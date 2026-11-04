//! 관계형·고정형 회귀: 사진의 포함·열 정렬 관계와 독립 Print의 2열·3줄 개수.
//! 주석 갱신: 2026-11-04T21:32:48+09:00; 직전 파일 커밋: 2026-11-04T17:50:30+09:00 (c312ea2c87cddbb8608f649a0d742eac1ab6107c).
//! 2026-11-04 원 head 7e7a86fef9 측정: Visual Sweep raw, 96dpi, PDFium, 색상 경계 보정 전 2px 내용 실루엣.
//! textbox_tac_picture_rows.hwp 전 1쪽(1쪽): Native 96.84744% / fresh optimized WASM 96.85057%; 각 전쪽 최저 동일.
//! 정식 근거: mydocs/pr/assets/pr8585/head-native.tsv, head-wasm.tsv와 대응 provenance·PNG; 독립 PDF는 pdf/issue7524/.
//! 원 head의 마지막 사진 줄 정렬은 아직 다르며, 해당 관계 검사는 원 head FAIL을 확인한다. 보정 후 실측은 검토 기록에 연결한다.
//!
//! [#7524] 저장 줄이 없는 글상자 문단에서 여러 글자처럼 취급 그림이 한 줄로 늘어서 상자 밖으로
//! 나가던 결함의 회귀 가드.
//!
//! # 무엇이 깨져 있었나
//!
//! 글상자 문단의 fresh 조판이 비-TAC 그림이 없는 문단을 프레임에서 사양해 45자 합성
//! 줄바꿈이 남았다. 합성 줄바꿈은 그림 폭을 모르므로 그림 6장이 한 줄이 되고, 글상자
//! 배치는 TAC 개체를 모두 첫 개체의 줄에 두고 x 를 누적해 상자 오른쪽 밖으로 내보냈다.
//!
//! # 기대값의 출처
//!
//! 재현물은 실문서(기관 서식, 저장 LINE_SEG 없음)를 글자 치환·그림 교체로 익명화한 것이다.
//! 같은 파일을 한컴 웹한글기안기로 연 인쇄 출력(PDF)에서 글상자 안 그림 6장은 상자 폭에 맞춰
//! 한 줄에 2장씩 3줄로 놓이고, 모두 상자 안에 있다.
//!
//! # 검사하는 것
//!
//! 글상자 안 큰 그림들의 상자 포함, 서로 겹치지 않음, 여러 줄로 나뉨(관계 검사).

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue7524/textbox_tac_picture_rows.hwp";
/// 도장 같은 작은 그림을 빼고 사진만 고르는 폭 하한(px).
const PHOTO_MIN_WIDTH: f64 = 100.0;

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
    fn right(&self) -> f64 {
        self.x + self.w
    }
    fn bottom(&self) -> f64 {
        self.y + self.h
    }
    fn overlaps(&self, other: &Rect) -> bool {
        self.x + 0.5 < other.right()
            && other.x + 0.5 < self.right()
            && self.y + 0.5 < other.bottom()
            && other.y + 0.5 < self.bottom()
    }
}

/// (글상자, 그 안의 사진들) 중 사진이 가장 많은 글상자.
fn photo_textbox(node: &RenderNode, best: &mut Option<(Rect, Vec<Rect>)>) {
    if matches!(node.node_type, RenderNodeType::TextBox) {
        let mut photos = Vec::new();
        collect_photos(node, &mut photos);
        if best.as_ref().is_none_or(|(_, p)| photos.len() > p.len()) {
            *best = Some((Rect::of(node), photos));
        }
    }
    for child in &node.children {
        photo_textbox(child, best);
    }
}

fn collect_photos(node: &RenderNode, out: &mut Vec<Rect>) {
    if matches!(node.node_type, RenderNodeType::Image(_)) && node.bbox.width >= PHOTO_MIN_WIDTH {
        out.push(Rect::of(node));
    }
    for child in &node.children {
        collect_photos(child, out);
    }
}

fn load_photos() -> (Rect, Vec<Rect>) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).expect("재현물 읽기");
    let doc = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let tree = doc.build_page_render_tree(0).expect("1쪽 렌더 트리");
    let mut best = None;
    photo_textbox(&tree.root, &mut best);
    let (textbox, photos) = best.expect("사진이 든 글상자가 없다 — 시험 설정 오류");
    assert_eq!(
        photos.len(),
        6,
        "글상자 안 사진은 6장이어야 한다: {photos:?}"
    );
    (textbox, photos)
}

#[test]
fn photos_stay_inside_the_textbox() {
    let (textbox, photos) = load_photos();
    for photo in &photos {
        assert!(
            photo.x + 0.5 >= textbox.x
                && photo.right() <= textbox.right() + 0.5
                && photo.y + 0.5 >= textbox.y
                && photo.bottom() <= textbox.bottom() + 0.5,
            "사진이 글상자 밖으로 나간다 — 그림 폭을 모르는 합성 줄바꿈이 한 줄에 몰았거나, \
             세로 정렬이 한 줄 높이만 쳤다. 상자 {textbox:?}, 사진 {photo:?}"
        );
    }
}

#[test]
fn photos_wrap_into_rows_without_overlap() {
    let (_, photos) = load_photos();
    let mut tops: Vec<i64> = photos.iter().map(|p| p.y.round() as i64).collect();
    tops.sort_unstable();
    tops.dedup();
    assert_eq!(
        tops.len(),
        3,
        "독립 Print의 사진 6장은 2열·3줄이다: {tops:?}"
    );
    let mut columns: Option<Vec<f64>> = None;
    for top in tops {
        let mut xs: Vec<f64> = photos
            .iter()
            .filter(|photo| photo.y.round() as i64 == top)
            .map(|photo| photo.x)
            .collect();
        xs.sort_by(f64::total_cmp);
        assert_eq!(
            xs.len(),
            2,
            "각 사진 줄은 같은 두 열을 소유해야 한다: {xs:?}"
        );
        if let Some(expected) = &columns {
            for (actual, previous) in xs.iter().zip(expected) {
                assert!(
                    (actual - previous).abs() <= 0.5,
                    "말미 공백은 사진 열의 정렬 원점을 바꾸지 않는다: {expected:?} / {xs:?}"
                );
            }
        } else {
            columns = Some(xs);
        }
    }
    for (i, a) in photos.iter().enumerate() {
        for b in &photos[i + 1..] {
            assert!(!a.overlaps(b), "사진끼리 겹친다: {a:?} / {b:?}");
        }
    }
}
