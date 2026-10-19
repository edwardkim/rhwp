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

/// 독립 한컴 Print의 2쪽 그림은 소유 줄·본문 상단을 공유하고 용지 상대 비율을 유지한다.
/// 전체 3쪽 Native/fresh WASM의 시각 gate와 확대 판독을 먼저 확인한다.
fn assert_picture_line_geometry(rel: &str) {
    use rhwp::model::{control::Control, shape::SizeCriterion};

    fn collect<'a>(node: &'a serde_json::Value, out: &mut Vec<&'a serde_json::Value>) {
        out.push(node);
        for child in node["children"].as_array().into_iter().flatten() {
            collect(child, out);
        }
    }
    fn image_attr(tag: &str, name: &str) -> f64 {
        tag.split(&format!("{name}=\""))
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(|| panic!("그림 속성 {name} 누락"))
    }
    fn close_ratio(actual: f64, expected: f64, tolerance: f64, relation: &str) {
        assert!(
            (actual - expected).abs() <= tolerance,
            "{relation}: 비율 {actual} != {expected}"
        );
    }

    let bytes =
        std::fs::read(sample(rel)).unwrap_or_else(|error| panic!("검증 입력 {rel}: {error}"));
    let source = rhwp::parser::parse_document(&bytes).expect("원본 파싱");
    let picture = match &source.sections[0].paragraphs[1].controls[0] {
        Control::Picture(picture) => picture,
        _ => panic!("원본 둘째 문단의 첫 개체는 그림이어야 한다"),
    };
    assert!(picture.common.treat_as_char);
    assert!(matches!(
        picture.common.width_criterion,
        SizeCriterion::Paper | SizeCriterion::Absolute
    ));
    assert_eq!(picture.common.height_criterion, SizeCriterion::Paper);
    let out_dir: PathBuf = std::env::temp_dir().join(format!(
        "rhwp-p122-relation-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    for command in ["export-svg", "export-render-tree"] {
        let out = Command::new(rhwp_bin())
            .args([
                command,
                &sample(rel),
                "-p",
                "1",
                "-o",
                &out_dir.to_string_lossy(),
            ])
            .output()
            .expect("2쪽 출력 실행");
        assert!(
            out.status.success(),
            "{command}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let files: Vec<_> = std::fs::read_dir(&out_dir)
        .unwrap()
        .map(|x| x.unwrap().path())
        .collect();
    let svg = std::fs::read_to_string(
        files
            .iter()
            .find(|p| p.extension().is_some_and(|e| e == "svg"))
            .expect("SVG 출력"),
    )
    .unwrap();
    let json = std::fs::read_to_string(
        files
            .iter()
            .find(|p| p.extension().is_some_and(|e| e == "json"))
            .expect("render tree 출력"),
    )
    .unwrap();
    std::fs::remove_dir_all(&out_dir).expect("검사 산출물 정리");
    let root: serde_json::Value = serde_json::from_str(&json).expect("render tree JSON");
    let mut nodes = Vec::new();
    collect(&root, &mut nodes);
    let images: Vec<_> = nodes
        .iter()
        .copied()
        .filter(|n| n["type"] == "Image")
        .collect();
    assert_eq!(images.len(), 1, "2쪽 그림은 누락·중복 없이 한 번 출력한다");
    assert_eq!(images[0]["pi"], 1, "그림의 문단 소유");
    assert_eq!(images[0]["ci"], 0, "그림의 개체 소유");
    let lines: Vec<_> = nodes
        .iter()
        .copied()
        .filter(|n| n["type"] == "TextLine" && n["pi"] == 1)
        .collect();
    assert_eq!(lines.len(), 1, "원본 그림 문단은 한 줄을 소유한다");
    assert!(
        lines[0]["children"]
            .as_array()
            .unwrap()
            .iter()
            .any(|n| n == images[0]),
        "그림은 자기 문단의 줄 안에 있어야 한다"
    );
    let body = nodes
        .iter()
        .copied()
        .find(|n| n["type"] == "Body")
        .expect("본문 영역");
    assert_eq!(
        lines[0]["bbox"]["y"], body["bbox"]["y"],
        "소유 줄과 본문 상단"
    );
    let image_tags: Vec<_> = svg.split("<image ").skip(1).collect();
    assert_eq!(image_tags.len(), 1, "최종 SVG에도 그림을 한 번 출력한다");
    let tag = image_tags[0];
    let svg_tag = svg.split('>').next().expect("SVG 루트");
    let paper_width = image_attr(svg_tag, "width");
    let paper_height = image_attr(svg_tag, "height");
    let page = &source.sections[0].section_def.page_def;
    // 정수 HWPUNIT 저장 기준선의 한 단위 오차만 무차원 비율로 허용한다.
    close_ratio(
        image_attr(tag, "y") / paper_height,
        f64::from(page.margin_top + page.margin_header) / f64::from(page.height),
        1.0 / f64::from(page.height),
        "그림과 본문 상단",
    );
    close_ratio(
        image_attr(tag, "x") / paper_width,
        f64::from(page.margin_left) / f64::from(page.width),
        1.0 / f64::from(page.width),
        "그림과 본문 시작",
    );
    close_ratio(
        image_attr(tag, "width") / paper_width,
        match picture.common.width_criterion {
            SizeCriterion::Paper => f64::from(picture.common.width) / 10000.0,
            SizeCriterion::Absolute => f64::from(picture.common.width) / f64::from(page.width),
            _ => unreachable!(),
        },
        f64::EPSILON * 8.0,
        "너비 기준과 용지의 비율",
    );
    close_ratio(
        image_attr(tag, "height") / paper_height,
        f64::from(picture.common.height) / 10000.0,
        f64::EPSILON * 8.0,
        "용지 상대 높이",
    );
}

#[test]
fn p122_second_page_picture_uses_paper_relative_size_at_body_top() {
    assert_picture_line_geometry("samples/p122.hwp");
}

/// 비율값과 현재 HWPUNIT가 다른 한컴 정상 저장본도 자기 줄의 원점을 유지한다.
#[test]
fn relative_picture_current_size_keeps_the_owned_line_origin() {
    for name in ["false", "true"] {
        assert_picture_line_geometry(&format!(
            "tests/fixtures/issue_7642/relative-quarter-affect-{name}-hancom-2020.hwp"
        ));
    }
}
