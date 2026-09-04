//! [Issue #6697] 표 칸 안 문단이 **블록 표**(`treat_as_char=false`)를 품으면 그 문단의
//! **자기 글자**가 어느 쪽에도 그려지지 않았다.
//!
//! `Task #573` 이 인라인 TAC 표에서 같은 결함을 고쳤고 블록 표는 "텍스트 흐름 외부"라며
//! ELSE 분기에 남았다. 글자가 없는 호스트 문단이 대다수라 오래 숨어 있었다.
//!
//! `80550` 30쪽 캡션 `<향후 10년간 폐농업용 지게차 해체 수익 계산>` 21자:
//!
//! ```text
//!   devel            30쪽 캡션 0회
//!   이 수정          30쪽 캡션 1회      넘침·용지밖·겹침·글자겹침 지표 전부 동일
//!   한/글 2020       30쪽 캡션 1회
//! ```
//!
//! ⚠ 이 수정은 **`#6716`(칸 안 자리차지 중첩 표의 `vertOffset`)이 먼저 들어간 뒤**라야
//! 쓸모가 있다. 표가 제자리로 가기 전에 글자만 되살리면 캡션이 표 머리행과 겹친다.
//!
//! ⚠ 렌더 트리의 `TextRun` 은 run 단위로 쪼개진다(`"<향후 10"` + `"년간 …"`).
//! 문자열 전체로 찾으면 **있는데도 0건**으로 보인다 — 이어 붙여서 봐야 한다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// 재현물은 코퍼스 문서다.
///
/// `hwpdocs_10k_share/opinion_downloads/농림축산식품부/
///  80550_(규제영향분석서) 농업기계화 촉진법 시행규칙 일부개정령(안).hwpx`
///
/// `RHWP_ISSUE6697_SAMPLE` 로 덮어쓸 수 있다.
fn sample() -> Option<Vec<u8>> {
    if let Ok(path) = std::env::var("RHWP_ISSUE6697_SAMPLE") {
        return std::fs::read(path).ok();
    }
    let roots = [
        r"C:\Users\planet\hwpdocs_10k_share\opinion_downloads\농림축산식품부",
        r"D:\hwpdocs_10k_share\opinion_downloads\농림축산식품부",
    ];
    for base in roots {
        let Ok(entries) = std::fs::read_dir(base) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with("80550_") && name.ends_with(".hwpx") {
                return std::fs::read(entry.path()).ok();
            }
        }
    }
    None
}

fn collect_text(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        collect_text(child, out);
    }
}

/// 블록 표를 앵커한 칸 문단의 캡션이 그 쪽에 그려져야 한다.
#[test]
fn cell_host_paragraph_caption_is_drawn_on_its_page() {
    let Some(bytes) = sample() else {
        return;
    };
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(core.page_count(), 31, "한/글 2020 과 같은 31쪽이어야 한다");

    let tree = core.build_page_render_tree(29).expect("30쪽 render tree");
    let mut text = String::new();
    collect_text(&tree.root, &mut text);
    let flat: String = text.chars().filter(|c| !c.is_whitespace()).collect();

    assert!(
        flat.contains("해체수익계산"),
        "블록 표를 앵커한 칸 문단의 캡션이 30쪽에 그려져야 한다 — #6697 회귀 \
         (수정 전 0회, 한/글 2020 은 1회)"
    );
}
