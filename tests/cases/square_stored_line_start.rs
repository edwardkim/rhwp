//! 2026-11-11 추가 관계형 검사: 종이 기준 그림의 세로 원점을 본문 좌표로 맞춰 후속 문단의 저장 줄 수와 cs를 보존한다.
//! 27쪽 보정 전 Native/fresh WASM: HWP5 99.50/99.50%, HWPX 97.69/97.69% → 각각 100.00/100.00%.
//! 같은 쪽 review·overlay에서 /home의 다음 줄 소속과 그림 오른쪽 배치를 판독했다.
//! 관계형은 원본 cs와 실제 단 원점의 차이를 비교하며 PDF 절대 px 좌표를 고정하지 않는다.
//! 정식 전후 증거: mydocs/pr/assets/planet6897_clean19_20261110/pr8734/maintainer/mount-{hwp5,hwpx}/.
//! 추가 관계형 검사: HWPX 16쪽의 균일·혼합 후속 줄은 원본 cs/sw와 같은 lane을 소비한다.
//! 2026-11-10 보정 후 같은 16쪽 Native/fresh WASM 2px raw: 100.00/100.00%. 같은 쪽 review·overlay 판독 후 소속 검사 추가.
//! 관계형 회귀 — 어울림(Square) 그림 문단의 글 줄은 저장 `cs`·`sw`를 그대로 쓴다.
//! 주석 갱신: 2026-11-11 01:06 KST
//! 측정일: 2026-11-07. raw = `silhouette_raw_match_percent`, 96 DPI, 독립 한컴 Print PDF.
//! 측정 값은 아래 표에 원본별로 적고, 정식 증거는 `mydocs/pr/assets/issue_8730/`에 둔다.
//!
//! ```text
//! 원본 (전체 쪽수)                              쪽  수정 전 N/W     수정 후 N/W      전쪽 최저 N/W
//! samples/hwp3-sample5.hwp (64쪽, 굴림 공급)    22  97.90/97.90     100.00/100.00    95.70/95.63
//! samples/hwp3-sample5-hwp5.hwp (64쪽, 굴림)    22  93.59/93.69     100.00/100.00    99.51/99.49
//! samples/issue6204/square_picture_band_host     1  80.82/79.48      92.15/93.42     92.15/93.42
//! ```
//!
//! 저장 줄의 시작 위치는 그림 왼쪽 바깥 여백·폭·오른쪽 바깥 여백을 모두 지난 자리다. 한컴이
//! 저장한 HWP5·HWPX 코퍼스와 `hwp3-sample5.hwp`의 한컴 변환본이 이를 보여 준다. 그래서
//! 배치에서 여백을 다시 더하면 그림 옆 줄, 그림 아래 전폭 줄, 그림이 오른쪽에 있어 `cs=0`인
//! 줄이 모두 여백만큼 밀린다. HWP3 파서는 왼쪽 여백을 빼고 있었고, 샘플의 좌우 여백이 같아
//! 배치의 가산이 그 부족분을 우연히 메웠다. 그래서 파서 값을 변환본과 직접 대조한다.
//!
//! 기대값은 구현 계산이 아니라 저장 값과 같은 쪽의 단 왼쪽 끝에서 얻는다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::shape::TextWrap;
use serde_json::Value;
use std::fs;
use std::path::Path;

const TOLERANCE_PX: f64 = 0.5;
const HWP3: &str = "samples/hwp3-sample5.hwp";
const HANCOM_HWP5: &str = "samples/hwp3-sample5-hwp5.hwp";
const RIGHT_PICTURE: &str = "samples/issue6204/square_picture_band_host.hwp";

fn bytes(sample: &str) -> Vec<u8> {
    fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(sample))
        .unwrap_or_else(|e| panic!("read {sample}: {e}"))
}

fn document(sample: &str) -> Document {
    rhwp::parser::parse_document(&bytes(sample)).unwrap_or_else(|e| panic!("parse {sample}: {e}"))
}

fn page_tree(sample: &str, page_index: u32) -> Value {
    let document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes(sample))
        .unwrap_or_else(|e| panic!("open {sample}: {e:?}"));
    let json = document
        .get_page_render_tree(page_index)
        .unwrap_or_else(|e| panic!("{sample} {}쪽: {e:?}", page_index + 1));
    serde_json::from_str(&json).expect("render tree json")
}

fn collect<'a>(node: &'a Value, kind: &str, out: &mut Vec<&'a Value>) {
    if node["type"] == kind {
        out.push(node);
    }
    for child in node["children"].as_array().into_iter().flatten() {
        collect(child, kind, out);
    }
}

fn bbox(node: &Value, key: &str) -> f64 {
    node["bbox"][key].as_f64().expect("bbox 좌표")
}

fn column_left(tree: &Value) -> f64 {
    let mut columns = Vec::new();
    collect(tree, "Column", &mut columns);
    assert!(!columns.is_empty(), "쪽에 단이 있어야 한다");
    bbox(columns[0], "x")
}

/// 문단의 렌더 줄 시작을 왼쪽부터 중복 없이 돌려준다.
fn line_starts(tree: &Value, para_index: u64) -> Vec<f64> {
    let mut lines = Vec::new();
    collect(tree, "TextLine", &mut lines);
    let mut starts: Vec<f64> = lines
        .iter()
        .filter(|line| line["pi"].as_u64() == Some(para_index))
        .map(|line| bbox(line, "x"))
        .collect();
    starts.sort_by(f64::total_cmp);
    starts.dedup_by(|a, b| (*a - *b).abs() < TOLERANCE_PX);
    starts
}

fn hu_to_px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

/// 관계형: Paper 좌표의 그림 세로 밴드를 본문 상대 저장 줄과 같은 축으로 대조한다.
/// 009b3cf42b의 HWPX 27쪽은 Native/fresh WASM 모두 97.69%지만 명령문이 그림 뒤에
/// 가려졌다. 독립 Print의 명령문 두 줄과 다음 명령 한 줄을 원본 소유 줄/차선으로 검사한다.
/// 전역 픽셀 좌표를 고정하지 않으며 글꼴 파일의 설치 유무에 의존하지 않는다.
fn assert_short_successor_source_rows(sample: &str) {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    fn descendants<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
        out.push(node);
        for child in &node.children {
            descendants(child, out);
        }
    }
    // 공개 JSON은 좌표를 소수 한 자리로 표시한다. 원본 HU 관계의 산술 검사는
    // 같은 프로덕션 트리의 정밀 좌표를 사용해 표시 반올림을 배치 결함으로 오인하지 않는다.
    let core = rhwp::document_core::DocumentCore::from_bytes(&bytes(sample)).unwrap();
    let tree = core.build_page_render_tree(26).unwrap();
    let mut nodes = Vec::new();
    descendants(&tree.root, &mut nodes);
    let left = nodes
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::Column(_)))
        .expect("원본의 본문 단")
        .bbox
        .x;
    for prefix in ["마운트는 다음과 같이", "$ mount /dev/hda3"] {
        let (pi, paragraph) = core.document().sections[0]
            .paragraphs
            .iter()
            .enumerate()
            .find(|(_, p)| p.text.starts_with(prefix))
            .expect("독립 Print 27쪽의 원본 명령 문단");
        let owned: Vec<_> = nodes
            .iter()
            .filter(|node| matches!(&node.node_type, RenderNodeType::TextLine(line) if line.para_index == Some(pi)))
            .collect();
        assert_eq!(
            owned.len(),
            paragraph.line_segs.len(),
            "명령문 저장 줄을 합치거나 누락하지 않는다"
        );
        for (line, segment) in owned.iter().zip(&paragraph.line_segs) {
            assert!(
                (line.bbox.x - left - hu_to_px(segment.column_start)).abs() < 0.01,
                "원본 저장 차선을 잃어 그림 뒤에 글자가 놓였다: {prefix}, actual={}, column_left={left}, source_start_hu={}",
                line.bbox.x, segment.column_start
            );
        }
    }
}

#[test]
fn paper_picture_lane_preserves_short_successor_paragraphs_and_every_source_row() {
    assert_short_successor_source_rows("samples/hwp3-sample5-hwpx.hwpx");
}

#[test]
fn paper_picture_lane_preserves_hwp5_mount_command_line_partition() {
    assert_short_successor_source_rows(HANCOM_HWP5);
}

fn square_picture_paragraphs(paragraphs: &[Paragraph]) -> Vec<usize> {
    paragraphs
        .iter()
        .enumerate()
        .filter(|(_, para)| {
            para.controls.iter().any(|control| {
                matches!(control, Control::Picture(picture)
                    if !picture.common.treat_as_char
                        && picture.common.text_wrap == TextWrap::Square
                        && picture.common.margin.right > 0)
            })
        })
        .map(|(index, _)| index)
        .collect()
}

fn widest_start(para: &Paragraph) -> (i32, i32) {
    para.line_segs
        .iter()
        .map(|seg| (seg.column_start, seg.segment_width))
        .max_by_key(|(start, _)| *start)
        .expect("저장 줄")
}

/// HWP3 파서의 어울림 줄은 같은 원본의 한컴 변환본과 같아야 한다. 좌우 여백이 같은
/// 샘플이라 배치의 가산이 파서의 부족분을 가리던 자리를 저장 값으로 직접 본다.
#[test]
fn hwp3_square_line_box_matches_the_hancom_conversion() {
    let hwp3 = document(HWP3);
    let converted = document(HANCOM_HWP5);
    let compared: Vec<usize> = square_picture_paragraphs(&converted.sections[0].paragraphs)
        .into_iter()
        .filter(|&pi| widest_start(&converted.sections[0].paragraphs[pi]).0 > 0)
        .collect();
    assert!(
        compared.len() >= 5,
        "왼쪽 그림 어울림 문단이 비교 대상이어야 한다: {compared:?}"
    );
    for pi in compared {
        assert_eq!(
            widest_start(&hwp3.sections[0].paragraphs[pi]),
            widest_start(&converted.sections[0].paragraphs[pi]),
            "pi={pi}: HWP3 파서 (cs, sw) 는 한컴 변환본 저장 값과 같아야 한다"
        );
    }
}

/// 왼쪽 그림: 좁은 줄은 단 왼쪽 + 저장 `cs`, 그림 아래 전폭 줄은 단 왼쪽에서 시작한다.
#[test]
fn left_picture_lines_start_at_their_stored_positions() {
    let converted = document(HANCOM_HWP5);
    for (sample, page_index, para_index) in [
        (HANCOM_HWP5, 3, 75_usize),
        (HANCOM_HWP5, 21, 605),
        (HWP3, 3, 75),
        (HWP3, 21, 605),
    ] {
        let tree = page_tree(sample, page_index);
        let left = column_left(&tree);
        let (stored_cs, _) = widest_start(&converted.sections[0].paragraphs[para_index]);
        assert!(stored_cs > 0, "pi={para_index}: 좁은 저장 줄이 있어야 한다");
        let starts = line_starts(&tree, para_index as u64);
        assert_eq!(
            starts.len(),
            2,
            "{sample} pi={para_index}: 전폭 줄과 좁은 줄이 함께 있어야 한다: {starts:?}"
        );
        assert!(
            (starts[0] - left).abs() < TOLERANCE_PX,
            "{sample} pi={para_index}: 전폭 줄 {} 은 단 왼쪽 {left} 이어야 한다",
            starts[0]
        );
        assert!(
            (starts[1] - (left + hu_to_px(stored_cs))).abs() < TOLERANCE_PX,
            "{sample} pi={para_index}: 좁은 줄 {} 은 단 왼쪽 + cs {stored_cs} 이어야 한다",
            starts[1]
        );
    }
}

/// 반례 — 그림이 오른쪽에 있으면 저장 `cs=0`이라 왼쪽 시작은 그림과 무관하다.
/// 줄 끝은 저장 `sw` 그대로 그림 왼쪽에서 끝난다.
#[test]
fn right_picture_keeps_the_left_edge_and_stored_line_end() {
    let source = document(RIGHT_PICTURE);
    let seg = &source.sections[0].paragraphs[1].line_segs[0];
    assert_eq!(seg.column_start, 0, "반례는 cs=0 줄이어야 한다");
    let tree = page_tree(RIGHT_PICTURE, 0);
    let left = column_left(&tree);
    let mut lines = Vec::new();
    collect(&tree, "TextLine", &mut lines);
    let first = lines
        .iter()
        .find(|line| line["pi"].as_u64() == Some(1))
        .expect("pi=1 첫 줄");
    assert!(
        (bbox(first, "x") - left).abs() < TOLERANCE_PX,
        "그림 옆 왼쪽 시작 {} 은 단 왼쪽 {left} 이어야 한다",
        bbox(first, "x")
    );
    let right = bbox(first, "x") + bbox(first, "w");
    assert!(
        (right - (left + hu_to_px(seg.segment_width))).abs() < TOLERANCE_PX,
        "줄 끝 {right} 은 단 왼쪽 + sw {} 이어야 한다",
        seg.segment_width
    );
}

#[test]
fn uniform_and_mixed_successor_rows_keep_their_source_picture_lane() {
    const HWPX: &str = "samples/hwp3-sample5-hwpx.hwpx";
    let source = document(HWPX);
    let mut rendered =
        rhwp::wasm_api::HwpDocument::from_bytes(&bytes(HWPX)).expect("한컴 HWPX 저장본");
    rendered.set_hangul2024_compat(false);
    let tree: Value = serde_json::from_str(
        &rendered
            .get_page_render_tree(15)
            .expect("그림과 후속 문단이 있는 16쪽"),
    )
    .expect("배치 JSON");
    let left = column_left(&tree);
    let mut runs = Vec::new();
    collect(&tree, "TextRun", &mut runs);
    let mut texts = std::collections::BTreeMap::<u64, String>::new();
    for run in runs {
        if let (Some(index), Some(text)) = (run["pi"].as_u64(), run["text"].as_str()) {
            texts
                .entry(index)
                .or_default()
                .extend(text.chars().filter(|ch| !ch.is_whitespace()));
        }
    }
    let mut lines = Vec::new();
    collect(&tree, "TextLine", &mut lines);
    for prefix in ["하드디스크는 하나 이상의", "CPU와 디스크는 디스크제어기"]
    {
        let original = source.sections[0]
            .paragraphs
            .iter()
            .find(|paragraph| paragraph.text.starts_with(prefix))
            .expect("원본 문단");
        let expected_prefix: String = prefix.chars().filter(|ch| !ch.is_whitespace()).collect();
        let (&owner, _) = texts
            .iter()
            .find(|(_, text)| text.starts_with(&expected_prefix))
            .expect("내용으로 찾은 실제 배치 문단");
        let owned: Vec<_> = lines
            .iter()
            .filter(|line| line["pi"].as_u64() == Some(owner))
            .collect();
        assert_eq!(
            owned.len(),
            original.line_segs.len(),
            "{prefix}: 저장 줄의 누락·중복"
        );
        for (line, stored) in owned.into_iter().zip(&original.line_segs) {
            assert!(
                (bbox(line, "x") - left - hu_to_px(stored.column_start)).abs() < TOLERANCE_PX,
                "{prefix}: 개체 옆 줄은 원본 lane의 시작을 지켜야 한다: {line}"
            );
            assert!(
                (bbox(line, "w") - hu_to_px(stored.segment_width)).abs() < TOLERANCE_PX,
                "{prefix}: 개체 아래 전폭 줄까지 원본 lane의 폭을 지켜야 한다: {line}"
            );
        }
    }
}

// 2026-11-11 00:44 KST 추가 관계형 검사. 고용서비스 보고서 79쪽의 독립 Print와
// Native 94.23%에서 본문이 그림 위로 흐르는 실제 차이를 확인했다. 줄 수·원본 차선·
// 그림과의 비겹침을 검사하며 절대 PDF px를 고정하지 않는다. 보정 후 같은 79쪽의
// Native/fresh WASM 2px raw는 각각 100.00/100.00%. 인접78·80쪽도 양쪽100.00%.
// 증거: mydocs/pr/assets/planet6897_clean19_20261110/pr8734/maintainer/paragraph-picture-successor/.
const EMPLOYMENT: &str = "tests/fixtures/pr8734/employment_service_interim_report.hwp";

fn employment_source_frame(
    change: impl FnOnce(&mut rhwp::model::image::Picture),
) -> (rhwp::document_core::DocumentCore, [usize; 2], usize) {
    let mut core = rhwp::document_core::DocumentCore::from_bytes(&bytes(EMPLOYMENT)).unwrap();
    let mut source = core.document().clone();
    let paragraphs = &source.sections[0].paragraphs;
    let host = paragraphs.iter().position(|paragraph| {
        paragraph.controls.iter().any(|control| matches!(control, Control::Picture(picture)
            if picture.common.text_wrap == TextWrap::Square
                && picture.caption.as_ref().is_some_and(|caption|
                    caption.paragraphs.iter().any(|paragraph| paragraph.text.contains("7대 기본원칙 모형")))))
    }).expect("독립 Print의 7대 기본원칙 그림");
    let owners = [
        "민간위탁기관은 고용서비스의 품질을 향상하기 위하여",
        "민간위탁기관의 고용서비스품질경영 7대 기본원칙은",
    ]
    .map(|prefix| {
        paragraphs
            .iter()
            .enumerate()
            .skip(host + 1)
            .find(|(_, p)| p.text.starts_with(prefix))
            .map(|(index, _)| index)
            .expect("해당 그림 뒤의 원본 문단")
    });
    let picture = source.sections[0].paragraphs[host]
        .controls
        .iter_mut()
        .find_map(|control| {
            if let Control::Picture(picture) = control {
                Some(picture)
            } else {
                None
            }
        })
        .unwrap();
    change(picture);
    core.set_document(source);
    (core, owners, host)
}

fn employment_nodes<'a>(
    node: &'a rhwp::renderer::render_tree::RenderNode,
    out: &mut Vec<&'a rhwp::renderer::render_tree::RenderNode>,
) {
    out.push(node);
    for child in &node.children {
        employment_nodes(child, out);
    }
}

#[test]
fn paragraph_picture_preserves_successor_source_rows_and_avoids_its_caption_band() {
    use rhwp::renderer::render_tree::RenderNodeType;
    let (core, owners, host) = employment_source_frame(|_| {});
    let tree = core.build_page_render_tree(78).expect("독립 Print 79쪽");
    let mut nodes = Vec::new();
    employment_nodes(&tree.root, &mut nodes);
    let image = nodes.iter().find(|node| matches!(&node.node_type, RenderNodeType::Image(image) if image.para_index == Some(host))).expect("같은 쪽의 원본 그림").bbox;
    let left = nodes
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::Column(_)))
        .unwrap()
        .bbox
        .x;
    for owner in owners {
        let source = &core.document().sections[0].paragraphs[owner];
        let actual: Vec<_> = nodes.iter().filter(|node| matches!(&node.node_type, RenderNodeType::TextLine(line) if line.para_index == Some(owner))).collect();
        assert_eq!(
            actual.len(),
            source.line_segs.len(),
            "그림 옆 저장 줄을 전폭으로 합치지 않는다: {}",
            source.text
        );
        for (line, stored) in actual.iter().zip(&source.line_segs) {
            assert!(
                (line.bbox.x - left - hu_to_px(stored.column_start)).abs() < 0.01,
                "저장 줄의 왼쪽 소속"
            );
            assert!(
                (line.bbox.width - hu_to_px(stored.segment_width)).abs() < 0.01,
                "실제 그림 옆 차선 폭"
            );
            assert!(
                line.bbox.x + line.bbox.width <= image.x,
                "그림 뒤에 본문이 가려지지 않는다"
            );
        }
        let rendered: String = nodes
            .iter()
            .filter_map(|node| match &node.node_type {
                RenderNodeType::TextRun(run) if run.para_index == Some(owner) => {
                    Some(run.text.as_str())
                }
                _ => None,
            })
            .flat_map(str::chars)
            .filter(|c| !c.is_whitespace())
            .collect();
        let expected: String = source.text.chars().filter(|c| !c.is_whitespace()).collect();
        assert_eq!(rendered, expected, "차선 보존 중 본문 누락·중복이 없다");
    }
}

fn paragraph_picture_without_current_frame_reflows(
    change: impl FnOnce(&mut rhwp::model::image::Picture),
) {
    use rhwp::renderer::render_tree::RenderNodeType;
    let (core, owners, _) = employment_source_frame(change);
    let tree = core.build_page_render_tree(78).unwrap();
    let mut nodes = Vec::new();
    employment_nodes(&tree.root, &mut nodes);
    for owner in owners {
        let lines:Vec<_>=nodes.iter().filter(|node| matches!(&node.node_type, RenderNodeType::TextLine(line) if line.para_index==Some(owner))).collect();
        assert!(
            lines.len()
                < core.document().sections[0].paragraphs[owner]
                    .line_segs
                    .len(),
            "실제 그림 배제가 없는 곳의 낡은 좁은 줄은 일반 흐름으로 재구성한다"
        );
    }
}

#[test]
fn paragraph_picture_on_another_vertical_band_does_not_own_successor_rows() {
    paragraph_picture_without_current_frame_reflows(|picture| {
        picture.common.vertical_offset += 35000
    });
}

#[test]
fn paragraph_picture_outside_the_column_does_not_own_successor_rows() {
    paragraph_picture_without_current_frame_reflows(|picture| {
        picture.common.horizontal_offset += 42520
    });
}
