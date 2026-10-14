//! #7902: 원본 표의 분할 내용과 이어받기 뒤 그림을 함께 보호한다.
//!
//! 독립 Print PDF와 Native/fresh WASM 전쪽 증거는
//! mydocs/working/task_m100_7902_stage2.md의 candidate-a5-leading-band-dev에 있다.
//! overfill 43·44쪽과 nested HWP/HWPX 104쪽은 양쪽 모두 98% 이상이다.
//! 위치 검사와 별도로 원본 셀의 내용 보존·소유·정렬 관계를 검사한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::{control::Control, paragraph::Paragraph, table::Table};
use serde_json::Value;
use std::{path::Path, process::Command, sync::OnceLock};

const OVERFILL: &str = "samples/table_giant_cell_overfill.hwpx";
const NESTED_HWP: &str = "samples/issue1949_giant_cell_nested_tables_perf.hwp";
const NESTED_HWPX: &str = "samples/issue1949_giant_cell_nested_tables_perf.hwpx";

fn hft_number_baseline(sample: &str) {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory = Trees(
        std::env::temp_dir().join(format!("rhwp-7902-number-{}-{stamp}", std::process::id())),
    );
    std::fs::create_dir(&directory.0).unwrap();
    let bin = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
    let output = Command::new(bin)
        .arg("export-svg")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join(sample))
        .args(["--compat", "2022", "--profile", "print", "-p", "28", "-o"])
        .arg(&directory.0)
        .output()
        .expect("29쪽의 최종 글리프 출력");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let files: Vec<_> = std::fs::read_dir(&directory.0)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "svg"))
        .collect();
    assert_eq!(files.len(), 1, "29쪽 하나만 내보낸다");
    let svg = std::fs::read_to_string(&files[0]).unwrap();
    let xml = roxmltree::Document::parse(&svg).unwrap();
    let glyphs: Vec<_> = xml
        .descendants()
        .filter(|node| node.has_tag_name("text"))
        .collect();
    // 독립 Print의 이 쪽은 본문 없이 '- 7 -'만 있다. 빈 본문 때문에
    // 번호까지 빠지거나 중복되는 것을 위치 검사와 별도로 검출한다.
    assert_eq!(
        glyphs
            .iter()
            .filter_map(|node| node.text())
            .collect::<String>(),
        "-7-"
    );
    for glyph in glyphs {
        let y: f64 = glyph.attribute("y").unwrap().parse().unwrap();
        // Print PDF의 실제 797.039978pt 원점. Native98.47328%, WASM100%
        // 페이지에 한해 허용되는 ±2px이며 구현의 descent 값을 재인용하지 않는다.
        assert!((y - 1062.719971).abs() <= 2.0, "{sample}: 기준선 {y}");
        assert_eq!(glyph.attribute("font-size"), Some("16"), "원본12pt");
        let bold = glyph.attribute("font-weight") == Some("bold")
            || glyph
                .attribute("stroke-width")
                .is_some_and(|width| width.parse::<f64>().unwrap() > 0.0);
        assert!(bold, "원본 쪽 번호 스타일의 진하게 속성");
    }
}

#[test]
fn hwp_generated_hft_number_keeps_print_baseline_and_content() {
    hft_number_baseline("samples/issue6782/1480000-201900042-chemical-product-labeling-study.hwp");
}

#[test]
fn hwpx_generated_hft_number_keeps_print_baseline_and_content() {
    hft_number_baseline("tests/fixtures/pr7704_manual_reflow/6761-control.hwpx");
}

#[cfg(feature = "native-skia")]
#[test]
fn native_generated_hft_font_trace_uses_paint_aliases() {
    use rhwp::{document_core::DocumentCore, renderer::skia::SkiaLayerRenderer};

    for sample in [
        "samples/issue6782/1480000-201900042-chemical-product-labeling-study.hwp",
        "tests/fixtures/pr7704_manual_reflow/6761-control.hwpx",
    ] {
        let bytes = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(sample)).unwrap();
        let core = DocumentCore::from_bytes(&bytes).unwrap();
        let trace: Value = serde_json::from_str(
            &SkiaLayerRenderer::new()
                .get_font_decision_trace(&core, 28, r#"{"maxCharacters":16}"#)
                .unwrap(),
        )
        .unwrap();
        let records = trace["records"].as_array().unwrap();
        assert!(!records.is_empty(), "생성 번호도 진단 대상이다");
        for record in records {
            let native = &record["paint"]["native"];
            // 한양중고딕의 설치 TTF 이름은 HY중고딕이다. 실제 paint가
            // 이 별칭을 찾는데 진단만 generic 후보를 바로 고르면 안 된다.
            let candidates = native["candidates"].as_array().unwrap();
            assert_eq!(candidates[0], "한양중고딕", "원 face 우선");
            assert_eq!(
                candidates[1], "HY중고딕",
                "generic보다 원 face의 설치 별칭 우선"
            );
        }
    }
}

struct Trees(std::path::PathBuf);

impl Drop for Trees {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn pages(sample: &str) -> &'static Vec<Value> {
    static OVERFILL_PAGES: OnceLock<Vec<Value>> = OnceLock::new();
    static HWP_PAGES: OnceLock<Vec<Value>> = OnceLock::new();
    static HWPX_PAGES: OnceLock<Vec<Value>> = OnceLock::new();
    let cache = match sample {
        OVERFILL => &OVERFILL_PAGES,
        NESTED_HWP => &HWP_PAGES,
        NESTED_HWPX => &HWPX_PAGES,
        _ => panic!("등록하지 않은 원본: {sample}"),
    };
    cache.get_or_init(|| {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory =
            Trees(std::env::temp_dir().join(format!("rhwp-7902-{}-{stamp}", std::process::id())));
        std::fs::create_dir(&directory.0).expect("검사 전용 임시 경로");
        let bin = std::env::var("CARGO_BIN_EXE_rhwp")
            .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
        let output = Command::new(bin)
            .arg("export-render-tree")
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join(sample))
            .args(["--compat", "2022", "-o"])
            .arg(&directory.0)
            .output()
            .expect("원본 최종 렌더 트리");
        assert!(
            output.status.success(),
            "{sample}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let mut files: Vec<_> = std::fs::read_dir(&directory.0)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
            .collect();
        files.sort();
        files
            .iter()
            .map(|path| serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap())
            .collect()
    })
}

fn nodes(node: &Value) -> Vec<&Value> {
    let mut result = vec![node];
    if let Some(children) = node["children"].as_array() {
        for child in children {
            result.extend(nodes(child));
        }
    }
    result
}

fn compact(text: &str) -> String {
    text.chars()
        .filter(|ch| !ch.is_whitespace() && !ch.is_control() && *ch != '\u{fffc}')
        .collect()
}

fn text(node: &Value) -> String {
    nodes(node)
        .iter()
        .filter_map(|node| node["text"].as_str())
        .collect()
}

fn cell(page: &Value, row: u16, col: u16) -> &Value {
    let matches: Vec<_> = nodes(page)
        .into_iter()
        .filter(|node| node["type"] == "Cell" && node["row"] == row && node["col"] == col)
        .collect();
    assert_eq!(matches.len(), 1, "원본 셀 ({row},{col})의 단일 쪽 소유");
    matches[0]
}

fn top(node: &Value) -> f64 {
    node["bbox"]["y"].as_f64().unwrap()
}

fn bottom(node: &Value) -> f64 {
    top(node) + node["bbox"]["h"].as_f64().unwrap()
}

fn source_table(paragraphs: &[Paragraph]) -> Option<&Table> {
    for paragraph in paragraphs {
        for control in &paragraph.controls {
            if let Control::Table(table) = control {
                if table.cells.iter().any(|cell| {
                    cell.row == 21
                        && cell.col == 8
                        && cell.paragraphs.iter().any(|p| p.text.contains("기인"))
                }) {
                    return Some(table);
                }
                for cell in &table.cells {
                    if let Some(found) = source_table(&cell.paragraphs) {
                        return Some(found);
                    }
                }
            }
        }
    }
    None
}

#[test]
fn original_cells_survive_the_print_page_cuts_without_loss_or_duplication() {
    let bytes = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(OVERFILL)).unwrap();
    let document = rhwp::parser::parse_document(&bytes).unwrap();
    let source = document
        .sections
        .iter()
        .find_map(|section| source_table(&section.paragraphs))
        .expect("원본 재시작을 가진 표");
    let actual = pages(OVERFILL);
    assert_eq!(actual.len(), 48, "독립 Print PDF와 전체 쪽수 일치");
    for (row, col, first, last, prefix_end, suffix_start) in [
        (21, 8, 43, 44, "비율의변화에기인", "한다."),
        (24, 6, 44, 45, "감소될수있", "다."),
    ] {
        let original = source.cell_at(row, col).expect("원본 셀");
        let expected = compact(
            &original
                .paragraphs
                .iter()
                .map(|paragraph| paragraph.text.as_str())
                .collect::<String>(),
        );
        let prefix = compact(&text(cell(&actual[first - 1], row, col)));
        let suffix = compact(&text(cell(&actual[last - 1], row, col)));
        assert!(prefix.ends_with(prefix_end), "앞 조각의 원본 내용 컷");
        assert!(suffix.starts_with(suffix_start), "뒤 조각의 이어받기 컷");
        assert_eq!(format!("{prefix}{suffix}"), expected, "누락·중복·순서 변경");
    }
}

#[test]
fn closed_source_frames_keep_center_alignment_instead_of_extra_line_spacing() {
    let actual = pages(OVERFILL);
    // Print PDF43쪽 마지막 가로 괘선: 790.414978pt → 1053.886637px.
    // 선언보다 큰 원본 잉크 끝 이후에 마지막 줄간격까지 더한 프레임을 검출한다.
    let first_frame = cell(&actual[42], 21, 8);
    assert!(
        (bottom(first_frame) - 1_053.886_637_369_791_7).abs() <= 2.0,
        "43쪽 첫 프레임에 마지막 줄간격을 중복 예약함: {}",
        bottom(first_frame)
    );
    // 첫 프레임의 잉크가 선언 높이보다 큰 셀과 선언 높이에 여유가 있는 대조군이다.
    // 마지막 줄간격을 다시 예약하거나 정렬을 Top으로 덮으면 대칭성이 깨진다.
    for (page, row, col) in [(43, 21, 6), (43, 21, 8), (44, 24, 6)] {
        let frame = cell(&actual[page - 1], row, col);
        let lines: Vec<_> = nodes(frame)
            .into_iter()
            .filter(|node| node["type"] == "TextLine" && !text(node).trim().is_empty())
            .collect();
        assert!(lines.len() > 1, "한 줄짜리 대조군으로 대신하지 않음");
        let upper = top(lines[0]) - top(frame);
        let lower = bottom(frame) - bottom(lines[lines.len() - 1]);
        assert!(upper >= 0.0 && lower >= 0.0, "선택 줄의 셀 내부 포함");
        assert!(
            (upper - lower).abs() <= 0.3,
            "{page}쪽 ({row},{col}) 원본 가운데 정렬: 위 {upper}, 아래 {lower}"
        );
    }
}

fn assert_following_image(sample: &str) {
    let actual = pages(sample);
    assert_eq!(actual.len(), 115, "독립 Print PDF 전체 쪽수");
    let page = &actual[103];
    let all = nodes(page);
    let images: Vec<_> = all
        .iter()
        .copied()
        .filter(|n| n["type"] == "Image")
        .collect();
    assert_eq!(images.len(), 1, "104쪽 후속 그림의 단일 소유");
    let image = images[0];
    let tables: Vec<_> = all
        .iter()
        .copied()
        .filter(|node| node["type"] == "Table" && node["rows"] == 14 && node["cols"] == 2)
        .collect();
    assert_eq!(tables.len(), 1, "앞 조각의 14×2 중첩 표");
    // 두 독립 Print PDF의 동일한 이어받기 표 위끝: 62.211975pt.
    assert!(
        (top(tables[0]) - 82.949_300_130_208_33).abs() <= 2.0,
        "{sample}: 제목행 반복 여부와 무관하게 원본 위 여백을 보존해야 함"
    );
    assert!(
        bottom(tables[0]) <= top(image),
        "그림은 이어받은 표 뒤에 배치"
    );
    // 독립 Print PDF 104쪽 그림 bbox의 위끝(72pt→96dpi 환산)이다.
    // 두 형식의 해당 쪽은 Native/fresh WASM 모두 98% 이상이며 직접 판독했다.
    // 중복 아래 여백이 있던 516.7px를 허용하지 않고 ±2px 범위를 유지한다.
    assert!(
        (top(image) - 514.157_333_333_333_3).abs() <= 2.0,
        "{sample}: 원본 아래 여백을 한 번만 소비한 후속 그림, 실제 {}",
        top(image)
    );
}

#[test]
fn hwp_nested_continuation_does_not_charge_bottom_margin_twice() {
    assert_following_image(NESTED_HWP);
}

#[test]
fn hwpx_nested_continuation_does_not_charge_bottom_margin_twice() {
    assert_following_image(NESTED_HWPX);
}
