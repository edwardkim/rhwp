//! [Issue #6737] 선행 공백이 실린 인라인 TAC 표가 391px 오른쪽에서 시작해 셀 4개가
//! 용지 밖으로 사라졌다.
//!
//! `stored_ladder_gives_tac_table_its_own_line` 은 저장 사다리가 그 표에 **자기 줄**을
//! 줬는지 보고, 줬으면 앞 공백 폭(leading)을 표의 x 에 싣지 않는다(`#6167`).
//! 그런데 그 판정이 **축이 다른 두 값을 견주고 있었다.**
//!
//! ```text
//! 156487948 pi=0   표의 문자 축 위치      40
//!                  저장 `text_start`      72   ← HWP5 축(확장 컨트롤 1개당 8 유닛)
//! ```
//!
//! 앞에 확장 컨트롤(그림 2개)이 있어 두 축이 32 만큼 벌어지고, 판정이 영원히 거짓이
//! 된다. `#6167` 이 이 술어를 세울 때 쓴 문서(113424)는 선행 컨트롤이 없어 두 축이
//! **우연히 겹쳤을 뿐**이다.
//!
//! 축 환산 대신 **기하**로 막는다 — `leading + 표폭` 이 단폭을 **한 글자(16px) 이상**
//! 넘으면 그 leading 은 실제가 아니다. 한컴은 그런 표를 다음 줄 좌단에 둔다.
//!
//! ```text
//!            표 x 시작 .. 끝        1쪽 글자
//! 수정 전     467.6 .. 1103.1        748     ← 오른쪽 309.4px 이 용지(793.7) 밖
//! 수정 후      77.5 ..  713.0        818
//! 한/글 2024   76.8 ..  713.4        818
//! ```
//!
//! ⚠ 여유 한 글자(16px)는 필러 기반 leading 을 지키기 위한 것이다 — `복학원서.hwp`
//! (`#677` 골든)는 leading 5.13px + 표 642.5 로 단폭을 **5.1px 만** 넘는데, 그 축은
//! `#1195` 한컴 실측으로 보정된 별도 축이다. 여유 없이 자르면 그 골든이 깨진다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// 재현물은 코퍼스 문서다.
///
/// `hwpdocs_10k_share/korea_downloads/해양경찰청/
///  156487948_해양경찰청, 해양오염방제관련 소관법령 마련 용역 완료.hwp`
///
/// ⚠ `.hwp` 를 `samples/` 에 넣으면 `ir_field_sweep_baseline` 이 `samples/` 전체를
/// 스윕한다. `RHWP_ISSUE6737_SAMPLE` 로 덮어쓸 수 있다.
fn sample() -> Option<Vec<u8>> {
    if let Ok(path) = std::env::var("RHWP_ISSUE6737_SAMPLE") {
        return std::fs::read(path).ok();
    }
    let roots = [
        r"C:\Users\planet\hwpdocs_10k_share\korea_downloads\해양경찰청",
        r"D:\hwpdocs_10k_share\korea_downloads\해양경찰청",
    ];
    for base in roots {
        let Ok(entries) = std::fs::read_dir(base) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with("156487948_") && name.ends_with(".hwp") {
                return std::fs::read(entry.path()).ok();
            }
        }
    }
    None
}

fn collect(node: &RenderNode, tables: &mut Vec<(f64, f64)>, runs: &mut Vec<(f64, f64, String)>) {
    match &node.node_type {
        RenderNodeType::Table(_) => tables.push((node.bbox.x, node.bbox.x + node.bbox.width)),
        RenderNodeType::TextRun(run) => {
            runs.push((node.bbox.x, node.bbox.x + node.bbox.width, run.text.clone()))
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, tables, runs);
    }
}

/// 표는 본문 왼쪽에서 시작하고 용지 안에 들어와야 한다.
#[test]
fn inline_tac_table_with_leading_spaces_starts_at_the_body_left() {
    let Some(bytes) = sample() else {
        return;
    };
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(core.page_count(), 2, "한/글 2024 와 같은 2쪽이어야 한다");

    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let paper_right = tree.root.bbox.x + tree.root.bbox.width;
    let mut tables = Vec::new();
    let mut runs = Vec::new();
    collect(&tree.root, &mut tables, &mut runs);

    let header = tables
        .iter()
        .copied()
        .max_by(|a, b| (a.1 - a.0).partial_cmp(&(b.1 - b.0)).unwrap())
        .expect("1쪽에 표가 있어야 한다");

    assert!(
        header.1 <= paper_right + 0.5,
        "머리글 표가 용지 밖으로 나가면 안 된다 — #6737 회귀 \
         (표 오른쪽 {:.1} > 용지 {paper_right:.1}; 수정 전 1103.1 vs 793.7)",
        header.1
    );
    assert!(
        header.0 < 120.0,
        "머리글 표는 본문 왼쪽에서 시작해야 한다 — #6737 회귀 \
         (표 왼쪽 {:.1}; 수정 전 467.6, 한/글 76.8)",
        header.0
    );
}

/// 용지 밖으로 나가 인쇄에서 사라지던 셀 글자가 **용지 안**에 있어야 한다.
///
/// ⚠ 렌더 트리에는 결함이 있어도 노드가 남는다(용지 밖 좌표). 존재만 보면 음성
/// 대조를 통과하므로 **x 가 용지 안인지**를 봐야 한다.
#[test]
fn cells_pushed_off_the_paper_are_back_inside_the_paper() {
    let Some(bytes) = sample() else {
        return;
    };
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let paper_right = tree.root.bbox.x + tree.root.bbox.width;
    let mut tables = Vec::new();
    let mut runs = Vec::new();
    collect(&tree.root, &mut tables, &mut runs);

    for needle in ["총", "담당부서", "담당계장", "이종남"] {
        let hit = runs
            .iter()
            .find(|(_, _, t)| t.replace(' ', "").contains(needle));
        let (x0, x1, text) =
            hit.unwrap_or_else(|| panic!("셀 {needle:?} 글자가 1쪽에 있어야 한다 — #6737 회귀"));
        assert!(
            *x1 <= paper_right + 0.5,
            "셀 {text:?} 이 용지 안에 있어야 한다 — #6737 회귀              (x {x0:.1}..{x1:.1} > 용지 {paper_right:.1})"
        );
    }
}
