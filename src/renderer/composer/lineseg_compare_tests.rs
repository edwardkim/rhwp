//! LINE_SEG 일치율 측정 통합 테스트
//!
//! HWP 파일을 로드하여 원본 LINE_SEG와 reflow_line_segs() 결과를 비교한다.
//! samples/ 디렉토리에 테스트 파일이 없으면 건너뜀.

#[cfg(test)]
mod tests {
    use crate::model::paragraph::LineSeg;
    use crate::renderer::composer::lineseg_compare::*;
    use crate::renderer::composer::reflow_line_segs;
    use crate::renderer::page_layout::PageLayoutInfo;
    use std::path::Path;

    /// HWP 파일을 파싱하여 Document + ResolvedStyleSet 반환
    fn load_raw(
        path: &str,
    ) -> Option<(
        crate::model::document::Document,
        crate::renderer::style_resolver::ResolvedStyleSet,
    )> {
        let p = Path::new(path);
        if !p.exists() {
            eprintln!("테스트 파일 없음: {} — 건너뜀", path);
            return None;
        }
        let data = std::fs::read(p).ok()?;
        let document = crate::parser::parse_document(&data).ok()?;
        let styles = crate::renderer::style_resolver::resolve_styles(
            &document.doc_info,
            96.0, // DEFAULT_DPI
        );
        Some((document, styles))
    }

    /// 섹션의 모든 본문 문단에 대해 LINE_SEG 비교 수행
    fn compare_section(
        document: &mut crate::model::document::Document,
        styles: &crate::renderer::style_resolver::ResolvedStyleSet,
        section_idx: usize,
        dpi: f64,
    ) -> SectionLineSegReport {
        let section = &document.sections[section_idx];
        let page_def = &section.section_def.page_def;

        let total_paragraphs = section.paragraphs.len();
        let mut paragraph_diffs = Vec::new();
        let mut compared = 0usize;
        let mut line_count_match = 0usize;
        let mut line_break_match = 0usize;
        let mut all_match = 0usize;

        for para_idx in 0..total_paragraphs {
            let para = &section.paragraphs[para_idx];

            // 빈 문단이나 LINE_SEG가 없는 문단은 건너뜀
            if para.line_segs.is_empty() || para.text.is_empty() {
                continue;
            }

            // line_height가 0인 문단은 원본 LINE_SEG가 없는 것 (HWPX 등)
            if para.line_segs[0].line_height == 0 {
                continue;
            }

            // 원본 LINE_SEG 복사
            let original_line_segs: Vec<LineSeg> = para.line_segs.clone();

            // ColumnDef 조회
            let column_def = find_column_def_for_paragraph(&section.paragraphs, para_idx);
            let layout = PageLayoutInfo::from_page_def(page_def, &column_def, dpi);
            let col_area = &layout.column_areas[0]; // 기본 단

            // 문단 여백 계산
            let para_style = styles.para_styles.get(para.para_shape_id as usize);
            let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
            let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
            let available_width = col_area.width - margin_left - margin_right;

            // reflow 실행 (문단을 임시 clone하여 원본 보존)
            let mut para_clone = para.clone();
            reflow_line_segs(&mut para_clone, available_width, styles, dpi);

            // 비교
            let diff = compare_line_segs(para_idx, &original_line_segs, &para_clone.line_segs);

            compared += 1;
            if diff.line_count_match {
                line_count_match += 1;
            }
            if diff.line_breaks_match() {
                line_break_match += 1;
            }
            if diff.all_match() {
                all_match += 1;
            }

            paragraph_diffs.push(diff);
        }

        SectionLineSegReport {
            section_idx,
            total_paragraphs,
            compared_paragraphs: compared,
            line_count_match_count: line_count_match,
            line_break_match_count: line_break_match,
            all_match_count: all_match,
            paragraph_diffs,
        }
    }

    /// 문단에 적용되는 ColumnDef 찾기 (DocumentCore::find_column_def_for_paragraph와 동일 로직)
    fn find_column_def_for_paragraph(
        paragraphs: &[crate::model::paragraph::Paragraph],
        para_idx: usize,
    ) -> crate::model::page::ColumnDef {
        use crate::model::control::Control;
        use crate::model::page::ColumnDef;
        let mut last_cd = ColumnDef::default();
        for (i, para) in paragraphs.iter().enumerate() {
            if i > para_idx {
                break;
            }
            for ctrl in &para.controls {
                if let Control::ColumnDef(cd) = ctrl {
                    last_cd = cd.clone();
                }
            }
        }
        last_cd
    }

    /// 단일 HWP 파일에 대해 전체 섹션 LINE_SEG 비교 리포트 생성
    fn run_comparison(path: &str) -> Option<Vec<SectionLineSegReport>> {
        let (mut document, styles) = load_raw(path)?;
        let dpi = 96.0;
        let sec_count = document.sections.len();

        let mut reports = Vec::new();
        for sec_idx in 0..sec_count {
            let report = compare_section(&mut document, &styles, sec_idx, dpi);
            reports.push(report);
        }
        Some(reports)
    }

    // ─── 일치율 측정 테스트 ───

    #[test]
    fn test_lineseg_compare_basic() {
        let Some(reports) = run_comparison("samples/basic/BookReview.hwp") else {
            return;
        };
        let report_text = format_report(&reports);
        eprintln!("\n=== BookReview.hwp ===\n{}", report_text);

        let total_compared: usize = reports.iter().map(|r| r.compared_paragraphs).sum();
        if total_compared == 0 {
            eprintln!("비교 대상 문단이 0개 — 건너뜀");
        }
    }

    #[test]
    fn test_lineseg_compare_table_test() {
        let Some(reports) = run_comparison("samples/hwp_table_test.hwp") else {
            return;
        };
        let report_text = format_report(&reports);
        eprintln!("\n=== hwp_table_test.hwp ===\n{}", report_text);

        let total_compared: usize = reports.iter().map(|r| r.compared_paragraphs).sum();
        assert!(total_compared > 0, "비교 대상 문단이 0개");
    }

    #[test]
    fn test_lineseg_compare_hongbo() {
        let Some(reports) = run_comparison("samples/20250130-hongbo.hwp") else {
            return;
        };
        let report_text = format_report(&reports);
        eprintln!("\n=== 20250130-hongbo.hwp ===\n{}", report_text);

        let total_compared: usize = reports.iter().map(|r| r.compared_paragraphs).sum();
        assert!(total_compared > 0, "비교 대상 문단이 0개");
    }

    // ─── 문자별 폭 진단 (Task 400) ───

    /// 줄별 텍스트를 추출하고 rhwp 측정 폭 vs available_width를 비교
    #[test]
    fn test_lineseg_width_diagnosis_basic() {
        use crate::renderer::composer::find_active_char_shape;
        use crate::renderer::layout::{estimate_text_width, resolved_to_text_style};
        use crate::renderer::style_resolver::detect_lang_category;

        let Some((document, styles)) = load_raw("samples/lseg-01-basic.hwp") else {
            return;
        };
        let dpi = 96.0;
        let section = &document.sections[0];
        let page_def = &section.section_def.page_def;
        let column_def = find_column_def_for_paragraph(&section.paragraphs, 0);
        let layout = PageLayoutInfo::from_page_def(page_def, &column_def, dpi);
        let col_area = &layout.column_areas[0];

        for (pi, para) in section.paragraphs.iter().enumerate() {
            if para.line_segs.is_empty() || para.text.is_empty() {
                continue;
            }
            if para.line_segs[0].line_height == 0 {
                continue;
            }

            let para_style = styles.para_styles.get(para.para_shape_id as usize);
            let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
            let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
            let available_width = col_area.width - margin_left - margin_right;
            let available_hwp = crate::renderer::px_to_hwpunit(available_width, dpi);

            let text_chars: Vec<char> = para.text.chars().collect();

            eprintln!(
                "\n=== 문단 {} (줄 {}개, 가용폭 {:.1}px = {} HU) ===",
                pi,
                para.line_segs.len(),
                available_width,
                available_hwp
            );

            for (li, ls) in para.line_segs.iter().enumerate() {
                let utf16_start = ls.text_start as usize;
                let utf16_end = if li + 1 < para.line_segs.len() {
                    para.line_segs[li + 1].text_start as usize
                } else {
                    para.char_offsets
                        .last()
                        .map(|&o| o as usize + 1)
                        .unwrap_or(text_chars.len())
                };

                // UTF-16 offset → char index 변환
                let char_start = para
                    .char_offsets
                    .iter()
                    .position(|&o| o as usize >= utf16_start)
                    .unwrap_or(0);
                let char_end = para
                    .char_offsets
                    .iter()
                    .position(|&o| o as usize >= utf16_end)
                    .unwrap_or(text_chars.len());
                let line_text: String = text_chars[char_start..char_end.min(text_chars.len())]
                    .iter()
                    .collect();

                // TextStyle 생성
                let active_cs_id = find_active_char_shape(&para.char_shapes, char_start as u32);
                let first_ch = line_text.chars().next().unwrap_or('가');
                let lang = detect_lang_category(first_ch);
                let ts = resolved_to_text_style(&styles, active_cs_id as u32, lang);

                // 줄 전체 폭 측정
                let measured_width = estimate_text_width(&line_text, &ts);
                let measured_hwp = crate::renderer::px_to_hwpunit(measured_width, dpi);
                let orig_seg_width = ls.segment_width;

                // 문자별 개별 폭 합산
                let mut hangul_count = 0usize;
                let mut latin_count = 0usize;
                let mut space_count = 0usize;
                let mut punct_count = 0usize;
                let mut hangul_total = 0.0f64;
                let mut latin_total = 0.0f64;
                let mut space_total = 0.0f64;
                let mut punct_total = 0.0f64;

                for ch in line_text.chars() {
                    let cw = estimate_text_width(&ch.to_string(), &ts);
                    if ch >= '\u{AC00}' && ch <= '\u{D7AF}' {
                        hangul_count += 1;
                        hangul_total += cw;
                    } else if ch.is_ascii_alphabetic() {
                        latin_count += 1;
                        latin_total += cw;
                    } else if ch == ' ' {
                        space_count += 1;
                        space_total += cw;
                    } else {
                        punct_count += 1;
                        punct_total += cw;
                    }
                }

                eprintln!(
                    "  L{}: chars=[{}..{}) len={} | measured={:.1}px({}HU) seg_width={}HU delta={}",
                    li,
                    char_start,
                    char_end,
                    line_text.chars().count(),
                    measured_width,
                    measured_hwp,
                    orig_seg_width,
                    measured_hwp - orig_seg_width
                );
                eprintln!(
                    "    font={} size={:.3}px bold={} ratio={:.2} ls={:.2}",
                    ts.font_family, ts.font_size, ts.bold, ts.ratio, ts.letter_spacing
                );
                eprintln!(
                    "    한글:{}자({:.1}px) 영문:{}자({:.1}px) 공백:{}자({:.1}px) 기타:{}자({:.1}px)",
                    hangul_count, hangul_total, latin_count, latin_total,
                    space_count, space_total, punct_count, punct_total
                );

                // 첫 5글자의 개별 폭 출력
                let detail_chars: Vec<(char, f64)> = line_text
                    .chars()
                    .take(10)
                    .map(|ch| (ch, estimate_text_width(&ch.to_string(), &ts)))
                    .collect();
                let detail_str: Vec<String> = detail_chars
                    .iter()
                    .map(|(c, w)| format!("'{}'{:.2}", c, w))
                    .collect();
                eprintln!("    처음10자: {}", detail_str.join(" "));
            }
        }
    }

    /// 원본 vs reflow 줄바꿈 경계에서 마지막 글자의 누적 폭 비교
    /// 줄바꿈 차이가 발생하는 정확한 글자와 폭을 추적
    #[test]
    fn test_lineseg_boundary_char_analysis() {
        use crate::renderer::composer::find_active_char_shape;
        use crate::renderer::layout::{estimate_text_width, resolved_to_text_style};
        use crate::renderer::style_resolver::detect_lang_category;

        let paths = ["samples/lseg-03-spacing.hwp", "samples/lseg-01-basic.hwp"];
        for path in &paths {
            let Some((document, styles)) = load_raw(path) else {
                continue;
            };
            let dpi = 96.0;
            let section = &document.sections[0];
            let page_def = &section.section_def.page_def;
            let column_def = find_column_def_for_paragraph(&section.paragraphs, 0);
            let layout = PageLayoutInfo::from_page_def(page_def, &column_def, dpi);
            let col_area = &layout.column_areas[0];

            eprintln!("\n========== {} ==========", path);

            for (pi, para) in section.paragraphs.iter().enumerate() {
                if pi > 0 {
                    break;
                } // 첫 문단만
                if para.line_segs.is_empty() || para.text.is_empty() {
                    continue;
                }
                if para.line_segs[0].line_height == 0 {
                    continue;
                }

                let para_style = styles.para_styles.get(para.para_shape_id as usize);
                let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
                let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
                let available_width = col_area.width - margin_left - margin_right;
                let available_hwp = crate::renderer::px_to_hwpunit(available_width, dpi);

                let text_chars: Vec<char> = para.text.chars().collect();

                // reflow 결과
                let mut para_clone = para.clone();
                crate::renderer::composer::reflow_line_segs(
                    &mut para_clone,
                    available_width,
                    &styles,
                    dpi,
                );

                eprintln!(
                    "문단 {} (가용폭={}HU={:.1}px)",
                    pi, available_hwp, available_width
                );

                // 각 줄의 경계 글자 비교
                for li in 0..para.line_segs.len().max(para_clone.line_segs.len()) {
                    let orig_ts = para.line_segs.get(li).map(|l| l.text_start);
                    let refl_ts = para_clone.line_segs.get(li).map(|l| l.text_start);

                    // 원본 줄의 마지막 글자 위치에서의 누적 폭
                    if let (Some(ots), Some(next_ots)) =
                        (orig_ts, para.line_segs.get(li + 1).map(|l| l.text_start))
                    {
                        // 이 줄의 글자 범위 (char index)
                        let char_start = para
                            .char_offsets
                            .iter()
                            .position(|&o| o >= ots)
                            .unwrap_or(0);
                        let char_end = para
                            .char_offsets
                            .iter()
                            .position(|&o| o >= next_ots)
                            .unwrap_or(text_chars.len());

                        // 줄 텍스트의 각 글자 폭을 누적
                        let mut cum = 0.0f64;
                        let mut last_char = ' ';
                        for ci in char_start..char_end.min(text_chars.len()) {
                            let ch = text_chars[ci];
                            let utf16_pos = if ci < para.char_offsets.len() {
                                para.char_offsets[ci]
                            } else {
                                ci as u32
                            };
                            let style_id = find_active_char_shape(&para.char_shapes, utf16_pos);
                            let lang = detect_lang_category(ch);
                            let ts = resolved_to_text_style(&styles, style_id, lang);
                            let cw = estimate_text_width(&ch.to_string(), &ts);
                            cum += cw;
                            last_char = ch;
                        }
                        let cum_hwp = crate::renderer::px_to_hwpunit(cum, dpi);

                        // 다음 글자(줄 넘침 원인)
                        let next_char = text_chars.get(char_end).copied().unwrap_or('?');
                        let next_utf16 = if char_end < para.char_offsets.len() {
                            para.char_offsets[char_end]
                        } else {
                            0
                        };
                        let next_style_id = find_active_char_shape(&para.char_shapes, next_utf16);
                        let next_lang = detect_lang_category(next_char);
                        let next_ts = resolved_to_text_style(&styles, next_style_id, next_lang);
                        let next_cw = estimate_text_width(&next_char.to_string(), &next_ts);
                        let next_cum_hwp = crate::renderer::px_to_hwpunit(cum + next_cw, dpi);

                        let delta_ts = refl_ts.map(|r| r as i64 - next_ots as i64).unwrap_or(0);

                        eprintln!(
                            "  L{}: 원본ts={} reflow_ts={} | 줄폭={}HU(남은={}HU) | 끝='{}' 다음='{}'({:.1}px) → +다음={}HU {} | delta_next_ts={}",
                            li, ots, refl_ts.unwrap_or(0),
                            cum_hwp, available_hwp - cum_hwp,
                            last_char, next_char, next_cw, next_cum_hwp,
                            if next_cum_hwp > available_hwp { "초과" } else { "여유" },
                            delta_ts
                        );
                    }
                }
            }
        }
    }

    /// 원본 줄바꿈 위치에서의 텍스트 폭 vs available_width 비교
    /// 한컴이 줄을 나누는 정확한 지점에서 rhwp가 측정한 폭을 확인
    #[test]
    fn test_lineseg_linebreak_width_analysis() {
        use crate::renderer::composer::{find_active_char_shape, tokenize_paragraph, BreakToken};
        use crate::renderer::layout::{estimate_text_width, resolved_to_text_style};
        use crate::renderer::style_resolver::detect_lang_category;

        let Some((document, styles)) = load_raw("samples/lseg-01-basic.hwp") else {
            return;
        };
        let dpi = 96.0;
        let section = &document.sections[0];
        let page_def = &section.section_def.page_def;
        let column_def = find_column_def_for_paragraph(&section.paragraphs, 0);
        let layout = PageLayoutInfo::from_page_def(page_def, &column_def, dpi);
        let col_area = &layout.column_areas[0];

        for (pi, para) in section.paragraphs.iter().enumerate() {
            if para.line_segs.is_empty() || para.text.is_empty() {
                continue;
            }
            if para.line_segs[0].line_height == 0 {
                continue;
            }

            let para_style = styles.para_styles.get(para.para_shape_id as usize);
            let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
            let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
            let available_width = col_area.width - margin_left - margin_right;
            let indent = para_style.map(|s| s.indent).unwrap_or(0.0);

            let text_chars: Vec<char> = para.text.chars().collect();

            let english_break_unit = para_style.map(|s| s.english_break_unit).unwrap_or(0);
            let korean_break_unit = para_style.map(|s| s.korean_break_unit).unwrap_or(0);
            eprintln!("\n=== 문단 {} (줄 {}개, 가용폭={:.1}px, 들여쓰기={:.1}px, ko_break={}, en_break={}) ===",
                pi, para.line_segs.len(), available_width, indent, korean_break_unit, english_break_unit);

            // 원본 LINE_SEG의 줄바꿈 위치 기준으로 각 줄 텍스트 폭 측정
            for (li, ls) in para.line_segs.iter().enumerate() {
                let utf16_start = ls.text_start as usize;
                let utf16_end = if li + 1 < para.line_segs.len() {
                    para.line_segs[li + 1].text_start as usize
                } else {
                    // 마지막 줄: 문단 끝까지
                    para.char_offsets
                        .last()
                        .map(|&o| o as usize + 1)
                        .unwrap_or(text_chars.len())
                };

                let char_start = para
                    .char_offsets
                    .iter()
                    .position(|&o| o as usize >= utf16_start)
                    .unwrap_or(0);
                let char_end = para
                    .char_offsets
                    .iter()
                    .position(|&o| o as usize >= utf16_end)
                    .unwrap_or(text_chars.len());
                let line_text: String = text_chars[char_start..char_end.min(text_chars.len())]
                    .iter()
                    .collect();
                let is_last_line = li + 1 >= para.line_segs.len();

                // 줄 텍스트에서 trailing space 제거하여 측정 (줄 끝 공백은 줄바꿈 시 흡수됨)
                let trimmed = line_text.trim_end();
                let trailing_spaces = line_text.len() - trimmed.len();

                // 문자별 개별 폭 합산 (양자화 없이)
                let mut raw_total = 0.0f64;
                let mut char_count_hangul = 0usize;
                let mut char_count_space = 0usize;
                let mut char_count_other = 0usize;

                for (ci, ch) in trimmed.chars().enumerate() {
                    let pos = char_start + ci;
                    let utf16_pos = if pos < para.char_offsets.len() {
                        para.char_offsets[pos]
                    } else {
                        pos as u32
                    };
                    let style_id = find_active_char_shape(&para.char_shapes, utf16_pos);
                    let lang = detect_lang_category(ch);
                    let ts = resolved_to_text_style(&styles, style_id, lang);
                    let cw = estimate_text_width(&ch.to_string(), &ts);
                    raw_total += cw;

                    if ch >= '\u{AC00}' && ch <= '\u{D7AF}' {
                        char_count_hangul += 1;
                    } else if ch == ' ' {
                        char_count_space += 1;
                    } else {
                        char_count_other += 1;
                    }
                }

                let eff_width = if li == 0 {
                    (available_width - indent.max(0.0)).max(1.0)
                } else {
                    available_width
                };
                let margin = eff_width - raw_total;

                eprintln!(
                    "  L{}: chars=[{}..{}) len={} (trailing_sp={}) | sum={:.1}px eff_width={:.1}px margin={:.1}px {}",
                    li, char_start, char_end, trimmed.chars().count(), trailing_spaces,
                    raw_total, eff_width, margin,
                    if is_last_line { "(마지막줄)" } else if margin < 0.0 { "*** 초과! ***" } else { "" }
                );
                eprintln!(
                    "    한글:{} 공백:{} 기타:{}",
                    char_count_hangul, char_count_space, char_count_other
                );
            }

            // 토큰화 결과 분석 — fill_lines에 전달되는 토큰들의 폭 합산
            let english_break_unit = para_style.map(|s| s.english_break_unit).unwrap_or(0);
            let korean_break_unit = para_style.map(|s| s.korean_break_unit).unwrap_or(0);
            let tokens = tokenize_paragraph(
                &text_chars,
                &para.char_offsets,
                &para.char_shapes,
                &styles,
                english_break_unit,
                korean_break_unit,
            );
            let mut token_width_sum = 0.0f64;
            let mut token_count = 0usize;
            for tok in &tokens {
                match tok {
                    BreakToken::Text {
                        width,
                        start_idx,
                        end_idx,
                        ..
                    } => {
                        token_width_sum += width;
                        token_count += 1;
                        if token_count <= 10 {
                            let tok_text: String =
                                text_chars[*start_idx..*end_idx].iter().collect();
                            eprintln!(
                                "    T[{}]: Text({:.1}px) [{}..{}) \"{}\"",
                                token_count - 1,
                                width,
                                start_idx,
                                end_idx,
                                tok_text
                            );
                        }
                    }
                    BreakToken::Space { width, idx, .. } => {
                        token_width_sum += width;
                        token_count += 1;
                        if token_count <= 10 {
                            eprintln!(
                                "    T[{}]: Space({:.1}px) idx={}",
                                token_count - 1,
                                width,
                                idx
                            );
                        }
                    }
                    _ => {
                        token_count += 1;
                    }
                }
            }
            eprintln!(
                "  토큰 {}개, 폭 합계={:.1}px (available={:.1}px)",
                tokens.len(),
                token_width_sum,
                available_width
            );

            // rhwp reflow 결과와 비교
            let mut para_clone = para.clone();
            crate::renderer::composer::reflow_line_segs(
                &mut para_clone,
                available_width,
                &styles,
                dpi,
            );
            eprintln!("  --- reflow 결과: {}줄 ---", para_clone.line_segs.len());
            for (li, ls) in para_clone.line_segs.iter().enumerate() {
                let orig_ts = para.line_segs.get(li).map(|o| o.text_start);
                eprintln!(
                    "  R{}: text_start={} (원본={})",
                    li,
                    ls.text_start,
                    orig_ts.map(|t| t.to_string()).unwrap_or("없음".into())
                );
            }
        }
    }

    // ─── 통제된 샘플 (lseg-*) 개별 비교 ───

    #[test]
    fn test_lineseg_compare_lseg_samples() {
        let sample_files = [
            "samples/lseg-01-basic.hwp",
            "samples/lseg-02-mixed.hwp",
            "samples/lseg-03-spacing.hwp",
            "samples/lseg-04-indent.hwp",
            "samples/lseg-05-tab.hwp",
            "samples/lseg-06-multisize.hwp",
        ];

        for path in &sample_files {
            if let Some(reports) = run_comparison(path) {
                eprintln!("\n=== {} ===", path);
                for r in &reports {
                    eprintln!(
                        "  섹션{}: 문단 {}/{} | 줄수={:.0}% 줄바꿈={:.0}% 전체={:.0}%",
                        r.section_idx,
                        r.compared_paragraphs,
                        r.total_paragraphs,
                        r.line_count_match_rate(),
                        r.line_break_match_rate(),
                        r.all_match_rate(),
                    );
                    let avg = r.avg_field_deltas();
                    if avg.lines_compared > 0 {
                        eprintln!(
                            "    오차: ts={:.1} lh={:.1} bl={:.1} ls={:.1} sw={:.1}",
                            avg.text_start,
                            avg.line_height,
                            avg.baseline_distance,
                            avg.line_spacing,
                            avg.segment_width
                        );
                    }
                    // 불일치 문단 상세 출력
                    for pd in &r.paragraph_diffs {
                        if !pd.all_match() {
                            eprintln!(
                                "    pi={}: 줄 {}→{} {}",
                                pd.para_idx,
                                pd.original_line_count,
                                pd.reflow_line_count,
                                if pd.line_count_match {
                                    "필드차이"
                                } else {
                                    "줄수불일치"
                                }
                            );
                            for fd in &pd.field_diffs {
                                if !fd.all_match() {
                                    eprintln!(
                                        "      L{}: ts={} lh={} th={} bl={} ls={} sw={} vp={}",
                                        fd.line_idx,
                                        fd.text_start_delta,
                                        fd.line_height_delta,
                                        fd.text_height_delta,
                                        fd.baseline_distance_delta,
                                        fd.line_spacing_delta,
                                        fd.segment_width_delta,
                                        fd.vertical_pos_delta
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // ─── 한컴 저장본 vs rhwp reflow 비교 ───

    #[test]
    fn test_lineseg_compare_hancom_saved() {
        let mut pairs: Vec<(String, String)> = std::fs::read_dir("samples/")
            .unwrap()
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                if name.starts_with("re-") && name.ends_with("-hancom.hwp") {
                    let base = name.replace("-hancom.hwp", ".hwp");
                    Some((format!("samples/{}", name), format!("samples/{}", base)))
                } else {
                    None
                }
            })
            .collect();

        if pairs.is_empty() {
            eprintln!("한컴 저장본 없음 — 건너뜀");
            return;
        }

        let mut total_compared = 0usize;
        let mut total_line_match = 0usize;
        let mut total_break_match = 0usize;
        let mut total_all_match = 0usize;

        for (hancom_path, _rhwp_path) in &pairs {
            let Some(reports) = run_comparison(hancom_path) else {
                continue;
            };
            let base = std::path::Path::new(hancom_path)
                .file_stem()
                .unwrap()
                .to_string_lossy();

            for r in &reports {
                total_compared += r.compared_paragraphs;
                total_line_match += r.line_count_match_count;
                total_break_match += r.line_break_match_count;
                total_all_match += r.all_match_count;

                eprintln!(
                    "{}: 줄수={:.0}% 줄바꿈={:.0}% 전체={:.0}% ({}/{})",
                    base,
                    r.line_count_match_rate(),
                    r.line_break_match_rate(),
                    r.all_match_rate(),
                    r.compared_paragraphs - r.all_match_count,
                    r.compared_paragraphs,
                );

                // 불일치 상세 출력
                for pd in &r.paragraph_diffs {
                    if !pd.all_match() {
                        for fd in &pd.field_diffs {
                            if !fd.all_match() {
                                eprintln!(
                                    "  L{}: ts={} lh={} th={} bl={} ls={} sw={}",
                                    fd.line_idx,
                                    fd.text_start_delta,
                                    fd.line_height_delta,
                                    fd.text_height_delta,
                                    fd.baseline_distance_delta,
                                    fd.line_spacing_delta,
                                    fd.segment_width_delta
                                );
                            }
                        }
                    }
                }
            }
        }

        let rate = |n: usize, d: usize| {
            if d == 0 {
                0.0
            } else {
                n as f64 / d as f64 * 100.0
            }
        };
        eprintln!("\n=== 한컴 저장본 전체 요약 ===");
        eprintln!("비교: {}문단", total_compared);
        eprintln!(
            "줄 수 일치율: {:.1}%",
            rate(total_line_match, total_compared)
        );
        eprintln!(
            "줄바꿈 위치 일치율: {:.1}%",
            rate(total_break_match, total_compared)
        );
        eprintln!(
            "전체 필드 일치율: {:.1}%",
            rate(total_all_match, total_compared)
        );
    }

    /// 전체 samples/ 대상 일괄 비교 (nocapture로 실행하여 리포트 확인)
    #[test]
    fn test_lineseg_compare_all_samples() {
        let sample_files = [
            "samples/basic/BookReview.hwp",
            "samples/basic/KTX.hwp",
            "samples/hwp_table_test.hwp",
            "samples/table-001.hwp",
            "samples/20250130-hongbo.hwp",
            "samples/field-01.hwp",
            "samples/inner-table-01.hwp",
            "samples/eq-01.hwp",
        ];

        let mut all_reports = Vec::new();

        for path in &sample_files {
            if let Some(reports) = run_comparison(path) {
                eprintln!("\n--- {} ---", path);
                for r in &reports {
                    eprintln!(
                        "  섹션{}: 줄수={:.0}% 줄바꿈={:.0}% 전체={:.0}% ({}/{})",
                        r.section_idx,
                        r.line_count_match_rate(),
                        r.line_break_match_rate(),
                        r.all_match_rate(),
                        r.all_match_count,
                        r.compared_paragraphs,
                    );
                }
                all_reports.extend(reports);
            }
        }

        if !all_reports.is_empty() {
            eprintln!("\n{}", format_report(&all_reports));
        }
    }
}
