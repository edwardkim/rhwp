
fn missing_lineseg_legacy_bullet_requires_regenerated_space_metric(
    para: &Paragraph,
    composed: &ComposedParagraph,
    styles: &ResolvedStyleSet,
) -> bool {
    let has_tight_leading_body_style = para.char_shapes.get(1).is_some_and(|cs_ref| {
        cs_ref.start_pos <= 3
            && styles
                .char_styles
                .get(cs_ref.char_shape_id as usize)
                .map(|cs| cs.letter_spacing <= -3.0)
                .unwrap_or(false)
    });

    para.line_segs.is_empty()
        && para.controls.is_empty()
        && para.text.starts_with('\u{F03C5}')
        && has_tight_leading_body_style
        && composed
            .lines
            .iter()
            .flat_map(|line| &line.runs)
            .any(|run| {
                let style = run.text_style(styles);
                hancom_regenerated_space_width(&style).is_some()
            })
}

/// 단일 ComposedLine 을 셀 가용 너비에 맞춰 다중 ComposedLine 으로 분할.
///
/// 분할 단위: 공백 단어 경계 우선, 단일 단어가 너비 초과 시 글자 단위 break.
/// 각 분할 줄의 메타데이터 (line_height/baseline/segment_width 등) 는 원본 보존.
fn split_composed_line_by_width(
    src: &ComposedLine,
    first_width_px: f64,
    cont_width_px: f64,
    styles: &ResolvedStyleSet,
    char_break: bool,
    space_condense: f64,
    regenerated_line_space_metric: bool,
    fit_tolerance_px: f64,
    regenerate_half_spaces: bool,
) -> Vec<ComposedLine> {
    let mut result: Vec<ComposedLine> = Vec::new();
    // [#2070] 내어쓰기(intent<0) 이중 폭: 첫 출력 줄은 first_width, 이후 연속
    // 줄은 cont_width 로 판정한다 (80168 조문 문단 첫줄 넓게/연속 좁게).
    let limit = |res: &Vec<ComposedLine>| -> f64 {
        if res.is_empty() {
            first_width_px
        } else {
            cont_width_px
        }
    };
    let mut current_runs: Vec<ComposedTextRun> = Vec::new();
    let mut current_width = 0.0;
    // [#2070] 한양신명조 사다리 v3/v4 확정 규칙: (a) 줄 채움 판정은 공백 폭을
    // 문단 condense% 만큼 압축해 계산(공백 압축), (b) 줄끝 초과 공백 1개는
    // 다음 줄로 넘기지 않고 현재 줄에 매달림(hang).
    let mut space_w = 0.0;
    let mut hung = false;
    let mut current_char_start = src.char_start;
    let mut chars_in_line = 0usize;
    let mut current_run_text = String::new();
    let mut current_run_template: Option<ComposedTextRun> = None;
    let text_width = |text: &str, style: &TextStyle| {
        let measured =
            estimate_regenerated_line_text_width(text, style, regenerated_line_space_metric);
        if !regenerate_half_spaces {
            return measured;
        }
        let spaces = text.chars().filter(|value| *value == ' ').count();
        if spaces == 0 {
            return measured;
        }
        let stored = estimate_regenerated_line_text_width(" ", style, false);
        measured + spaces as f64 * (regenerated_half_space_width(style) - stored)
    };

    let flush_run =
        |runs: &mut Vec<ComposedTextRun>, text: &mut String, template: &Option<ComposedTextRun>| {
            if !text.is_empty() {
                if let Some(t) = template {
                    runs.push(ComposedTextRun {
                        text: std::mem::take(text),
                        char_style_id: t.char_style_id,
                        lang_index: t.lang_index,
                        neutral_space_origin: None,
                        char_overlap: t.char_overlap.clone(),
                        footnote_marker: t.footnote_marker,
                        display_text: None,
                        supplemental_metrics_blocked: t.supplemental_metrics_blocked,
                        inserted_control_text: t.inserted_control_text,
                        space_metric: t.space_metric,
                        regenerated_font_metrics: t.regenerated_font_metrics,
                    });
                } else {
                    text.clear();
                }
            }
        };

    let push_line = |result: &mut Vec<ComposedLine>,
                     runs: &mut Vec<ComposedTextRun>,
                     current_char_start: &mut usize,
                     chars_in_line: &mut usize,
                     current_width: &mut f64| {
        if !runs.is_empty() {
            result.push(ComposedLine {
                runs: std::mem::take(runs),
                line_height: src.line_height,
                baseline_distance: src.baseline_distance,
                segment_width: src.segment_width,
                column_start: src.column_start,
                line_spacing: src.line_spacing,
                has_line_break: false,
                char_start: *current_char_start,
            });
            *current_char_start += *chars_in_line;
            *chars_in_line = 0;
            *current_width = 0.0;
        }
    };

    for run in &src.runs {
        let ts = run.text_style(styles);
        // 현재 run 의 template 변경 (char_style 다른 run 들 처리)
        if current_run_template
            .as_ref()
            .map(|t| {
                t.char_style_id != run.char_style_id
                    || t.lang_index != run.lang_index
                    || (styles.supplemental_metrics.is_some()
                        && (t.supplemental_metrics_blocked != run.supplemental_metrics_blocked
                            || t.inserted_control_text != run.inserted_control_text))
            })
            .unwrap_or(true)
        {
            flush_run(
                &mut current_runs,
                &mut current_run_text,
                &current_run_template,
            );
            current_run_template = Some(run.clone());
        }
        // [#2169] 줄나눔 기준 '글자'(korean_break_unit==0) — 글자 단위 채움.
        // 한글은 이 모드에서 어절 경계 무시하고 줄을 채운다 (80168 r10:
        // "또/는", "필요/한" 글자 분리, 한글 5줄 vs 어절 래핑 6줄).
        if char_break {
            for ch in run.text.chars() {
                let ch_str: String = std::iter::once(ch).collect();
                let ch_width = text_width(&ch_str, &ts);
                if std::env::var("RHWP_RAZOR").is_ok()
                    && src.runs.iter().any(|r| r.text.contains("도조례로 정하는"))
                {
                    eprintln!(
                        "RZ: ch={:?} w={:.2} cur={:.2} spw={:.2} cnd={:.2} limit={:.2} fam={:?}",
                        ch,
                        ch_width,
                        current_width,
                        space_w,
                        space_condense,
                        limit(&result),
                        ts.font_family.split(',').next().unwrap_or("")
                    );
                }
                let eff = current_width - space_w * space_condense;
                let over = eff + ch_width > limit(&result) + fit_tolerance_px && chars_in_line > 0;
                if over && ch == ' ' && !hung {
                    // 줄끝 초과 공백 1개 hang — 줄바꿈 없이 현재 줄에 계상.
                    hung = true;
                } else if over {
                    // [#2244] 행두 금칙: 새 줄이 금칙 문자(마침표 등)로 시작하지
                    // 않도록 직전 글자를 함께 다음 줄로 이월한다 — 한컴 2024 저장
                    // 오라클 정합 ("적용한 | 다.111…", LINE_SEG [...,128]).
                    // 직전 글자가 같은 run 안에 있고(스타일 경계 아님) 공백이
                    // 아니며 줄에 2자 이상 남을 때만 1자 retraction.
                    let carried: Option<(char, f64)> = if is_line_start_forbidden(ch)
                        && chars_in_line > 1
                        && current_run_text
                            .chars()
                            .last()
                            .is_some_and(|p| p != ' ' && !is_line_start_forbidden(p))
                    {
                        current_run_text.pop().map(|prev| {
                            let prev_str: String = std::iter::once(prev).collect();
                            let prev_w = text_width(&prev_str, &ts);
                            current_width -= prev_w;
                            chars_in_line -= 1;
                            (prev, prev_w)
                        })
                    } else {
                        None
                    };
                    flush_run(
                        &mut current_runs,
                        &mut current_run_text,
                        &current_run_template,
                    );
                    push_line(
                        &mut result,
                        &mut current_runs,
                        &mut current_char_start,
                        &mut chars_in_line,
                        &mut current_width,
                    );
                    space_w = 0.0;
                    hung = false;
                    if let Some((pch, pw)) = carried {
                        current_run_text.push(pch);
                        current_width += pw;
                        chars_in_line += 1;
                    }
                }
                current_run_text.push(ch);
                current_width += ch_width;
                if ch == ' ' {
                    space_w += ch_width;
                }
                chars_in_line += 1;
            }
            continue;
        }
        // run 텍스트를 단어 단위로 분할 (공백 포함)
        let mut word = String::new();
        for ch in run.text.chars() {
            word.push(ch);
            // 공백 또는 마지막 글자 직전이 단어 경계
            if ch == ' ' || ch == '\t' {
                let word_width = text_width(&word, &ts);
                // 현재 단어가 추가되면 max_width 초과하는지 검사
                if current_width - space_w * space_condense + word_width
                    > limit(&result) + fit_tolerance_px
                    && (chars_in_line > 0 || !current_run_text.is_empty())
                {
                    // 현재 줄을 flush 후 새 줄 시작
                    flush_run(
                        &mut current_runs,
                        &mut current_run_text,
                        &current_run_template,
                    );
                    push_line(
                        &mut result,
                        &mut current_runs,
                        &mut current_char_start,
                        &mut chars_in_line,
                        &mut current_width,
                    );
                    space_w = 0.0;
                    hung = false;
                }
                // 단어 자체가 max_width 초과 시 글자 단위 break
                if word_width > limit(&result) && current_width == 0.0 {
                    for wch in word.chars() {
                        let wch_str: String = std::iter::once(wch).collect();
                        let wch_width = text_width(&wch_str, &ts);
                        if current_width - space_w * space_condense + wch_width > limit(&result)
                            && chars_in_line > 0
                        {
                            flush_run(
                                &mut current_runs,
                                &mut current_run_text,
                                &current_run_template,
                            );
                            push_line(
                                &mut result,
                                &mut current_runs,
                                &mut current_char_start,
                                &mut chars_in_line,
                                &mut current_width,
                            );
                            space_w = 0.0;
                            hung = false;
                        }
                        current_run_text.push(wch);
                        current_width += wch_width;
                        chars_in_line += 1;
                    }
                } else {
                    current_run_text.push_str(&word);
                    current_width += word_width;
                    space_w += text_width(" ", &ts) * word.matches(' ').count() as f64;
                    chars_in_line += word.chars().count();
                }
                word.clear();
            }
        }
        // run 끝에 남은 단어 처리
        if !word.is_empty() {
            let word_width = text_width(&word, &ts);
            if current_width - space_w * space_condense + word_width > limit(&result)
                && (chars_in_line > 0 || !current_run_text.is_empty())
            {
                flush_run(
                    &mut current_runs,
                    &mut current_run_text,
                    &current_run_template,
                );
                push_line(
                    &mut result,
                    &mut current_runs,
                    &mut current_char_start,
                    &mut chars_in_line,
                    &mut current_width,
                );
                space_w = 0.0;
                hung = false;
            }
            // 단어 자체가 max_width 초과 시 글자 단위 break
            if word_width > limit(&result) && current_width == 0.0 {
                for wch in word.chars() {
                    let wch_str: String = std::iter::once(wch).collect();
                    let wch_width = text_width(&wch_str, &ts);
                    if current_width - space_w * space_condense + wch_width > limit(&result)
                        && chars_in_line > 0
                    {
                        flush_run(
                            &mut current_runs,
                            &mut current_run_text,
                            &current_run_template,
                        );
                        push_line(
                            &mut result,
                            &mut current_runs,
                            &mut current_char_start,
                            &mut chars_in_line,
                            &mut current_width,
                        );
                        space_w = 0.0;
                        hung = false;
                    }
                    current_run_text.push(wch);
                    current_width += wch_width;
                    chars_in_line += 1;
                }
            } else {
                current_run_text.push_str(&word);
                current_width += word_width;
                chars_in_line += word.chars().count();
            }
        }
    }
    // 마지막 줄 flush
    flush_run(
        &mut current_runs,
        &mut current_run_text,
        &current_run_template,
    );
    push_line(
        &mut result,
        &mut current_runs,
        &mut current_char_start,
        &mut chars_in_line,
        &mut current_width,
    );
    space_w = 0.0;
    hung = false;

    if result.is_empty() {
        // 안전장치: 절대 빈 결과 반환하지 않음
        result.push(src.clone());
    }
    result
}
