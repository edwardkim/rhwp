//! 관계형·고정형 — [#9223] 단 폭을 채운 어울림 개체는 밀린 글줄이 넘어가도 떠나는 쪽에 남는다.
//! 주석 갱신: 2026-11-20 13:40 KST (새 파일)
//!
//! 입력: 공개 보도자료(산업통상자원부 「이산화탄소저장활용법 하위법령 제정안 입법예고」, HWPX, 3쪽)
//! `samples/issue9223/ccus-press-release-square-band.hwpx`.
//! 기준: 한컴 engine 2020 Print PDF `pdf/issue9223/ccus-press-release-square-band-2020.pdf`(3쪽).
//! `pi=16` 은 [0] 사각형 글상자(어울림, 세로 문단 기준 오프셋 2943HU, 48797×4287HU — 단 폭 48188HU
//! 이상)와 [1] 글자처럼 취급한 1×3 표(저장 줄 vpos 0, 높이 3013)를 가진다. 앞 문단은 1쪽 63449HU 에서
//! 끝나 줄만이면 1쪽에 들어가지만, 개체가 줄 자리와 겹쳐 줄을 `2943 + 4287` 아래로 밀어 줄이 2쪽 맨
//! 위로 넘어갔다. 한/글은 개체를 1쪽 아래(앞 흐름 끝 + 오프셋)에 남긴다. 수정 전 rhwp 는 개체도 2쪽에
//! 그려 `pi=18` 제목 표를 덮었다.
//! - 고정형: 전체 쪽수 3(= PDF).
//! - 관계형: 사각형은 1쪽에 있고 앞 표(`pi=15`) 아래에 있다. 2쪽에는 그 사각형이 없고, 글자처럼
//!   취급한 표(`pi=16`)가 제목 표(`pi=18`) 위에 있다.
//!
//! 측정(2026-11-20, 이 파일과 같은 커밋의 코드, base = upstream/devel `0a437a7eef`,
//! `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`, raw =
//! `silhouette_raw_match_percent`, 글꼴 `ttfs/opensource` — 문서 본 글꼴 없는 환경):
//! - Native: 3쪽 / 3쪽, 1쪽 99.97(base 90.54)·2쪽 99.74(base 87.82), 전쪽 최저 99.74
//! - fresh WASM(`scripts/wasm-pack-locked.sh --target web`): 3쪽 / 3쪽, 1쪽 99.82·2쪽 97.45, 전쪽 최저 97.45
//! - 증거: `mydocs/pr/assets/issue_9223/`(native·wasm 전쪽 TSV, 1·2쪽 review·overlay PNG)
//! - 1쪽 말미의 사각형(안내 띠)은 한컴 PDF 에서도 본문 바닥(1028.0px)을 넘어 1031px 까지 그려진다.
//!   `tests/fixtures/body_overflow_baseline.tsv` 에 1건으로 등록했다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/issue9223/ccus-press-release-square-band.hwpx";
/// 한컴 engine 2020 Print PDF 의 쪽수.
const PDF_PAGES: u32 = 3;
/// 어울림 사각형과 글자처럼 취급한 표의 호스트 문단, 그 앞 표와 뒤 제목 표의 호스트 문단.
const HOST_PARA: usize = 16;
const PREVIOUS_TABLE_PARA: usize = 15;
const NEXT_TABLE_PARA: usize = 18;

fn collect<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        collect(child, out);
    }
}

/// 본문 직속(칸 밖) 사각형 가운데 호스트 문단의 것.
fn band_frames(nodes: &[&RenderNode]) -> Vec<BoundingBox> {
    nodes
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::Rectangle(rect)
                if rect.para_index == Some(HOST_PARA)
                    && rect.control_index == Some(0)
                    && rect.cell_index.is_none() =>
            {
                Some(node.bbox)
            }
            _ => None,
        })
        .collect()
}

fn table(nodes: &[&RenderNode], para: usize) -> Option<BoundingBox> {
    nodes.iter().find_map(|node| match &node.node_type {
        RenderNodeType::Table(t) if t.para_index == Some(para) && t.cell_context.is_none() => {
            Some(node.bbox)
        }
        _ => None,
    })
}

#[test]
fn column_wide_square_band_stays_on_the_page_its_line_left() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("fixture 누락 {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 열기");
    assert_eq!(
        core.page_count(),
        PDF_PAGES,
        "전체 쪽수가 한컴 PDF 와 다르다"
    );

    // 1쪽: 사각형이 앞 표 아래에 있다.
    let first = core.build_page_render_tree(0).expect("1쪽");
    let mut nodes = Vec::new();
    collect(&first.root, &mut nodes);
    let bands = band_frames(&nodes);
    assert_eq!(bands.len(), 1, "어울림 사각형은 1쪽에 한 번 있어야 한다");
    let previous = table(&nodes, PREVIOUS_TABLE_PARA).expect("1쪽 앞 표");
    assert!(
        bands[0].y >= previous.y + previous.height - 0.5,
        "사각형 위 {:.2} 은 앞 표 아래 {:.2} 보다 아래여야 한다",
        bands[0].y,
        previous.y + previous.height
    );
    assert!(
        table(&nodes, HOST_PARA).is_none(),
        "밀린 글줄의 표는 1쪽에 없어야 한다"
    );

    // 2쪽: 사각형은 없고, 넘어온 표가 제목 표 위에 있다.
    let second = core.build_page_render_tree(1).expect("2쪽");
    let mut nodes = Vec::new();
    collect(&second.root, &mut nodes);
    assert!(
        band_frames(&nodes).is_empty(),
        "어울림 사각형은 2쪽에 없어야 한다"
    );
    let host = table(&nodes, HOST_PARA).expect("2쪽에 넘어온 표");
    let next = table(&nodes, NEXT_TABLE_PARA).expect("2쪽 제목 표");
    assert!(
        host.y + host.height <= next.y + 0.5,
        "넘어온 표 아래 {:.2} 은 제목 표 위 {:.2} 보다 위여야 한다",
        host.y + host.height,
        next.y
    );
}
