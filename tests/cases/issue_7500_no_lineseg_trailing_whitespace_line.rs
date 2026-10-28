//! [#7500] 저장 줄이 없는 문단에서 끝의 공백이 별도 줄로 떨어지던 결함의 회귀 가드.
//!
//! # 무엇이 깨져 있었나
//!
//! 저장 LINE_SEG 가 없는 문단은 `compose_lines` 가 45자마다 줄을 끊는 합성 줄바꿈을 쓴다.
//! 이 합성은 끊고 남은 글자가 공백뿐이어도 그 공백을 새 줄로 만들었다. 한/글은 말미
//! 공백에 줄상자를 주지 않으므로(#7160) 그 줄 수만큼 뒤 내용이 밀린다.
//!
//! - 글자처럼 취급 표 하나와 공백 91자만 든 host 문단이 45+45+1 세 줄이 되어, 뒤의
//!   쪽 크기 표가 다음 쪽으로 넘어갔다. 표 앞 공백이 줄 끝에서 잘려 표가 줄 가운데에 앉았다.
//! - 그림 3장을 나란히 띄운 셀의 공백 문단은 합성 두 줄 경계에서 행이 잘려 그림이 두 쪽에
//!   걸쳤다. 공백을 흡수해 한 줄이 되면 이번에는 행 나눔 장부가 나란한 그림 높이를
//!   합산해(측정·배치는 띠 하나) 행이 통째로 다음 쪽으로 밀렸다.
//!
//! # 기대값의 출처
//!
//! 재현물은 실문서(기관 서식, 저장 LINE_SEG 없음)를 글자 치환·그림 교체로 익명화한 것이다.
//! 그림 띠 재현물은 결함과 무관한 떠 있는 표 2개와 글상자 3개를 지운 최소판이다.
//! 같은 파일을 한컴 웹한글기안기로 연 인쇄 출력(PDF)에서:
//!
//! - 표 host 문서는 1쪽이다. 글자처럼 취급 표는 앞 공백 뒤 줄 오른쪽 끝에 있고, 뒤의 큰 표는
//!   그 표 줄 아래에서 같은 쪽에 시작한다.
//! - 그림 띠 문서의 1쪽에는 그림 행이 통째로 놓이고, 그림 3장은 같은 띠에 나란히 있다.
//!
//! # 검사하는 것
//!
//! 쪽 수, 큰 표의 쪽 소속과 host 표 줄과의 앞뒤 순서, host 표의 줄 안 위치(단 가운데 기준),
//! 그림 3장의 1쪽 소속·표 조각 포함·같은 띠 여부.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const TAC_HOST_SAMPLE: &str = "samples/issue7500/tac_table_host_spaces.hwp";
const PICTURE_BAND_SAMPLE: &str = "samples/issue7500/picture_band_cell_spaces.hwp";
/// 글자처럼 취급 표와 공백만 든 host 문단.
const HOST_PARA: usize = 1;
/// host 뒤의 쪽 크기 표를 소유한 문단.
const BIG_TABLE_PARA: usize = 3;
/// 그림 띠 셀을 가진 표를 소유한 문단.
const PICTURE_TABLE_PARA: usize = 3;

#[derive(Clone, Copy, Debug)]
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

impl Rect {
    fn of(node: &RenderNode) -> Self {
        Self {
            x: node.bbox.x,
            y: node.bbox.y,
            w: node.bbox.width,
            h: node.bbox.height,
        }
    }
    fn bottom(&self) -> f64 {
        self.y + self.h
    }
}

fn load(sample: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = std::fs::read(&path).expect("재현물 읽기");
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

/// 본문 직속 표(셀 안 표 제외).
fn top_level_table(node: &RenderNode, para: usize, in_table: bool) -> Option<Rect> {
    if let RenderNodeType::Table(t) = &node.node_type {
        if !in_table && t.para_index == Some(para) {
            return Some(Rect::of(node));
        }
    }
    let inside = in_table || matches!(node.node_type, RenderNodeType::Table(_));
    node.children
        .iter()
        .find_map(|child| top_level_table(child, para, inside))
}

fn column(node: &RenderNode) -> Option<Rect> {
    if matches!(node.node_type, RenderNodeType::Column(_)) {
        return Some(Rect::of(node));
    }
    node.children.iter().find_map(column)
}

/// 본문 직속 표 안 그림들.
fn images_in_table(node: &RenderNode, para: usize, in_target: bool, out: &mut Vec<Rect>) {
    let mut inside = in_target;
    match &node.node_type {
        RenderNodeType::Table(t) if !in_target && t.para_index == Some(para) => inside = true,
        RenderNodeType::Image(_) if in_target => out.push(Rect::of(node)),
        _ => {}
    }
    for child in &node.children {
        images_in_table(child, para, inside, out);
    }
}

#[test]
fn tac_table_host_keeps_single_page() {
    assert_eq!(
        load(TAC_HOST_SAMPLE).page_count(),
        1,
        "웹한글기안기 기준 1쪽 — 공백만 남은 합성 줄이 뒤 표를 다음 쪽으로 민다"
    );
}

#[test]
fn big_table_follows_host_table_line_on_first_page() {
    let doc = load(TAC_HOST_SAMPLE);
    let tree = doc.build_page_render_tree(0).expect("1쪽 렌더 트리");
    let host = top_level_table(&tree.root, HOST_PARA, false).expect("host 표");
    let big = top_level_table(&tree.root, BIG_TABLE_PARA, false)
        .expect("1쪽에 큰 표가 없다 — 공백 합성 줄이 표를 다음 쪽으로 밀었다");
    assert!(
        big.y + 0.5 >= host.bottom(),
        "큰 표는 host 표 줄 아래에서 시작해야 한다: host 아래끝 {:.1}, 큰 표 위 {:.1}",
        host.bottom(),
        big.y
    );
}

#[test]
fn tac_table_sits_after_leading_spaces_at_line_end() {
    let doc = load(TAC_HOST_SAMPLE);
    let tree = doc.build_page_render_tree(0).expect("1쪽 렌더 트리");
    let col = column(&tree.root).expect("단");
    let host = top_level_table(&tree.root, HOST_PARA, false).expect("host 표");
    let center = col.x + col.w / 2.0;
    assert!(
        host.x > center && host.x + host.w <= col.x + col.w + 0.5,
        "글자처럼 취급 표는 앞 공백 뒤 줄 오른쪽 끝(단 오른쪽 절반 안)에 앉는다. 단 {:.1}..{:.1}, \
         표 {:.1}..{:.1} — 앞 공백이 합성 줄 끝에서 잘리면 표가 줄 가운데로 온다",
        col.x,
        col.x + col.w,
        host.x,
        host.x + host.w
    );
}

#[test]
fn side_by_side_pictures_row_stays_whole_on_first_page() {
    let doc = load(PICTURE_BAND_SAMPLE);
    let tree = doc.build_page_render_tree(0).expect("1쪽 렌더 트리");
    let fragment = top_level_table(&tree.root, PICTURE_TABLE_PARA, false).expect("1쪽 표 조각");
    let mut images = Vec::new();
    images_in_table(&tree.root, PICTURE_TABLE_PARA, false, &mut images);
    assert_eq!(
        images.len(),
        3,
        "나란한 그림 3장이 모두 1쪽에 있어야 한다 — 행 나눔 장부가 띠를 그림 수만큼 \
         합산하면 행이 통째로 다음 쪽으로 밀리거나 행 중간에서 잘린다. 1쪽 그림 {images:?}"
    );
    for img in &images {
        assert!(
            img.y + 0.5 >= fragment.y && img.bottom() <= fragment.bottom() + 0.5,
            "그림이 1쪽 표 조각 밖으로 나간다: 그림 {img:?}, 조각 {fragment:?}"
        );
    }
    let band_top = images.iter().map(|r| r.y).fold(f64::MIN, f64::max);
    let band_bottom = images.iter().map(Rect::bottom).fold(f64::MAX, f64::min);
    assert!(
        band_top < band_bottom,
        "나란한 그림 3장은 같은 띠에 있어야 한다(세로로 겹치는 구간이 있어야 함): {images:?}"
    );
}
