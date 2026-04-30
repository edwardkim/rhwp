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
}
