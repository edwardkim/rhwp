//! 관계형·고정형 — [#9045] 나뉜 1열 표 끝 행의 쪽 상자 정렬과 칸 안 자리차지 그림 띠.
//! 주석 갱신: 2026-11-20 01:04 KST (새 파일)
//!
//! 입력: 중소벤처기업부 사업공고 첨부 HWPX(공개, #9038 첨부) `samples/issue9038/sme_announcement_split_table_successor.hwpx`.
//! 기준: 같은 게시글의 한컴 PDF(`Hwp 2018 10.0.0.14515`, 46쪽)
//! `pdf/issue9038/sme_announcement_split_table_successor-hancom-2018.pdf`.
//! 25행 1열 RowBreak 표(pi=251)가 43~46쪽에 나뉜다.
//! - 고정형: 전체 쪽수 46(= PDF).
//! - 관계형(44쪽): 13번째 칸(저장 높이 40645HU)이 중첩 표의 첫 행 뒤에서 나뉜다. 조각 상자는 단
//!   아래끝 − 표 바깥 아래 여백까지이고 중첩 표는 그 상자의 가운데에 있다. 둘째 행은 45쪽 맨 위다.
//! - 관계형(43쪽): 자리차지 그림(오프셋 300HU + 높이 4191HU) 바닥에서 다음 문단의 그림 줄(저장 vpos
//!   4491)이 시작한다. 글 한 줄과 자리차지 그림 넷을 가진 칸은 글줄 위끝~가장 아래 그림 바닥을
//!   칸 가운데에 둔다.
//! - 관계형(45쪽): 글이 있는 호스트의 자리차지 그림은 글줄 아래에 차례로 놓이고 겹치지 않으며
//!   칸 안에서 끝난다(음수 오프셋 -48883HU 그림은 앞 그림 바닥).
//!
//! 측정(2026-11-20, 이 파일과 같은 커밋의 코드, base = #9038 수정 `498ad0fcec`(upstream/devel
//! `9440d05e7d` 위), `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`, raw =
//! `silhouette_raw_match_percent`, 글꼴 `ttfs/opensource` — 문서 본 글꼴 없는 환경):
//! - Native: 46쪽 / 46쪽, 43쪽 95.58 · 44쪽 99.95 · 45쪽 97.67, 전쪽 최저 90.83(17쪽)
//! - fresh WASM(`scripts/wasm-pack-locked.sh --target web`): 46쪽 / 46쪽, 43쪽 95.60 · 44쪽 99.95 ·
//!   45쪽 97.64, 전쪽 최저 93.38(17쪽)
//! - 수정 전(base, SVG raster 예비 측정): 43쪽 75.15 · 44쪽 63.43 · 45쪽 65.08
//! - 증거: `mydocs/pr/assets/issue_9045/`(native·wasm TSV, 43~45쪽 review·overlay PNG)
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/issue9038/sme_announcement_split_table_successor.hwpx";
/// 한컴 Print PDF 의 전체 쪽수.
const PDF_PAGES: u32 = 46;

fn open() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("fixture 누락 {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 열기");
    assert_eq!(
        core.page_count(),
        PDF_PAGES,
        "전체 쪽수가 한컴 PDF 와 다르다"
    );
    core
}

fn ink(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.extend(run.text.chars().filter(|c| !c.is_whitespace()));
    }
    for child in &node.children {
        ink(child, out);
    }
}

fn ink_of(node: &RenderNode) -> String {
    let mut out = String::new();
    ink(node, &mut out);
    out
}

fn collect<'a>(
    node: &'a RenderNode,
    pred: &dyn Fn(&RenderNode) -> bool,
    out: &mut Vec<&'a RenderNode>,
) {
    if pred(node) {
        out.push(node);
    }
    for child in &node.children {
        collect(child, pred, out);
    }
}

fn all<'a>(node: &'a RenderNode, pred: &dyn Fn(&RenderNode) -> bool) -> Vec<&'a RenderNode> {
    let mut out = Vec::new();
    collect(node, pred, &mut out);
    out
}

fn is_cell(node: &RenderNode) -> bool {
    matches!(node.node_type, RenderNodeType::TableCell(_))
}

fn is_image(node: &RenderNode) -> bool {
    matches!(node.node_type, RenderNodeType::Image(_))
}

fn bottom(node: &RenderNode) -> f64 {
    node.bbox.y + node.bbox.height
}

/// 글이 `needle` 로 시작하는 칸 가운데 가장 안쪽(가장 낮은) 칸.
fn innermost_cell<'a>(root: &'a RenderNode, needle: &str) -> &'a RenderNode {
    all(root, &|n| is_cell(n) && ink_of(n).starts_with(needle))
        .into_iter()
        .min_by(|a, b| a.bbox.height.total_cmp(&b.bbox.height))
        .unwrap_or_else(|| panic!("「{needle}」 로 시작하는 칸이 없다"))
}

/// `inner` 를 담은 칸 가운데 가장 안쪽 칸.
fn innermost_cell_containing<'a>(root: &'a RenderNode, inner: &RenderNode) -> &'a RenderNode {
    all(root, &|n| {
        is_cell(n) && n.bbox.y < inner.bbox.y - 0.5 && ink_of(n) == ink_of(inner)
    })
    .into_iter()
    .max_by(|a, b| a.bbox.y.total_cmp(&b.bbox.y))
    .expect("중첩 표를 담은 바깥 칸 조각")
}

fn images_top_down(cell: &RenderNode) -> Vec<&RenderNode> {
    let mut images = all(cell, &is_image);
    images.sort_by(|a, b| a.bbox.y.total_cmp(&b.bbox.y));
    images
}

fn first_line_top(cell: &RenderNode) -> f64 {
    all(cell, &|n| {
        matches!(n.node_type, RenderNodeType::TextLine(_))
    })
    .into_iter()
    .map(|line| line.bbox.y)
    .fold(f64::INFINITY, f64::min)
}

/// 44쪽: 25행 1열 표의 13번째 칸은 중첩 표의 첫 행 뒤에서 나뉜다. 한/글은 그 칸의 첫 조각
/// 상자를 쪽 아래까지 두고(저장 칸 높이 40645HU ≥ 쪽이 남긴 상자), 조각 내용을 가운데에 둔다.
#[test]
fn split_end_row_fragment_centers_nested_table_in_page_box() {
    let core = open();
    let tree = core.build_page_render_tree(43).expect("44쪽");
    let nested = all(&tree.root, &|n| {
        matches!(&n.node_type, RenderNodeType::Table(t) if t.row_count == 2 && t.col_count == 1)
            && ink_of(n).contains("대표자신분증진위확인(주민등록증")
    })
    .into_iter()
    .next()
    .expect("44쪽에 「① 대표자 신분증 진위확인」 중첩 표가 있어야 한다");
    let host = innermost_cell_containing(&tree.root, nested);
    let column_bottom = all(&tree.root, &|n| {
        matches!(n.node_type, RenderNodeType::Column(_))
    })
    .into_iter()
    .map(bottom)
    .next()
    .expect("본문 단");
    // 조각 상자는 칸 위끝부터 단 아래끝에서 표 바깥 아래 여백(141HU)을 뺀 자리까지다.
    let box_bottom = column_bottom - 141.0 * 96.0 / 7200.0;
    let above = nested.bbox.y - host.bbox.y;
    let below = box_bottom - bottom(nested);
    assert!(
        (above - below).abs() <= 1.0,
        "나뉜 끝 행 조각의 중첩 표는 쪽이 정한 상자의 가운데에 있어야 한다: 위 {above:.2} 아래 {below:.2}"
    );
    let heading = all(&tree.root, &|n| {
        matches!(n.node_type, RenderNodeType::TextLine(_))
            && ink_of(n).contains("대표자신분증진위확인→②필수서류자동제출")
    })
    .into_iter()
    .next()
    .expect("「5. 서류제출」 안내 줄");
    assert!(
        nested.bbox.y >= bottom(heading),
        "중첩 표 상자는 머리 글 아래에서 시작해야 한다"
    );
    assert!(
        ink_of(&core.build_page_render_tree(44).expect("45쪽").root)
            .starts_with("②필수서류자동제출"),
        "중첩 표의 둘째 행은 45쪽 맨 위에서 시작해야 한다"
    );
}

/// 43쪽: 글자 없는 호스트의 자리차지 그림(오프셋 300HU, 높이 4191HU) 바닥에서 다음 문단의
/// 글자처럼 취급 그림 줄이 시작한다(저장 vpos 4491 = 300 + 4191).
#[test]
fn next_stored_line_closes_cell_picture_band() {
    let core = open();
    let tree = core.build_page_render_tree(42).expect("43쪽");
    let heading_top = all(&tree.root, &|n| {
        matches!(n.node_type, RenderNodeType::TextLine(_)) && ink_of(n).contains("2.신청자금선택")
    })
    .into_iter()
    .map(|line| line.bbox.y)
    .next()
    .expect("「2. 신청자금 선택」 줄");
    let mut images: Vec<_> = all(&tree.root, &is_image)
        .into_iter()
        .filter(|image| image.bbox.width > 500.0 && bottom(image) <= heading_top)
        .collect();
    images.sort_by(|a, b| a.bbox.y.total_cmp(&b.bbox.y));
    let [float, inline] = images.as_slice() else {
        panic!("「1.」 그림 칸에 그림 둘이 있어야 한다: {}", images.len());
    };
    assert!(
        float.bbox.height < inline.bbox.height,
        "위 그림이 자리차지 그림(낮은 띠)이어야 한다"
    );
    assert!(
        (bottom(float) - inline.bbox.y).abs() <= 0.5,
        "자리차지 그림 바닥에서 다음 그림 줄이 시작해야 한다: {} / {}",
        bottom(float),
        inline.bbox.y
    );

    // 같은 쪽 「① 기업정보 …」 칸: 글 한 줄과 자리차지 그림 넷. 내용은 글줄 위끝부터 가장 아래
    // 그림 바닥까지이고 칸 세로 가운데에 놓인다.
    let cell = innermost_cell(&tree.root, "①기업정보및관할지역본지부선택");
    let images = images_top_down(cell);
    assert_eq!(images.len(), 4, "칸의 자리차지 그림 넷");
    let above = first_line_top(cell) - cell.bbox.y;
    let below = bottom(cell) - images.iter().map(|i| bottom(i)).fold(0.0, f64::max);
    assert!(
        (above - below).abs() <= 1.0,
        "글줄과 그림 묶음이 칸 가운데에 있어야 한다: 위 {above:.2} 아래 {below:.2}"
    );
}

/// 45쪽: 글이 있는 호스트 문단의 자리차지 그림.
#[test]
fn text_host_topbottom_pictures_do_not_overlap() {
    let core = open();
    let tree = core.build_page_render_tree(44).expect("45쪽");

    // 「① 신청금액 …」: 그림(오프셋 1317HU, 높이 2834HU) 바닥에서 다음 문단 그림 줄(vpos 4151).
    let cell = innermost_cell(&tree.root, "①신청금액등신청정보입력");
    let images = images_top_down(cell);
    let [tab, body] = images.as_slice() else {
        panic!("「① 신청금액」 칸에 그림 둘: {}", images.len());
    };
    assert!(
        first_line_top(cell) < tab.bbox.y && (bottom(tab) - body.bbox.y).abs() <= 0.5,
        "글줄 → 탭 그림 → 본문 그림 순서로 이어져야 한다"
    );
    assert!(
        bottom(body) <= bottom(cell) + 0.5,
        "본문 그림은 칸 안에서 끝나야 한다: {} > {}",
        bottom(body),
        bottom(cell)
    );

    // 「② 기업정보 활용 …」: 둘째 그림은 음수 오프셋(-48883HU)이라 문단 위로 올라가지 않고,
    // 겹침을 허용하지 않으므로 첫 그림(오프셋 1362HU) 아래에 놓인다.
    let cell = innermost_cell(&tree.root, "②기업정보활용을위한체크리스트진행");
    let images = images_top_down(cell);
    let [tab, body] = images.as_slice() else {
        panic!("「② 기업정보 활용」 칸에 그림 둘: {}", images.len());
    };
    assert!(
        tab.bbox.height < body.bbox.height,
        "위 그림이 탭 그림(낮은 띠)이어야 한다"
    );
    assert!(
        first_line_top(cell) < tab.bbox.y && (bottom(tab) - body.bbox.y).abs() <= 0.5,
        "글줄 → 탭 그림 → 본문 그림 순서로 겹치지 않고 이어져야 한다: {} / {}",
        bottom(tab),
        body.bbox.y
    );
    assert!(
        bottom(body) <= bottom(cell) + 0.5,
        "본문 그림은 칸 안에서 끝나야 한다"
    );
}
