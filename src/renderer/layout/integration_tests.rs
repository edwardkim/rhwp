//! 레이아웃 통합 테스트
//!
//! 실제 HWP 파일을 로딩하여 페이지네이션 + 레이아웃 결과를 검증한다.
//! samples/ 디렉토리에 테스트 파일이 없으면 건너뜀.

#[cfg(test)]
mod tests {
    use std::path::Path;

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

    #[test]
    fn test_exam_math_page_count() {
        let Some(core) = load_document("samples/exam_math.hwp") else {
            return;
        };
        let page_count = core.page_count();
        assert!(
            page_count >= 18,
            "exam_math.hwp 페이지 수 18 이상 (실제: {})",
            page_count
        );
    }

    // ─── 2단 레이아웃 검증 ───

    #[test]
    fn test_exam_math_two_column_layout() {
        let Some(core) = load_document("samples/exam_math.hwp") else {
            return;
        };
        // 1페이지: 2단 레이아웃이어야 함
        let pages = &core.pagination;
        if let Some(result) = pages.first() {
            if let Some(page) = result.pages.first() {
                assert!(
                    page.column_contents.len() >= 2,
                    "exam_math.hwp 1페이지는 2단 이상 (실제: {}단)",
                    page.column_contents.len()
                );
            }
        }
    }

    // ─── 머리말 검증 ───

    #[test]
    fn test_exam_math_no_header_on_first_page() {
        let Some(core) = load_document("samples/exam_math_no.hwp") else {
            return;
        };
        let pages = &core.pagination;
        if let Some(result) = pages.first() {
            if let Some(page) = result.pages.first() {
                assert!(
                    page.active_header.is_none(),
                    "exam_math_no.hwp 1페이지에는 머리말이 없어야 함"
                );
            }
        }
    }

    #[test]
    fn test_exam_math_header_from_second_page() {
        let Some(core) = load_document("samples/exam_math_no.hwp") else {
            return;
        };
        let pages = &core.pagination;
        if let Some(result) = pages.first() {
            if result.pages.len() > 1 {
                let page2 = &result.pages[1];
                assert!(
                    page2.active_header.is_some(),
                    "exam_math_no.hwp 2페이지부터 머리말이 있어야 함"
                );
            }
        }
    }

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

    #[test]
    fn test_equation_svg_content() {
        let Some(core) = load_document("samples/exam_math.hwp") else {
            return;
        };
        let svg = core.render_page_svg_native(0).unwrap_or_default();
        let has_content = svg.contains("<path") || svg.contains("<text");
        assert!(has_content, "수식 페이지 SVG에 렌더링 요소가 있어야 함");
    }

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

    /// Task #469: cross-column 으로 이어지는 paragraph border 박스의 좌·우 세로선이
    /// col_top 위(헤더선 영역) 까지 침범하지 않는지 검증.
    ///
    /// exam_kor.hwp 페이지 2 우측 단의 (나) 박스(border_fill_id=7)는 좌측 단 마지막 줄
    /// 부터 이어지는 partial_start 케이스. 수정 전: 좌·우 세로선이 y=196.55 (헤더선)
    /// 부터 시작. 수정 후: y >= 211.65 (body top, 단 시작 좌표) 이상에서 시작.
    #[test]
    fn test_469_partial_start_box_does_not_cross_col_top() {
        let Some(core) = load_document("samples/exam_kor.hwp") else {
            return;
        };
        let svg = core.render_page_svg_native(1).unwrap_or_default();
        assert!(!svg.is_empty(), "페이지 2 SVG 가 비어있음");

        // 우측 단 영역: x ≈ 582..1005. 페이지 본문 상단(col_top) ≈ 211.65 px.
        // 헤더 가로선은 단일 (y=196.55, x1≈117, x2≈1005, y2 동일) 으로 전체 폭에 그어짐.
        // 우측 단 범위(x1 in [580, 1010]) 의 수직선(y1 != y2) 들은 y1 >= 200 이어야 함.
        let mut violations: Vec<(f64, f64, f64)> = Vec::new();
        for chunk in svg.split("<line ").skip(1) {
            // 다음 '/>' 또는 '>' 이전까지의 속성 파싱
            let end = chunk.find("/>").or_else(|| chunk.find('>')).unwrap_or(chunk.len());
            let attrs = &chunk[..end];
            let parse_attr = |name: &str| -> Option<f64> {
                let pat = format!("{}=\"", name);
                let i = attrs.find(&pat)? + pat.len();
                let j = i + attrs[i..].find('"')?;
                attrs[i..j].parse::<f64>().ok()
            };
            let (x1, y1, x2, y2) = match (parse_attr("x1"), parse_attr("y1"), parse_attr("x2"), parse_attr("y2")) {
                (Some(a), Some(b), Some(c), Some(d)) => (a, b, c, d),
                _ => continue,
            };
            // 수직선(y1 != y2 && x1 == x2) 만 검사
            if (y1 - y2).abs() < 0.5 || (x1 - x2).abs() >= 0.5 {
                continue;
            }
            // 우측 단 영역
            if x1 >= 580.0 && x1 <= 1010.0 {
                let y_top = y1.min(y2);
                if y_top < 200.0 {
                    violations.push((x1, y1, y2));
                }
            }
        }
        assert!(
            violations.is_empty(),
            "우측 단 수직선이 헤더선 영역(y<200) 까지 침범: {:?}",
            violations
        );
    }

    /// Task #470: cross-paragraph vpos-reset 미인식 (cv != 0)
    ///
    /// 21_언어_기출_편집가능본.hwp 페이지 1 의 pi=10 ("적합성 검증이란…") 은
    /// HWP 인코딩상 col 1 시작 (first vpos=9014, pi=9 last vpos=90426).
    /// `cv == 0` 가드만 있던 시절: pi=10 partial 2줄이 col 0 에 강제 삽입되어 overflow.
    /// 수정 후: 전체가 col 1 첫 항목으로 이동.
    #[test]
    fn test_470_cross_paragraph_vpos_reset_with_column_header_offset() {
        let Some(core) = load_document("samples/21_언어_기출_편집가능본.hwp") else {
            return;
        };
        let dump = core.dump_page_items(Some(0));
        assert!(!dump.is_empty(), "페이지 1 dump 가 비어있음");

        // 페이지 1 의 단 0 / 단 1 섹션을 분리해서 pi=10 위치 검증
        // dump 형식 예:
        //   === 페이지 1 ...
        //     단 0 (...)
        //       PartialParagraph pi=10 ...
        //     단 1 (...)
        //       FullParagraph pi=10 ...
        let mut col0_block = String::new();
        let mut col1_block = String::new();
        let mut current_col: i32 = -1;
        for line in dump.lines() {
            if line.trim_start().starts_with("단 0") {
                current_col = 0;
                continue;
            }
            if line.trim_start().starts_with("단 1") {
                current_col = 1;
                continue;
            }
            // 다음 페이지로 넘어가면 중단
            if line.starts_with("=== 페이지") && current_col >= 0 {
                break;
            }
            match current_col {
                0 => col0_block.push_str(line),
                1 => col1_block.push_str(line),
                _ => {}
            }
            col0_block.push('\n');
            col1_block.push('\n');
        }

        // pi=10 이 단 0 에 등장하면 안 됨, 단 1 에는 등장해야 함.
        let col0_has_pi10 = col0_block.contains("pi=10");
        let col1_has_pi10 = col1_block.contains("pi=10");
        assert!(!col0_has_pi10,
            "pi=10 이 col 0 에 배치되어 있음 (cross-column vpos-reset 미감지). col 0 dump:\n{}",
            col0_block);
        assert!(col1_has_pi10,
            "pi=10 이 col 1 에 등장해야 함. col 1 dump:\n{}",
            col1_block);
    }

    /// Task #471: cross-column 박스 검출(Task #468) 이 stroke_sig 머지(Task #321 v6) 와
    /// 불일치하여 좌측 단 (가) 박스 하단에 잘못된 가로선이 그려지는 회귀.
    ///
    /// 21_언어_기출_편집가능본.hwp 페이지 1: pi=6(bf=7) + pi=7~9(bf=4) 가 stroke_sig
    /// 동일하여 한 그룹으로 머지. 그룹의 g.0=7 (첫 range bf). 다음 paragraph pi=10 은
    /// bf=4. bf_id 비교로는 7 != 4 → partial_end 미설정 → 4면 Rectangle 으로 그려져
    /// 하단 가로선 발생.
    #[test]
    fn test_471_cross_column_box_no_bottom_line_in_col0() {
        let Some(core) = load_document("samples/21_언어_기출_편집가능본.hwp") else {
            return;
        };
        let svg = core.render_page_svg_native(0).unwrap_or_default();
        assert!(!svg.is_empty(), "페이지 1 SVG 가 비어있음");

        // body bottom = 1436.2. cross-column 박스의 하단 가로선이 그려진다면
        // y ≈ 1436~1442 부근에 stroke 가 있는 4면 rect 또는 가로 line 이 존재.
        // body_clip 안의 좌측 단 영역 (x in [120, 542]) 에서 stroke 가 있는
        // rect 또는 가로 line 의 bottom_y 가 1300 보다 큰 항목이 있는지 검사.
        //
        // 현 구조: 잘못 그려진 단일 Rectangle (4면 stroke) — `<rect ... fill="none"
        // stroke="#000000" stroke-width="0.5"/>` x≈128, y≈558, w≈402, h≈880 (ends_y≈1438).
        let mut violations: Vec<String> = Vec::new();
        for chunk in svg.split("<rect ").skip(1) {
            let end = chunk.find("/>").or_else(|| chunk.find('>')).unwrap_or(chunk.len());
            let attrs = &chunk[..end];
            // stroke 가 있는 rect 만 (fill 만 있는 rect 는 paragraph background)
            if !attrs.contains("stroke=\"#000000\"") && !attrs.contains("stroke=\"#000\"") {
                continue;
            }
            let parse_attr = |name: &str| -> Option<f64> {
                let pat = format!("{}=\"", name);
                let i = attrs.find(&pat)? + pat.len();
                let j = i + attrs[i..].find('"')?;
                attrs[i..j].parse::<f64>().ok()
            };
            let (x, y, w, h) = match (parse_attr("x"), parse_attr("y"), parse_attr("width"), parse_attr("height")) {
                (Some(a), Some(b), Some(c), Some(d)) => (a, b, c, d),
                _ => continue,
            };
            // 좌측 단 영역의 4면 stroke rect 로 bottom 이 col_bottom 근처
            if x >= 120.0 && x <= 542.0 && (x + w) <= 545.0 && (y + h) > 1300.0 {
                violations.push(format!("rect x={} y={} w={} h={} ends_y={}", x, y, w, h, y + h));
            }
        }
        assert!(violations.is_empty(),
            "좌측 단 (가) 박스에 4면 stroke rect 가 그려짐 (cross-column 검출 실패): {:?}",
            violations);
    }

    /// Task #490: 빈 텍스트 + TAC 수식만 있는 셀 paragraph 의 alignment 적용.
    ///
    /// 케이스: `samples/exam_science.hwp` 페이지 1 의 3번 표 (pi=12, 4행×4열,
    /// "이온 결합 화합물") 의 셀 7 (행1, 열3) "전체 전자의 양" 컬럼 28 수식.
    /// 셀 paragraph 는 text_len=0 + ctrls=1 (수식) 구조. 수정 전: empty-runs
    /// 분기 (`paragraph_layout.rs:2227`) 가 `inline_x = effective_col_x +
    /// effective_margin_left` 로 좌측 고정 → 28 수식이 셀 좌측에 정렬.
    /// 수정 후: paragraph alignment(Center) 따라 align_offset 적용 → 수식이
    /// 셀 중앙 부근에 정렬.
    ///
    /// 검증: 28 수식의 그룹 transform x 좌표가 수정 전(x≈358) 보다 우측
    /// (x>400) 으로 이동했는지 확인. 셀 7 영역(x≈336..478) 의 좌측 1/3
    /// 범위(<395) 에 있으면 결함, 그 이후면 alignment 정상 적용.
    #[test]
    fn test_490_empty_para_with_tac_equation_respects_alignment() {
        let Some(core) = load_document("samples/exam_science.hwp") else {
            return;
        };
        let svg = core.render_page_svg_native(0).unwrap_or_default();
        assert!(!svg.is_empty(), "exam_science 페이지 1 SVG 가 비어있음");

        // 28 수식 위치 추출. SVG 구조: <g transform="translate(X, Y) scale(...)">
        //                              <text x="0" y="...">28</text>
        //                              </g>
        // "28" 텍스트 직전의 group transform x 좌표를 찾는다.
        let needle = ">28<";
        let mut found_xs: Vec<f64> = Vec::new();
        let mut search_start = 0;
        while let Some(pos) = svg[search_start..].find(needle) {
            let abs_pos = search_start + pos;
            let context_start = abs_pos.saturating_sub(2000);
            let context = &svg[context_start..abs_pos];
            // 가장 가까운 직전 `<g transform="translate(X` 패턴 찾기
            if let Some(g_rel) = context.rfind("<g transform=\"translate(") {
                let after_translate = &context[g_rel + "<g transform=\"translate(".len()..];
                if let Some(comma) = after_translate.find(',') {
                    if let Ok(x) = after_translate[..comma].parse::<f64>() {
                        // y 좌표로 3번 표 영역 (y ≈ 1040..1090) 인지 확인
                        let after_comma = &after_translate[comma + 1..];
                        if let Some(close_paren) = after_comma.find(')') {
                            if let Ok(y) = after_comma[..close_paren].parse::<f64>() {
                                if (1040.0..1090.0).contains(&y) {
                                    found_xs.push(x);
                                }
                            }
                        }
                    }
                }
            }
            search_start = abs_pos + needle.len();
        }

        assert!(
            !found_xs.is_empty(),
            "Task #490: 3번 표 영역(y∈[1040,1090])의 28 수식 transform 을 찾지 못함"
        );

        // 셀 7 영역: x≈336.8..478.0 (140 px). 좌측 1/4 한계: 372.
        // 수정 전: x≈358.7 (좌측 정렬). 수정 후: x>=400 (alignment 적용).
        for x in &found_xs {
            assert!(
                *x >= 380.0,
                "Task #490: 28 수식이 좌측 정렬됨 (x={:.1} < 380). 셀 paragraph alignment 적용 안 됨",
                x
            );
        }
    }

    /// Task #565: 텍스트가 있는 줄의 TAC 표 문단에서 수식이 fallback 좌표에
    /// 겹치지 않고 각 줄의 인라인 위치에 배치되는지 검증.
    ///
    /// 케이스: `samples/exam_science.hwp` 12/15/18/19번 문제.
    /// 12번 문단에는 TAC 표 1개 뒤에 `rmX`, `rmA`, `rmB`, `rmC`, `rmD`,
    /// `m-4`, `m-2`, `m+2`, `m+4` 수식 9개가 있고, 나머지 보고 문단도
    /// 같은 "TAC 표 + TAC 수식" 흐름을 가진다.
    /// 수정 전: 9개 수식이 모두 같은 fallback 좌표(x≈534.8, y≈1262.0)에 겹침.
    #[test]
    fn test_565_exam_science_inline_tac_equations_are_not_collapsed() {
        let Some(core) = load_document("samples/exam_science.hwp") else {
            return;
        };

        fn collect_equations(
            node: &crate::renderer::render_tree::RenderNode,
            para_index: usize,
            expected_controls: &std::collections::BTreeSet<usize>,
            out: &mut Vec<(usize, i32, i32)>,
        ) {
            if let crate::renderer::render_tree::RenderNodeType::Equation(eq) = &node.node_type {
                if eq.section_index == Some(0)
                    && eq.para_index == Some(para_index)
                    && eq.control_index
                        .map(|ci| expected_controls.contains(&ci))
                        .unwrap_or(false)
                {
                    let ci = eq.control_index.unwrap();
                    out.push((
                        ci,
                        (node.bbox.x * 10.0).round() as i32,
                        (node.bbox.y * 10.0).round() as i32,
                    ));
                }
            }
            for child in &node.children {
                collect_equations(child, para_index, expected_controls, out);
            }
        }

        let cases: &[(u32, usize, &[usize], usize, usize, usize)] = &[
            (1, 61, &[1, 2, 3, 4, 5, 6, 7, 8, 9], 8, 5, 2),
            (2, 79, &[0, 2, 4, 5, 6, 7, 8, 9, 10], 8, 5, 2),
            (3, 110, &[0, 2, 3, 4, 5, 6, 7], 6, 4, 2),
            (3, 118, &[0, 1, 2, 3, 4, 5, 6, 8], 7, 4, 2),
        ];

        for &(page_index, para_index, controls, min_positions, min_xs, min_ys) in cases {
            let tree = core
                .build_page_render_tree(page_index)
                .expect("페이지 렌더 트리 생성");
            let expected_controls: std::collections::BTreeSet<usize> =
                controls.iter().copied().collect();

            let mut equations = Vec::new();
            collect_equations(&tree.root, para_index, &expected_controls, &mut equations);
            equations.sort_by_key(|(ci, _, _)| *ci);

            assert_eq!(
                equations.len(),
                controls.len(),
                "Task #565: pi={} TAC 수식이 한 번씩 렌더되어야 함: {:?}",
                para_index,
                equations
            );

            let unique_positions: std::collections::BTreeSet<(i32, i32)> =
                equations.iter().map(|(_, x, y)| (*x, *y)).collect();
            let unique_xs: std::collections::BTreeSet<i32> =
                equations.iter().map(|(_, x, _)| *x).collect();
            let unique_ys: std::collections::BTreeSet<i32> =
                equations.iter().map(|(_, _, y)| *y).collect();

            assert!(
                unique_positions.len() >= min_positions,
                "Task #565: pi={} TAC 수식이 같은 좌표에 겹침: {:?}",
                para_index,
                equations
            );
            assert!(
                unique_xs.len() >= min_xs && unique_ys.len() >= min_ys,
                "Task #565: pi={} TAC 수식이 줄별 인라인 위치로 분산되지 않음: {:?}",
                para_index,
                equations
            );
        }

        fn collect_problem_19_answer_line(
            node: &crate::renderer::render_tree::RenderNode,
            eq_x: &mut Option<f64>,
            table_x: &mut Option<f64>,
            text_x: &mut Option<f64>,
        ) {
            match &node.node_type {
                crate::renderer::render_tree::RenderNodeType::Equation(eq)
                    if eq.section_index == Some(0)
                        && eq.para_index == Some(120)
                        && eq.control_index == Some(0) =>
                {
                    *eq_x = Some(node.bbox.x);
                }
                crate::renderer::render_tree::RenderNodeType::Table(table)
                    if table.section_index == Some(0)
                        && table.para_index == Some(120)
                        && table.control_index == Some(1) =>
                {
                    *table_x = Some(node.bbox.x);
                }
                crate::renderer::render_tree::RenderNodeType::TextRun(run)
                    if run.section_index == Some(0)
                        && run.para_index == Some(120)
                        && run.char_start == Some(3)
                        && run.text.trim_start().starts_with("은?") =>
                {
                    *text_x = Some(node.bbox.x);
                }
                _ => {}
            }
            for child in &node.children {
                collect_problem_19_answer_line(child, eq_x, table_x, text_x);
            }
        }

        let tree = core.build_page_render_tree(3).expect("페이지 4 렌더 트리 생성");
        let mut eq_x = None;
        let mut table_x = None;
        let mut text_x = None;
        collect_problem_19_answer_line(&tree.root, &mut eq_x, &mut table_x, &mut text_x);

        let eq_x = eq_x.expect("Task #565: 19번 답문항 선행 수식 x TIMES 렌더");
        let table_x = table_x.expect("Task #565: 19번 답문항 분수 TAC 표 렌더");
        let text_x = text_x.expect("Task #565: 19번 답문항 본문 텍스트 렌더");
        assert!(
            eq_x < table_x && table_x < text_x,
            "Task #565: 같은 text position 의 TAC 수식/표가 텍스트 앞 순서로 배치되어야 함: \
             eq_x={eq_x}, table_x={table_x}, text_x={text_x}"
        );
    }

    /// Task #489: Picture+Square wrap (어울림) 호스트 paragraph 의 텍스트가
    /// 그림 영역을 침범하지 않고 LINE_SEG.cs/sw 좁아진 영역에 정상 배치되는지 검증.
    ///
    /// 케이스: `samples/exam_science.hwp` 페이지 1 컬럼 1 (단 1) 의 5번 문제 본문
    /// (pi=21). HWP IR: 그림(11250×10230 HU, wrap=Square, horz_align=Right) +
    /// 6 줄 LINE_SEG cs=0, sw=19592 (~261px, 컬럼 너비 ~412px 에서 그림 너비
    /// 만큼 좁아짐).
    ///
    /// 수정 전: 풀컬럼 너비로 justify → 텍스트가 그림 영역(x=807..957) 침범.
    /// 수정 후: segment_width 적용 → 텍스트 우측 끝이 x≈798 이내, 그림과 겹치지 않음.
    #[test]
    fn test_489_picture_square_wrap_text_does_not_overlap_image() {
        let Some(core) = load_document("samples/exam_science.hwp") else {
            return;
        };
        let svg = core.render_page_svg_native(0).unwrap_or_default();
        assert!(!svg.is_empty(), "exam_science 페이지 1 SVG 가 비어있음");

        // ─── 그림 위치 파싱 ───────────────────────────────────────
        // pi=21 ci=0 그림: width=150 (= 39.7mm @ 75 HU/px), height≈136.
        // 다른 그림(width=258, 110, 102 등) 과 구분되도록 width 기준으로 식별.
        fn parse_attr_f64(s: &str, key: &str) -> Option<f64> {
            let pat = format!("{}=\"", key);
            let p = s.find(&pat)?;
            let val_start = p + pat.len();
            let rest = &s[val_start..];
            let q = rest.find('"')?;
            rest[..q].parse().ok()
        }
        let mut img_rect: Option<(f64, f64, f64, f64)> = None;
        for chunk in svg.split("<image").skip(1) {
            let end = chunk.find("/>").unwrap_or(chunk.len());
            let attrs = &chunk[..end];
            let w = parse_attr_f64(attrs, "width").unwrap_or(0.0);
            let h = parse_attr_f64(attrs, "height").unwrap_or(0.0);
            // pi=21 그림 식별: width≈150 (148~152) AND height≈136 (134~138)
            if (w - 150.0).abs() < 2.0 && (h - 136.4).abs() < 2.0 {
                let x = parse_attr_f64(attrs, "x").unwrap_or(0.0);
                let y = parse_attr_f64(attrs, "y").unwrap_or(0.0);
                img_rect = Some((x, y, w, h));
                break;
            }
        }
        let (img_x, img_y, img_w, img_h) = img_rect
            .expect("Task #489: pi=21 ci=0 그림 (width≈150 height≈136) 을 SVG 에서 찾지 못함");
        let img_left = img_x;
        let img_right = img_x + img_w;
        let img_top = img_y;
        let img_bottom = img_y + img_h;

        // ─── 텍스트 위치 파싱 ─────────────────────────────────────
        // <text ... transform="translate(x,y) ...">텍스트</text>
        let mut overlap_chars: Vec<(f64, f64, String)> = Vec::new();
        for chunk in svg.split("<text").skip(1) {
            let close = chunk.find('>').unwrap_or(chunk.len());
            let header = &chunk[..close];
            let body_end = chunk.find("</text>").unwrap_or(chunk.len());
            let body = &chunk[close + 1..body_end];

            let trans_pat = "transform=\"translate(";
            let Some(tp) = header.find(trans_pat) else { continue };
            let trans_str_start = tp + trans_pat.len();
            let trans_rest = &header[trans_str_start..];
            let Some(close_paren) = trans_rest.find(')') else { continue };
            let trans_args = &trans_rest[..close_paren];
            let mut parts = trans_args.split(',');
            let x: f64 = match parts.next().and_then(|s| s.trim().parse().ok()) {
                Some(v) => v,
                None => continue,
            };
            let y: f64 = match parts.next().and_then(|s| s.trim().parse().ok()) {
                Some(v) => v,
                None => continue,
            };

            // 그림의 수직 영역 안에서 그림 가로 영역에 있는 텍스트는 침범.
            if y > img_top && y < img_bottom && x >= img_left && x < img_right {
                overlap_chars.push((x, y, body.to_string()));
            }
        }

        assert!(
            overlap_chars.is_empty(),
            "Task #489: pi=21 텍스트가 그림 영역(x={:.1}..{:.1} y={:.1}..{:.1}) 에 침범: {:?}",
            img_left, img_right, img_top, img_bottom, overlap_chars,
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

    /// Task #537: TAC `<보기>` 표 직후 첫 답안(①) → 다음 답안(②) gap 이
    /// IR `LINE_SEG.vpos` delta 와 일치해야 한다.
    ///
    /// 21_언어_기출_편집가능본.hwp 페이지 2 의 3번 문제 답안:
    ///   pi=37 = TAC <보기> 표
    ///   pi=38 = ① 답 (3 라인, lh=1100, ls=716)
    ///   pi=39 = ② 답 (3 라인, lh=1100, ls=716)
    ///   pi=40 = ③ 답 (2 라인)
    ///
    /// IR vpos delta:
    ///   pi=38→pi=39: 5448 HU = 72.64 px (= 3*(lh+ls))
    ///   pi=39→pi=40: 5448 HU = 72.64 px
    ///
    /// 버그(수정 전): lazy_base 가 sequential drift(trailing-ls 제외) 를
    /// 동결시켜 pi=39 부터 IR_vpos − 716 HU 위치에 배치 →
    /// ①→② gap 이 63.09 px (4732 HU, 1 ls 부족) 으로 좁아짐.
    /// 수정 후: ①→② gap == ②→③ gap == 72.64 px.
    #[test]
    fn test_537_first_answer_after_tac_table_line_spacing() {
        let Some(core) = load_document("samples/21_언어_기출_편집가능본.hwp") else {
            return;
        };
        let svg = core.render_page_svg_native(1).unwrap_or_default();
        assert!(!svg.is_empty(), "페이지 2 SVG 가 비어있음");

        // SVG 에서 ① ② ③ 의 baseline y 추출.
        // 형식: <text transform="translate(X,Y) ...">①</text>
        // X 는 단 0 시작 부근 (좌측 단 답안 마커 위치).
        let extract_y = |needle: char| -> Option<f64> {
            for chunk in svg.split("<text ") {
                if let Some(pos) = chunk.rfind(&format!(">{}</text>", needle)) {
                    // 같은 chunk 의 transform 에서 두 번째 숫자(=y) 파싱
                    let attrs = &chunk[..pos];
                    let tr = attrs.find("translate(")?;
                    let after = &attrs[tr + "translate(".len()..];
                    let close = after.find(')')?;
                    let inside = &after[..close];
                    let mut parts = inside.split(',');
                    let _x = parts.next()?.trim();
                    let y = parts.next()?.trim().parse::<f64>().ok()?;
                    return Some(y);
                }
            }
            None
        };

        let y1 = extract_y('①').expect("① not found in page 2 SVG");
        let y2 = extract_y('②').expect("② not found in page 2 SVG");
        let y3 = extract_y('③').expect("③ not found in page 2 SVG");

        let gap_12 = y2 - y1;
        let gap_23 = y3 - y2;

        // pi=38 (3 라인), pi=39 (3 라인) 동일 ParaShape → 두 gap 이 같아야 함.
        // IR vpos delta 5448 HU = 72.64 px. 부동소수 톨러런스 0.5 px.
        assert!(
            (gap_12 - gap_23).abs() < 0.5,
            "①→② gap({:.2}) 와 ②→③ gap({:.2}) 가 일치해야 함. \
             y1={:.2}, y2={:.2}, y3={:.2}. \
             버그(수정 전): gap_12=63.09, gap_23=72.64.",
            gap_12, gap_23, y1, y2, y3
        );

        // IR delta 정합 검증: 5448 HU = 72.64 px.
        let expected_gap = (5448.0_f64 * 96.0 / 7200.0_f64);
        assert!(
            (gap_12 - expected_gap).abs() < 0.5,
            "①→② gap({:.2}) 가 IR vpos delta({:.2}) 와 일치해야 함",
            gap_12, expected_gap
        );
    }

    /// Task #539: InFrontOfText + treat_as_char Shape 호스트 paragraph 직후
    /// 다음 paragraph 의 줄간격이 IR vpos delta 와 일치해야 한다.
    ///
    /// 21_언어_기출_편집가능본.hwp 페이지 7:
    ///   pi=145 = "68혁명 이후..." (controls=1: InFrontOfText tac=true Shape, 글박스)
    ///   pi=146 = "르포르는 1789년..."
    ///
    /// IR vpos delta (pi=145 last seg → pi=146 first seg):
    ///   pi=145 last seg vpos≈24969+1100+716 = 26785 = pi=146 first seg vpos. delta = 1816 HU = 24.21 px.
    ///
    /// 버그(수정 전): pi=145 의 InFrontOfText Shape 가 `prev_has_overlay_shape`
    /// 가드를 발동시켜 pi=146 의 vpos correction 자체를 skip → drift 716 HU(=9.55 px)
    /// 잔존하여 pi=145 마지막 line(y=555) → pi=146 첫 line(y=569.81) gap = 14.67 px.
    /// 수정 후: gap = 24.21 px (IR delta 정확).
    #[test]
    fn test_539_paragraph_after_overlay_shape_host() {
        let Some(core) = load_document("samples/21_언어_기출_편집가능본.hwp") else {
            return;
        };

        // 페이지 7 SVG 에서 col 0 영역의 '르' 첫 등장 (pi=146 첫 글자)
        // 과 그 이전 줄의 글자 baseline y 추출
        let svg = core.render_page_svg_native(6).unwrap_or_default();
        assert!(!svg.is_empty(), "페이지 7 SVG 가 비어있음");

        // col 0 (x≈140-200) 의 글자별 (y, x, char) 수집
        let mut points: Vec<(f64, f64, String)> = Vec::new();
        for chunk in svg.split("<text ") {
            if let Some(close) = chunk.find("</text>") {
                let attrs_and_content = &chunk[..close];
                if let Some(gt) = attrs_and_content.find('>') {
                    let attrs = &attrs_and_content[..gt];
                    let content = &attrs_and_content[gt + 1..];
                    if content.chars().count() != 1 { continue; }
                    let tr = match attrs.find("translate(") {
                        Some(p) => p + "translate(".len(),
                        None => continue,
                    };
                    let close_paren = match attrs[tr..].find(')') {
                        Some(p) => p,
                        None => continue,
                    };
                    let inside = &attrs[tr..tr + close_paren];
                    let mut parts = inside.split(',');
                    let x = parts.next().and_then(|s| s.trim().parse::<f64>().ok());
                    let y = parts.next().and_then(|s| s.trim().parse::<f64>().ok());
                    if let (Some(x), Some(y)) = (x, y) {
                        if (130.0..=200.0).contains(&x) && (500.0..=700.0).contains(&y) {
                            points.push((y, x, content.to_string()));
                        }
                    }
                }
            }
        }
        points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        // '르' 의 첫 등장 (pi=146 의 첫 글자) y
        let leporeut_y = points.iter()
            .find(|(_, _, c)| c == "르")
            .map(|(y, _, _)| *y)
            .expect("페이지 7 col 0 에서 '르' 를 찾을 수 없음");

        // '르' 직전 line 의 글자 y 찾기 (gap 측정용)
        let prev_line_y = points.iter()
            .filter(|(y, _, _)| *y < leporeut_y - 0.5)
            .last()
            .map(|(y, _, _)| *y)
            .expect("'르' 직전 line 을 찾을 수 없음");

        let gap = leporeut_y - prev_line_y;
        let expected_gap = 1816.0_f64 * 96.0 / 7200.0;  // 24.21 px

        assert!(
            (gap - expected_gap).abs() < 0.5,
            "pi=145 last line(y={:.2}) → pi=146 first line '르'(y={:.2}) gap({:.2}) 가 \
             IR vpos delta({:.2} px = 1816 HU) 와 일치해야 함. \
             버그(수정 전): gap=14.67 (1 ls=716 HU 부족, prev_has_overlay_shape 가드로 \
             vpos correction skipped).",
            prev_line_y, leporeut_y, gap, expected_gap
        );
    }

    /// Task #539: 페이지 9 의 PartialParagraph + InFrontOfText Shape 호스트 케이스.
    ///
    /// 21_언어_기출_편집가능본.hwp 페이지 9 col 0:
    ///   pi=181 (lines 8..13) = PartialParagraph (controls=1: InFrontOfText tac=true Shape)
    ///   pi=182 = "더불어 수피즘의 의식에..."
    ///
    /// IR vpos delta:
    ///   pi=181 line 12 vpos=7264, pi=182 line 0 vpos=9080. delta = 1816 HU = 24.21 px.
    ///
    /// 버그(수정 전): pi=181 의 InFrontOfText Shape 로 인해 pi=182 의 vpos correction
    /// skipped → gap = 14.67 px (1 ls 부족).
    #[test]
    fn test_539_partial_paragraph_after_overlay_shape() {
        let Some(core) = load_document("samples/21_언어_기출_편집가능본.hwp") else {
            return;
        };
        let svg = core.render_page_svg_native(8).unwrap_or_default();
        assert!(!svg.is_empty(), "페이지 9 SVG 가 비어있음");

        // col 0 영역 (x≈140-540) y 분포 (페이지 9 col 0 상단)
        let mut points: Vec<(f64, f64, String)> = Vec::new();
        for chunk in svg.split("<text ") {
            if let Some(close) = chunk.find("</text>") {
                let attrs_and_content = &chunk[..close];
                if let Some(gt) = attrs_and_content.find('>') {
                    let attrs = &attrs_and_content[..gt];
                    let content = &attrs_and_content[gt + 1..];
                    if content.chars().count() != 1 { continue; }
                    let tr = match attrs.find("translate(") {
                        Some(p) => p + "translate(".len(),
                        None => continue,
                    };
                    let close_paren = match attrs[tr..].find(')') {
                        Some(p) => p,
                        None => continue,
                    };
                    let inside = &attrs[tr..tr + close_paren];
                    let mut parts = inside.split(',');
                    let x = parts.next().and_then(|s| s.trim().parse::<f64>().ok());
                    let y = parts.next().and_then(|s| s.trim().parse::<f64>().ok());
                    if let (Some(x), Some(y)) = (x, y) {
                        if (140.0..=540.0).contains(&x) && (250.0..=400.0).contains(&y) {
                            points.push((y, x, content.to_string()));
                        }
                    }
                }
            }
        }
        points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        // '더' 의 첫 등장 (pi=182 의 첫 글자)
        let deobureo_y = points.iter()
            .find(|(_, _, c)| c == "더")
            .map(|(y, _, _)| *y)
            .expect("페이지 9 col 0 에서 '더' 를 찾을 수 없음");

        let prev_line_y = points.iter()
            .filter(|(y, _, _)| *y < deobureo_y - 0.5)
            .last()
            .map(|(y, _, _)| *y)
            .expect("'더' 직전 line 을 찾을 수 없음");

        let gap = deobureo_y - prev_line_y;
        let expected_gap = 1816.0_f64 * 96.0 / 7200.0;  // 24.21 px

        assert!(
            (gap - expected_gap).abs() < 0.5,
            "pi=181 last line(y={:.2}) → pi=182 first line '더'(y={:.2}) gap({:.2}) 가 \
             IR vpos delta({:.2} px = 1816 HU) 와 일치해야 함. \
             버그(수정 전): gap=14.67 (PartialParagraph 의 overlay Shape 가드로 skipped).",
            prev_line_y, deobureo_y, gap, expected_gap
        );
    }
}
