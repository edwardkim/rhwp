//! [#7095] 쪽을 넘기는 1×1 `RowBreak` 표의 조각 상자가 표의 `outer_margin_top` 을 열지 않아
//! 그 표에 담긴 쪽 전체가 1.9px 위에 있었다.
//!
//! 한컴 engine 2020 정본(`pdf/tac_object_host_line_height-2020.pdf`, job
//! `94f14dc5-422a-462a-b4e6-810ef36ff98d`)을 `pdftocairo -svg` 벡터 좌표로 재면
//! (96dpi 환산, 경로 자신의 `transform` 반영):
//!
//! | 축 | 정본 | 수정 전 |
//! | --- | ---: | ---: |
//! | 2쪽 바깥 표 상자 | 47.15 .. 1042.54 | 45.60 .. 1025.70 |
//! | 3쪽 바깥 표 상자 | 47.15 .. 1042.54 (같다) | 45.60 .. 1006 |
//! | 10쪽(마지막 조각) | 47.15 .. 920.27 (줄어든다) | — |
//! | 2쪽 칸 첫 내용(제목 상자) | 57.06 | 47.2 |
//! | 2쪽 칸 마지막 내용(`※` 상자 하단) | 1032.63 | — |
//!
//! 2쪽 위 여백 9.91 / 아래 여백 9.91 로 **정확히 대칭**이고, 칸 padding 141HU(1.88px)를 뺀
//! 여유 16.0 을 반씩 나눈 예측(57.03 / 1032.66)이 실측과 **0.03px** 로 맞는다. 곧
//!
//! 1. 조각 상자 상단 = 본문 상단 + 표 `outer_margin_top`
//! 2. 마지막이 아니고 쪽 상단에서 시작하는 조각의 상자 하단
//!    = 본문 하단 − `outer_margin_bottom` − 100HU
//! 3. 칸 내용은 그 상자 안에서 칸 `valign`(이 문서는 Center)으로 배치
//!
//! 위 표의 정본 좌표는 PDF 원좌표다. 한/글 PDF 는 A4 쪽을 595×841pt 로 내 내용이 0.99895 배
//! 축소되므로, 그 척도를 걷으면 2쪽 상자 하단은 1043.6 이다. 본문 하단 1046.93 −
//! 바깥 여백 1.88 − 100HU(1.33) = 1043.72 와 맞는다. 100HU 는 여백·테두리·쪽 기하
//! 돌연변이 7종에서 흔들리지 않은 상수다(#7095).
//!
//! 1·2 는 이 시험이 잠근다. 2 를 페이지네이터 예산 없이 렌더러에만 넣으면 내용이 칸 밖으로
//! 밀린다(PR #7098 실측 overfill overflow_cell +4줄). 그래서 같은 술어로 예산에서도 뺀다.
//!
//! 반례: 표가 쪽 **중간**에서 시작하는 조각(156645214 19쪽, 표 시작 y≈217)은 늘리지 않는다 —
//! 늘리면 내용이 정본보다 8px 아래로 밀린다(정본 대비 −3 → +5). 마지막 조각도 늘리지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue7062/tac_object_host_line_height.hwp";

#[test]
fn projected_picture_fragment_does_not_claim_a_stored_page_frame() {
    let core = load_sample("samples/issue2004_cell_image_stack.hwp");
    let nodes = page_nodes(&core, 4);
    let frame = nodes
        .iter()
        .find(|node| {
            matches!(node.node_type, RenderNodeType::Table { .. })
                && (node.bbox.width - 635.0).abs() < 1.0
        })
        .expect("outer picture frame")
        .bbox;
    // Existing Hancom PDF p5: path y=143.121094pt under matrix(1,0,0,-1,0,841).
    // Normalize its 841pt page to the source 84188HU: bottom = 931.48px.
    // Allow 2px for the existing picture projection metric difference; the
    // incorrect full-page inference moves this real printed border to 1023px.
    let pdf_bottom = (841.0 - 143.121094) / 841.0 * (84188.0 / 75.0);
    assert!(
        (frame.y + frame.height - pdf_bottom).abs() < 2.0,
        "projected content frame must end at its picture: {:?}, PDF bottom={pdf_bottom}",
        frame
    );
    assert_eq!(core.page_count(), 8);
}

fn load() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open")
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

fn page_nodes(core: &DocumentCore, page_index: u32) -> Vec<RenderNode> {
    let page = core
        .build_page_render_tree(page_index)
        .expect("render tree");
    let mut refs = Vec::new();
    walk(&page.root, &mut refs);
    refs.into_iter().cloned().collect()
}

/// 본문을 담은 바깥 1×1 조각 표(폭 676px).
fn fragment_box(nodes: &[RenderNode]) -> BoundingBox {
    nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::Table { .. } if (n.bbox.width - 676.0).abs() < 2.0 => Some(n.bbox),
            _ => None,
        })
        .max_by(|a, b| a.height.total_cmp(&b.height))
        .expect("바깥 조각 표")
}

#[test]
fn issue_7095_fragment_box_opens_the_table_outer_top_margin() {
    let core = load();
    let outer = fragment_box(&page_nodes(&core, 1));

    // 상단 = 본문 상단 45.35 + outer_margin_top 141HU(1.88) = 47.2 (정본 47.15).
    // 수정 전에는 본문 상단(45.6)에 그대로 붙었다.
    assert!(
        (outer.y - 47.2).abs() < 0.5,
        "#7095: 조각 상자 상단은 본문 상단 + 바깥여백(47.2)이어야 한다 — 수정 전 45.6: {:.2}",
        outer.y
    );

    // 3쪽 조각도 같은 상단이다(같은 표의 다른 조각).
    let outer_p3 = fragment_box(&page_nodes(&core, 2));
    assert!(
        (outer_p3.y - 47.2).abs() < 0.5,
        "#7095: 3쪽 조각 상단도 47.2 여야 한다: {:.2}",
        outer_p3.y
    );

    // 비끝 조각 상자 하단 = 본문 하단 1046.93 − 바깥 아래 여백 1.88 − 100HU(1.33) = 1043.72.
    // 정본 1042.54 는 PDF 쪽 척도(0.99895)가 걸린 값이고 걷으면 1043.6 이다.
    // 수정 전에는 내용 컷에서 끝나 2쪽 1025.9 · 3쪽 1006.7 이었다.
    for (label, frag) in [("2쪽", outer), ("3쪽", outer_p3)] {
        let bottom = frag.y + frag.height;
        assert!(
            (bottom - 1043.72).abs() < 1.0,
            "#7095: {label} 비끝 조각 상자 하단은 본문 하단 − 바깥여백 − 100HU(1043.72)여야 한다: {bottom:.2}"
        );
    }
}

#[test]
fn issue_7095_last_fragment_and_downstream_contracts_hold() {
    let core = load();
    // 마지막 조각(10쪽)은 쪽 상자로 늘리지 않는다 — 정본도 내용에 맞춰 줄어든다(정본 아래 920.27).
    // 비끝 조각 상자 아래(1043.72)보다 확실히 위에서 끝나야 한다.
    let last_frag = fragment_box(&page_nodes(&core, 9));
    let last_bottom = last_frag.y + last_frag.height;
    assert!(
        (last_frag.y - 47.2).abs() < 0.5 && last_bottom < 1000.0,
        "#7095: 마지막 조각도 같은 상단(47.2)이고 내용에 맞춰 줄어든다: y={:.2} bottom={:.1}",
        last_frag.y,
        last_bottom
    );

    // #7079 계약: 도해 그림 → 뒤 표 간격 410.0(저장 사다리)은 이 변경과 무관하다.
    let nodes = page_nodes(&core, 1);
    let image = nodes
        .iter()
        .find_map(|n| match &n.node_type {
            RenderNodeType::Image(_) if (n.bbox.height - 400.0).abs() < 1.0 => Some(n.bbox),
            _ => None,
        })
        .expect("도해 그림");
    let table = nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::Table { .. } if n.bbox.y > image.y + 100.0 => Some(n.bbox),
            _ => None,
        })
        .min_by(|a, b| a.y.total_cmp(&b.y))
        .expect("그림 뒤 표");
    assert!(
        (table.y - image.y - 410.0).abs() < 1.0,
        "#7095: 그림 → 뒤 표 410.0(#7079)은 유지되어야 한다: {:.1}",
        table.y - image.y
    );

    assert_eq!(core.page_count(), 10, "#7095: 쪽수는 10 이어야 한다");
}

fn load_sample(rel: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open")
}

/// `para_index` 가 같은 표 노드의 상자들.
fn tables_of_para(nodes: &[RenderNode], para: usize, rows: u16, cols: u16) -> Vec<BoundingBox> {
    nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::Table(t)
                if t.para_index == Some(para) && t.row_count == rows && t.col_count == cols =>
            {
                Some(n.bbox)
            }
            _ => None,
        })
        .collect()
}

#[test]
fn issue_7095_first_fragment_starting_at_page_top_is_pinned_too() {
    // 30269 10쪽은 표(pi136)의 **첫** 조각인데 쪽 상단에서 시작한다. 정본 상자 아래는
    // 본문 아래 1028.01 − 바깥 아래 여백 283HU(3.77) − 100HU(1.33) = 1022.91 이다.
    // 수정 전에는 내용 행 높이(마지막 줄 뒤 줄간격 포함)로 끝나 1028.3, 이어짐 조건으로만
    // 고정하면 1032.1 로 정본 상자를 9px 넘었다.
    let core = load_sample("samples/issue6023/30269_reform_recommendation.hwp");
    let boxes = tables_of_para(&page_nodes(&core, 9), 136, 1, 1);
    let frag = boxes.first().expect("30269 10쪽 조각 표");
    let bottom = frag.y + frag.height;
    assert!(
        (frag.y - 98.27).abs() < 1.0 && (bottom - 1022.91).abs() < 1.0,
        "#7095: 30269 10쪽 조각 상자는 위 98.27 · 아래 1022.91 이어야 한다: y={:.2} bottom={:.2}",
        frag.y,
        bottom
    );
    assert_eq!(
        core.page_count(),
        22,
        "#7095: 30269 쪽수는 정본과 같은 22 여야 한다"
    );
}

/// Edited IR flow contract, separate from the unmodified Hancom fixtures above.
/// Page height is varied across a row-fitting boundary; every numbered unit must
/// survive once inside its own fragment and following text must remain after it.
#[test]
fn fragment_budget_preserves_units_with_visible_and_empty_hosts() {
    use rhwp::model::{
        control::Control,
        paragraph::Paragraph,
        shape::TextWrap,
        table::{Cell, Table, TablePageBreak},
    };
    for visible_host in [false, true] {
        for page_height in (21000..=24000).step_by(100) {
            let mut core = load();
            let mut doc = core.document().clone();
            doc.sections.truncate(1);
            let section = &mut doc.sections[0];
            section.section_def.page_def.height = page_height;
            section.section_def.page_def.margin_top = 1500;
            section.section_def.page_def.margin_bottom = 1500;
            section.section_def.page_def.margin_header = 0;
            section.section_def.page_def.margin_footer = 0;
            let make_para = |text: String| {
                let mut p = Paragraph::new_empty();
                p.char_count = text.encode_utf16().count() as u32 + 1;
                p.char_offsets = (0..text.len() as u32).collect(); // ASCII contract markers
                p.text = text;
                p.invalidate_layout_inputs();
                p
            };
            let mut cell = Cell::new_empty(0, 0, 30000, 60000, 0);
            cell.paragraphs = (0..40).map(|i| make_para(format!("UNIT{i:02}"))).collect();
            let mut table = Table {
                row_count: 1,
                col_count: 1,
                row_sizes: vec![1],
                cells: vec![cell],
                page_break: TablePageBreak::RowBreak,
                outer_margin_top: 141,
                outer_margin_bottom: 900,
                ..Default::default()
            };
            table.common.width = 30000;
            table.common.height = 60000;
            table.common.vertical_offset = if visible_host { 6000 } else { 0 };
            table.common.text_wrap = TextWrap::TopAndBottom;
            table.common.vert_rel_to = rhwp::model::shape::VertRelTo::Para;
            table.rebuild_grid();
            let mut host = make_para(if visible_host {
                "HOST".into()
            } else {
                String::new()
            });
            host.char_count += 8; // table control follows the host text
            host.controls = vec![Control::Table(Box::new(table))];
            if visible_host {
                assert_eq!(
                    host.control_text_positions(),
                    vec![4],
                    "visible-host placement must own the text-tail control"
                );
            }
            section.paragraphs = vec![host, make_para("AFTER".into())];
            core.set_document(doc);
            let mut found = vec![0; 40];
            let mut last_table = None;
            let mut following = None;
            fn inspect(
                n: &RenderNode,
                page: u32,
                bounds: Option<BoundingBox>,
                found: &mut [usize],
                last: &mut Option<(u32, f64)>,
                after: &mut Option<(u32, f64)>,
            ) {
                let bounds = if matches!(n.node_type, RenderNodeType::TableCell(_)) {
                    Some(n.bbox)
                } else {
                    bounds
                };
                if matches!(n.node_type, RenderNodeType::Table(_)) {
                    *last = Some((page, n.bbox.y + n.bbox.height));
                }
                if let RenderNodeType::TextRun(t) = &n.node_type {
                    if let Some(index) = t
                        .text
                        .strip_prefix("UNIT")
                        .and_then(|s| s.parse::<usize>().ok())
                    {
                        found[index] += 1;
                        let cell = bounds.expect("unit must stay in a cell");
                        assert!(
                            n.bbox.y >= cell.y - 0.5
                                && n.bbox.y + n.bbox.height <= cell.y + cell.height + 0.5,
                            "unit {} escapes page {} cell {:?}: {:?}",
                            index,
                            page,
                            cell,
                            n.bbox
                        );
                    }
                    if t.text == "AFTER" {
                        *after = Some((page, n.bbox.y));
                    }
                }
                for child in &n.children {
                    inspect(child, page, bounds, found, last, after);
                }
            }
            let dump = core.dump_page_items_json(None);
            for page in 0..core.page_count() {
                let nodes = page_nodes(&core, page);
                let continuation = dump[page as usize]["columns"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .flat_map(|column| column["items"].as_array().unwrap())
                    .any(|item| item["kind"] == "partialTable" && item["isContinuation"] == true);
                if continuation {
                    let body = nodes
                        .iter()
                        .find(|n| matches!(n.node_type, RenderNodeType::Body { .. }))
                        .unwrap();
                    let frame = nodes
                        .iter()
                        .find(|n| matches!(n.node_type, RenderNodeType::Table(_)))
                        .unwrap();
                    assert!((frame.bbox.y - body.bbox.y - 141.0 / 75.0).abs() < 0.05,
                        "host {visible_host}, height {page_height}, page {page}: continuation opens top margin once: body {:?}, table {:?}", body.bbox, frame.bbox);
                }
                inspect(
                    &core.build_page_render_tree(page).unwrap().root,
                    page,
                    None,
                    &mut found,
                    &mut last_table,
                    &mut following,
                );
            }
            assert!(
                found.iter().all(|&n| n == 1),
                "host {visible_host}, height {page_height}: units {found:?}"
            );
            let last = last_table.expect("table");
            let after = following.expect("following paragraph survives");
            assert!(
                after.0 > last.0 || (after.0 == last.0 && after.1 >= last.1 - 0.5),
                "following {after:?} precedes table end {last:?}"
            );
        }
    }
}

/// Stored mid-page cut advance includes a trailing blank interval. It is not
/// interchangeable with the physical border height. Independent committed PDFs
/// have 157 and 18 pages; subtracting the inset again produces 158 and 19.
#[test]
fn stored_midpage_cut_keeps_its_last_source_unit() {
    for (source, pages) in [
        ("samples/80168_regulatory_analysis.hwp", 157),
        ("samples/rowbreak-problem-pages.hwp", 18),
    ] {
        let bytes = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(source)).unwrap();
        let core = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(
            core.page_count(),
            pages,
            "{source}: do not cut a source unit early by counting unpainted trailing space twice"
        );
    }
}

/// 쪽 트리에서 `prefix` 로 시작하는 글줄(공백 무시)의 기준선 y.
fn line_baseline(nodes: &[RenderNode], prefix: &str) -> Option<f64> {
    let key: String = prefix.chars().filter(|c| !c.is_whitespace()).collect();
    nodes.iter().find_map(|line| {
        if !matches!(line.node_type, RenderNodeType::TextLine(_)) {
            return None;
        }
        let runs: Vec<_> = line
            .children
            .iter()
            .filter_map(|child| match &child.node_type {
                RenderNodeType::TextRun(run) => Some((child.bbox, run)),
                _ => None,
            })
            .collect();
        let text: String = runs
            .iter()
            .flat_map(|(_, run)| run.text.chars())
            .filter(|c| !c.is_whitespace())
            .collect();
        if !text.starts_with(&key) {
            return None;
        }
        runs.first().map(|(bbox, run)| bbox.y + run.baseline)
    })
}

#[test]
fn issue_7095_pinned_fragment_centres_its_own_content_in_the_box() {
    // 쪽이 정한 비끝 조각 상자(47.2..1043.7) 안에서 칸(`valign=Center`)의 내용은 **그 조각의
    // 내용 높이**로 가운데 정렬된다. 정본 3~9쪽의 줄은 모두 `칸 내용 위 + 저장 vpos` 에 쪽별
    // 상수 하나를 더한 자리에 있고(쪽 안 편차 0.6px 이하), 그 상수가 `(상자 안 높이 − 조각
    // 내용 높이) / 2` 와 맞는다. 조각 내용은 꼬리 빈 문단의 줄 상자까지 센다(4·5·6쪽).
    //
    // 기대값은 한/글 2020 정본 PDF 의 글줄 기준선이다(PyMuPDF span origin, 96dpi, 쪽 척도
    // 841/841.88 을 걷은 값). 수정 전 rhwp 는 내용을 상자 위에 붙여 첫 줄이 4쪽 13.8 ·
    // 6쪽 3.9 · 7쪽 7.6 · 9쪽 5.4px 위에 있었다. 5·8쪽은 내용이 상자를 거의 채워 여유가
    // 1px 안팎이라 수정 전에도 맞는 대조군이다.
    let core = load();
    //
    // 2·3쪽은 표 host(p18, 저장 LINE_SEG 없음) 뒤 빈 문단 p19 의 저장 되감김(vpos 0)이 한/글
    // 쪽 경계다. 그 되감김을 못 알아보면 p19 가 2쪽 끝에 남아 2쪽은 4.3px, 3쪽은 7.0px 어긋난다.
    let cases: [(u32, &str, f64, &str, f64); 8] = [
        (1, "◈금융지주가", 175.70, "중장기 과제는", 1025.07),
        (
            2,
            "업무위탁과 겸직 관련",
            93.06,
            "* 금융지주그룹 소속",
            1036.60,
        ),
        (
            3,
            "❶은행대출이 어려운",
            92.74,
            "* (현행) 59개 금융업무",
            1005.69,
        ),
        (4, "①핵심업무(28개", 66.47, "사전에 겸직승인을", 1021.55),
        (5, "그룹내 정보공유와", 78.16, "②이용기간 적정성의", 1005.21),
        (
            6,
            "(6) 고객정보 제공내역",
            74.80,
            "대출을 상환하거나",
            1014.50,
        ),
        (
            7,
            "□자회사등이 해외법인에",
            78.96,
            "* 영국, 미국, 호주",
            1039.17,
        ),
        (
            8,
            "신사업 진출 및 투자",
            93.22,
            "보유할 수 없기 때문에",
            1035.32,
        ),
    ];
    for (page, first, first_pdf, last, last_pdf) in cases {
        let nodes = page_nodes(&core, page);
        for (label, prefix, expected) in [("첫", first, first_pdf), ("끝", last, last_pdf)] {
            let actual = line_baseline(&nodes, prefix)
                .unwrap_or_else(|| panic!("{}쪽 {label} 줄 `{prefix}`", page + 1));
            assert!(
                (actual - expected).abs() < 1.5,
                "#7095: {}쪽 {label} 줄 `{prefix}` 기준선은 정본 {expected:.2} 이어야 한다: {actual:.2}",
                page + 1
            );
        }
    }
}

#[test]
fn issue_7095_stale_cell_height_keeps_content_fitted_box_top_anchored() {
    // 반례: 1382000 `pi=90` 은 칸 `valign=Center` 이지만 저장 칸 높이가 282HU(칸 여백뿐)다.
    // 한/글은 분할 칸의 저장 높이를 조각 상자 높이의 합으로 적는데(`pi=95` 4335px ↔ 정본 합
    // 4338px), 이 칸은 그렇지 않고 정본 16~19쪽 상자도 모두 내용에 맞춘다(아래 977·980·970·897).
    // 정본 17쪽 칸 내용은 상자 위 3px 에서 시작한다. 쪽 상자로 늘린 뒤 가운데 정렬하면 14px 내려간다.
    let core = load_sample("samples/task2430/1382000_domestic_violence_survey.hwp");
    let nodes = page_nodes(&core, 16);
    let cell = nodes
        .iter()
        .filter(|n| matches!(n.node_type, RenderNodeType::TableCell(_)))
        .max_by(|a, b| a.bbox.height.total_cmp(&b.bbox.height))
        .expect("17쪽 1×1 조각 칸")
        .bbox;
    let first_line = nodes
        .iter()
        .filter(|n| matches!(n.node_type, RenderNodeType::TextLine(_)))
        .filter(|n| n.bbox.y >= cell.y - 0.5)
        .map(|n| n.bbox.y)
        .fold(f64::INFINITY, f64::min);
    assert!(
        first_line - cell.y < 4.0,
        "#7095: 저장 높이가 쪽을 덮지 않는 칸은 내용을 상자 위에 둔다: cell_y={:.1} first_line={first_line:.1}",
        cell.y
    );
}

/// 쪽 트리에서 본문 단 바로 아래 항목들(표·글줄)의 (종류, 위, 아래).
fn column_children(core: &DocumentCore, page_index: u32) -> Vec<(bool, f64, f64)> {
    let page = core
        .build_page_render_tree(page_index)
        .expect("render tree");
    fn column(node: &RenderNode) -> Option<&RenderNode> {
        if matches!(node.node_type, RenderNodeType::Column(_)) {
            return Some(node);
        }
        node.children.iter().find_map(column)
    }
    column(&page.root)
        .expect("본문 단")
        .children
        .iter()
        .map(|n| {
            (
                matches!(n.node_type, RenderNodeType::Table(_)),
                n.bbox.y,
                n.bbox.y + n.bbox.height,
            )
        })
        .collect()
}

#[test]
fn issue_7095_terminal_fragment_box_takes_the_stored_cell_height_remainder() {
    // 한/글은 분할된 칸의 저장 높이를 조각 상자 높이의 합으로 적는다. 저장 높이는 칸의
    // 최소 높이라 끝 조각 상자는 `max(내용, 저장 높이 − 앞 조각 상자 합)` 이다.
    //
    // - 7062: 저장 700976HU = 9346.35px ↔ 정본 상자 500.30 + 996.43×8 + 874.04 = 9345.78px.
    //   정본 10쪽 상자 47.20..921.24 (PDF 벡터, 쪽 척도 제거). 수정 전 rhwp 는 내용에 맞춰
    //   901.3 에서 끝났다.
    // - 1382000 `pi=95`: 저장 4335.25px ↔ 정본 합 4335.38px. 정본 30쪽 상자 아래 923.87,
    //   수정 전 842.0.
    let core = load();
    let (_, top, bottom) = column_children(&core, 9)
        .into_iter()
        .find(|(is_table, _, _)| *is_table)
        .expect("7062 10쪽 끝 조각");
    assert!(
        (top - 47.2).abs() < 0.5 && (bottom - 921.24).abs() < 1.0,
        "#7095: 7062 10쪽 끝 조각 상자는 정본 47.20..921.24 여야 한다: {top:.2}..{bottom:.2}"
    );

    let survey = load_sample("samples/task2430/1382000_domestic_violence_survey.hwp");
    let (_, _, bottom) = column_children(&survey, 29)
        .into_iter()
        .find(|(is_table, _, _)| *is_table)
        .expect("1382000 30쪽 끝 조각");
    assert!(
        (bottom - 923.87).abs() < 1.0,
        "#7095: 1382000 30쪽 끝 조각 상자 아래는 정본 923.87 이어야 한다: {bottom:.2}"
    );
}

#[test]
fn issue_7095_terminal_fragment_does_not_push_the_next_paragraph_past_its_stored_top() {
    // 반례: rowbreak-problem-pages `pi=13` 은 저장 칸 높이가 상자 합이 아니다. 나머지 규칙만
    // 쓰면 끝 조각이 16px 늘어 뒤 문단이 한/글 저장 자리(첫 줄 vpos 21751HU → 본문 위 94.5 +
    // 290.0 = 384.5)를 넘고 문서가 18 → 19쪽이 된다. 다음 문단의 저장 자리가 늘림의 상한이다.
    let core = load_sample("samples/rowbreak-problem-pages.hwp");
    let first_line_after_table = column_children(&core, 13)
        .into_iter()
        .skip_while(|(is_table, _, _)| !*is_table)
        .find(|(is_table, _, _)| !*is_table)
        .map(|(_, top, _)| top)
        .expect("14쪽 끝 조각 뒤 문단");
    assert!(
        (first_line_after_table - 384.5).abs() < 1.5,
        "#7095: 끝 조각 뒤 문단은 한/글 저장 자리 384.5 에 있어야 한다: {first_line_after_table:.2}"
    );
    assert_eq!(
        core.page_count(),
        18,
        "#7095: rowbreak-problem-pages 는 한/글과 같은 18쪽"
    );
}
