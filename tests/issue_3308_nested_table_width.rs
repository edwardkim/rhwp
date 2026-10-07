//! Issue #3308 회귀 가드 — 좁은 중첩 표의 선언 폭과 셀 안쪽 가운데 배치.
//!
//! 한컴 2020으로 다시 출력한 9쪽 기준 PDF와 같은 원본을 사용한다.
//! `직인` 글자 원점은 오른쪽 셀의 안쪽 여백 때문에 셀 경계와 다르므로
//! 절대 x좌표 대신 포함 관계를 검사한다. 현재 head의 전쪽 시각 검증은 별도다.

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

#[test]
fn narrow_nested_table_keeps_declared_width_and_centers() {
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

    // 원본의 중첩 표는 부모 셀보다 확실히 좁으며, 셀 안에서 가운데 정렬된다.
    let width_ratio = nested.w / parent.w;
    assert!(
        (0.5..0.8).contains(&width_ratio),
        "중첩 표가 부모 폭으로 늘어났다: 폭 비율 {width_ratio:.3}"
    );
    let left_gap = nested.x - parent.x;
    let right_gap = parent.x + parent.w - nested.x - nested.w;
    assert!(
        (left_gap - right_gap).abs() <= parent.w * 0.02,
        "중첩 표가 셀 가운데를 벗어났다: 좌 {left_gap:.1}, 우 {right_gap:.1}"
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
