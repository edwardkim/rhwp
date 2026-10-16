//! 레이아웃 통합 테스트
//!
//! 실제 HWP 파일을 로딩하여 페이지네이션 + 레이아웃 결과를 검증한다.
//! samples/ 디렉토리에 테스트 파일이 없으면 건너뜀.

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::model::page::{ColumnDef, PageDef};
    use crate::model::paragraph::{LineSeg, Paragraph};
    use crate::model::style::{BorderLine, BorderLineType};
    use crate::renderer::composer::compose_paragraph;
    use crate::renderer::layout::LayoutEngine;
    use crate::renderer::page_layout::PageLayoutInfo;
    use crate::renderer::pagination::{ColumnContent, PageContent, PageItem};
    use crate::renderer::render_tree::{RenderNode, RenderNodeType};
    use crate::renderer::style_resolver::{
        ResolvedBorderStyle, ResolvedCharStyle, ResolvedParaStyle, ResolvedStyleSet,
    };

    /// 테스트용 DocumentCore 생성 헬퍼
    fn load_document(path: &str) -> Option<crate::document_core::DocumentCore> {
        let p = Path::new(path);
        if !p.exists() {
            eprintln!("테스트 파일 없음: {} — 건너뜀", path);
            return None;
        }
        let data = std::fs::read(p).ok()?;
        crate::document_core::DocumentCore::from_bytes(&data).ok()
    }

    fn collect_render_nodes<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
        out.push(node);
        for child in &node.children {
            collect_render_nodes(child, out);
        }
    }

    #[derive(Debug)]
    struct ParaBorderNodeCounts {
        stroked_rects: usize,
        vertical_lines: usize,
        horizontal_lines: usize,
        line_summaries: Vec<String>,
    }

    fn render_synthetic_para_border_counts(borders: [BorderLine; 4]) -> ParaBorderNodeCounts {
        let engine = LayoutEngine::with_default_dpi();
        let page_def = PageDef {
            width: 59528,
            height: 84188,
            margin_left: 8504,
            margin_right: 8504,
            margin_top: 5669,
            margin_bottom: 4252,
            margin_header: 4252,
            margin_footer: 4252,
            margin_gutter: 0,
            ..Default::default()
        };
        let layout = PageLayoutInfo::from_page_def_default(&page_def, &ColumnDef::default());

        let text = "가로선만 있는 문단 테두리".to_string();
        let paragraphs = vec![Paragraph {
            char_count: text.chars().count() as u32 + 1,
            char_offsets: (0..text.chars().count() as u32).collect(),
            text,
            para_shape_id: 0,
            line_segs: vec![LineSeg {
                line_height: 400,
                baseline_distance: 320,
                segment_width: 30000,
                ..Default::default()
            }],
            ..Default::default()
        }];
        let composed: Vec<_> = paragraphs.iter().map(compose_paragraph).collect();

        let styles = ResolvedStyleSet {
            page_number_char_style_id: None,
            page_number_font_is_hft: false,
            hwp3_variant: false,
            hft_ascii_halfwidth: false,
            char_styles: vec![ResolvedCharStyle::default()],
            para_styles: vec![ResolvedParaStyle {
                border_fill_id: 1,
                ..Default::default()
            }],
            border_styles: vec![ResolvedBorderStyle {
                borders,
                fill_color: None,
                pattern: None,
                gradient: None,
                image_fill: None,
                diagonal_attr: 0,
                diagonal: Default::default(),
                center_line: Default::default(),
            }],
            numberings: Vec::new(),
            bullets: Vec::new(),
            kerning_measurement_context: None,
            horizontal_shaping_context: None,
            supplemental_metrics: None,
        };

        let page_content = PageContent {
            page_index: 0,
            page_number: 0,
            page_number_restarted: false,
            section_index: 0,
            layout,
            column_contents: vec![ColumnContent {
                column_index: 0,
                start_height: 0.0,
                endnote_flow: false,
                items: vec![PageItem::FullParagraph { para_index: 0 }],
                zone_layout: None,
                zone_y_offset: 0.0,
                wrap_around_paras: Vec::new(),
                used_height: 0.0,
                wrap_anchors: std::collections::HashMap::new(),
                overlay_continuations: Vec::new(),
                overlay_cuts: Vec::new(),
                inline_placements: Default::default(),
                inline_flow_plans: Default::default(),
                stored_paragraph_flow_ends: Default::default(),
                stored_column_top_spacings: Default::default(),
                paragraph_float_placements: Default::default(),
                float_pushed_line_spacings: Default::default(),
            }],
            active_header: None,
            active_footer: None,
            page_number_pos: None,
            page_hide: None,
            footnotes: Vec::new(),
            active_master_page: None,
            extra_master_pages: Vec::new(),
            ladder_band_tables: Vec::new(),
        };

        let tree = engine.build_render_tree(
            &page_content,
            &paragraphs,
            &paragraphs,
            &paragraphs,
            &composed,
            &styles,
            &Default::default(),
            &[],
            None,
            &[],
            None,
            0,
            &[],
        );

        let mut nodes = Vec::new();
        collect_render_nodes(&tree.root, &mut nodes);

        let stroked_rects = nodes
            .iter()
            .filter(|node| match &node.node_type {
                RenderNodeType::Rectangle(rect) => rect.style.stroke_width > 0.0,
                _ => false,
            })
            .count();
        let vertical_lines = nodes
            .iter()
            .filter(|node| match &node.node_type {
                RenderNodeType::Line(line) => {
                    (line.x1 - line.x2).abs() < 0.5 && (line.y1 - line.y2).abs() > 0.5
                }
                _ => false,
            })
            .count();
        let line_summaries: Vec<String> = nodes
            .iter()
            .filter_map(|node| match &node.node_type {
                RenderNodeType::Line(line) => Some(format!(
                    "line x1={:.1} y1={:.1} x2={:.1} y2={:.1}",
                    line.x1, line.y1, line.x2, line.y2
                )),
                _ => None,
            })
            .collect();
        let horizontal_lines = nodes
            .iter()
            .filter(|node| match &node.node_type {
                RenderNodeType::Line(line) => {
                    (line.y1 - line.y2).abs() < 0.5 && (line.x1 - line.x2).abs() > 0.5
                }
                _ => false,
            })
            .count();

        ParaBorderNodeCounts {
            stroked_rects,
            vertical_lines,
            horizontal_lines,
            line_summaries,
        }
    }

    // ─── 페이지 수 검증 ───

    #[test]
    fn test_hwpspec_w_page_count() {
        let Some(core) = load_document("samples/hwpspec-w.hwp") else {
            return;
        };
        let page_count = core.page_count();
        assert!(
            page_count >= 170,
            "hwpspec-w.hwp 페이지 수 170 이상 (실제: {})",
            page_count
        );
    }

    // ─── 2단 레이아웃 검증 ───

    // ─── 머리말 검증 ───

    // ─── 표 분할(PartialTable) 검증 ───

    #[test]
    fn test_hwpspec_w_table_split() {
        let Some(core) = load_document("samples/hwpspec-w.hwp") else {
            return;
        };
        use crate::renderer::pagination::PageItem;
        let has_partial_table = core.pagination.iter().any(|result| {
            result.pages.iter().any(|p| {
                p.column_contents.iter().any(|cc| {
                    cc.items
                        .iter()
                        .any(|item| matches!(item, PageItem::PartialTable { .. }))
                })
            })
        });
        assert!(
            has_partial_table,
            "hwpspec-w.hwp에는 페이지 분할된 표(PartialTable)가 있어야 함"
        );
    }

    // ─── SVG 내보내기 검증 ───

    #[test]
    fn test_export_svg_produces_output() {
        let Some(core) = load_document("samples/hwpspec-w.hwp") else {
            return;
        };
        let svg = core.render_page_svg_native(0).unwrap_or_default();
        assert!(!svg.is_empty(), "SVG 출력이 비어있으면 안 됨");
        assert!(svg.contains("<svg"), "SVG 출력에 <svg 태그가 있어야 함");
        assert!(svg.contains("</svg>"), "SVG 출력에 </svg> 태그가 있어야 함");
    }

    #[test]
    fn test_export_svg_contains_text() {
        let Some(core) = load_document("samples/hwpspec-w.hwp") else {
            return;
        };
        let svg = core.render_page_svg_native(0).unwrap_or_default();
        assert!(svg.contains("<text"), "SVG에 텍스트 요소가 있어야 함");
    }

    // ─── 수식 렌더링 검증 ───

    // ─── 다중 페이지 렌더링 회귀 테스트 ───

    #[test]
    fn test_hwpspec_w_multi_page_render() {
        let Some(core) = load_document("samples/hwpspec-w.hwp") else {
            return;
        };
        for page_idx in 0..16u32 {
            let svg = core.render_page_svg_native(page_idx).unwrap_or_default();
            assert!(!svg.is_empty(), "페이지 {} SVG가 비어있음", page_idx + 1);
        }
    }

    // ─── 문단 테두리 검증 ───

    #[test]
    fn test_1_3_paragraph_border() {
        let Some(core) = load_document("samples/1-3.hwp") else {
            return;
        };
        let svg = core.render_page_svg_native(0).unwrap_or_default();
        assert!(
            svg.contains("<rect") || svg.contains("<path"),
            "1-3.hwp에 문단 테두리/배경 렌더링 요소가 있어야 함"
        );
    }

    /// Task #1205 RED: 문단 borderFill 의 left/right 가 NONE 이면 top/bottom 이
    /// visible 이더라도 4면 stroke rectangle 이나 좌우 수직선을 만들면 안 된다.
    #[test]
    fn task_1205_para_border_none_sides_do_not_render_vertical_edges() {
        let hidden = BorderLine {
            line_type: BorderLineType::None,
            width: 1,
            color: 0,
        };
        let visible = BorderLine {
            line_type: BorderLineType::Solid,
            width: 1,
            color: 0,
        };
        let counts = render_synthetic_para_border_counts([hidden, hidden, visible, visible]);

        assert_eq!(
            counts.stroked_rects, 0,
            "left/right NONE 문단 border 는 4면 stroke rectangle 으로 렌더되면 안 됨"
        );
        assert_eq!(
            counts.vertical_lines, 0,
            "left/right NONE 문단 border 는 좌우 수직선을 렌더하면 안 됨"
        );
        assert!(
            counts.horizontal_lines >= 1,
            "top/bottom SOLID 문단 border 는 가로선을 렌더해야 함. lines={:?}",
            counts.line_summaries
        );
    }

    /// Task #1205 Stage 3: 4면이 모두 같은 visible stroke 이고 partial skip 이
    /// 없을 때만 기존 단일 Rectangle stroke 경로를 사용할 수 있다.
    #[test]
    fn task_1205_rect_stroke_path_requires_four_visible_same_stroke() {
        let hidden = BorderLine {
            line_type: BorderLineType::None,
            width: 1,
            color: 0,
        };
        let visible = BorderLine {
            line_type: BorderLineType::Solid,
            width: 1,
            color: 0,
        };
        let different_width = BorderLine {
            width: 2,
            ..visible
        };

        assert!(
            super::super::para_border_can_use_rect_stroke(
                &[visible, visible, visible, visible],
                false,
                false,
            ),
            "4면 동일 visible 문단 border 는 기존 Rectangle stroke 경로를 사용할 수 있어야 함"
        );
        assert!(
            !super::super::para_border_can_use_rect_stroke(
                &[hidden, hidden, visible, visible],
                false,
                false,
            ),
            "NONE side 가 있으면 Rectangle stroke 경로를 사용하면 안 됨"
        );
        assert!(
            !super::super::para_border_can_use_rect_stroke(
                &[visible, visible, different_width, visible],
                false,
                false,
            ),
            "side별 stroke 가 다르면 Rectangle stroke 경로를 사용하면 안 됨"
        );
        assert!(
            !super::super::para_border_can_use_rect_stroke(
                &[visible, visible, visible, visible],
                false,
                true,
            ),
            "partial skip 이 있으면 Rectangle stroke 경로를 사용하면 안 됨"
        );
    }

    /// Issue #3257: 마지막 공백 뒤의 TAC 그림도 가운데 정렬 폭에 포함해야 한다.
    ///
    /// HWP 2020 기준에서 4쪽 `용도별 서버 구성`은 본문 안쪽 약 x=129px에서 시작한다.
    /// 수정 전에는 끝 위치 TAC가 폭 계산에서 빠져 공백만 Center 정렬한 뒤 그림을 붙여
    /// x=416.9px로 밀리고 페이지 오른쪽에서 잘렸다.
    #[test]
    fn issue_3257_centered_trailing_picture_uses_full_line_width() {
        let Some(core) = load_document("samples/issue3257/webhangul_product_spec_v1.1.hwp") else {
            return;
        };
        let tree = core
            .build_page_render_tree(3)
            .expect("#3257: 제품규격서 4페이지 render tree 생성 실패");

        fn find_picture<'a>(node: &'a RenderNode, out: &mut Option<&'a RenderNode>) {
            if let RenderNodeType::Image(image) = &node.node_type {
                if image.para_index == Some(75) && image.control_index == Some(0) {
                    *out = Some(node);
                }
            }
            for child in &node.children {
                find_picture(child, out);
            }
        }

        let mut picture_node = None;
        find_picture(&tree.root, &mut picture_node);
        let picture = picture_node.expect("#3257: pi=75 ci=0 TAC 그림 노드를 찾지 못함");
        assert!(
            (picture.bbox.x - 129.8).abs() < 2.0,
            "#3257: 끝 위치 TAC 그림의 Center 정렬 좌표가 기준에서 벗어남: x={:.1}",
            picture.bbox.x
        );
        assert!(
            picture.bbox.x + picture.bbox.width < 718.5,
            "#3257: 그림이 본문 우측을 넘음: x={:.1}, width={:.1}",
            picture.bbox.x,
            picture.bbox.width
        );
    }

    /// Issue #1838 회귀 가드: 셀 폭을 초과하는 텍스트(공백 포함)가 파일 lineseg
    /// (짧은 값 기준 단일 줄, 외부 도구 값 교체 시나리오)와 부정합해도 **모든 글자가
    /// SVG 에 방출**되어야 한다 (조용한 글리프 탈락 금지).
    ///
    /// 진단 확정 사항 (basic-table-01 합성):
    /// - compose/렌더트리/SVG 모두 전체 텍스트 보존 — 글리프 탈락 없음.
    /// - SVG 는 per-cluster(<text> 글자 단위) 방출이라 "앞토큰" 같은 다글자
    ///   부분문자열 검색은 항상 실패한다 — 존재 검사는 글자 단위로 해야 함.
    /// - 초과분은 cell clipPath 안에 있어 시각적으로만 잘린다 (별도 개선 축:
    ///   한컴은 열었을 때 재줄바꿈해 2줄 표시).
    #[test]
    fn test_1838_overwide_cell_text_keeps_all_glyphs_in_svg() {
        use crate::model::control::Control;

        let Some(mut core) = load_document("samples/hwpx/basic-table-01.hwpx") else {
            return;
        };
        let value = "앞토큰 뒤토큰뒤토큰(괄호포함내용내용내용)";
        {
            let para = &mut core.document.sections[0].paragraphs[0];
            let Some(Control::Table(table)) = para.controls.get_mut(2) else {
                panic!("#1838: basic-table-01 문단 0 ctrl[2] 가 표가 아님");
            };
            // 실제 파일 상태 충실 재현: text/char_offsets/char_count 는 파서 정합값,
            // line_segs 는 짧은 원본값 기준 단일 줄로 잔존.
            let cell_para = &mut table.cells[0].paragraphs[0];
            assert_eq!(cell_para.line_segs.len(), 1, "#1838 전제: 단일 lineseg");
            cell_para.text = value.to_string();
            let n = cell_para.text.chars().count() as u32;
            cell_para.char_offsets = (0..n).collect();
            cell_para.char_count = n + 1;
        }
        let svg = core.render_page_svg_native(0).unwrap_or_default();
        assert!(!svg.is_empty(), "#1838: SVG 비어있음");

        // per-cluster 방출이므로 글자 단위로 존재 검사 (공백 제외 전 글자)
        let missing: Vec<char> = value
            .chars()
            .filter(|c| *c != ' ')
            .filter(|c| !svg.contains(&format!(">{c}<")))
            .collect();
        assert!(
            missing.is_empty(),
            "#1838: 셀 폭 초과 텍스트의 글자가 SVG 에서 누락됨: {missing:?}"
        );
    }

    #[test]
    fn test_layer_svg_matches_legacy_for_basic_text_sample() {
        let Some(core) = load_document("samples/lseg-01-basic.hwp") else {
            return;
        };
        let legacy = core.render_page_svg_legacy_native(0).unwrap_or_default();
        let layered = core.render_page_svg_layer_native(0).unwrap_or_default();
        assert_eq!(
            layered, legacy,
            "layer SVG는 기본 텍스트 샘플에서 legacy SVG와 동일해야 함"
        );
    }

    #[test]
    fn test_layer_svg_matches_legacy_for_table_sample() {
        let Some(core) = load_document("samples/hwp_table_test.hwp") else {
            return;
        };
        let legacy = core.render_page_svg_legacy_native(0).unwrap_or_default();
        let layered = core.render_page_svg_layer_native(0).unwrap_or_default();
        assert_eq!(
            layered, legacy,
            "layer SVG는 표 샘플에서 legacy SVG와 동일해야 함"
        );
    }

    // ─── Task #634: 한컴 호환 — 쪽번호 표시/미표시 ───
    //
    // 새 한컴 PDF (samples/aift.pdf, 1-up portrait, 2026-05-06 갱신) 측정 결과:
    //
    // | rhwp page | 한컴 footer | 메커니즘 |
    // |-----------|-------------|---------|
    // | 1 (cover disclaimer)   | **표시**   | PageNumberPos 등록 후 표시 |
    // | 2 (사업계획서 표지)      | 미표시     | 미해결 (PageHide 없음, 별도 issue) |
    // | 3 (요약문)              | 미표시     | 미해결 (PageHide 없음, 별도 issue) |
    // | 4 (목차)                | 미표시     | PageHide on para 2.34 ✓ |
    // | 5 (별첨 목차)            | 미표시     | PageHide on para 2.54 ✓ |
    // | 6 (본문 시작)            | **표시**   | 정상 (rhwp 도 표시) |
    // | 7+ (NewNumber 발화 후)  | **표시**   | 정상 (rhwp 도 표시) |
    //
    // **잘못된 가설 H1'' (NewNumber 게이팅) revert**: 한컴은 NewNumber 발화와 무관하게
    // 페이지 1, 6 에 쪽번호 표시. 게이팅은 한컴 동작이 아님.
    //
    // 페이지 2, 3 미표시 메커니즘은 **별도 issue 분리**.

    /// SVG 에서 특정 y 좌표 (`±0.5px` 허용) 의 `<text>` 요소 개수를 센다.
    /// 지정한 y(96dpi SVG 단위) 행의 `<text>` 개수.
    ///
    /// [#3048] 쪽 번호를 한글 정합값 10pt 로 교정하면서(종전 7.5pt) 줄 baseline 이
    /// +4.44px 이동했다 — 아래 #634 가드들의 y 상수는 그만큼 갱신되어 있다.
    /// 글자 수(`- N -` = 3개)는 불변이라 가드의 의도는 그대로다.
    fn count_text_at_y(svg: &str, target_y: f64) -> usize {
        let mut count = 0;
        for line in svg.lines() {
            if !line.contains("<text") {
                continue;
            }
            // y="..." 추출
            if let Some(s) = line.find(" y=\"") {
                let rest = &line[s + 4..];
                if let Some(e) = rest.find('"') {
                    if let Ok(y) = rest[..e].parse::<f64>() {
                        if (y - target_y).abs() < 0.5 {
                            count += 1;
                        }
                    }
                }
            }
        }
        count
    }

    /// [Task #683] pr-149.hwp 빈 paragraph + Para-relative TopAndBottom 그림
    /// cluster 간 거리 정합 검증.
    /// - 한컴 한글 2022 PDF 정합: 그림 cluster 당 18864 HU (= image_height + line(lh+ls)).
    /// - 버그 (수정 전): cluster = 17280 HU (image_height + 0, line baseline 누락).
    /// - 수정: image-paragraph result_y 에 line(lh+ls) 추가 (layout_shape_item Picture 분기).
    #[test]
    fn test_task683_pr149_image_cluster_spacing() {
        let Some(core) = load_document("samples/pr-149.hwp") else {
            return;
        };
        let svg = core.render_page_svg_native(0).unwrap_or_default();
        assert!(!svg.is_empty(), "pr-149.hwp page 0 SVG 생성");

        // SVG 에서 <image ... y="..."/> 의 y 값 추출
        let mut image_ys: Vec<f64> = Vec::new();
        for cap in svg.split("<image ").skip(1) {
            if let Some(idx) = cap.find("y=\"") {
                let rest = &cap[idx + 3..];
                if let Some(end) = rest.find('"') {
                    if let Ok(y) = rest[..end].parse::<f64>() {
                        image_ys.push(y);
                    }
                }
            }
        }
        image_ys.sort_by(|a, b| a.partial_cmp(b).unwrap());

        assert_eq!(image_ys.len(), 3, "그림 3개 (원본/회색조/흑백)");

        let cluster1 = image_ys[1] - image_ys[0];
        let cluster2 = image_ys[2] - image_ys[1];

        // 18864 HU @ 96 dpi = 251.52 px. ±3 px tolerance for sub-pixel rounding.
        let pdf_cluster_px: f64 = 18864.0 * 96.0 / 7200.0;
        assert!(
            (cluster1 - pdf_cluster_px).abs() < 3.0,
            "image1→image2 cluster {:.2} px 가 PDF {:.2} px (±3) 와 일치해야 함. \
             버그(수정 전): 230.4 px (= 17280 HU, line 누락).",
            cluster1,
            pdf_cluster_px
        );
        assert!(
            (cluster2 - pdf_cluster_px).abs() < 3.0,
            "image2→image3 cluster {:.2} px 가 PDF {:.2} px (±3) 와 일치해야 함.",
            cluster2,
            pdf_cluster_px
        );
    }

    /// 표시 여부는 자동 쪽번호 런의 최종 기준선에서 실제 SVG 글자를 확인한다.
    /// 위치의 정확성은 독립 PDF를 쓰는 정식 기준선 검사에서 별도로 확인한다.
    fn count_automatic_page_number_glyphs(
        core: &crate::document_core::DocumentCore,
        page: u32,
        svg: &str,
    ) -> usize {
        let tree = core.build_page_render_tree(page).expect("쪽번호 배치");
        tree.root
            .children
            .iter()
            .filter(|node| matches!(node.node_type, RenderNodeType::Footer))
            .flat_map(|footer| &footer.children)
            .flat_map(|line| &line.children)
            .filter_map(|node| match &node.node_type {
                RenderNodeType::TextRun(run) if run.para_index.is_none() => {
                    Some(count_text_at_y(svg, node.bbox.y + run.baseline))
                }
                _ => None,
            })
            .sum()
    }

    /// Task #634: aift.hwp 페이지 1 (표지 고지 "※ 동 사업...") 은 PageNumberPos
    /// 등록 페이지로 한컴이 "- 1 -" 표시. rhwp 도 표시되어야 함 (회귀 방지).
    #[test]
    fn test_634_aift_page1_shows_page_number() {
        let Some(core) = load_document("samples/aift.hwp") else {
            return;
        };
        let svg = core.render_page_svg_native(0).unwrap_or_default();
        let count = count_automatic_page_number_glyphs(&core, 0, &svg);
        assert_eq!(
            count, 3,
            "aift.hwp 페이지 1 (표지 고지, PageNumberPos 등록 페이지) 은 \
             \"- 1 -\" 3글자 표시되어야 함 (한컴 일치)."
        );
    }

    /// Task #634: aift.hwp 페이지 6 (본문 시작) 은 한컴이 쪽번호 표시.
    /// rhwp 도 표시되어야 함 (회귀 방지).
    #[test]
    fn test_634_aift_page6_shows_page_number() {
        let Some(core) = load_document("samples/aift.hwp") else {
            return;
        };
        let svg = core.render_page_svg_native(5).unwrap_or_default();
        let count = count_automatic_page_number_glyphs(&core, 5, &svg);
        assert_eq!(
            count, 3,
            "aift.hwp 페이지 6 (본문 시작) 은 한컴이 \"- N -\" 표시. \
             rhwp 도 3글자 표시되어야 함."
        );
    }

    /// Task #634: aift.hwp 페이지 7 (□ 배경, NewNumber 발화) 부터 쪽번호 표시 (회귀 방지).
    #[test]
    fn test_634_aift_page7_shows_page_number() {
        let Some(core) = load_document("samples/aift.hwp") else {
            return;
        };
        let svg = core.render_page_svg_native(6).unwrap_or_default();
        let count = count_automatic_page_number_glyphs(&core, 6, &svg);
        assert_eq!(
            count, 3,
            "aift.hwp 페이지 7 (NewNumber 발화) 은 \"- 1 -\" 3글자 표시되어야 함."
        );
    }

    /// Task #634: aift.hwp 페이지 4 (목차) 는 PageHide page_num=true (paragraph 2.34)
    /// 적용으로 쪽번호 미표시. rhwp 도 미표시 (기존 PageHide 지원).
    #[test]
    fn test_634_aift_page4_pagehide_no_page_number() {
        let Some(core) = load_document("samples/aift.hwp") else {
            return;
        };
        let svg = core.render_page_svg_native(3).unwrap_or_default();
        let count = count_automatic_page_number_glyphs(&core, 3, &svg);
        assert_eq!(
            count, 0,
            "aift.hwp 페이지 4 는 PageHide page_num=true (paragraph 2.34) 로 미표시."
        );
    }

    /// Task #634: aift.hwp 페이지 5 (별첨 목차) 는 PageHide (paragraph 2.54) 적용 미표시.
    #[test]
    fn test_634_aift_page5_pagehide_no_page_number() {
        let Some(core) = load_document("samples/aift.hwp") else {
            return;
        };
        let svg = core.render_page_svg_native(4).unwrap_or_default();
        let count = count_automatic_page_number_glyphs(&core, 4, &svg);
        assert_eq!(
            count, 0,
            "aift.hwp 페이지 5 는 PageHide page_num=true (paragraph 2.54) 로 미표시."
        );
    }

    /// Task #634: 2022년 국립국어원 페이지 1 (표지) 은 PageHide page_num=true 적용 미표시.
    #[test]
    fn test_634_gukrip_page1_pagehide_no_page_number() {
        let Some(core) = load_document("samples/2022년 국립국어원 업무계획.hwp") else {
            return;
        };
        let svg = core.render_page_svg_native(0).unwrap_or_default();
        let count = count_automatic_page_number_glyphs(&core, 0, &svg);
        assert_eq!(
            count, 0,
            "국립국어원 페이지 1 은 PageHide (paragraph 0.19) 로 미표시."
        );
    }

    /// 국립국어원 같은 입력의 한컴2020·2022 PDF3쪽은 "- 1 -"을 표시한다.
    /// 과거 특정 y에서0글자라는 검사로 숨김을 추정한 기대값은 독립 출력과 다르다.
    #[test]
    fn test_634_gukrip_page3_shows_page_number() {
        let Some(core) = load_document("samples/2022년 국립국어원 업무계획.hwp") else {
            return;
        };
        let svg = core.render_page_svg_native(2).unwrap_or_default();
        let count = count_automatic_page_number_glyphs(&core, 2, &svg);
        assert_eq!(
            count, 3,
            "국립국어원 같은 입력의 한컴 PDF3쪽 자동 번호는 표시되어야 함."
        );
    }

    /// Task #634: hwp3-sample.hwp (NewNumber 0개) 페이지 1 부터 표시 (회귀 방지).
    #[test]
    fn test_634_no_newnumber_doc_shows_page_numbers_from_page1() {
        let Some(core) = load_document("samples/hwp3-sample.hwp") else {
            return;
        };
        let svg = core.render_page_svg_native(0).unwrap_or_default();
        let count = count_automatic_page_number_glyphs(&core, 0, &svg);
        assert_eq!(
            count, 3,
            "hwp3-sample.hwp 페이지 1 (NewNumber 0개) 은 쪽번호 표시되어야 함 (회귀 방지)."
        );
    }

    // 페이지 2, 3 (사업계획서 표지/요약문) 미표시 메커니즘은 별도 issue.
    // 한컴: PageHide 없는데도 미표시. rhwp: 표시 (현재). 메커니즘 미확인 — 후속 분석 필요.

    // ─── Task #705: 셀 안 PageHide 컨트롤 페이지네이션 매핑 ───
    //
    // 메인테이너 권위 측정 (PR #638 코멘트):
    //   aift.hwp s0/p[1]/Table[0]/셀[167]/p[3]/ctrl[0]
    //   PageHide(header=true footer=true master=true border=true fill=true page_num=true)
    //
    // Stage 0 본 환경 측정 (mydocs/tech/investigations/issue-705/probes/inspect_705.rs):
    //   - aift.hwp 셀 안 PageHide 2건 (s0/셀[167] full6, s1/셀[31] page_num)
    //   - 본문 PageHide 2건 (s2/p[34], s2/p[54])
    //
    // 결함 #1 (pagination/engine.rs:519-544): 본문 paragraph 만 순회 →
    //   셀 안 PageHide 무시 → page.page_hide 가 None 으로 남음.

    #[test]
    fn test_705_aift_page2_cell_pagehide_collected() {
        let Some(core) = load_document("samples/aift.hwp") else {
            return;
        };
        // page 2 (global_idx=1, section=0, page_num=2)
        // 외부 paragraph s0/p[1] (Table 35x27, tac=false) 의 셀[167]/p[3] PageHide
        let page = core
            .pagination
            .first()
            .and_then(|pr| pr.pages.get(1))
            .expect("aift.hwp page 2 존재");
        assert!(
            page.page_hide.is_some(),
            "aift.hwp page 2: 셀[167]/p[3] PageHide 가 page.page_hide 로 매핑되어야 함 \
             (현재 None: 결함 #1 — pagination/engine.rs 가 셀 안 paragraph 미순회)"
        );
        assert!(
            page.page_hide.as_ref().unwrap().hide_page_num,
            "aift.hwp page 2: hide_page_num=true 여야 함"
        );
    }

    #[test]
    fn test_705_aift_page2_cell_pagehide_six_fields() {
        let Some(core) = load_document("samples/aift.hwp") else {
            return;
        };
        let page = core
            .pagination
            .first()
            .and_then(|pr| pr.pages.get(1))
            .expect("aift.hwp page 2 존재");
        let ph = page
            .page_hide
            .as_ref()
            .expect("aift.hwp page 2: page_hide 채워져야 함 (결함 #1)");
        // 메인테이너 권위 측정: 6 필드 모두 true
        assert!(ph.hide_header, "hide_header=true (감추기 다이얼로그 6항목)");
        assert!(ph.hide_footer, "hide_footer=true");
        assert!(ph.hide_master_page, "hide_master_page=true");
        assert!(ph.hide_border, "hide_border=true");
        assert!(ph.hide_fill, "hide_fill=true");
        assert!(ph.hide_page_num, "hide_page_num=true");
    }

    #[test]
    fn test_705_aift_page3_cell_pagehide_collected() {
        let Some(core) = load_document("samples/aift.hwp") else {
            return;
        };
        // page 3 (global_idx=2, section=1, page_num=3)
        // 외부 paragraph s1/p[0] 의 셀[31]/p[0] PageHide (page_num 만 true)
        let page = core
            .pagination
            .get(1)
            .and_then(|pr| pr.pages.first())
            .expect("aift.hwp page 3 존재");
        let ph = page.page_hide.as_ref().expect(
            "aift.hwp page 3: 셀[31]/p[0] PageHide 가 page.page_hide 로 매핑되어야 함 \
                     (현재 None: 결함 #1)",
        );
        assert!(
            ph.hide_page_num,
            "aift.hwp page 3: hide_page_num=true (셀[31]/p[0] '최종 목표')"
        );
    }

    #[test]
    fn test_705_aift_cell_pagehides_total_count() {
        let Some(core) = load_document("samples/aift.hwp") else {
            return;
        };
        // 본문 PageHide 2건 (s2/p[34], s2/p[54]) + 셀 안 PageHide 2건 (s0/셀[167], s1/셀[31])
        // = 최소 4 페이지에 page_hide 매핑되어야 함
        let count = core
            .pagination
            .iter()
            .flat_map(|pr| pr.pages.iter())
            .filter(|p| p.page_hide.is_some())
            .count();
        assert!(
            count >= 4,
            "aift.hwp page_hide 매핑 페이지 4건 이상 (실제: {}). \
             셀 안 PageHide (s0/셀[167] + s1/셀[31]) 가 누락된 가능성: 결함 #1",
            count
        );
    }

    #[test]
    fn test_705_kor2022_cell_pagehide_collected() {
        // Stage 0 측정: 본문 PageHide 1건 + 셀 안 PageHide 1건 (셀[0]/p[5] -----P "Ⅱ. 2022년 정책방향")
        let Some(core) = load_document("samples/2022년 국립국어원 업무계획.hwp") else {
            return;
        };
        let count = core
            .pagination
            .iter()
            .flat_map(|pr| pr.pages.iter())
            .filter(|p| p.page_hide.is_some())
            .count();
        assert!(
            count >= 2,
            "국립국어원 업무계획.hwp page_hide 매핑 페이지 2건 이상 (실제: {}). \
             셀 안 PageHide 누락 가능성: 결함 #1",
            count
        );
    }

    #[test]
    fn test_705_ktx_cell_pagehide_collected() {
        // Stage 0 측정: 본문 PageHide 1건 + 셀 안 PageHide 1건 (셀[10]/p[0] -----P "Ⅰ. 사업 개요")
        let Some(core) = load_document("samples/KTX.hwp") else {
            return;
        };
        let count = core
            .pagination
            .iter()
            .flat_map(|pr| pr.pages.iter())
            .filter(|p| p.page_hide.is_some())
            .count();
        assert!(
            count >= 2,
            "KTX.hwp page_hide 매핑 페이지 2건 이상 (실제: {}). \
             셀 안 PageHide 누락 가능성: 결함 #1",
            count
        );
    }

    /// treat_as_char(TAC) table host-line trailing spacing must be applied
    /// even when the table's control index differs from its line-seg index.
    ///
    /// `layout_table_item` looks up the host line via
    /// `para.line_segs.get(control_index)`, but `control_index` indexes the
    /// paragraph's CONTROL array. When invisible controls (SectionDef,
    /// ColumnDef, bookmarks, ...) precede the table, the lookup misses and
    /// the host line's `line_spacing` is silently dropped, pulling all
    /// following content up by that amount.
    ///
    /// Fixture: synthetic samples/tac-host-spacing.hwpx — host paragraph is
    /// [secPr, colPr, bookmark, TAC table(h=2994, outMargin 283)] with a
    /// Hancom-style lineSegArray (host vertsize=3560, spacing=600); the next
    /// paragraph's authoritative vertical_pos is 4160 HWPUNIT.
    ///
    /// Expected: first char of "NEXT PARAGRAPH" baseline at
    ///   (4251+2834+4160)/75 + 0.85*1000/75 = ~161.3 px.
    /// Buggy (spacing dropped): ~153.3 px.
    #[test]
    fn test_tac_host_line_spacing_with_preceding_invisible_controls() {
        let Some(core) = load_document("samples/tac-host-spacing.hwpx") else {
            return;
        };
        let svg = core.render_page_svg_native(0).unwrap_or_default();
        assert!(!svg.is_empty(), "fixture page 1 SVG is empty");

        let find_text_y = |needle: &str| -> Option<f64> {
            for chunk in svg.split("<text").skip(1) {
                let Some(gt) = chunk.find('>') else {
                    continue;
                };
                let attrs = &chunk[..gt];
                let body = &chunk[gt + 1..chunk.find("</text>").unwrap_or(chunk.len())];
                if body.trim() == needle {
                    return attrs
                        .split("y=\"")
                        .nth(1)
                        .and_then(|r| r.split('\"').next())
                        .and_then(|v| v.parse::<f64>().ok());
                }
            }
            None
        };

        // 한컴 PDF 기준 CELL baseline은 셀 중앙 쪽이다. 회귀 전에는 셀 하단
        // 근처(~133.5px)에 그려져 세로 가운데 정렬이 적용되지 않았다.
        let cell_y = find_text_y("C").expect("'C' of CELL not found in SVG");
        assert!(
            (cell_y - 122.1).abs() < 2.0,
            "CELL baseline {} px — expected ~122.1 from Hancom PDF centered cell text",
            cell_y
        );

        let y = find_text_y("N").expect("'N' of NEXT PARAGRAPH not found in SVG");
        assert!(
            (y - 161.3).abs() < 2.0,
            "next paragraph baseline {} px — expected ~161.3 (file lineSegArray              vertical_pos=4160); ~153.3 means the TAC host line_spacing was dropped",
            y
        );
    }

    // === Issue #4334 분해 1단계: 히트테스트 tie-break 회귀 pin (서수화 전 안전망) ===
    //
    // 한컴 권위 샘플 `samples/textbox-under-image.hwp` — 글상자(InFrontOfText,
    // plane 3) 가 이미지(Square, plane 2) 앞에 있다(task1280_v2 로 이미 검증된
    // 사실, `task1280_v2_control_layout_exposes_plane_z_order_stable_index` 참고).
    // 겹침 클릭 시 "위에 보이는 개체(글상자) 선택" 이 한컴 편집기 육안 대조로 확정된
    // 기대값이다(`mydocs/report/task_m100_1280_v2_report.md`). 이 테스트는
    // `get_page_control_layout_native` 가 노출하는 `(plane, zOrder, stableIndex)` 를
    // TS `controlTopKey`(input-handler-picture.ts:104-105) 와 동일하게 사전식
    // 최댓값으로 비교해 실제로 글상자가 이기는지 Rust 쪽에서 고정한다.
    //
    // 이 케이스는 plane 차이(3 > 2)로 이미 결정되어 stableIndex 자체는 tie-break에
    // 관여하지 않는다 — 여러 실제 fixture(issue1921/59043, task2093/1192000,
    // aift.hwp, 21_언어_기출_편집가능본.hwp 등, 수십~수백 페이지)를 뒤졌지만 stableIndex
    // 로만 갈리는 **공간적으로 실제 겹치는** 실제 문서 사례를 찾지 못했다 — layer=None
    // 폴백을 쓰는 인라인 컨트롤끼리는 흐름(flow) 특성상 서로 겹치지 않고, topAndBottom
    // 류 float 도 서로 겹치지 않게 배치되는 경향이 있다. 그래서 stableIndex 갈래(양쪽
    // 공식 각각의 현재 값)는 `layout/tests.rs`의
    // `issue_4334_paper_node_sort_key_no_longer_depends_on_node_id`(신규)와
    // `task1197_paper_nodes_sort_by_plane_z_order_and_stable_index`(기존)가 Rust
    // 내부 정렬 단위에서 고정한다.
    //
    // [#4334 갱신] `stableIndex` 는 이제 스칼라가 아니라 문서 경로 배열이다
    // (`[secIdx, paraIdx, ...cell 경로, controlIdx]`) — 이 fixture 는 셀 중첩이 없어
    // `[secIdx, paraIdx, controlIdx]` 3원소. 값 자체가 `next_id()` 카운터가 아니라
    // `secIdx/paraIdx/controlIdx` 를 그대로 반영하므로, 아래 JSON 의 `controlIdx`
    // 필드와 정확히 일치해야 한다(우연이 아니라 `doc_path_for_node` 의 정의).
    #[test]
    fn issue_4334_stage1_textbox_under_image_top_object_pin() {
        fn parse_controls(json: &str) -> Vec<(String, i64, i64, Vec<i64>)> {
            fn extract_i64(slice: &str, key: &str) -> i64 {
                let needle = format!("\"{}\":", key);
                let start = slice
                    .find(&needle)
                    .unwrap_or_else(|| panic!("{key} 필드 없음: {slice}"))
                    + needle.len();
                let tail = &slice[start..];
                let end = tail
                    .find(|c: char| c == ',' || c == '}')
                    .unwrap_or(tail.len());
                tail[..end]
                    .trim()
                    .parse()
                    .unwrap_or_else(|_| panic!("{key} 파싱 실패: {slice}"))
            }
            fn extract_i64_array(slice: &str, key: &str) -> Vec<i64> {
                let needle = format!("\"{}\":[", key);
                let start = slice
                    .find(&needle)
                    .unwrap_or_else(|| panic!("{key} 배열 필드 없음: {slice}"))
                    + needle.len();
                let tail = &slice[start..];
                let end = tail
                    .find(']')
                    .unwrap_or_else(|| panic!("{key} 배열 종료 없음: {slice}"));
                let body = tail[..end].trim();
                if body.is_empty() {
                    return Vec::new();
                }
                body.split(',')
                    .map(|s| {
                        s.trim()
                            .parse()
                            .unwrap_or_else(|_| panic!("{key} 원소 파싱 실패: {slice}"))
                    })
                    .collect()
            }
            let marker = "\"type\":\"";
            let positions: Vec<usize> = json.match_indices(marker).map(|(i, _)| i).collect();
            positions
                .iter()
                .enumerate()
                .map(|(n, &start)| {
                    let end = positions.get(n + 1).copied().unwrap_or(json.len());
                    let slice = &json[start..end];
                    let type_start = start + marker.len();
                    let type_end = json[type_start..].find('"').unwrap() + type_start;
                    let type_name = json[type_start..type_end].to_string();
                    (
                        type_name,
                        extract_i64(slice, "plane"),
                        extract_i64(slice, "zOrder"),
                        extract_i64_array(slice, "stableIndex"),
                    )
                })
                .collect()
        }

        let data = std::fs::read("samples/textbox-under-image.hwp");
        let Ok(data) = data else {
            eprintln!("테스트 파일 없음: samples/textbox-under-image.hwp — 건너뜀");
            return;
        };
        let mut core = crate::document_core::DocumentCore::from_bytes(&data).expect("load");
        core.paginate();
        let json = core
            .get_page_control_layout_native(0)
            .expect("control layout for page 0");

        let controls = parse_controls(&json);
        assert_eq!(controls.len(), 2, "글상자+이미지 2개 컨트롤 기대: {json}");

        // TS `controlTopKey`/`isAboveControl` 재현: (plane, zOrder, stableIndex) 사전식
        // 최댓값이 최상단. `Vec<i64>` 의 `Ord` 는 원소별 비교 후 길이로, TS 쪽 새
        // `compareLexArrays` 와 동일 의미.
        let winner = controls
            .iter()
            .max_by_key(|(_, plane, z, stable)| (*plane, *z, stable.clone()))
            .expect("빈 목록 아님");

        assert_eq!(
            winner.0, "shape",
            "겹침 클릭 시 글상자(위)가 이미지(아래)를 이겨야 한다(한컴 권위 샘플 실측). \
             controls={controls:?}"
        );
        // 원본 값도 그대로 pin — 바뀌면 이 테스트가 즉시 알려준다.
        let shape = controls.iter().find(|c| c.0 == "shape").unwrap();
        let image = controls.iter().find(|c| c.0 == "image").unwrap();
        assert_eq!(
            (shape.1, shape.2),
            (3, 0),
            "shape plane/zOrder: {controls:?}"
        );
        assert_eq!(
            (image.1, image.2),
            (2, 1),
            "image plane/zOrder: {controls:?}"
        );
        // [#4334] stableIndex = [secIdx, paraIdx, controlIdx] — 글상자 controlIdx=2,
        // 이미지 controlIdx=3(둘 다 secIdx=0, paraIdx=0, 셀 중첩 없음). doc_path_for_node
        // 정의를 그대로 반영하는 값이라 controlIdx 가 바뀌면 이 값도 같이 바뀐다.
        assert_eq!(shape.3, vec![0, 0, 2], "shape stableIndex: {controls:?}");
        assert_eq!(image.3, vec![0, 0, 3], "image stableIndex: {controls:?}");
    }

    // === Issue #4334 "문서 경로 배열" 구현 후 잔여 확인 — collect_controls/
    // sort_paper_render_nodes 두 집합에서 문서 위치를 못 만드는 노드가 남아있는지 실측 ===
    //
    // 1단계 실측(42/1680, 폐기된 pin)에서 원인 셋을 찾아 고쳤다 — 전부 "host 는 있지만
    // 안 이어져 있던" 플러밍 결손:
    //   - TAC(text-as-char) 중첩 표 — `table_cell_content.rs`(`layout_embedded_table`)가
    //     `enclosing_ctx`(호스트 경로) 를 갖고 있으면서도 `TableNode.section_index/
    //     para_index/control_index/cell_context` 를 전부 `None` 으로 버렸다. 이제
    //     `enclosing_ctx` 를 그대로 옮겨 담는다.
    //   - 바탕쪽(master page) `Control::Picture` — `layout.rs`의 `build_master_page` 가
    //     `Control::Table`/`Shape` 분기는 이미 바탕쪽 로컬 `(pi, ci)` 를 넘기면서
    //     `Control::Picture` 분기만 `None, None` 을 넘겼다. 이제 `Some(pi), Some(ci)`.
    //   - 재귀 중첩 표(recursive nested table) 3곳(`table_layout.rs` 2곳,
    //     `table_partial.rs` 1곳) — `enclosing_cell_ctx`(`nested_ctx`) 는 넘기면서
    //     `table_meta` 는 `None` 이라 para/control 이 항상 비었다. 이제 `nested_ctx`
    //     경로의 마지막 두 항목에서 유도한다(기존에 이미 있던
    //     `layout_partial_table_item` 패턴과 동일).
    //
    // 실측: Table/Equation/Image 1,680개 중 **42 → 24개**로 줄었다. 남은 24개는 전부
    // Image 이고 **하나의 원인**으로 수렴한다 — `render_cell_background`
    // (`table_layout.rs:3773`, 표 셀 배경 무늬/이미지 채우기) 가 `fill_color`/
    // `gradient`/`pattern` 배경과 같은 자리에서 만드는 **이미지 채우기 장식**은
    // `ImageNode::new(..)` 기본값(`section_index`/`para_index`/`control_index`
    // 전부 `None`)을 그대로 쓴다. 이 함수는 애초에 section/para/control/cell_context
    // 를 매개변수로 받지 않는다 — 셀 지오메트리(`cell_x/y/w/h`)와 테두리 스타일만
    // 받는 순수 장식 렌더러. 이 이미지 장식은 **독립된 문서 Control 이 아니다** — 자기가
    // 채우는 셀의 border-fill 스타일에서 파생된 값이라 "이 이미지의 문서 위치"라는
    // 질문 자체가 그 셀과 별개로는 의미가 없다(#4334 코디네이터 질문 1번 — host 조차
    // 없는 카테고리, 스칼라든 배열이든 자기 자신의 문서 위치는 없다). `render_cell_background`
    // 에 문서 경로 매개변수를 추가로 뚫는 건(호출부 전부 갱신) 이번 1단계 범위 밖이라
    // 하지 않았다 — `doc_path_for_node` 는 이 경우 빈 경로로 결정적으로 폴백한다
    // (`paper_node_sort_key` 참고, node.id 는 안 읽는다).
    //
    // `sort_paper_render_nodes` 가 정렬하는 집합(`paper_images`, 정렬 후
    // `tree.root.children` 에 그대로 붙는다) 은 이번 라운드에서 손대지 않은 축이라
    // 여전히 `layer=None` 비중이 크다(56%, PageBg/Header/Footer/Body/MasterPage/
    // FootnoteArea 제외) — `layer` 유무와 `doc_path_for_node` 성공 여부는 서로 다른
    // 축이라(레이어 없어도 para/control 은 있을 수 있음) 이 값 자체는 "문서 위치를
    // 못 구한다"는 뜻이 아니다. 참고용으로 계속 측정한다.
    //
    // 셀 중첩(코디네이터 질문 4번, 유지): Equation/Image 1,680개 중 391개(23%)가
    // `cell_index`/`cell_context` 를 갖는다 — 드문 예외가 아니다. `doc_path_for_node`
    // 는 Table/Image 는 전체 `cell_context`(다단계 경로)를, Rectangle/Line/Ellipse/
    // Path/Equation 은 단일 레벨(`cell_index`/`cell_para_index`/
    // `outer_table_control_index`, Task #1138/#1151 패턴)을 반영한다.
}
