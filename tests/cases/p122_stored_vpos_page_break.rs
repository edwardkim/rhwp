//! [#5907] 문단 간 저장 vpos 되감김을 쪽 경계로 읽는다.
//!
//! `samples/p122.hwp` 는 본문 문단이 3개뿐인데 세 문단의 `PARA_LINE_SEG` 가 전부
//! `vertical_pos = 0` 이다. 앞 문단이 단 맨 위에서 시작해 0 보다 아래에서 끝났는데
//! 다음 문단이 다시 맨 위를 주장하므로, 한/글은 그 사이에서 쪽을 넘긴 것이다
//! (정본 `pdf/p122-2022.pdf` 3쪽, 문서에 저장된 `PrvImage` 1쪽 썸네일도 공백).
//!
//! 수정 전 rhwp 는 세 문단을 한 쪽에 쌓아 1쪽만 만들고 그림을 본문 상단이 아니라
//! 21.3px 아래에 그렸다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample(rel: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(rel)
        .to_string_lossy()
        .into_owned()
}

fn dump_pages(rel: &str) -> String {
    let out = Command::new(rhwp_bin())
        .args(["dump-pages", &sample(rel)])
        .output()
        .expect("dump-pages 실행 실패");
    assert!(out.status.success(), "dump-pages 실패: {rel}");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn page_headers(dump: &str) -> Vec<&str> {
    dump.lines()
        .filter(|line| line.starts_with("=== 페이지 "))
        .collect()
}

/// 세 문단이 각각 한 쪽씩 차지한다 — 한/글 정본과 같은 3쪽.
#[test]
fn p122_stored_vpos_reset_yields_three_pages() {
    let dump = dump_pages("samples/p122.hwp");
    let pages = page_headers(&dump);
    assert_eq!(
        pages.len(),
        3,
        "p122 는 한/글 정본과 같이 3쪽이어야 한다.\n{dump}"
    );
}

/// 그림은 2쪽에 홀로 놓인다 — 1쪽·3쪽은 빈 문단만.
#[test]
fn p122_picture_sits_alone_on_second_page() {
    let dump = dump_pages("samples/p122.hwp");
    let blocks: Vec<&str> = dump.split("=== 페이지 ").skip(1).collect();
    assert_eq!(blocks.len(), 3, "쪽 블록 3개가 필요하다.\n{dump}");
    assert!(
        !blocks[0].contains("Shape"),
        "1쪽에는 그림이 없어야 한다.\n{}",
        blocks[0]
    );
    assert!(
        blocks[1].contains("Shape") && blocks[1].contains("그림"),
        "2쪽에 그림이 있어야 한다.\n{}",
        blocks[1]
    );
    assert!(
        !blocks[2].contains("Shape"),
        "3쪽에는 그림이 없어야 한다.\n{}",
        blocks[2]
    );
}

/// 2쪽 SVG 의 그림은 본문 영역 맨 위(상단 여백 20mm + 머리말 15mm = 35mm)에 놓이고,
/// HWP5 `PAPER` 기준 크기(425.20% × 222.38%)를 유지한다.
#[test]
fn p122_second_page_picture_uses_paper_relative_size_at_body_top() {
    let out_dir: PathBuf = std::env::temp_dir().join(format!(
        "rhwp-p122-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let out = Command::new(rhwp_bin())
        .args([
            "export-svg",
            &sample("samples/p122.hwp"),
            "-p",
            "1",
            "-o",
            &out_dir.to_string_lossy(),
        ])
        .output()
        .expect("export-svg 실행 실패");
    assert!(out.status.success(), "export-svg 실패");

    let svg_path = std::fs::read_dir(&out_dir)
        .unwrap_or_else(|e| panic!("{}: {e}", out_dir.display()))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|ext| ext == "svg"))
        .expect("2쪽 SVG 가 생성되어야 한다");
    let svg = std::fs::read_to_string(&svg_path).expect("SVG 읽기 실패");
    let _ = std::fs::remove_dir_all(&out_dir);

    let image_tag = svg.split("<image ").nth(1).unwrap_or_else(|| {
        panic!(
            "2쪽에 <image> 가 있어야 한다:\n{}",
            &svg[..svg.len().min(600)]
        )
    });
    let y = image_tag
        .split("y=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .and_then(|v| v.parse::<f64>().ok())
        .expect("<image> 의 y 값을 읽어야 한다");
    let width = image_tag
        .split("width=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .and_then(|v| v.parse::<f64>().ok())
        .expect("<image> 의 width 값을 읽어야 한다");
    let height = image_tag
        .split("height=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .and_then(|v| v.parse::<f64>().ok())
        .expect("<image> 의 height 값을 읽어야 한다");

    // 35mm = 35 / 25.4 * 96 px = 132.28px (본문 영역 상단).
    let body_top_px = 35.0 / 25.4 * 96.0;
    assert!(
        (y - body_top_px).abs() < 1.0,
        "그림은 본문 상단({body_top_px:.2}px)에 놓여야 하는데 y={y}"
    );
    // A4 210×297mm @96dpi에서 425.20%×222.38%.
    let paper_width_px = 210.0 / 25.4 * 96.0;
    let paper_height_px = 297.0 / 25.4 * 96.0;
    let expected_width = paper_width_px * 4.252;
    let expected_height = paper_height_px * 2.2238;
    assert!(
        (width - expected_width).abs() < 2.0,
        "그림 너비는 PAPER 425.20%({expected_width:.2}px)여야 하는데 width={width}"
    );
    assert!(
        (height - expected_height).abs() < 2.0,
        "그림 높이는 PAPER 222.38%({expected_height:.2}px)여야 하는데 height={height}"
    );
}
