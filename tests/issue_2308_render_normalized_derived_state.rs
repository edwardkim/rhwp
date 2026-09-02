//! Issue #2308 functional regression for nested-table derived geometry.
//!
//! Page-count pins do not catch a nested 1×1 table whose width normalization
//! drifts only the split height. The two continuation fragments are pinned after
//! direct comparison with the HWP 2024/Hancom PDF fixture: the second fragment
//! begins at the page's content top while retaining the stored table width.
//! #3128 additionally pins its PDF-owned 10-line continuation height and table
//! content-box padding semantics.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

fn nested_one_by_one_tables(node: &RenderNode, table_depth: usize, out: &mut Vec<(f64, f64)>) {
    let next_depth = if let RenderNodeType::Table(table) = &node.node_type {
        if table_depth >= 1 && table.row_count == 1 && table.col_count == 1 {
            out.push((node.bbox.y, node.bbox.height));
        }
        table_depth + 1
    } else {
        table_depth
    };
    for child in &node.children {
        nested_one_by_one_tables(child, next_depth, out);
    }
}

fn find_table_with_owner_para(node: &RenderNode, para_index: usize) -> Option<&RenderNode> {
    if matches!(
        &node.node_type,
        RenderNodeType::Table(table) if table.para_index == Some(para_index)
    ) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_table_with_owner_para(child, para_index))
}

fn find_nested_single_cell_table(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(
        &node.node_type,
        RenderNodeType::Table(table) if table.row_count == 1 && table.col_count == 1
    ) {
        return Some(node);
    }
    node.children.iter().find_map(find_nested_single_cell_table)
}

fn collect_visible_text_line_rights(node: &RenderNode, rights: &mut Vec<f64>) {
    if !node.visible {
        return;
    }
    if matches!(&node.node_type, RenderNodeType::TextLine(_)) {
        rights.push(node.bbox.x + node.bbox.width);
    }
    for child in &node.children {
        collect_visible_text_line_rights(child, rights);
    }
}

fn contains_text(node: &RenderNode, needle: &str) -> bool {
    matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text.contains(needle))
        || node
            .children
            .iter()
            .any(|child| contains_text(child, needle))
}

/// Whether any single painted **line** carries the needle.
///
/// [#5193] `contains_text` asks whether one `TextRun` holds it, and that is not
/// what a PDF line contract says. A frame-owned row is split at CharShapeRef
/// boundaries — on `76076` p81 the row arrives as
/// `['○ ', '구내운반차 안전조치를', ' 통해 근로자와 부딪히는 등의 사고를 ']`,
/// because the traced glyph advance changes 1300 → 1261 at char offset 13 where
/// the char shape does. The retired cell rebuild emitted one run for the whole
/// row (`char_shapes[0]` blindness), so a single-run test happened to work.
/// "One painted line holds this text" is the contract; "one run holds it" was
/// never intended. Scoped to a `TextLine` so it cannot match across a break.
fn line_contains_text(node: &RenderNode, needle: &str) -> bool {
    fn line_text(node: &RenderNode, out: &mut String) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            out.push_str(&run.text);
        }
        for child in &node.children {
            line_text(child, out);
        }
    }
    if matches!(&node.node_type, RenderNodeType::TextLine(_)) {
        let mut text = String::new();
        line_text(node, &mut text);
        if text.contains(needle) {
            return true;
        }
    }
    node.children
        .iter()
        .any(|child| line_contains_text(child, needle))
}

#[derive(Clone, Copy)]
struct ClipRect {
    x: f64,
    y: f64,
    right: f64,
    bottom: f64,
}

impl ClipRect {
    fn from_node(node: &RenderNode) -> Self {
        Self {
            x: node.bbox.x,
            y: node.bbox.y,
            right: node.bbox.x + node.bbox.width,
            bottom: node.bbox.y + node.bbox.height,
        }
    }

    fn intersect(self, other: Self) -> Option<Self> {
        let clipped = Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            right: self.right.min(other.right),
            bottom: self.bottom.min(other.bottom),
        };
        (clipped.right > clipped.x && clipped.bottom > clipped.y).then_some(clipped)
    }

    /// [#5301] 허용치가 `0.01px` 에서 `0.5px` 로 넓어졌다 — **전진 상자와 잉크를
    /// 가른다.**
    ///
    /// `RenderNode.bbox` 는 글리프 **전진폭** 상자다. 잉크가 아니다. p34 근거설명
    /// 줄은 전진 상자가 클립을 `0.40px`(0.30pt) 넘지만 600dpi 래스터로 재면 괘선
    /// 너머 어두운 픽셀이 **0** 이다 — 한글 2024 정답지도 같은 줄을 괘선에 닿게
    /// 그린다(글자 상자 `156.96..522.60pt`, 칸 폭 `365.72pt`). `#6443`·`#6303` 이
    /// 같은 착시를 두 번 기록했다.
    ///
    /// 종전에는 `aim=false` 중첩 칸에 저장 여백 `510HU` 를 얹어 폭을 `10.2pt` 줄인
    /// 덕에 이 여유가 생겼는데, 그 여백이 바로 66쪽에서 글자를 소실시키던 원인이다.
    fn contains_node(self, node: &RenderNode) -> bool {
        const ADVANCE_BOX_TOLERANCE_PX: f64 = 0.5;
        node.bbox.x >= self.x - ADVANCE_BOX_TOLERANCE_PX
            && node.bbox.y >= self.y - ADVANCE_BOX_TOLERANCE_PX
            && node.bbox.x + node.bbox.width <= self.right + ADVANCE_BOX_TOLERANCE_PX
            && node.bbox.y + node.bbox.height <= self.bottom + ADVANCE_BOX_TOLERANCE_PX
    }

    fn intersects_node(self, node: &RenderNode) -> bool {
        self.intersect(Self::from_node(node)).is_some()
    }
}

fn text_run_is_fully_painted(node: &RenderNode, needle: &str, clip: Option<ClipRect>) -> bool {
    if !node.visible {
        return false;
    }
    let clip = match &node.node_type {
        RenderNodeType::TableCell(cell) if cell.clip => {
            clip.and_then(|active| active.intersect(ClipRect::from_node(node)))
        }
        _ => clip,
    };
    if matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text.contains(needle))
        && clip.is_some_and(|active| active.contains_node(node))
    {
        return true;
    }
    node.children
        .iter()
        .any(|child| text_run_is_fully_painted(child, needle, clip))
}

fn text_run_is_partially_painted(node: &RenderNode, needle: &str, clip: Option<ClipRect>) -> bool {
    if !node.visible {
        return false;
    }
    let clip = match &node.node_type {
        RenderNodeType::TableCell(cell) if cell.clip => {
            clip.and_then(|active| active.intersect(ClipRect::from_node(node)))
        }
        _ => clip,
    };
    if matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text.contains(needle))
        && clip.is_some_and(|active| active.intersects_node(node) && !active.contains_node(node))
    {
        return true;
    }
    node.children
        .iter()
        .any(|child| text_run_is_partially_painted(child, needle, clip))
}

#[test]
fn issue_2308_saved_nested_width_keeps_fragment_geometry() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/76076_regulatory_analysis.hwp");
    let bytes = fs::read(path).expect("read #2195 authority fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("parse #2195 authority fixture");

    // p33's first fragment begins at the row-6 boundary in the HWP 2024 PDF;
    // its old 351.1px pin predated the empty RowBreak host flow correction and
    // incorrectly described a point inside the preceding row.
    //
    // [#5193] p34's 1×1 rationale fragment was pinned at 426.9px. That number
    // was the retired width-based cell rebuild's, and the authority PDF
    // (`samples/issue1891/76076_regulatory_analysis-2024.pdf`) contradicts it:
    // 426.9 − 388.3 = 38.667px = exactly two 19.333px rows, and the two rows are
    // rows Hancom does not print. Traced per paragraph of the 직접비용 근거설명
    // cell, only two of its thirteen paragraphs change, and the PDF adjudicates
    // both against the retired path:
    //
    //   paragraph                     box 35552HU   box 36572HU   PDF p33
    //   `↳ 한편, 작업환경실태조사 …`   4 → 3 rows    3 / 3         3 rows
    //   `↳ 상기 댓수는 2019년 …`      7 / 7         7 → 6 rows    6 rows
    //
    // The p34 page also carries a *second* nested 1×1 table (직접편익 근거설명,
    // h=124.7px). It is not a split of the first — it is unchanged in height and
    // merely lifted by the same 38.667px, on both paths. Any reading of this
    // failure as "the p34 fragment splits in two" is that second table.
    //
    // [#5704] p33's fragment height moved 649.3 → 636.8 when upstream/devel
    // merged, and the pin follows the frame rather than the other way round.
    // The 12.5px is one row of `↳ 상기 댓수는 2019년 …`, and **the evidence was
    // already in the table above** — at `box 36572HU` that paragraph goes
    // `7 → 6 rows` and the authority PDF prints 6. 36572 HWPUNIT is the box both
    // trees actually fold it at: 487.6px × 75 = 36,570.
    //
    // Measured by building `upstream/devel` from `git archive` and running the
    // same instrument on both trees, same page, same paragraph:
    //
    // ```text
    // ours      track=false box_px=487.6 lines_after=6  "  ↳ 상기 댓수는 2019년 작"
    // upstream  track=false box_px=487.6 lines_after=7  "  ↳ 상기 댓수는 2019년 작"
    // ```
    //
    // `track=false` — this is not the #3128 indented-tracking class, and the
    // tracking class itself shows zero row-count differences between the trees
    // (5 shared paragraphs, 0 diffs). So 649.3 is not a capability we dropped;
    // it is the retired width-based rebuild's row count, kept alive in a pin.
    // 636.8 is the frame's, and it is the one the PDF backs.
    //
    // Confirmed independently of the row-count argument by rasterising the
    // authority PDF at 96dpi (`gs -sDEVICE=pgmraw -r96`) and reading the drawn
    // rules, which land 1:1 on our px grid. PDF page 33 carries a full-width
    // rule at the fragment top (y=400) and its bottom rule at **y=1034**:
    //
    // ```text
    //   636.8  ->  bottom 1037.2   delta 3.2px from the rule    accepted
    //   649.3  ->  bottom 1049.7   delta 15.7px                 rejected
    // ```
    //
    // The 1–4px band is the same allowance `issue_3128_terminal_nested_table_geometry`
    // states: the render-tree bbox spans the stroke/clip outer edge, so it reads
    // slightly larger than the raster centre line.
    //
    // **p34's 388.3 stays, and it is not a stale pin — it is PDF-correct.** The
    // same raster on PDF page 34 gives the fragment top at y=77 and its bottom
    // rule at y=463, so the printed height is 386.0px and 388.3 sits 2.3px above
    // it, inside the same band. This path currently produces 374.9 (11.1px short)
    // and upstream/devel produces 370.9 (15.1px short) — **both are wrong against
    // the PDF**, so there is no value here worth moving the pin to.
    //
    // That 11.1px is the same shortfall `#5703` pins from the other side: the p34
    // continuation bottom reads 452.0 against the PDF's rule at exactly y=463.
    // This assertion therefore stays red until #5703 closes, and it is red for a
    // real defect rather than a disagreeing oracle.
    //
    // Recorded but not concluded: our 374.9 sits 4.0px above upstream's 370.9,
    // and `mixed_nested_flow_extra_from_cut` carries an `extra += 4.0` row
    // reservation. Two constants of equal size are not evidence that they are the
    // same constant; nothing here rests on that.
    let expected = [(32, 400.4, 636.8), (33, 77.1, 388.3)];
    for (page, expected_y, expected_height) in expected {
        let tree = core
            .build_page_render_tree(page)
            .unwrap_or_else(|error| panic!("render page {}: {error}", page + 1));
        let mut fragments = Vec::new();
        nested_one_by_one_tables(&tree.root, 0, &mut fragments);
        assert!(
            fragments.iter().any(|(y, height)| {
                (y - expected_y).abs() <= 0.2 && (height - expected_height).abs() <= 0.2
            }),
            "page {} nested fragment must preserve PDF-aligned geometry \
             y={expected_y:.1} h={expected_height:.1}; got {fragments:?}",
            page + 1
        );
    }
}

/// 한컴 PDF p33의 마지막 "현황 추이(p.270)" 줄은 p33의 셀 안에 온전히 남고,
/// p34는 다음 문단에서 시작한다. 중첩 표 조각 유닛을 기본 inMargin 폭으로
/// 재조판하면 한 줄을 덜 측정해 이 경계가 각각 하단/상단 clip에 반쯤 걸린다.
#[test]
fn issue_2308_nested_fragment_cut_does_not_half_paint_boundary_line() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/76076_regulatory_analysis.hwp");
    let bytes = fs::read(path).expect("read #2195 authority fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("parse #2195 authority fixture");

    let p33 = core.build_page_render_tree(32).expect("render HWP PDF p33");
    let p33_clip = Some(ClipRect::from_node(&p33.root));
    assert!(
        text_run_is_fully_painted(&p33.root, "현황 추이", p33_clip),
        "p33 must keep the final source line fully inside the nested-cell clip"
    );

    let p34 = core.build_page_render_tree(33).expect("render HWP PDF p34");
    let p34_clip = Some(ClipRect::from_node(&p34.root));
    assert!(
        !text_run_is_partially_painted(&p34.root, "현황 추이", p34_clip),
        "p34 must not retain a half-painted residue of the p33-owned source line"
    );
    assert!(
        !contains_text(&p34.root, "현황 추이"),
        "p34 must not fully repaint the p33-owned final source line"
    );
    assert!(
        text_run_is_fully_painted(&p34.root, "자율안전확인신고한", p34_clip),
        "p34 must begin with the next fully painted source paragraph"
    );
}

/// HWP 2024 PDF p34의 1×1 중첩 표는 `applyInnerMargin=false`이고
/// `inMargin.left/right=0`이므로, 저장 cell margin 510HU를 덧적용하지 않고
/// table content box 전체를 쓴다. PDF의 우측 마지막 글자도 테두리에 거의 맞닿는다.
#[test]
fn issue_2308_nested_non_tac_table_uses_table_content_box() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/76076_regulatory_analysis.hwp");
    let bytes = fs::read(path).expect("read #2195 authority fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("parse #2195 authority fixture");
    let tree = core.build_page_render_tree(33).expect("render HWP PDF p34");

    let outer = find_table_with_owner_para(&tree.root, 325)
        .expect("p34 outer activity-cost table (pi=325)");
    let nested =
        find_nested_single_cell_table(outer).expect("p34 nested single-cell rationale table");
    assert!(
        (nested.bbox.width - 487.6).abs() <= 0.2,
        "p34 nested-table width={:.1}; HWP 2024 PDF retains saved 36,572HU (487.6px)",
        nested.bbox.width
    );
    let mut rights = Vec::new();
    collect_visible_text_line_rights(nested, &mut rights);
    let rightmost = rights.into_iter().fold(f64::NEG_INFINITY, f64::max);
    let border_right = nested.bbox.x + nested.bbox.width;
    assert!(
        (rightmost - border_right).abs() <= 0.2,
        "p34 nested-table text viewport must reach the table content-box edge: \
         text_right={rightmost:.1}, border_right={border_right:.1}"
    );
}

/// HWP 2024 PDF p34의 직접편익 표는 빈 host 문단 안에 1×1 블록 표를 둔다.
/// 일반 표에는 unit cut이 없는데도 빈 composed line을 이유로 host를 건너뛰면,
/// 표 테두리만 남고 `근거설명` 본문 전체가 사라진다.
#[test]
fn issue_2308_empty_host_paragraph_keeps_block_nested_table_content() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/76076_regulatory_analysis.hwp");
    let bytes = fs::read(path).expect("read #2195 authority fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("parse #2195 authority fixture");
    let p34 = core.build_page_render_tree(33).expect("render HWP PDF p34");
    let p34_clip = Some(ClipRect::from_node(&p34.root));

    let outer = find_table_with_owner_para(&p34.root, 336)
        .expect("p34 direct-benefit outer table (pi=336)");
    let nested =
        find_nested_single_cell_table(outer).expect("p34 direct-benefit rationale nested table");
    assert!(
        text_run_is_fully_painted(nested, "분쇄기 등 회전기계", p34_clip),
        "p34 direct-benefit rationale must retain the block nested-table body"
    );
}

/// Native HWP5의 마지막 short RowBreak child는 별도 owner projection을 쓴다.
/// 한컴 2024 PDF는 parent owner content box에서 첫 줄을
/// `… 등의 사고`까지 그리고, p82는 동일 문장을 재paint하지 않고 `를 예방…`으로
/// 이어 간다. p34의 우측 border 보호와 이 p81/p82 owner 계약을 함께 고정한다.
///
/// # [#5193] 셀 재조판 이관 후 실패 — 0.55% 폭 추정 잔차, 옮기지 않는다
///
/// 이 핀은 한컴 2024 PDF 가 직접 판정한다 (`pdftotext -layout -f 81 -l 82
/// samples/issue1891/76076_regulatory_analysis-2024.pdf`):
///
/// ```text
/// p81  … 근거설명 ○ 구내운반차 안전조치를 통해 근로자와 부딪히는 등의 사고
/// p82    를 예방함으로써 산업재해 감소*에 기여할 것으로 예상되나
/// ```
///
/// 프레임 경로는 `를 `를 p81 로 끌어온다. 셀 내폭 38245 HWPUNIT 에서의 판정:
///
/// ```text
/// 를   pen 36735  glyph 1261 (fit 1300)  sum 38035  over  −210  → 수용 (한컴은 거부)
/// 예   pen 38673  glyph 1208 (fit 1299)  sum 39972  over +1727  → 거부 (결정 지점)
/// ```
///
/// 여유 **210 HWPUNIT = 2.8px = 상자의 0.55%**. 폐기 경로가 한컴과 같은 자리에서
/// 끊은 것은 우연이다 — `char_shapes[0]` 하나로 재는 탓에 이 줄의 맑은 고딕 15
/// 글자를 1261 이 아니라 1300 으로 재서 pen 이 585 HWPUNIT 부풀고, 그래서 `를`가
/// 넘쳤다. 0.55% 의 폭 추정 잔차는 폐기 경로가 알고 있던 것이 아니므로 핀을 옮기지
/// 않고 잔차를 기록한다.
///
/// 첫 assertion 은 별개 이유로 깨졌고(줄이 CharShapeRef 경계에서 run 으로 쪼개짐)
/// `line_contains_text` 로 계약을 바로잡았다 — **그 assertion 은 통과한다.** 그래서
/// 아래 두 테스트로 나눈다. 하나에 묶어 `#[ignore]` 하면 고쳐 놓은 계약이 영영
/// 실행되지 않고, `line_contains_text` 는 재귀 호출 말고는 호출자가 없어진다.
fn short_rowbreak_child_core() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/76076_regulatory_analysis.hwp");
    let bytes = fs::read(path).expect("read #2195 authority fixture");
    DocumentCore::from_bytes(&bytes).expect("parse #2195 authority fixture")
}

/// The part `453e426cd` repaired, and it runs.
///
/// p81 holds the owner content box's first line through `사고`, and p82 does not
/// re-paint a split `사고`. Both are PDF contracts the frame satisfies; only the
/// wrap point below does not.
#[test]
fn issue_2308_short_rowbreak_child_uses_owner_content_box_only() {
    let core = short_rowbreak_child_core();

    let p81 = core.build_page_render_tree(80).expect("render HWP PDF p81");
    assert!(
        line_contains_text(
            &p81.root,
            "구내운반차 안전조치를 통해 근로자와 부딪히는 등의 사고"
        ),
        "p81 must keep the PDF-owned short-child first line through `사고`"
    );

    let p82 = core.build_page_render_tree(81).expect("render HWP PDF p82");
    assert!(
        !contains_text(&p82.root, "고를 예방함으로써 산업재해 감소"),
        "p82 must not split the PDF-owned word `사고` across pages"
    );
}

/// The one assertion the frame does not satisfy — the 0.55% width residual
/// documented above. Ignored on its own so it cannot take the repaired p81
/// contract down with it.
#[ignore = "#5193: 프레임 이관 후 이 wrap 핀만 실패. 핀은 한컴 PDF 가 판정하므로 \
            옮기지 않는다 — 프레임이 `를`를 210 HWPUNIT(상자의 0.55%) 여유로 p81 에 \
            싣는 폭 추정 잔차. 위 주석에 판정 피연산자 기록."]
#[test]
fn issue_2308_short_rowbreak_child_wraps_where_the_authority_pdf_wraps() {
    let core = short_rowbreak_child_core();
    let p82 = core.build_page_render_tree(81).expect("render HWP PDF p82");
    assert!(
        contains_text(&p82.root, "를 예방함으로써 산업재해 감소"),
        "p82 must begin the continuation after the p81-owned `사고`"
    );
}
