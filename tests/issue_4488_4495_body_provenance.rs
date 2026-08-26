//! [Issue #4488/#4495] 공개 저수준 `parse_document → Document 직접 변경 →
//! serialize_document` 경로에서 본문 변경이 Section `raw_stream`(#4488)과 하위
//! 컨트롤 raw 레코드(#4495)에 가려져 조용히 사라지던 계약을 고정한다.
//!
//! - 무변경 문서는 Section 레코드 스트림 바이트를 정확히 재사용한다.
//! - 공개 본문 텍스트·중첩 표 셀 텍스트 직접 변경은 저장·재로드 뒤 유지된다.
//! - 표 CTRL_HEADER 의 모델 필드(`common`) 직접 변경도 유지된다 — raw_ctrl_data
//!   가 남아 있어도 봉인 불일치로 IR 합성 경로를 탄다.
//! - 공개 `raw_stream` 을 다른 바이트로 교체해도 봉인으로 승인되지 않는다.

use rhwp::model::control::Control;
use rhwp::{parse_document, serialize_document};

const SAMPLE: &str = "samples/2026_oss_rst.hwp";
const TABLE_SAMPLE: &str = "samples/basic/issue2007_nested_cell_pagination_42065.hwp";

fn read_sample(rel: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

#[test]
fn unchanged_parse_serialize_reuses_section_raw_bytes() {
    let doc = parse_document(&read_sample(SAMPLE)).expect("parse");
    let original: Vec<Vec<u8>> = doc
        .sections
        .iter()
        .map(|s| {
            assert!(
                s.raw_provenance.is_some(),
                "파서가 만든 문서에는 Section 봉인이 있어야 한다 (#4488)"
            );
            s.raw_stream.clone().expect("HWP5 Section raw 캐시")
        })
        .collect();

    let out = serialize_document(&doc).expect("serialize");
    let reparsed = parse_document(&out).expect("reparse");
    for (i, sec) in reparsed.sections.iter().enumerate() {
        assert_eq!(
            sec.raw_stream.as_ref(),
            Some(&original[i]),
            "무변경 구역 {i} 의 레코드 스트림은 바이트 그대로 통과해야 한다"
        );
    }
}

/// 본문 문단 텍스트를 같은 길이로 바꿔치기 — 길이 불변이라 char_count·char_shapes
/// 경계에 영향을 주지 않는 가장 보수적인 공개 직접 변경.
fn swap_first_char_of_longest_para(
    doc: &mut rhwp::model::document::Document,
) -> (usize, usize, String) {
    let (si, pi) = {
        let mut best = (0usize, 0usize, 0usize);
        for (si, sec) in doc.sections.iter().enumerate() {
            for (pi, para) in sec.paragraphs.iter().enumerate() {
                let len = para.text.chars().count();
                if len > best.2 {
                    best = (si, pi, len);
                }
            }
        }
        assert!(best.2 >= 3, "샘플 전제: 본문 텍스트 문단이 있어야 한다");
        (best.0, best.1)
    };
    let para = &mut doc.sections[si].paragraphs[pi];
    let mut chars: Vec<char> = para.text.chars().collect();
    let idx = chars
        .iter()
        .position(|c| *c != 'Ｘ' && !c.is_control())
        .expect("교체할 글자");
    chars[idx] = 'Ｘ'; // 전각 문자 — UTF-16 1유닛 유지
    para.text = chars.into_iter().collect();
    (si, pi, doc.sections[si].paragraphs[pi].text.clone())
}

#[test]
fn public_body_text_mutation_survives_save_reload() {
    let mut doc = parse_document(&read_sample(SAMPLE)).expect("parse");
    let (si, pi, mutated) = swap_first_char_of_longest_para(&mut doc);

    let out = serialize_document(&doc).expect("serialize");
    let reparsed = parse_document(&out).expect("reparse");
    assert_eq!(
        reparsed.sections[si].paragraphs[pi].text, mutated,
        "공개 본문 텍스트 직접 변경이 저장·재로드 뒤 유지돼야 한다 (#4488)"
    );
}

/// 첫 번째 표를 (구역, 문단, 컨트롤) 좌표로 찾는다.
fn find_first_table(doc: &rhwp::model::document::Document) -> (usize, usize, usize) {
    for (si, sec) in doc.sections.iter().enumerate() {
        for (pi, para) in sec.paragraphs.iter().enumerate() {
            for (ci, ctrl) in para.controls.iter().enumerate() {
                if matches!(ctrl, Control::Table(_)) {
                    return (si, pi, ci);
                }
            }
        }
    }
    panic!("샘플 전제: 표가 있어야 한다");
}

#[test]
fn nested_table_cell_text_mutation_survives_save_reload() {
    let mut doc = parse_document(&read_sample(TABLE_SAMPLE)).expect("parse");
    let (si, pi, ci) = find_first_table(&doc);
    let Control::Table(table) = &mut doc.sections[si].paragraphs[pi].controls[ci] else {
        unreachable!()
    };
    let (cell_idx, cp_idx) = {
        let mut found = None;
        'outer: for (celli, cell) in table.cells.iter().enumerate() {
            for (cpi, cp) in cell.paragraphs.iter().enumerate() {
                if cp.text.chars().count() >= 2 && cp.text.chars().any(|c| !c.is_control()) {
                    found = Some((celli, cpi));
                    break 'outer;
                }
            }
        }
        found.expect("샘플 전제: 텍스트 있는 셀 문단")
    };
    let cp = &mut table.cells[cell_idx].paragraphs[cp_idx];
    let mut chars: Vec<char> = cp.text.chars().collect();
    let idx = chars.iter().position(|c| !c.is_control()).unwrap();
    chars[idx] = 'Ｘ';
    cp.text = chars.into_iter().collect();
    let mutated = cp.text.clone();

    let out = serialize_document(&doc).expect("serialize");
    let reparsed = parse_document(&out).expect("reparse");
    let Control::Table(table2) = &reparsed.sections[si].paragraphs[pi].controls[ci] else {
        panic!("표가 유지돼야 한다")
    };
    assert_eq!(
        table2.cells[cell_idx].paragraphs[cp_idx].text, mutated,
        "중첩 표 셀 텍스트 직접 변경이 저장·재로드 뒤 유지돼야 한다 (#4488)"
    );
}

#[test]
fn table_common_mutation_survives_despite_raw_ctrl_data() {
    let mut doc = parse_document(&read_sample(TABLE_SAMPLE)).expect("parse");
    let (si, pi, ci) = find_first_table(&doc);
    let new_margin_top = {
        let Control::Table(table) = &mut doc.sections[si].paragraphs[pi].controls[ci] else {
            unreachable!()
        };
        assert!(
            !table.raw_ctrl_data.is_empty(),
            "샘플 전제: HWP5 표는 raw_ctrl_data 를 갖는다"
        );
        assert!(
            table.raw_ctrl_seal.is_some(),
            "파서가 만든 표에는 CTRL_HEADER 봉인이 있어야 한다 (#4495)"
        );
        let v = table.common.margin.top + 37;
        table.common.margin.top = v;
        v
    };

    let out = serialize_document(&doc).expect("serialize");
    let reparsed = parse_document(&out).expect("reparse");
    let Control::Table(table2) = &reparsed.sections[si].paragraphs[pi].controls[ci] else {
        panic!("표가 유지돼야 한다")
    };
    assert_eq!(
        table2.common.margin.top, new_margin_top,
        "표 CTRL_HEADER 모델 필드 직접 변경이 raw_ctrl_data 에 가려지면 안 된다 (#4495)"
    );
}

#[test]
fn unchanged_table_keeps_raw_ctrl_bytes_when_body_elsewhere_changes() {
    // 다른 곳(본문 문단)이 바뀌어 구역이 재생성돼도, 변경되지 않은 표의
    // CTRL_HEADER 는 원본 raw 바이트를 유지해야 한다(하위 raw 전면 폐기 금지).
    let mut doc = parse_document(&read_sample(TABLE_SAMPLE)).expect("parse");
    let (si, pi, ci) = find_first_table(&doc);
    let original_raw = {
        let Control::Table(table) = &doc.sections[si].paragraphs[pi].controls[ci] else {
            unreachable!()
        };
        table.raw_ctrl_data.clone()
    };
    swap_first_char_of_longest_para(&mut doc);

    let out = serialize_document(&doc).expect("serialize");
    let reparsed = parse_document(&out).expect("reparse");
    let Control::Table(table2) = &reparsed.sections[si].paragraphs[pi].controls[ci] else {
        panic!("표가 유지돼야 한다")
    };
    assert_eq!(
        table2.raw_ctrl_data, original_raw,
        "무변경 표의 CTRL_HEADER raw 는 유지돼야 한다"
    );
}

#[test]
fn swapped_section_raw_stream_is_not_honored() {
    let mut doc = parse_document(&read_sample(SAMPLE)).expect("parse");
    let para_count = doc.sections[0].paragraphs.len();
    doc.sections[0].raw_stream = Some(vec![0xDE; 64]);

    let out = serialize_document(&doc).expect("교체 raw 는 무시되고 모델로 재생성돼야 한다");
    let reparsed = parse_document(&out).expect("산출물은 정상 문서여야 한다");
    assert_eq!(
        reparsed.sections[0].paragraphs.len(),
        para_count,
        "산출물은 교체 바이트가 아니라 모델 상태를 담아야 한다 (#4488)"
    );
}
