//! 관계형 및 고정형 회귀: 원본 번호 수준·마커·왕복 보존과 빈 머리 본문 거리의 관계. 양쪽 100.00% 완료 1쪽의 독립 Print 본문 시작은 ±1 CSSpx로 보호한다.
//! 주석 갱신: 2026-11-06T08:25:01+09:00 (Asia/Seoul).
//! 직전 파일 변경: 5172f2d55c73b2ed33d4d3657d81e769bc9004c2, 작성 2026-11-06T08:13:58+09:00, 커밋 2026-11-06T08:13:58+09:00.
//! 측정일: 2026-11-06 KST. 제품 source 86709ac16711820e61baa3b87e2c4ddaac23573e.
//! Print 프로필·96 DPI·PDFium·색상 경계 보정 전 2px raw. 독립 PDF와 전체 쪽수가 같다.
//! 대응 실문서·API 저장본의 전쪽 결과이며 입력 개입·합성 분기의 전수 시각 측정으로 주장하지 않는다.
//! levels.hwp (8527-levels-hwp-2022): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! 고정 1쪽: Native 100.0%, fresh WASM 100.0%; 같은 영역의 독립 Print 확대 판독 완료.
//! levels.hwpx (8527-levels-hwpx-2022): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! 고정 1쪽: Native 100.0%, fresh WASM 100.0%; 같은 영역의 독립 Print 확대 판독 완료.
//! 원본·PDF SHA, 명령·원본별 전쪽 TSV·정밀 raw·대표 PNG:
//! mydocs/pr/assets/semanticist21_20261105/validation-r6/README.md 및 measurements.md.
//! 원본 형식 계약:
//! 관계형·고정형: 검사 대상 — 한/글 12.30이 저장한 8~10수준 개요·번호 문단.
//! 입력·기준: `tests/fixtures/numbering_levels/levels.hwp`·`levels.hwpx`(한/글 12.30 Mac 저장본, 저장자 이름만
//! 바꿈). 개요 1·7·8·9·10수준과 번호(1. 1.1. 1.1.1. 견본) 1·8·9·10수준 문단이 있다.
#![cfg(not(target_arch = "wasm32"))]
//! 한/글은 이 문서의 번호 문단을 `1.`·`1.1.1.1.1.1.1.1.`·`1.1.1.1.1.1.1.1.1.`·
//! `1.1.1.1.1.1.1.1.1.1.`로, 개요 9·10수준은 번호 서식이 비어 번호 없이 그린다.
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;
use std::io::{Cursor, Read};

fn fixture(name: &str) -> HwpDocument {
    let path = format!(
        "{}/tests/fixtures/numbering_levels/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    HwpDocument::from_bytes(&std::fs::read(path).unwrap()).unwrap()
}

/// 문단 수준과 위에서부터의 번호 마커 글(원문 자리 없는 글).
fn markers(doc: &HwpDocument) -> (Vec<u8>, Vec<String>) {
    let document = doc.document();
    let levels = document.sections[0]
        .paragraphs
        .iter()
        .map(|p| document.doc_info.para_shapes[p.para_shape_id as usize].para_level)
        .collect();
    let layout: Value = serde_json::from_str(&doc.get_page_text_layout_native(0).unwrap()).unwrap();
    let texts = layout["runs"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|run| run.get("charStart").is_none())
        .filter_map(|run| run["text"].as_str().map(str::to_string))
        .collect();
    (levels, texts)
}

#[test]
fn hancom_levels_eight_to_ten_keep_their_level_and_number_after_saving() {
    let number = |n: usize| "1.".repeat(n);
    for name in ["levels.hwp", "levels.hwpx"] {
        let doc = fixture(name);
        let check = |doc: &HwpDocument, label: &str| {
            let (levels, texts) = markers(doc);
            assert_eq!(levels, [0, 6, 7, 8, 9, 0, 7, 8, 9], "{label}");
            // 개요 1·7·8수준과 번호 1·8·9·10수준만 번호를 그린다. 개요 9·10수준은 서식이 비었다.
            assert_eq!(texts.len(), 7, "{label}: {texts:?}");
            assert_eq!(
                texts[3..],
                [number(1), number(8), number(9), number(10)],
                "{label}"
            );
        };
        check(&doc, name);
        for hwpx in [false, true] {
            let saved = if hwpx {
                doc.export_hwpx_native().unwrap()
            } else {
                doc.export_hwp_native().unwrap()
            };
            if hwpx {
                // 한/글처럼 8~10수준 머리는 2016 문단 네임스페이스 case 에만 두고 default 는 머리 없음이다.
                let mut header = String::new();
                zip::ZipArchive::new(Cursor::new(&saved))
                    .unwrap()
                    .by_name("Contents/header.xml")
                    .unwrap()
                    .read_to_string(&mut header)
                    .unwrap();
                assert!(header.contains(concat!(
                    r#"<hp:switch><hp:case hp:required-namespace="http://www.hancom.co.kr/hwpml/2016/paragraph">"#,
                    r#"<hh:heading type="NUMBER" idRef="2" level="9"/></hp:case><hp:default>"#,
                    r#"<hh:heading type="NONE" idRef="0" level="0"/></hp:default></hp:switch>"#,
                )));
            }
            check(
                &HwpDocument::from_bytes(&saved).unwrap(),
                &format!("{name} → hwpx={hwpx}"),
            );
        }
    }
}

/// 한/글 5.1은 번호 레코드의 7수준 시작 번호 뒤에 8~10수준 머리 정보·형식 문자열 3개와 시작 번호 3개를
/// 덧붙인다. 모델에서 다시 쓴 레코드가 한/글 저장본과 바이트 단위로 같다.
#[test]
fn hancom_numbering_records_are_rewritten_byte_for_byte() {
    let mut doc = fixture("levels.hwp");
    let hancom: Vec<Vec<u8>> = doc
        .document()
        .doc_info
        .numberings
        .iter()
        .map(|n| n.raw_data.clone().unwrap())
        .collect();
    let formats = |doc: &HwpDocument| -> Vec<Vec<String>> {
        doc.document()
            .doc_info
            .numberings
            .iter()
            .map(|n| n.level_formats.to_vec())
            .collect()
    };
    assert_eq!(formats(&doc)[0][7..], ["^8", "", ""]);
    assert_eq!(formats(&doc)[1][9], "^1.^2.^3.^4.^5.^6.^7.^8.^9.^:.");
    let info = &mut doc.document_mut().doc_info;
    for numbering in &mut info.numberings {
        numbering.raw_data = None;
    }
    info.raw_stream_dirty = true;
    let saved = HwpDocument::from_bytes(&doc.export_hwp_native().unwrap()).unwrap();
    let rewritten: Vec<Vec<u8>> = saved
        .document()
        .doc_info
        .numberings
        .iter()
        .map(|n| n.raw_data.clone().unwrap())
        .collect();
    assert_eq!(rewritten, hancom);
}

/// 개요 찾아가기도 8~10수준을 제 수준으로 보인다. 9·10수준은 번호 서식이 비어 목록에 넣지 않는다.
#[test]
fn outline_navigation_lists_level_eight_on_its_own_level() {
    for name in ["levels.hwp", "levels.hwpx"] {
        let navigation: Value =
            serde_json::from_str(&fixture(name).get_outline_navigation_native().unwrap()).unwrap();
        let levels: Vec<u64> = navigation["outline"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item["level"].as_u64().unwrap())
            .collect();
        assert_eq!(levels, [1, 7, 8], "{name}");
    }
}

fn assert_empty_outline_head_body_distance(name: &str, distance: Option<i16>) {
    let file = format!(
        "{}/tests/fixtures/numbering_levels/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    let mut doc = HwpDocument::from_bytes(&std::fs::read(file).unwrap()).unwrap();
    if let Some(distance) = distance {
        let id = usize::from(doc.document().sections[0].section_def.outline_numbering_id) - 1;
        let mut model = doc.document().clone();
        for level in [8, 9] {
            model.doc_info.numberings[id].heads[level].text_distance = distance;
        }
        // 원시 IR만 바꾸지 않고 공개 setter로 번호 머리·스타일 파생 상태도 재구축한다.
        doc.set_document(model);
    }
    let document = doc.document();
    let paragraphs = &document.sections[0].paragraphs;
    let layout: Value = serde_json::from_str(&doc.get_page_text_layout_native(0).unwrap()).unwrap();
    let runs = layout["runs"].as_array().unwrap();
    let first_marker = runs
        .iter()
        .find(|r| r["text"] == "1." && r.get("charStart").is_none())
        .unwrap();
    let reference_x = first_marker["x"].as_f64().unwrap();
    for index in [3, 4] {
        let para = &paragraphs[index];
        let style = &document.doc_info.para_shapes[para.para_shape_id as usize];
        let numbering = &document.doc_info.numberings
            [usize::from(document.sections[0].section_def.outline_numbering_id) - 1];
        let level = usize::from(style.para_level);
        assert!(numbering.level_formats[level].is_empty());
        let head = numbering.heads[level];
        assert_eq!(
            head.text_distance,
            distance.unwrap_or(50),
            "원본 HWP/HWPX의 상대 거리와 거리 0 대조군"
        );
        assert_eq!(head.attr & 3, 0, "원본 머리의 왼쪽 정렬");
        assert_eq!(head.attr & (1 << 4), 0, "원본 머리 거리의 상대 단위");
        let shape = para.char_shape_id_at(0).unwrap();
        let nominal_size = document.doc_info.char_shapes[shape as usize].base_size as f64 / 75.0;
        let source_delta = (para.line_segs[0].column_start
            - paragraphs[0].line_segs[0].column_start) as f64
            / 75.0;
        let body = runs
            .iter()
            .find(|r| r["paraIdx"].as_u64() == Some(index as u64) && r["charStart"] == 0)
            .unwrap();
        let actual_ratio =
            (body["x"].as_f64().unwrap() - reference_x - source_delta) / nominal_size;
        let source_ratio = f64::from(head.text_distance) / 100.0;
        // 두 JSON 원점이 각각 0.1 단위로 반올림되는 정밀도만 허용한다.
        assert!(
            (actual_ratio - source_ratio).abs() <= 0.11 / nominal_size,
            "{name} 문단 {index}: 본문 상대 거리 비율 {actual_ratio}, 원본 {source_ratio}"
        );
    }
    assert_eq!(
        runs.iter().filter(|r| r.get("charStart").is_none()).count(),
        7,
        "빈 머리는 공통 기하만 점유하고 별도 마커 run을 만들지 않는다"
    );
}
#[test]
fn hwp_empty_heads_keep_distance() {
    assert_empty_outline_head_body_distance("levels.hwp", None);
}
#[test]
fn hwpx_empty_heads_keep_distance() {
    assert_empty_outline_head_body_distance("levels.hwpx", None);
}

#[test]
fn hwp_zero_distance_control() {
    assert_empty_outline_head_body_distance("levels.hwp", Some(0));
}
#[test]
fn hwpx_zero_distance_control() {
    assert_empty_outline_head_body_distance("levels.hwpx", Some(0));
}

/// 저장 LineSeg의 단위 관계와 별개로 같은 완료 페이지의 실제 본문 원점을 보호한다.
#[test]
fn empty_outline_body_origins_match_independent_print_page_one() {
    for name in ["levels.hwp", "levels.hwpx"] {
        let doc = fixture(name);
        assert_eq!(doc.page_count(), 1, "{name}: 독립 Print 전체 쪽수");
        let layout: Value =
            serde_json::from_str(&doc.get_page_text_layout_native(0).unwrap()).unwrap();
        let runs = layout["runs"].as_array().unwrap();
        for (index, print_points) in [(3, 180.022435), (4, 189.977040)] {
            let run = runs
                .iter()
                .find(|r| r["paraIdx"].as_u64() == Some(index) && r["charStart"] == 0)
                .unwrap();
            let actual = run["x"].as_f64().unwrap();
            let independent = print_points * 96.0 / 72.0;
            assert!(
                (actual - independent).abs() <= 1.0,
                "{name} 문단 {index}: 독립 Print 본문 시작 ±1 CSSpx: {actual} / {independent}"
            );
            // 언어 슬롯이 다른 한글·숫자는 별도 run이므로 문단의 원문 순서로 확인한다.
            let mut body_runs: Vec<_> = runs
                .iter()
                .filter(|r| r["paraIdx"].as_u64() == Some(index) && r["charStart"].is_number())
                .collect();
            body_runs.sort_by_key(|r| r["charStart"].as_u64().unwrap());
            let text: String = body_runs
                .iter()
                .map(|r| r["text"].as_str().unwrap())
                .collect();
            let expected = if index == 3 {
                "개요9 문단"
            } else {
                "개요10 문단"
            };
            assert_eq!(text, expected, "{name} 문단 {index}: 본문 순서·누락·중복");
        }
    }
}
