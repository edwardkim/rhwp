//! [#7422] 칸 문단을 **다시 조합**하는 경로도 문단 좌우 여백을 뺀 상자를 쓴다.
//!
//! # 무엇이 깨져 있었나
//!
//! `#7410`(`#7407` A2a)이 편집 경로에서 「칸 문단 상자도 본문과 같은 문단 여백 계약을
//! 쓴다」로 통일했는데, `renderer/composer.rs` 의 **재조합 경로 세 곳**은 그대로
//! `ParagraphBox::content_width_px`(칸 안쪽 폭 전체)를 쓰고 있었다. 그래서 같은 문단이
//! 어느 경로로 왔느냐에 따라 다른 프레임 폭을 받았다 —
//! `ParagraphBox::body` 의 주석이 경고하는 상태 그대로다.
//!
//! 이 문서에서 프레임이 잘라 주는 폭이 문단 좌우 여백만큼 넓어, 줄마다 글자를 더 먹고
//! 마지막 줄이 짧아졌다.
//!
//! # 기대값의 출처 — 한/글 출력 PDF
//!
//! `pdf/78494-virtual-convergence-industry-decree-2020.pdf` 37쪽의 **첫 줄**이
//! `법인 또는 단체일 것` 이다(96dpi 래스터 직접 판독). rhwp 는 수정 전 그 자리를
//! `단체일 것` 로 끊어, 앞 줄이 `… 가능한 법인 또는` 로 끝났다.
//!
//! ```text
//! 한/글 정본     … 지원센터의      / 설치 목적 … 가능한 / 법인 또는 단체일 것
//! 수정 전 rhwp   … 지원센터의 설치 / 목적 … 법인 또는   / 단체일 것
//! 수정 후 rhwp   … 지원센터의      / 설치 목적 … 가능한 / 법인 또는 단체일 것
//! ```
//!
//! # 이 검사가 말하지 않는 것
//!
//! 이 쪽에는 이 변경과 무관한 세로 어긋남이 남아 있다(정본은 같은 칸이 앞 쪽에서
//! 이어진다). 여기서 잠그는 것은 **그 칸의 줄 나눔**뿐이다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// 한/글이 저장한 실제 문서(이름은 `.hwpx` 지만 CFB/HWP5 본이다).
const DOC: &str = "samples/issue6776/78494-virtual-convergence-industry-decree.hwpx";
/// 0-based. 한/글 출력 PDF 의 37쪽과 같은 쪽이다.
const PAGE: u32 = 36;

fn collect_runs(node: &RenderNode, out: &mut Vec<(f64, f64, String)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push((node.bbox.y, node.bbox.x, run.text.clone()));
    }
    for child in &node.children {
        collect_runs(child, out);
    }
}

/// 대상 칸의 줄 나눔이 한/글 출력과 같다.
#[test]
fn a_recomposed_cell_keeps_the_break_hancom_printed() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(DOC);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{DOC} 읽기: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let page = core.build_page_render_tree(PAGE).expect("37쪽 render tree");

    let mut runs = Vec::new();
    collect_runs(&page.root, &mut runs);
    runs.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // 같은 baseline 의 run 을 한 줄로 모은다.
    let mut lines: Vec<(f64, String)> = Vec::new();
    for (y, _x, text) in runs {
        match lines.last_mut() {
            Some((prev_y, buf)) if (*prev_y - y).abs() < 0.5 => buf.push_str(&text),
            _ => lines.push((y, text)),
        }
    }

    let joined: Vec<String> = lines
        .iter()
        .map(|(_, text)| text.chars().filter(|c| !c.is_whitespace()).collect())
        .collect();

    // 전제 확인: 이 쪽에 대상 칸의 문장이 실제로 있어야 한다.
    assert!(
        joined.iter().any(|line| line.contains("지원센터의")),
        "정답지 전제가 깨졌다 — 37쪽에서 대상 문장을 찾지 못했다. 쪽 구성이 바뀌었으면          이 검사의 쪽 번호부터 다시 정해야 한다. 줄={joined:?}"
    );

    assert!(
        joined.iter().any(|line| line == "법인또는단체일것"),
        "한/글 출력 37쪽 첫 줄 `법인 또는 단체일 것` 이 한 줄로 안 나온다 — 칸 프레임이          문단 좌우 여백만큼 넓으면 앞 줄이 `법인 또는` 까지 먹고 `단체일 것` 만 남는다.          줄={joined:?}"
    );
}
