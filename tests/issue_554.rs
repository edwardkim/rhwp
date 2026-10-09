//! Task #554 회귀 테스트
//!
//! HWP3 → HWP5/HWPX 변환본 페이지네이션 정합 검증.
//!
//! ## 본질
//!
//! 한컴이 HWP3 → HWP5/HWPX 변환 시 한글97의 "마지막 줄 tolerance" (1600 HU)
//! 동작이 누락되어 페이지 수가 +1 ~ +4 증가했다. Task #554 에서 변환본
//! 식별 휴리스틱 + 조건부 `margin_bottom -= 1600` 보정으로 정정.
//!
//! ## 식별 휴리스틱
//!
//! - HWPX: `<hh:head version="1.4">`
//! - HWP5: `(ParaShape/Paragraph < 0.05) AND (CharShape/Paragraph < 0.15) AND (Paragraph > 50)`
//!
//! Task #554 Stage 1 진단 결과 (27 fixture 100% 정확).

use std::fs;
use std::path::Path;

fn page_count(rel_path: &str) -> usize {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(rel_path);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {rel_path}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {rel_path}: {e:?}"));
    doc.page_count() as usize
}

// ───────── HWP3 변환본 (Task #554 핵심 대상): 정답 정합 ─────────

#[test]
fn hwp3_sample4_hwp5_36p() {
    // HWP3 변환본 (36페이지 정답): 한컴 정답 일치
    assert_eq!(page_count("samples/hwp3-sample4-hwp5.hwp"), 36);
}

#[test]
fn hwp3_sample5_hwp5_64p() {
    // HWP3 변환본 (64페이지 정답)
    assert_eq!(page_count("samples/hwp3-sample5-hwp5.hwp"), 64);
}

#[test]
fn hwp3_sample5_hwpx_64p() {
    // HWPX 변환본 (hwpml=1.4)
    assert_eq!(page_count("samples/hwp3-sample5-hwpx.hwpx"), 64);
}

// ───────── 알려진 잔존 -1 over-correct 해소 확인 ─────────
//
// #949 lineSegArray vertpos 보존 후 sample 변환본도 한컴 정답 16p에 도달했다.
// 이전에는 단일 -1600 HU 보정의 한계로 15p가 나왔고, 이 값을 known-limit 가드로
// 고정해 두었다. 이제는 정답 페이지 수를 회귀 가드로 검증한다.

#[test]
fn hwp3_sample_hwp5_16p() {
    // HWP3 변환본: 한컴 정답 16p
    assert_eq!(page_count("samples/hwp3-sample-hwp5.hwp"), 16);
}

#[test]
fn hwp3_sample_hwpx_16p() {
    // HWPX 변환본: 한컴 정답 16p
    assert_eq!(page_count("samples/hwp3-sample-hwpx.hwpx"), 16);
}

// ───────── HWP3 원본 회귀 0 (Task #460 보정과 충돌 없음) ─────────

#[test]
fn hwp3_sample_hwp3_16p() {
    use rhwp::document_core::DocumentCore;
    use rhwp::model::control::Control;
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

    fn nodes<'a>(node: &'a RenderNode, pred: &impl Fn(&RenderNode) -> bool) -> Vec<&'a RenderNode> {
        let mut result = Vec::new();
        if pred(node) {
            result.push(node);
        }
        for child in &node.children {
            result.extend(nodes(child, pred));
        }
        result
    }
    fn body_line(node: &RenderNode, pi: usize) -> bool {
        matches!(&node.node_type, RenderNodeType::TextLine(line)
            if line.para_index == Some(pi) && line.caption_owner.is_none())
    }
    fn gap_hu(gap: f64) -> i32 {
        (gap / rhwp::renderer::hwpunit_to_px(1, 96.0)).round() as i32
    }

    // issue_265.hwp와 같은 바이트의 원본이다. 전쪽 Native/fresh 및 독립 Print 16쪽을 대조한다.
    let bytes = fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwp3-sample.hwp"))
        .expect("HWP3 원본 읽기");
    let mut core = DocumentCore::from_bytes(&bytes).expect("HWP3 원본 열기");
    core.set_hangul2024_compat(false);
    assert_eq!(core.page_count(), 16, "원본의 전체 쪽 소속");
    let paragraphs = &core.document().sections[0].paragraphs;

    let host = paragraphs
        .iter()
        .position(|para| {
            para.controls.iter().any(|control| {
                let Control::Picture(picture) = control else {
                    return false;
                };
                picture.caption.as_ref().is_some_and(|caption| {
                    caption
                        .paragraphs
                        .iter()
                        .any(|paragraph| paragraph.text.contains("가상 서버의 일반적인 구조"))
                })
            })
        })
        .expect("그림 캡션 원본 소유자");
    let (ci, picture) = paragraphs[host]
        .controls
        .iter()
        .enumerate()
        .find_map(|(ci, control)| {
            if let Control::Picture(pic) = control {
                Some((ci, pic))
            } else {
                None
            }
        })
        .expect("그림 개체");
    assert!(
        picture.caption.as_ref().expect("캡션").spacing > 0,
        "원본 그림 정보의 세로 캡션 띠에는 텍스트 외 간격이 있음"
    );
    let page3 = core
        .build_page_render_tree(2)
        .expect("그림과 뒤 본문이 있는 3쪽");
    let images = nodes(&page3.root, &|node| {
        matches!(&node.node_type, RenderNodeType::Image(image)
        if image.para_index == Some(host) && image.control_index == Some(ci))
    });
    assert_eq!(images.len(), 1, "그림의 같은 쪽 소속과 중복 없음");
    let captions = nodes(&page3.root, &|node| {
        matches!(&node.node_type, RenderNodeType::TextLine(line)
        if line.caption_owner.is_some_and(|owner| owner.para_idx == host && owner.control_idx == ci))
    });
    assert_eq!(captions.len(), 1, "독립 Print와 같은 한 줄 캡션");
    assert!(
        captions[0].bbox.y > images[0].bbox.y + images[0].bbox.height,
        "캡션 간격을 지워 그림 하단에 붙이지 않음"
    );
    let blank = nodes(&page3.root, &|node| body_line(node, host + 1));
    assert_eq!(blank.len(), 1, "그림 뒤 빈 문단의 줄 점유");
    let host_line = &paragraphs[host].line_segs[0];
    let occupied = gap_hu(blank[0].bbox.y - captions[0].bbox.y - captions[0].bbox.height)
        - i32::from(picture.common.margin.bottom);
    assert!(
        occupied >= host_line.line_height + host_line.line_spacing,
        "그림 호스트의 저장 줄 진행은 캡션과 별도로 보존: {occupied}"
    );
    let after = nodes(&page3.root, &|node| body_line(node, host + 2));
    assert!(
        !after.is_empty() && after[0].bbox.y >= blank[0].bbox.y + blank[0].bbox.height,
        "뒤 본문은 빈 문단 뒤에서 시작"
    );

    let paragraph = paragraphs
        .iter()
        .position(|para| para.text.starts_with("NAT에 의한 가상 서버의 작업 흐름"))
        .expect("쪽을 잇는 원본 문단");
    let source = &paragraphs[paragraph].line_segs;
    let cut = source
        .windows(2)
        .position(|lines| lines[0].vertical_pos > 0 && lines[1].vertical_pos == 0)
        .expect("원본의 물리 쪽 경계")
        + 1;
    let mut rendered = Vec::new();
    for (page_index, range) in [(5, 0..cut), (6, cut..source.len())] {
        let page = core
            .build_page_render_tree(page_index)
            .expect("이어지는 본문 쪽");
        let lines = nodes(&page.root, &|node| body_line(node, paragraph));
        let indexes: Vec<usize> = lines
            .iter()
            .map(|node| {
                let RenderNodeType::TextLine(line) = &node.node_type else {
                    unreachable!()
                };
                line.line_index.expect("원본 줄 소유") as usize
            })
            .collect();
        assert_eq!(
            indexes,
            range.collect::<Vec<_>>(),
            "문단의 저장 쪽 경계와 누락/중복 없음"
        );
        rendered.extend(indexes);
    }
    assert_eq!(
        rendered.len(),
        source.len(),
        "이어진 문단의 모든 줄을 한 번씩 표시"
    );

    let page8 = core
        .build_page_render_tree(7)
        .expect("규칙 표와 뒤 본문의 8쪽");
    let tables = nodes(&page8.root, &|node| {
        matches!(&node.node_type, RenderNodeType::Table(_))
    });
    assert_eq!(tables.len(), 5, "독립 Print의 규칙 표와 네 패킷 표");
    let mut checked = 0;
    for table_node in &tables {
        let RenderNodeType::Table(table) = &table_node.node_type else {
            unreachable!()
        };
        let pi = table.para_index.expect("본문 표 소유");
        let ci = table.control_index.expect("표 컨트롤 소유");
        let Control::Table(original) = &paragraphs[pi].controls[ci] else {
            panic!("원본 표")
        };
        let next = nodes(&page8.root, &|node| body_line(node, pi + 1));
        if next.is_empty() {
            continue;
        }
        let spacing = paragraphs[pi].line_segs[0].line_spacing;
        assert!(
            spacing > 0,
            "백분율 문단의 호스트 글꼴 후행 간격을 표 높이와 구분"
        );
        let gap = gap_hu(next[0].bbox.y - table_node.bbox.y - table_node.bbox.height)
            - i32::from(original.outer_margin_bottom);
        assert!(
            gap >= spacing,
            "표 뒤 본문이 호스트 후행 띠를 침범하지 않음: {gap} < {spacing}"
        );
        checked += 1;
    }
    assert_eq!(checked, 4, "같은 쪽의 표 뒤 네 본문 간격을 확인");
}

#[test]
fn hwp3_sample5_hwp3_64p() {
    // HWP3 원본
    assert_eq!(page_count("samples/hwp3-sample5.hwp"), 64);
}

// ───────── 광범위 회귀 0 (휴리스틱 false positive 없음) ─────────

#[test]
fn task554_no_regression_2022_kuglip() {
    // 2022년 국립국어원: 단순 -1600 적용 시 -5 회귀였던 케이스
    // 휴리스틱 (PS/CS 비율) 로 변환본이 아니라 정확히 분류 → 보정 미적용
    // [Task #643] 페이지 분할 드리프트 정정 + Task #404 vpos_end 트레일링 ls 제외:
    //   pi=80 (page 6) 트레일링 ls 정정 + pi=39 (page 3) heading-orphan 가드 정정
    //   → 후속 페이지 압축 → 40 → 35 페이지 (HWP 원본 정합 회복)
    assert_eq!(page_count("samples/2022년 국립국어원 업무계획.hwp"), 35);
}

#[test]
fn task554_no_regression_aift() {
    // aift: 단순 -1600 적용 시 -1 회귀였던 케이스
    // Task #874 #1~#8 누적 정합 결과 한컴 PDF (pdf/aift-2022.pdf) 와 동일한 74p.
    assert_eq!(page_count("samples/aift.hwp"), 74);
}

#[test]
fn task554_no_regression_2025_donations_hwpx() {
    // 2025년 기부·답례품 HWPX: hwpml=1.5 (직접 작성) — 휴리스틱 미적용
    assert_eq!(
        page_count("samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx"),
        30
    );
}
