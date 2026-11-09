//! 저장 줄 없는 제목 문단(글앞 표 + 용지 기준 어울림 그림을 소유)이 쪽 끝에 걸릴 때의 쪽 나눔.
//!
//! # 무엇이 깨져 있었나
//!
//! 저장 LINE_SEG 가 없는 주간 점검표 서식은 쪽마다 "제목 문단 + 본문" 을 되풀이한다. 제목 문단은
//! 오른쪽 위 결재 칸(글앞으로 표)과 도장(용지 기준 어울림 그림)을 소유한다. 제목 글줄이 현재 쪽
//! 남은 자리에 들어가지 않아도 rhwp 는 그 글줄과 결재 칸을 현재 쪽 맨 아래에 그렸다. 다음 쪽 맨
//! 위에는 제목이 없고, 현재 쪽 글줄이 본문 영역 아래로 넘쳤다.
//!
//! 글앞 표 host 의 글줄은 데코 경로가 fit 없이 방출한다. 저장 줄 없는 글앞 표 host 의 사전 이월
//! 판정은 모든 개체가 글앞/글뒤일 때만 걸려, 용지 기준 어울림 그림이 함께 있으면 빠졌다.
//!
//! # 기대값의 출처
//!
//! 재현물 두 개는 공개 샘플(test-image.hwpx 그림, issue3738 표)을 조합한 합성 문서다. 제목
//! 문단 뒤 줄 수와 두 번째 결재 칸 높이가 다르다(`title_line_fits` 는 결재 칸을 5677→2600 HU 로
//! 줄여, 제목 글줄 높이에서 시작해도 본문 아래끝 안에서 끝난다). 같은 파일을 웹한글기안기로 연
//! 인쇄 PDF 에서:
//!
//! - 빈 줄 38개(`title_line_overflows`): 2쪽. 두 번째 제목은 2쪽 맨 위에 있고 1쪽에는 없다.
//! - 빈 줄 37개(`title_line_fits`): 2쪽. 두 번째 제목 글줄은 1쪽 맨 아래에 들어가고, 결재 칸도
//!   그 제목과 같은 1쪽에 있다.
//!
//! # 검사하는 것
//!
//! 쪽 수, 두 번째 제목 글줄의 쪽 소속, 그 쪽 본문 영역 안에 들어가는지, 제목과 결재 칸이 같은
//! 쪽인지, 다음 본문 줄보다 위인지. 쪽 아래로 넘치는 글앞 표를 rhwp 가 본문 아래끝 안으로 당겨
//! 그리는 기전(한컴은 넘친 채 둠)은 쪽 나눔과 별개라, `title_line_fits` 의 결재 칸은 넘치지 않게
//! 두어 그 기전이 섞이지 않게 했다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const OVERFLOW_SAMPLE: &str = "samples/issue8665/title_line_overflows.hwpx";
const FITS_SAMPLE: &str = "samples/issue8665/title_line_fits.hwpx";

#[derive(Clone, Copy, Debug)]
struct Rect {
    y: f64,
    h: f64,
}

impl Rect {
    fn of(node: &RenderNode) -> Self {
        Self {
            y: node.bbox.y,
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

fn body(node: &RenderNode) -> Option<Rect> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(Rect::of(node));
    }
    node.children.iter().find_map(body)
}

fn line_text(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        line_text(child, out);
    }
}

/// 단 직속 글줄(표·도형 안 글줄 제외) 중 글자가 `needle` 로 시작하는 줄.
fn column_line(node: &RenderNode, needle: &str) -> Option<Rect> {
    if matches!(node.node_type, RenderNodeType::Column(_)) {
        return node.children.iter().find_map(|child| {
            if !matches!(child.node_type, RenderNodeType::TextLine(_)) {
                return None;
            }
            let mut text = String::new();
            line_text(child, &mut text);
            text.trim_start()
                .starts_with(needle)
                .then(|| Rect::of(child))
        });
    }
    node.children.iter().find_map(|c| column_line(c, needle))
}

/// 쪽의 결재 칸 표들(셀 안 표 제외). 재현물의 표는 결재 칸뿐이다.
fn overlay_tables(node: &RenderNode, out: &mut Vec<Rect>) {
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        out.push(Rect::of(node));
        return;
    }
    for child in &node.children {
        overlay_tables(child, out);
    }
}

/// (두 번째 제목이 있는 쪽, 그 쪽의 제목 글줄, 본문 영역, 그 쪽 표들, 첫 본문 줄)
fn second_title(doc: &DocumentCore) -> (u32, Rect, Rect, Vec<Rect>, Option<Rect>) {
    let mut found = Vec::new();
    for page in 0..doc.page_count() {
        let tree = doc.build_page_render_tree(page).expect("렌더 트리");
        if let Some(line) = column_line(&tree.root, "TITLE 2") {
            let mut tables = Vec::new();
            overlay_tables(&tree.root, &mut tables);
            found.push((
                page,
                line,
                body(&tree.root).expect("본문 영역"),
                tables,
                column_line(&tree.root, "2-1"),
            ));
        }
    }
    assert_eq!(found.len(), 1, "두 번째 제목은 한 쪽에만 그려져야 한다");
    found.pop().unwrap()
}

#[test]
fn title_line_that_does_not_fit_moves_to_next_page_top() {
    let doc = load(OVERFLOW_SAMPLE);
    assert_eq!(doc.page_count(), 2, "웹한글기안기 기준 2쪽");
    let (page, title, body, tables, first_body_line) = second_title(&doc);
    assert_eq!(
        page, 1,
        "제목 글줄이 1쪽 남은 자리에 들지 않으면 2쪽으로 간다"
    );
    assert!(
        title.bottom() <= body.bottom() + 0.5,
        "제목 글줄은 본문 영역 안에 있어야 한다: {title:?} / {body:?}"
    );
    let first_body_line = first_body_line.expect("2쪽 첫 본문 줄");
    assert!(
        title.bottom() <= first_body_line.y + 0.5,
        "제목은 그 아래 본문 줄과 겹치지 않아야 한다: {title:?} / {first_body_line:?}"
    );
    assert_eq!(tables.len(), 1, "결재 칸은 제목과 같은 2쪽에 있어야 한다");
    assert!(
        (tables[0].y - title.y).abs() < title.h,
        "결재 칸은 제목 줄 높이에서 시작한다: {:?} / {title:?}",
        tables[0]
    );
}

#[test]
fn title_line_that_fits_stays_on_current_page() {
    let doc = load(FITS_SAMPLE);
    assert_eq!(doc.page_count(), 2, "웹한글기안기 기준 2쪽");
    let (page, title, body, tables, _) = second_title(&doc);
    assert_eq!(
        page, 0,
        "제목 글줄이 들어가면 1쪽에 남는다"
    );
    assert!(
        title.bottom() <= body.bottom() + 0.5,
        "제목 글줄은 본문 영역 안에 있어야 한다: {title:?} / {body:?}"
    );
    assert_eq!(
        tables.len(),
        2,
        "두 번째 결재 칸도 제목과 같은 1쪽에 있어야 한다: {tables:?}"
    );
}
