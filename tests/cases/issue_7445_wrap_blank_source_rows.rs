//! 탭만 있는 어울림 문단도 원본 저장 줄과 실제 본문 흐름의 소유를 보존한다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{model::control::Control, DocumentCore};
use serde_json::Value;
use std::{path::Path, process::Command};
fn walk<'a>(n: &'a Value, out: &mut Vec<&'a Value>) {
    out.push(n);
    if let Some(cs) = n["children"].as_array() {
        for c in cs {
            walk(c, out);
        }
    }
}
fn text(n: &Value) -> String {
    n["children"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|x| x["text"].as_str())
        .collect()
}
fn top(n: &Value) -> f64 {
    n["bbox"]["y"].as_f64().unwrap()
}
#[test]
fn closed_tab_rows_keep_their_space_before_the_following_note() {
    let input = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/task1725/text_footnote_tail_overpagination.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(&input).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(
        core.page_count(),
        242,
        "같은 원본의 독립 Print 쪽수와 문단 소유를 함께 확인한다"
    );
    let paragraphs = &core.document().sections[0].paragraphs;
    let owners: Vec<_> = paragraphs
        .windows(3)
        .enumerate()
        .filter(|(_, p)| {
            p[0].text.contains("Journey basis")
                && p[1].text.chars().all(char::is_whitespace)
                && p[2].text.starts_with("주)")
        })
        .collect();
    assert_eq!(
        owners.len(),
        1,
        "원문 호스트·탭 문단·다음 주석 소유를 유일하게 식별한다"
    );
    let (pi, source) = owners[0];
    assert!(source[0]
        .controls
        .iter()
        .any(|c| matches!(c,Control::Table(t)if !t.common.treat_as_char)));
    assert!(source[1].controls.is_empty());
    assert_eq!(source[1].text, "\t\t\t");
    let stored: Vec<_> = source[0]
        .line_segs
        .last()
        .into_iter()
        .chain(source[1].line_segs.iter())
        .chain(source[2].line_segs.first())
        .collect();
    assert!(
        stored
            .windows(2)
            .all(|p| p[0].vertical_pos + p[0].line_height + p[0].line_spacing == p[1].vertical_pos),
        "원시 저장 사다리의 앞뒤·내부 간격이 닫힌다"
    );
    struct Directory(std::path::PathBuf);
    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let out = Directory(
        std::env::temp_dir().join(format!("rhwp-wrap-space-{}-{stamp}", std::process::id())),
    );
    std::fs::create_dir(&out.0).unwrap();
    let bin = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
    let r = Command::new(bin)
        .arg("export-render-tree")
        .arg(input)
        .args(["--compat", "2024", "-p", "200", "-o"])
        .arg(&out.0)
        .output()
        .unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let tree: Value =
        serde_json::from_slice(&std::fs::read(out.0.join("render_tree_201.json")).unwrap())
            .unwrap();
    let mut nodes = Vec::new();
    walk(&tree, &mut nodes);
    let lines = |owner: usize| {
        nodes
            .iter()
            .copied()
            .filter(|n| n["type"] == "TextLine" && n["pi"].as_u64() == Some(owner as u64))
            .collect::<Vec<_>>()
    };
    let before = lines(pi);
    let blank = lines(pi + 1);
    let after = lines(pi + 2);
    assert_eq!(
        before.len(),
        1,
        "이 쪽은 호스트의 마지막 줄을 한 번 소유한다"
    );
    assert_eq!(
        blank.len(),
        source[1].line_segs.len(),
        "보이는 글자가 없어도 저장 탭 줄을 누락·재생하지 않는다"
    );
    assert_eq!(
        after.len(),
        source[2].line_segs.len(),
        "이어지는 주석 줄을 누락·재생하지 않는다"
    );
    let placed: Vec<_> = before
        .iter()
        .copied()
        .chain(blank.iter().copied())
        .chain(after.first().copied())
        .collect();
    assert_eq!(placed.len(), stored.len());
    for pair in placed.windows(2).zip(stored.windows(2)) {
        let (actual, expected) = pair;
        let delta = f64::from(expected[1].vertical_pos - expected[0].vertical_pos) / 75.0;
        assert!(
            (top(actual[1]) - top(actual[0]) - delta).abs() <= 0.1,
            "동일 source 줄 간격과 순서를 paint까지 보존한다"
        );
    }
    let normal = |s: &str| {
        s.chars()
            .filter(|c| !c.is_whitespace() && !c.is_control())
            .collect::<String>()
    };
    assert_eq!(
        normal(&after.iter().map(|n| text(n)).collect::<String>()),
        normal(&source[2].text),
        "주석 본문 전체의 순서·누락·중복을 보존한다"
    );
}
