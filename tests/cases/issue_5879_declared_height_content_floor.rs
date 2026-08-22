//! [#5879] 선언 높이 비례 축소는 **내용이 필요로 하는 높이 아래로 줄이지 않는다.**
//!
//! 종전(devel `0f9ceeb19a`): `fit_measured_table_to_declared_height` 가 측정 합의
//! 75%~135% 창 안이면 무조건 비례 축소했다. `samples/issue4514/sample1-repro.hwp` 19쪽의
//! 표는 측정 683.9px → 선언 565.8px (**scale 0.829**) 라 창을 통과했고, 3행 셀이
//! 589.2px 필요한데 488.1px 로 눌리면서 **글줄 4개가 셀 밖으로** 밀렸다.
//! 그 줄들은 그려지되 clip 아래라 보이지 않고, 표가 "쪽에 들어간다"고 판정돼
//! 분할되지도 않아 **다음 쪽에서 이어지지도 않았다**(#5784 와 같은 증상).
//!
//! 한글 정답지 `pdf/issue4514/sample1-repro-2020.pdf` 는 19쪽을
//! `… 원활히 연동되도록 구축` 에서 끝내고 `- 데이터 연계 시 …` 부터 20쪽에 둔다.
#![cfg(not(target_arch = "wasm32"))]

use std::process::Command;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

const SAMPLE: &str = "samples/issue4514/sample1-repro.hwp";
/// 정답지가 20쪽 머리에 두는 문장. 19쪽에 있으면 clip 아래로 밀려 사라진 상태다.
const SEAM: &str = "타 행정정보 시스템과의 연계";

fn out_dir(tag: &str) -> std::path::PathBuf {
    static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let nth = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "rhwp_issue_5879_{tag}_{}_{}",
        std::process::id(),
        nth
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("출력 디렉토리 생성");
    dir
}

fn run(args: &[&str]) {
    let done = Command::new(rhwp_bin())
        .current_dir(repo_root())
        .args(args)
        .output()
        .expect("rhwp 실행");
    assert!(
        done.status.success(),
        "{args:?} 실패: {}",
        String::from_utf8_lossy(&done.stderr)
    );
}

/// 쪽 번호(1 기준) 텍스트를 읽는다. 다중 쪽 산출은 `<stem>_NNN.txt` 다.
fn page_text(dir: &std::path::Path, page: usize) -> String {
    let path = std::fs::read_dir(dir)
        .expect("출력 디렉토리")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(&format!("_{page:03}.txt")))
        })
        .unwrap_or_else(|| panic!("{page}쪽 텍스트 산출물이 없다"));
    std::fs::read_to_string(path).expect("쪽 텍스트 읽기")
}

#[test]
fn page19_ends_at_the_oracle_seam_and_the_rest_moves_to_page20() {
    let dir = out_dir("text");
    run(&["export-text", SAMPLE, "-o", dir.to_str().expect("경로")]);

    let p19 = page_text(&dir, 19);
    let p20 = page_text(&dir, 20);

    assert!(
        !p19.contains(SEAM),
        "19쪽이 {SEAM:?} 를 담고 있다 — 선언 높이 축소로 셀 밖에 밀려 clip 이 지운 상태다"
    );
    assert!(
        p20.contains(SEAM),
        "정답지대로라면 {SEAM:?} 는 20쪽에 있어야 한다"
    );
}

#[test]
fn page19_draws_nothing_below_the_body_area() {
    let dir = out_dir("tree");
    run(&[
        "export-render-tree",
        SAMPLE,
        "-p",
        "18", // 0 기준 — 문서 19쪽
        "-o",
        dir.to_str().expect("경로"),
    ]);
    let tree_path = std::fs::read_dir(&dir)
        .expect("출력 디렉토리")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .find(|path| path.extension().is_some_and(|ext| ext == "json"))
        .expect("render tree 산출물");
    let root: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(tree_path).expect("render tree 읽기"))
            .expect("render tree 파싱");

    // Body 서브트리만 본다 — 꼬리말(쪽 번호)은 Body 형제라 본문 하한 아래가 정상이다.
    fn find_body(node: &serde_json::Value) -> Option<&serde_json::Value> {
        if node.get("type").and_then(|t| t.as_str()) == Some("Body") {
            return Some(node);
        }
        node.get("children")?.as_array()?.iter().find_map(find_body)
    }
    fn lowest_text_bottom(node: &serde_json::Value, acc: &mut f64) {
        if node.get("type").and_then(|t| t.as_str()) == Some("TextLine") {
            if let Some(bbox) = node.get("bbox") {
                let y = bbox.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let h = bbox.get("h").and_then(|v| v.as_f64()).unwrap_or(0.0);
                *acc = acc.max(y + h);
            }
        }
        if let Some(children) = node.get("children").and_then(|c| c.as_array()) {
            for child in children {
                lowest_text_bottom(child, acc);
            }
        }
    }

    let body = find_body(&root).expect("Body 노드");
    let bbox = body.get("bbox").expect("Body bbox");
    let body_bottom = bbox.get("y").and_then(|v| v.as_f64()).expect("Body y")
        + bbox.get("h").and_then(|v| v.as_f64()).expect("Body h");
    let mut lowest = f64::MIN;
    lowest_text_bottom(body, &mut lowest);

    assert!(
        lowest > f64::MIN,
        "19쪽 본문에 글줄이 하나도 없다 — 시험 전제가 깨졌다"
    );
    assert!(
        lowest <= body_bottom + 1.0,
        "본문 하한 {body_bottom:.1} 아래까지 글줄이 그려진다 (최하단 {lowest:.1}) — clip 이 지우는 글자다"
    );
}
