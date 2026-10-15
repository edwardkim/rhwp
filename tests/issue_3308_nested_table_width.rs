//! Issue #3308 회귀 가드 — 좁은 중첩 표의 선언 폭·원본 오프셋·셀 내부 포함 관계.
//!
//! 한컴 2020으로 다시 출력한 9쪽 기준 PDF와 같은 원본을 사용한다.
//! `직인` 글자 원점은 오른쪽 셀의 안쪽 여백 때문에 셀 경계와 다르므로
//! 절대 x좌표 대신 포함 관계를 검사한다. 현재 head의 전쪽 시각 검증은 별도다.

use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::shape::{HorzAlign, HorzRelTo};
use rhwp::model::table::Table;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

const FIXTURE: &str = "samples/task3307/issue3307_outline_number.hwpx";

#[derive(Clone, Copy)]
struct Box2d {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

fn bbox(node: &serde_json::Value) -> Box2d {
    let b = &node["bbox"];
    Box2d {
        x: b["x"].as_f64().expect("bbox.x"),
        y: b["y"].as_f64().expect("bbox.y"),
        w: b["w"].as_f64().expect("bbox.w"),
        h: b["h"].as_f64().expect("bbox.h"),
    }
}

fn seal_geometry<'a>(
    node: &'a serde_json::Value,
    ancestors: &mut Vec<&'a serde_json::Value>,
) -> Option<(Box2d, Box2d, Box2d, Box2d)> {
    if node["type"] == "TextRun" && node["text"].as_str().is_some_and(|s| s.starts_with("직인")) {
        let nested_index = ancestors.iter().rposition(|a| a["type"] == "Table")?;
        let outer_cell = ancestors[..nested_index]
            .iter()
            .rfind(|a| a["type"] == "Cell")?;
        let right_cell = ancestors[nested_index + 1..]
            .iter()
            .find(|a| a["type"] == "Cell")?;
        return Some((
            bbox(outer_cell),
            bbox(ancestors[nested_index]),
            bbox(right_cell),
            bbox(node),
        ));
    }
    ancestors.push(node);
    let found = node["children"].as_array().and_then(|children| {
        children
            .iter()
            .find_map(|child| seal_geometry(child, ancestors))
    });
    ancestors.pop();
    found
}

// 출력 좌표를 기대값으로 재사용하지 않고 원본의 폭·정렬·오프셋을 읽는다.
fn source_seal_placement(paragraphs: &[Paragraph]) -> Option<(f64, f64)> {
    paragraphs.iter().find_map(|paragraph| {
        paragraph.controls.iter().find_map(|control| {
            let Control::Table(table) = control else {
                return None;
            };
            source_seal_in_table(table)
        })
    })
}

fn source_seal_in_table(table: &Table) -> Option<(f64, f64)> {
    table.cells.iter().find_map(|parent| {
        for paragraph in &parent.paragraphs {
            for control in &paragraph.controls {
                let Control::Table(nested) = control else {
                    continue;
                };
                if nested.cells.iter().any(|cell| {
                    cell.paragraphs
                        .iter()
                        .any(|paragraph| paragraph.text.starts_with("직인"))
                }) {
                    assert!(!nested.common.treat_as_char, "원본 직인 표의 떠 있는 배치");
                    assert_eq!(nested.common.horz_align, HorzAlign::Left);
                    assert_eq!(nested.common.horz_rel_to, HorzRelTo::Column);
                    assert!(parent.width > 0, "원본 부모 셀 폭");
                    let padding = parent.effective_padding(&table.padding);
                    let offset = f64::from(nested.common.horizontal_offset as i32)
                        + f64::from(padding.left)
                        + f64::from(nested.outer_margin_left);
                    return Some((
                        f64::from(nested.common.width) / f64::from(parent.width),
                        offset / f64::from(parent.width),
                    ));
                }
            }
        }
        source_seal_placement(&parent.paragraphs)
    })
}

#[test]
fn narrow_nested_table_preserves_source_width_offset_and_containment() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("현재 시각")
        .as_nanos();
    let out_dir = PathBuf::from("output/pr-review/tests")
        .join(format!("issue3308-{}-{unique}", std::process::id()));
    std::fs::create_dir_all(&out_dir).expect("검사 출력 디렉터리");

    let status = Command::new(rhwp_bin())
        .args([
            "export-render-tree",
            FIXTURE,
            "-p",
            "6",
            "-o",
            out_dir.to_str().expect("UTF-8 경로"),
        ])
        .status()
        .expect("rhwp 실행");
    assert!(status.success(), "export-render-tree 실패");

    let tree: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(out_dir.join("render_tree_007.json")).expect("7쪽 렌더 트리"),
    )
    .expect("렌더 트리 JSON");
    let (parent, nested, right, seal) =
        seal_geometry(&tree, &mut Vec::new()).expect("직인 중첩 표 계층");

    let original = rhwp::parse_document(
        &std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(FIXTURE)).expect("원본 HWPX"),
    )
    .expect("원본 속성 파싱");
    let (source_width_ratio, source_offset_ratio) = original
        .sections
        .iter()
        .find_map(|section| source_seal_placement(&section.paragraphs))
        .expect("원본 직인 표의 폭과 왼쪽 오프셋");
    let width_ratio = nested.w / parent.w;
    assert!(
        (width_ratio - source_width_ratio).abs() <= 0.001,
        "중첩 표의 선언 폭 비율이 달라졌다: 실제 {width_ratio:.4}, 원본 {source_width_ratio:.4}"
    );
    let offset_ratio = (nested.x - parent.x) / parent.w;
    assert!(
        (offset_ratio - source_offset_ratio).abs() <= 0.001,
        "원본 왼쪽 기준 오프셋이 달라졌다: 실제 {offset_ratio:.4}, 원본 {source_offset_ratio:.4}"
    );
    assert!(
        nested.y >= parent.y && nested.y + nested.h <= parent.y + parent.h,
        "중첩 표가 부모 셀을 벗어났다"
    );
    // 렌더 트리의 bbox는 소수 첫째 자리로 반올림되므로 맞닿은 끝점에
    // 표 폭의 0.1%만 허용한다. 셀 위치를 고정 픽셀로 못박지 않는다.
    let rounded_edge = nested.w * 0.001;
    assert!(
        right.x >= nested.x - rounded_edge
            && right.x + right.w <= nested.x + nested.w + rounded_edge,
        "직인 셀이 중첩 표를 벗어났다"
    );
    assert!(
        seal.x >= right.x
            && seal.x + seal.w <= right.x + right.w
            && seal.y >= right.y
            && seal.y + seal.h <= right.y + right.h,
        "직인 글자가 오른쪽 셀 내부에 있지 않다"
    );

    std::fs::remove_dir_all(out_dir).expect("검사 산출물 정리");
}
