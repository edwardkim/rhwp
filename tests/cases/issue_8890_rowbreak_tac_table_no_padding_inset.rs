//! 관계형: 저장 줄 없는 공백 문단의 첫 RowBreak TAC 표의 가로 위치가 표 안 여백에 기대지 않는다(#8890).
//! 주석 갱신: 2026-11-10 14:30 KST(+09:00).
//! 측정: 2026-11-10, `scripts/visual_sweep.py` 실루엣 raw(4열 `silhouette_raw_match_percent`), 96DPI,
//!   source 9eedf98ecc(base upstream/devel 3170fddc48). 전체 2쪽(한/글 PDF 2쪽과 일치).
//!   1쪽 Native 92.41 / fresh WASM 92.83 (수정 전 devel Native 80.07), 2쪽 양쪽 100.00.
//!   전쪽 최저 Native 92.41 · fresh WASM 92.83. 1쪽 잔여 차이는 머리 로고 그림 크기(이 변경 밖).
//!   증거: `mydocs/pr/assets/issue_8890/{native,wasm,base-native}/36400288_silhouette.tsv`,
//!   `mydocs/pr/assets/issue_8890/36400288_p1_review.png`, `36400288_p1_overlay.png`.
//! 독립 기준: 한/글 2022(12.0.0.4547) Print PDF `pdf/issue8890/36400288_approval-hancom2022.pdf` 1쪽.
//!   6×4 표 세로 괘선 204.2 / 313.8 / 601.6px(96DPI) = 선언 열 너비(7704·8226·21604HU)를
//!   표 왼쪽 101.5px 에서 누적한 값. 그 왼쪽은 공백 3자 전진 + 바깥 여백뿐이고 안 여백 inset 이 없다.
//! 설치 글꼴 없이 검사한다. 좌표를 고정하지 않고 안 여백을 바꾼 같은 문서와의 관계만 본다.

use rhwp::{
    model::control::Control,
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

const INPUT: &str = "samples/issue8890/36400288_approval.hwpx";
/// 6×4 표 host 문단(공백 3자 + TAC 표, 저장 줄 없음).
const HOST: usize = 9;

fn table_left(core: &DocumentCore) -> f64 {
    fn walk(node: &RenderNode, out: &mut Vec<f64>) {
        if let RenderNodeType::Table(table) = &node.node_type {
            if table.row_count == 6 && table.col_count == 4 {
                out.push(node.bbox.x);
            }
        }
        for child in &node.children {
            walk(child, out);
        }
    }
    let page = core.build_page_render_tree(0).unwrap();
    let mut found = Vec::new();
    walk(&page.root, &mut found);
    assert_eq!(found.len(), 1, "1쪽의 6×4 표는 하나다");
    found[0]
}

fn load() -> DocumentCore {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    DocumentCore::from_bytes(&std::fs::read(root.join(INPUT)).expect("#8890 표본")).unwrap()
}

#[test]
fn rowbreak_tac_table_left_does_not_follow_cell_padding() {
    let original = load();
    let paragraph = &original.document().sections[0].paragraphs[HOST];
    // 원본 계약: 저장 줄이 없는 공백 전용 host, 첫 control 은 셀 단위로 나뉘는 TAC 표.
    assert!(
        paragraph.line_segs.is_empty()
            || paragraph.line_segs.iter().all(|s| s.tag & 0x8000_0000 != 0)
    );
    assert!(!paragraph.text.is_empty() && paragraph.text.chars().all(char::is_whitespace));
    let Control::Table(table) = &paragraph.controls[0] else {
        panic!("host 의 첫 control 은 표");
    };
    assert!(table.common.treat_as_char);
    assert_eq!((table.padding.left, table.padding.right), (510, 510));
    let left = table_left(&original);

    // 같은 문서에서 칸 안 여백만 좌우 대칭으로 줄인다. 안 여백은 칸 안 내용의 자리이며
    // 표 외곽의 가로 위치를 옮기지 않는다.
    let mut doc = load().document().clone();
    let Control::Table(table) = &mut doc.sections[0].paragraphs[HOST].controls[0] else {
        unreachable!()
    };
    table.padding.left = 141;
    table.padding.right = 141;
    let mut narrow = DocumentCore::new_empty();
    narrow.set_document(doc);
    let narrow_left = table_left(&narrow);
    assert!(
        (left - narrow_left).abs() < 0.01,
        "표 왼쪽 {left} 이 안 여백을 줄이자 {narrow_left} 로 움직였다"
    );
}
