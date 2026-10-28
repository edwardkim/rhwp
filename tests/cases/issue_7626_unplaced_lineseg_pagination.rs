//! #7626: 폭·원점이 없는 HWPX 줄 캐시는 실제 본문/표의 쪽 예산이 아니다.
//! 기대 쪽 소속과 줄 진행은 동일 원본의 한컴 Print PDF와 한컴 재저장 HWP에서 온다.
//! Native/fresh WASM 전체 2쪽 Sweep 최저 98.71% 확인 후 추가했다.
#![cfg(not(target_arch = "wasm32"))]

use serde_json::Value;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static SEQUENCE: AtomicUsize = AtomicUsize::new(0);

fn pages(sample: &str) -> Vec<Value> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let out = std::env::temp_dir().join(format!(
        "rhwp-7626-{}-{}",
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&out).unwrap();
    // nextest archive에서는 실행 호스트가 주입한 바이너리를 우선 사용한다.
    let binary =
        std::env::var_os("CARGO_BIN_EXE_rhwp").unwrap_or_else(|| env!("CARGO_BIN_EXE_rhwp").into());
    let output = Command::new(binary)
        .args(["export-render-tree", sample, "-o"])
        .arg(&out)
        .current_dir(root)
        .output()
        .expect("rhwp export-render-tree");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut paths = std::fs::read_dir(&out)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect::<Vec<_>>();
    paths.sort();
    let pages = paths
        .iter()
        .map(|path| serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap())
        .collect();
    std::fs::remove_dir_all(&out).unwrap();
    pages
}

fn body_nodes<'a>(node: &'a Value, out: &mut Vec<&'a Value>) {
    match node["type"].as_str() {
        Some("Table") => {
            out.push(node);
            return;
        }
        Some("TextLine") => out.push(node),
        _ => {}
    }
    if let Some(children) = node["children"].as_array() {
        for child in children {
            body_nodes(child, out);
        }
    }
}

fn body(page: &Value) -> &Value {
    page["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["type"] == "Body")
        .unwrap()
}

fn lines(page: &Value) -> Vec<&Value> {
    let mut result = Vec::new();
    body_nodes(body(page), &mut result);
    result
}

fn coordinate(node: &Value, name: &str) -> f64 {
    node["bbox"][name].as_f64().unwrap()
}

#[test]
fn issue_7626_keeps_every_body_paragraph_on_its_hancom_page() {
    let pages = pages("samples/issue7626/sample-document.hwpx");
    assert_eq!(pages.len(), 2, "한컴 Print와 같은 2쪽 — 수정 전 1쪽");
    let mut seen = Vec::new();
    for (index, page) in pages.iter().enumerate() {
        let nodes = lines(page);
        let bottom = coordinate(body(page), "y") + coordinate(body(page), "h");
        for node in nodes {
            // 빈 표 host의 TextLine은 기본 pi를 가질 수 있다. 실제 TextRun의
            // 소유자를 소비하고 표와 그 host가 같은 문단인 것은 중복 본문으로 세지 않는다.
            let paragraph = node["children"]
                .as_array()
                .and_then(|children| children.iter().find(|child| child["type"] == "TextRun"))
                .map_or(&node["pi"], |run| &run["pi"])
                .as_u64()
                .unwrap() as usize;
            assert_eq!(
                usize::from(paragraph >= 28),
                index,
                "pi{paragraph}의 쪽 소속"
            );
            assert!(
                coordinate(node, "y") + coordinate(node, "h") <= bottom + 0.1,
                "pi{paragraph}가 본문 밖으로 넘침"
            );
            seen.push(paragraph);
        }
    }
    seen.dedup();
    assert_eq!(
        seen,
        (0..41).collect::<Vec<_>>(),
        "본문/표 소유 순서·누락·중복"
    );
    fn text(node: &Value, out: &mut String) {
        if node["type"] == "TextRun" {
            out.push_str(node["text"].as_str().unwrap());
        }
        if let Some(children) = node["children"].as_array() {
            for child in children {
                text(child, out);
            }
        }
    }
    let mut rendered = String::new();
    for page in &pages {
        text(page, &mut rendered);
    }
    for section in 1..=4 {
        for part in ["heading", "point", "detail", "sub", "note"] {
            let token = format!("{{{{s{section}_{part}}}}}");
            assert_eq!(rendered.matches(&token).count(), 1, "{token} 누락/중복");
        }
    }
    for name in [
        "title",
        "summary",
        "table_title",
        "table_unit",
        "table_note",
        "th1",
        "th2",
        "th3",
        "td1",
        "td2",
        "td3",
    ] {
        let token = format!("{{{{{name}}}}}");
        assert_eq!(rendered.matches(&token).count(), 1, "{token} 누락/중복");
    }
}

#[test]
fn issue_7626_reserves_the_empty_end_run_before_the_table() {
    for (sample, line_count) in [
        ("samples/issue7626/sample-document.hwpx", 1),
        ("samples/issue7626/end-run-two-lines.hwpx", 2),
    ] {
        let pages = pages(sample);
        assert_eq!(pages.len(), 2, "{sample}");
        let nodes = lines(&pages[1]);
        let find = |pi| *nodes.iter().find(|node| node["pi"] == pi).unwrap();
        let units = nodes
            .iter()
            .copied()
            .filter(|node| node["type"] == "TextLine" && node["pi"] == 30)
            .collect::<Vec<_>>();
        assert_eq!(units.len(), line_count, "{sample}: 표 앞 줄 누락/중복");
        let unit = *units.last().unwrap();
        let table = find(31);
        let note = find(32);
        // 한컴 재저장 마지막 줄: th=1200HU, 다음 표까지 진행=1920HU.
        // 두 줄 대조군의 앞줄은 th=1000HU, 다음 줄까지 진행=1600HU다.
        // 문단 끝의 1200HU 글자 상자를 모든 줄에 적용하면 앞줄 진행이 틀린다.
        if line_count == 2 {
            assert!(
                (coordinate(units[1], "y") - coordinate(units[0], "y") - 1600.0 * 96.0 / 7200.0)
                    .abs()
                    <= 0.1,
                "{sample}: 끝 run 크기는 마지막 물리 줄에만 적용",
            );
        }
        assert!((coordinate(unit, "h") - 1200.0 * 96.0 / 7200.0).abs() <= 0.1);
        assert!(
            (coordinate(table, "y") - coordinate(unit, "y") - 1920.0 * 96.0 / 7200.0).abs() <= 0.1
        );
        assert!(coordinate(unit, "y") + coordinate(unit, "h") < coordinate(table, "y"));
        assert!(coordinate(table, "y") + coordinate(table, "h") < coordinate(note, "y"));
    }
}

#[test]
fn issue_7626_preserves_placed_hancom_rows_even_with_a_widthless_table_host() {
    for sample in [
        "samples/issue7626/hancom-resaved.hwp",
        "samples/issue7626/end-run-two-lines.hwp",
    ] {
        let pages = pages(sample);
        assert_eq!(pages.len(), 2, "{sample}");
        let nodes = lines(&pages[1]);
        let find = |pi| *nodes.iter().find(|node| node["pi"] == pi).unwrap();
        // 한컴 저장 vpos p29=2080, p30=4000: 유효 저장 높이 사다리를 보존한다.
        assert!(
            (coordinate(find(30), "y") - coordinate(find(29), "y") - 1920.0 * 96.0 / 7200.0).abs()
                <= 0.1,
            "{sample}: 저장 줄 진행",
        );
        assert_eq!(find(31)["type"], "Table");
    }
}
