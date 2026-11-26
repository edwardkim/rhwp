//! 관계형·고정형: 원 문단 여백과 독립 Print의 옛한글·후속 글자 원점을 보호한다.
//! 주석 갱신: 2026-11-22T07:56:56+09:00.
//! 2026-11-22, PDFium 96dpi, 색상 경계 보정 전 2px raw 실루엣으로 원본 전24쪽 측정.
//! 1쪽 Native 99.81% / fresh WASM 99.72%, 2쪽 양쪽 99.99%; 전쪽 최저 99.50% / 99.48%.
//! 근거: mydocs/pr/assets/issues7754_7765/source36/native/issue7754-plain/silhouette.tsv,
//! mydocs/pr/assets/issues7754_7765/source36/wasm/issue7754-plain/silhouette.tsv,
//! mydocs/pr/assets/issues7754_7765/source36/independent-pdf-coordinates.json.
//! 1·2쪽의 원본·독립 PDF 확대 판독은 source35/direct-review/와 새 PNG 동일성 증거로 연결한다.
//! 좌표는 독립 PDF p2 rawdict의 Type3 원점·후속 글자 원점이며 96/72로 환산해 ±2px만 허용한다.
//! 회귀 실행은 빈 명시 FontEnvironment로 저장 HFT 메트릭을 보존한다. 설치·비공개 글꼴을 요구하지 않는다.
//! 실제 glyph 윤곽·분할 공급은 기존 Native issue_8153_print_glyph_geometry의 합성 관계 검사로 별도 보호한다.
#![cfg(not(target_arch = "wasm32"))]

use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};
struct OutputDir(PathBuf);
impl Drop for OutputDir {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
fn render(command: &str, page: usize) -> OutputDir {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dir = OutputDir(root.join("output").join(format!(
        "test-7754-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    std::fs::create_dir_all(&dir.0).unwrap();
    let environment = dir.0.join("font-environment.json");
    std::fs::write(&environment,r#"{"id":"issue7754-minimum-font-contract","substitutions":{},"preserveSourceMetrics":true}"#).unwrap();
    let bin = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
    let mut cmd = Command::new(bin);
    cmd.arg(command)
        .arg(root.join("samples/HWP5-nopassword-123456.hwpx"))
        .args(["--compat", "2024", "--font-environment"])
        .arg(environment)
        .arg("-p")
        .arg(page.to_string())
        .arg("-o")
        .arg(&dir.0);
    if command == "export-svg" {
        cmd.args(["--profile", "print"]);
    }
    let out = cmd.output().unwrap();
    assert!(
        out.status.success(),
        "actual CLI output failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    dir
}
fn walk<'a>(node: &'a Value, out: &mut Vec<&'a Value>) {
    out.push(node);
    if let Some(c) = node.get("children").and_then(Value::as_array) {
        for child in c {
            walk(child, out);
        }
    }
}
#[test]
fn stored_full_paragraph_frame_does_not_apply_margin_twice() {
    let dir = render("export-render-tree", 0);
    let root: Value =
        serde_json::from_slice(&std::fs::read(dir.0.join("render_tree_001.json")).unwrap())
            .unwrap();
    let mut nodes = Vec::new();
    walk(&root, &mut nodes);
    let lines = |para| {
        nodes
            .iter()
            .copied()
            .filter(|n| {
                n["type"] == "TextLine"
                    && n["pi"] == para
                    && n["children"].as_array().is_some_and(|cs| {
                        cs.iter().any(|c| c["type"] == "TextRun" && c["pi"] == para)
                    })
            })
            .collect::<Vec<_>>()
    };
    let control = lines(6);
    let target = lines(8);
    assert!(!control.is_empty());
    // The original saved paragraph and independent Print p1 both contain four rows.
    assert_eq!(
        target.len(),
        4,
        "preserve actual original/Print line ownership"
    );
    for n in &target {
        assert_eq!(
            n["bbox"]["x"], control[0]["bbox"]["x"],
            "same complete paragraph origin; no double margin"
        );
        assert_eq!(
            n["bbox"]["w"], control[0]["bbox"]["w"],
            "same complete paragraph width"
        );
    }
    for pair in target.windows(2) {
        assert!(
            pair[0]["bbox"]["y"].as_f64().unwrap() + pair[0]["bbox"]["h"].as_f64().unwrap()
                <= pair[1]["bbox"]["y"].as_f64().unwrap(),
            "ordered nonoverlapping original rows"
        );
    }
    let text = target
        .iter()
        .flat_map(|l| l["children"].as_array().unwrap())
        .filter_map(|n| n["text"].as_str())
        .collect::<String>();
    let mut end = 0;
    for token in [
        "아래의 차례",
        "\u{f03a0}",
        "도구 상자",
        "하이퍼텍스트 이전 위치로",
        "입력-하이퍼텍스트-이전 위치로",
        "다시 차례로 되돌아옵니다.",
    ] {
        assert_eq!(
            text.matches(token).count(),
            1,
            "content occurs once: {token}"
        );
        let at = text.find(token).unwrap();
        assert!(at >= end, "original content order: {token}");
        end = at + token.len();
    }
}
#[test]
fn old_hangul_origins_match_independent_print_on_best_qualified_page() {
    let dir = render("export-svg", 1);
    let path = std::fs::read_dir(&dir.0)
        .unwrap()
        .map(|x| x.unwrap().path())
        .find(|p| p.extension().is_some_and(|e| e == "svg"))
        .unwrap();
    let svg = std::fs::read_to_string(path).unwrap();
    let xml = roxmltree::Document::parse(&svg).unwrap();
    let text: Vec<_> = xml
        .descendants()
        .filter(|n| n.has_tag_name("text"))
        .collect();
    let hits: Vec<_> = text
        .iter()
        .enumerate()
        .filter(|(_, n)| n.text() == Some("ᄒᆞᆫ"))
        .collect();
    assert_eq!(
        hits.len(),
        3,
        "each original Jamo heading occurs once without source rewriting"
    );
    // Independent PDF p2 rawdict F53A origins: (172.919998,115.079994),
    // (127.919998,264.479980), (127.919998,301.919983) pt, transformed by 96/72.
    for ((index, node), (x, y)) in hits.iter().zip([
        (230.559_998, 153.439_992),
        (170.559_998, 352.639_974),
        (170.559_998, 402.559_977),
    ]) {
        let coordinate =
            |n: roxmltree::Node<'_, '_>, name| n.attribute(name).unwrap().parse::<f64>().unwrap();
        let actual_x = coordinate(**node, "x");
        let baseline = coordinate(**node, "y");
        assert!(
            (actual_x - x).abs() <= 2.0,
            "independent old-Hangul origin: {actual_x} vs {x}"
        );
        assert!(
            (baseline - y).abs() <= 2.0,
            "independent Print baseline: {baseline} vs {y}"
        );
        let next = text[index + 1];
        assert_eq!(next.text(), Some("글"), "source sequence after old-Hangul");
        let following = coordinate(next, "x");
        assert!(
            (following - (x + 16.0)).abs() <= 2.0,
            "following 글 has its independent Print origin"
        );
        assert_eq!(
            coordinate(next, "y"),
            baseline,
            "same original line baseline"
        );
        assert!(following > actual_x, "cluster advances once");
    }
}
