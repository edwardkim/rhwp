//! 원본 공동 쪽 컷의 빈 밴드는 그 조각이 소유하고, 한 셀만 이어지는 쪽은 내용에서 닫힌다.
//! 동일 원본의 독립 한컴 Print에서 2쪽 외곽은 3쪽 외곽보다 아래에서 끝난다.
//! 절대 좌표 대신 두 원본 조각의 종료 관계를 검사한다.
#![cfg(not(target_arch = "wasm32"))]
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

struct OutputDir(PathBuf);

impl OutputDir {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("현재 시각")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "rhwp-5885-body-frame-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("쪽별 실제 출력 경로");
        Self(path)
    }
}

impl Drop for OutputDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn outer_frames(node: &Value, out: &mut Vec<(f64, f64)>) {
    if node["type"] == "Table" {
        let bbox = &node["bbox"];
        let y = bbox["y"].as_f64().expect("표 원점");
        let h = bbox["h"].as_f64().expect("표 높이");
        assert!(h > 0.0, "양수 물리 프레임");
        out.push((y, y + h));
        return;
    }
    if let Some(children) = node["children"].as_array() {
        for child in children {
            outer_frames(child, out);
        }
    }
}

#[test]
fn issue_5885_shared_body_frame_and_single_owner_tail_have_distinct_ends() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let input = root.join("samples/issue5885/3171199_design_capability_criteria.hwp");
    let output = OutputDir::new();
    let result = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("export-render-tree")
        .arg(&input)
        .arg("-o")
        .arg(&output.0)
        .args(["--compat", "2022"])
        .output()
        .expect("실제 Native 쪽 출력");
    assert!(
        result.status.success(),
        "Native 출력 실패: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let mut frames = Vec::new();
    for page in [2, 3] {
        let path = output.0.join(format!("render_tree_{page:03}.json"));
        let tree: Value =
            serde_json::from_slice(&fs::read(path).expect("쪽별 실제 트리")).expect("트리 JSON");
        let mut page_frames = Vec::new();
        outer_frames(&tree, &mut page_frames);
        assert_eq!(
            page_frames.len(),
            1,
            "해당 쪽은 같은 원본 표 조각 하나를 소유한다"
        );
        frames.push(page_frames[0]);
    }
    assert_eq!(
        frames[0].0, frames[1].0,
        "두 이어받기는 같은 본문 상단에서 시작한다"
    );
    assert!(frames[0].1 > frames[1].1,
        "공동 쪽 컷의 빈 밴드는 2쪽에 보존하고, 한 셀만 이어지는 3쪽은 그 내용에서 닫혀야 한다: {:?}", frames);
}
