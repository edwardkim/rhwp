
/// 문단의 텍스트를 줄별로 분할하고, 각 줄 내에서 CharShapeRef 경계에 따라 분할한다.
fn compose_lines(para: &Paragraph) -> Vec<ComposedLine> {
    if para.line_segs.is_empty() {
        // LineSeg가 없으면 텍스트를 ComposedLine 으로 분할
        if para.text.is_empty() {
            return Vec::new();
        }
        let default_style_id = para
            .char_shapes
            .first()
            .map(|cs| cs.char_shape_id)
            .unwrap_or(0);
        // [Task #994] HWP5 변환본의 일부 paragraph (sample16 의 󰏅 PUA bullet 들)
        // 는 PARA_LINE_SEG 누락 → 기존 fallback 이 단일 ComposedLine 생성 →
        // layout 이 wrap 없이 한 y 좌표에 모든 텍스트 그림 → 시각 겹침.
        // 임시 휴리스틱: 공백 기준 word wrap, ~45 chars/line (Korean 13pt 표준) 한도.
        // 정확한 line_height 는 corrected_line_height 가 layout 에서 보정 (max_fs * 1.6).
        // 향후 reflow_line_segs 정식 호출 시 본 휴리스틱 대체.
        // [Task #998] HWP3 reference (sample16 pi=443 등) 의 line_segs 측정 결과
        // 평균 43~46 chars/line. 기존 35 는 conservative — 매 paragraph +1 line
        // 발생 → 페이지 수 inflate (sample16-hwp5.hwp: 64 reference 대비 +3).
        // 45 로 조정하여 HWP3 정합 개선 (+1 까지 축소, 잔존 ParaShape 데이터 차이).
        let chars: Vec<char> = para.text.chars().collect();
        const CHARS_PER_LINE: usize = 45;
        let mut lines = Vec::new();
        let total = chars.len();
        let mut offset = 0;
        while offset < total {
            let max_end = (offset + CHARS_PER_LINE).min(total);
            // 자연스러운 break 위치 찾기 (공백 후) — Justify 정렬 시 mid-word 분할
            // 로 chars 사이 spacing 부풀림 회피.
            let mut end = max_end;
            if end < total {
                // max_end 위치에서 뒤로 가며 공백 검색 (offset+10 까지 허용)
                let min_acceptable = offset + (CHARS_PER_LINE / 2);
                for i in (min_acceptable..max_end).rev() {
                    if chars[i] == ' ' || chars[i] == '\t' {
                        end = i + 1; // 공백 포함하여 line 끝
                        break;
                    }
                }
            }
            // 강제 줄바꿈(0x0A)은 문자 수와 무관하게 그 자리에서 줄을 끝낸다 —
            // 한글은 이 문자에서 반드시 개행하므로, 무시하면 두 줄 분량이 한
            // 줄로 이어져 열 폭을 넘는다.
            if let Some(nl) = chars[offset..max_end].iter().position(|&c| c == '\n') {
                end = offset + nl + 1;
            }
            // [#7500] 남은 글자가 공백뿐이면 이 줄이 흡수한다 — 한/글은 말미 공백에 줄상자를
            // 주지 않는다(#7160 `absorb_whitespace_only_rows` 와 같은 규칙). 끊어 두면
            // 공백만 든 줄이 생겨 뒤 내용을 그 줄 수만큼 밀어낸다.
            if end < total
                && chars[end..]
                    .iter()
                    .all(|&c| matches!(c, ' ' | '\t' | '\u{3000}'))
            {
                end = total;
            }
            let line_text: String = chars[offset..end].iter().collect();
            let is_last_line = end >= total;
            // 이 폴백(PARA_LINE_SEG 누락 문단)도 CharShapeRef 경계를 존중한다 —
            // 종전에는 문단 전체를 `char_shapes[0]` 단일 run 으로 만들어, 첫
            // run 이 컨트롤 문자 구간(스트림 위주 앞부분)의 모양일 때 가시
            // 텍스트 전체가 그 모양으로 렌더됐다(제목이 15pt 흰색 bold 저장인데
            // 10pt 검정으로 — run 경계 (0,cs_a)(24,cs_b)(26,cs_c) 에서 cs_a 적용).
            // char_offsets 가 있으면 본문 경로와 같은 splitter 로 토큰화하고,
            // 없으면 종전 단일 default_style run 을 유지한다.
            let mut fallback_runs = if para.char_offsets.len() == total && !para.char_shapes.is_empty()
            {
                split_by_char_shapes(
                    &line_text,
                    offset,
                    end,
                    &para.char_offsets,
                    &para.char_shapes,
                )
            } else {
                split_runs_by_lang(vec![ComposedTextRun {
                    text: line_text,
                    char_style_id: default_style_id,
                    lang_index: 0,
                    neutral_space_origin: None,
                    char_overlap: None,
                    footnote_marker: None,
                    display_text: None,
                    supplemental_metrics_blocked: false,
                    inserted_control_text: false,
                    space_metric: crate::renderer::composer::SpaceMetric::Stored,
                    regenerated_font_metrics: false,
                    literal_figure_space: false,
                }])
            };
            for run in &mut fallback_runs {
                run.regenerated_font_metrics = true;
            }
            lines.push(ComposedLine {
                runs: fallback_runs,
                line_height: 400,
                baseline_distance: 320,
                segment_width: 0,
                column_start: 0,
                line_spacing: 0,
                // [Task #994] non-last synth wrap line 은 has_line_break=true 로 marking —
                // Justify 정렬 비활성화 (line 의 chars 가 column width 만큼 spread 되지 않음).
                // 마지막 line 은 false (기존 paragraph 동작 유지).
                has_line_break: !is_last_line,
                char_start: offset,
            });
            offset = end;
        }
        return lines;
    }

    let mut lines: Vec<ComposedLine> = Vec::new();
    let line_seg_count = para.line_segs.len();

    for line_idx in 0..line_seg_count {
        let line_seg = &para.line_segs[line_idx];
        let space_metric = para.line_space_metric(line_idx);
        let regenerated_font_metrics = text_metric_origin::regenerated_row(para, line_idx);
        let split_row_runs = |text: &str, start, end, offsets: &[u32], shapes: &[CharShapeRef]| {
            let mut runs = split_by_char_shapes(text, start, end, offsets, shapes);
            for run in &mut runs {
                run.space_metric = space_metric;
                run.regenerated_font_metrics = regenerated_font_metrics;
            }
            runs
        };

        // UTF-16 위치 기반으로 이 줄의 텍스트 범위 계산.
        // [#5961] 아래에서 `char_offsets`·`char_count` 로 투영하므로 HWP5 축으로 올려
        // 받는다. HWPX 출처 구역 첫 문단은 저장 `textpos` 가 그만큼 짧아서, 날값을 쓰면
        // 줄이 보정폭만큼 일찍 끊긴다(한글 대조 실측: 글자 54 를 46 으로 읽는다).
        let utf16_start = para.line_seg_text_start(line_idx);
        let utf16_end = if line_idx + 1 < line_seg_count {
            para.line_seg_text_start(line_idx + 1)
        } else {
            // 마지막 줄: char_count 또는 텍스트 끝까지
            if para.char_count > 0 {
                para.char_count
            } else {
                // char_count 미설정 시 텍스트 길이 기반 추정
                para.text.chars().count() as u32 + 1
            }
        };

        // UTF-16 위치 → 텍스트 문자 인덱스로 변환
        let (text_start, mut text_end) = utf16_range_to_text_range(
            &para.char_offsets,
            utf16_start,
            utf16_end,
            para.text.chars().count(),
        );
        if text_end < text_start {
            text_end = text_start;
        }
        // 이 줄의 텍스트 추출
        let line_text: String = para
            .text
            .chars()
            .skip(text_start)
            .take(text_end - text_start)
            .collect();

        // TAC 표 문단 감지
        let has_tac = para.controls.iter().any(
            |c| matches!(c, crate::model::control::Control::Table(t) if t.common.treat_as_char),
        );

        // 강제 줄넘김(\n) + TAC 표 문단 처리 (Task #19/Task #20)
        // 아래 범위와 take/skip은 문자 인덱스다. UTF-8 바이트 위치를 쓰면
        // 한글 제목 뒤 개행이 줄 끝을 넘어 표 줄까지 제목에 합쳐진다.
        let newline_pos = line_text.chars().position(|ch| ch == '\n');
        if let (true, Some(nl_pos)) = (has_tac, newline_pos) {
            let pre_text: String = line_text.chars().take(nl_pos).collect();
            let pre_end = text_start + nl_pos;
            let post_start = text_start + nl_pos + 1;
            let post_text: String = line_text.chars().skip(nl_pos + 1).collect();
            let post_text_clean = post_text.trim_end_matches('\n').to_string();
            // [#6300] 강제 줄나눔이 이 저장 줄의 끝이고, 다음 LINE_SEG 가 인라인
            // 개체일 때만 경계를 유지한다. 같은 저장 줄 안의 `\n`+표(Task #20)는
            // 기존처럼 `\n` 앞 텍스트를 이전 줄에 합친다. 이 가드 밖의 `\n` 은
            // off-canvas·overflow-cell 래칫을 키우지 않는다.
            let keep_stored_boundary = post_text_clean.is_empty()
                && line_idx + 1 < line_seg_count
                && tac_inline_object_starts_at(para, text_end)
                // The first stored row also owns its terminating break.
                // Requiring a previous row creates an extra empty row before
                // the already stored next table row (#7165).
                && lines.last().is_none_or(|prev| prev.char_start != text_start);

            if !pre_text.is_empty() && !lines.is_empty() && !keep_stored_boundary {
                // \n 앞 텍스트를 이전 ComposedLine에 합침 (한컴 방식: \n 전 전체가 한 줄)
                let prev: &mut ComposedLine = lines.last_mut().unwrap();
                let mut extra_runs = split_row_runs(
                    &pre_text,
                    text_start,
                    pre_end,
                    &para.char_offsets,
                    &para.char_shapes,
                );
                prev.runs.append(&mut extra_runs);
                prev.has_line_break = true;
            } else if !pre_text.is_empty() || keep_stored_boundary {
                // A terminating break owns its stored line box even with no
                // visible glyphs. Measurement and placement consume this same
                // row before the following inline object.
                let pre_runs = split_row_runs(
                    &pre_text,
                    text_start,
                    pre_end,
                    &para.char_offsets,
                    &para.char_shapes,
                );
                let pre_lh = if line_seg.text_height > 0
                    && line_seg.text_height < line_seg.line_height / 3
                {
                    line_seg.text_height
                } else {
                    line_seg.line_height
                };
                lines.push(ComposedLine {
                    runs: pre_runs,
                    line_height: pre_lh,
                    baseline_distance: line_seg.baseline_distance,
                    segment_width: line_seg.segment_width,
                    column_start: line_seg.column_start,
                    line_spacing: line_seg.line_spacing,
                    has_line_break: true,
                    char_start: text_start,
                });
            }

            // \n 이후: 표 줄 (빈 runs, 표는 layout에서 별도 처리)
            // [#6300] 다음 저장 줄이 인라인 개체면 빈 후속 줄을 여기서 만들지 않는다.
            if !(keep_stored_boundary && post_text_clean.is_empty()) {
                let post_runs = split_row_runs(
                    &post_text_clean,
                    post_start,
                    text_end,
                    &para.char_offsets,
                    &para.char_shapes,
                );
                lines.push(ComposedLine {
                    runs: post_runs,
                    line_height: line_seg.line_height,
                    baseline_distance: line_seg.baseline_distance,
                    segment_width: line_seg.segment_width,
                    column_start: line_seg.column_start,
                    line_spacing: line_seg.line_spacing,
                    has_line_break: post_text.ends_with('\n'),
                    char_start: post_start,
                });
            }
        } else {
            // 일반 처리: LINE_SEG 범위 안에 강제 줄바꿈(\n)이 있으면 실제 줄로 분할한다.
            // Shift+Enter는 문단을 새로 만들지 않지만 렌더러/커서/들여쓰기 계산에서는
            // 다음 visual line 이 별도 ComposedLine 이어야 한다.
            let line_chars: Vec<char> = line_text.chars().collect();
            let mut segment_start = 0usize;

            let corrected_lh = if has_tac
                && line_seg.text_height > 0
                && line_seg.text_height < line_seg.line_height / 3
            {
                line_seg.text_height
            } else {
                // 글자 테두리 등으로 텍스트 점유 높이가 명목 줄 높이를 넘을 수
                // 있다. 한컴 저장 줄(1000/1056/632)은 1688HU씩 전진한다.
                // 공통 구성 결과에 점유 높이를 싣고 측정과 paint가 함께 소비한다.
                stored_line_box_height(line_seg)
            };

            let mut push_segment = |segment_start: usize, segment_end: usize, has_break: bool| {
                let segment_text: String = line_chars[segment_start..segment_end].iter().collect();
                let segment_abs_start = text_start + segment_start;
                let segment_abs_end = text_start + segment_end;
                let runs = split_row_runs(
                    &segment_text,
                    segment_abs_start,
                    segment_abs_end,
                    &para.char_offsets,
                    &para.char_shapes,
                );
                lines.push(ComposedLine {
                    runs,
                    line_height: corrected_lh,
                    baseline_distance: line_seg.baseline_distance,
                    segment_width: line_seg.segment_width,
                    column_start: line_seg.column_start,
                    line_spacing: line_seg.line_spacing,
                    has_line_break: has_break,
                    char_start: segment_abs_start,
                });
            };

            for (rel_idx, ch) in line_chars.iter().enumerate() {
                if *ch == '\n' {
                    push_segment(segment_start, rel_idx, true);
                    segment_start = rel_idx + 1;
                }
            }

            if segment_start < line_chars.len() || !line_text.ends_with('\n') {
                push_segment(segment_start, line_chars.len(), false);
            }
        }
    }

    lines
}

/// UTF-16 위치 범위를 텍스트 문자 인덱스 범위로 변환한다.
pub(crate) fn utf16_range_to_text_range(
    char_offsets: &[u32],
    utf16_start: u32,
    utf16_end: u32,
    text_len: usize,
) -> (usize, usize) {
    if char_offsets.is_empty() {
        // 오프셋 정보가 없으면 1:1 매핑 가정
        let start = (utf16_start as usize).min(text_len);
        let end = (utf16_end as usize).min(text_len);
        return (start, end);
    }

    // char_offsets[i] >= utf16_start인 첫 번째 i가 text_start
    let text_start = char_offsets
        .iter()
        .position(|&off| off >= utf16_start)
        .unwrap_or(text_len);

    // char_offsets[i] >= utf16_end인 첫 번째 i가 text_end
    let text_end = char_offsets
        .iter()
        .position(|&off| off >= utf16_end)
        .unwrap_or(text_len);

    (text_start, text_end)
}

/// 줄 내 텍스트를 CharShapeRef 경계에 따라 다중 TextRun으로 분할한다.
fn split_by_char_shapes(
    line_text: &str,
    text_start: usize,
    text_end: usize,
    char_offsets: &[u32],
    char_shapes: &[CharShapeRef],
) -> Vec<ComposedTextRun> {
    if line_text.is_empty() {
        return Vec::new();
    }

    if char_shapes.is_empty() {
        return split_runs_by_lang(vec![ComposedTextRun {
            text: line_text.to_string(),
            char_style_id: 0,
            lang_index: 0,
            neutral_space_origin: None,
            char_overlap: None,
            footnote_marker: None,
            display_text: None,
            supplemental_metrics_blocked: false,
            inserted_control_text: false,
            space_metric: crate::renderer::composer::SpaceMetric::Stored,
            regenerated_font_metrics: false,
            literal_figure_space: false,
        }]);
    }

    // 이 줄 범위에 영향을 미치는 CharShapeRef 찾기
    //
    // [#915] CharShapeRef.start_pos 는 paragraph 텍스트의 UTF-16 stream offset
    // 이다 (해석 A). char_offsets[i] 가 가시문자 i 의 stream offset 이므로,
    // start_pos 이상인 첫 char_offsets 항목이 char_shape 적용 시작 가시문자다.
    //
    // 해석 이력: #884 가 start_pos 를 visible char index 로 해석(해석 B)하도록
    // 바꿨으나, 그 근거였던 table-in-tbox.hwp footer "충남중부권지사장" 의
    // "26pt" 판정이 오진(실제 HY수평선B 16pt — 한컴 폰트 패널 확인)이었다.
    // 해석 B 는 인라인 제어자가 문단 중간에 있는 경우(char_offsets gap) start_pos
    // 가 범위 밖으로 부풀려져 char_shape 가 통째 누락된다 (#915 — table-in-tbox
    // p2 "충남중부권지사" 가 1pt 로 렌더). 또한 paragraph_layout.rs /
    // line_breaking.rs 는 줄곧 해석 A 를 써 와서 #884 이후 composer 와 불일치
    // 상태였다 — 본 수정으로 전 경로가 해석 A 로 일관된다.
    let total_chars = char_offsets.len();
    // [#915] 줄 시작 가시문자의 stream offset — fallback active-shape 조회용.
    let line_stream_start = char_offsets
        .get(text_start)
        .copied()
        .unwrap_or(text_start as u32);
    let mut segments: Vec<(usize, u32)> = Vec::new();

    for cs in char_shapes {
        // start_pos(stream offset) 이상인 첫 가시문자가 char_shape 적용 시작점.
        let cs_visible_idx = char_offsets
            .iter()
            .position(|&off| off >= cs.start_pos)
            .unwrap_or(total_chars);
        // cs 가 이 줄 범위 밖이면 skip
        if cs_visible_idx >= text_end {
            continue;
        }
        let text_idx = cs_visible_idx.saturating_sub(text_start);
        segments.push((text_idx, cs.char_shape_id));
    }

    // 시작 인덱스로 정렬 (동일 인덱스 내에서는 원래 순서 유지)
    segments.sort_by_key(|&(idx, _)| idx);

    // 중복 시작 위치 제거: 동일 위치의 마지막 것(가장 최근 CharShapeRef)만 유지
    // 뒤에서부터 dedup하면 마지막 것이 유지됨
    segments.reverse();
    segments.dedup_by_key(|s| s.0);
    segments.reverse();

    // segments가 비어있으면 첫 번째 CharShapeRef 사용
    if segments.is_empty() {
        // 줄 시작 위치 이전의 마지막 CharShapeRef 찾기
        let style_id = find_active_char_shape(char_shapes, line_stream_start);
        return split_runs_by_lang(vec![ComposedTextRun {
            text: line_text.to_string(),
            char_style_id: style_id,
            lang_index: 0,
            neutral_space_origin: None,
            char_overlap: None,
            footnote_marker: None,
            display_text: None,
            supplemental_metrics_blocked: false,
            inserted_control_text: false,
            space_metric: crate::renderer::composer::SpaceMetric::Stored,
            regenerated_font_metrics: false,
            literal_figure_space: false,
        }]);
    }

    // TextRun 생성
    let chars: Vec<char> = line_text.chars().collect();
    let mut runs = Vec::new();

    for i in 0..segments.len() {
        let (start_idx, style_id) = segments[i];
        let end_idx = if i + 1 < segments.len() {
            segments[i + 1].0
        } else {
            chars.len()
        };

        if start_idx < end_idx && start_idx < chars.len() {
            let actual_end = end_idx.min(chars.len());
            let run_text: String = chars[start_idx..actual_end].iter().collect();
            if !run_text.is_empty() {
                runs.push(ComposedTextRun {
                    text: run_text,
                    char_style_id: style_id,
                    lang_index: 0,
                    neutral_space_origin: None,
                    char_overlap: None,
                    footnote_marker: None,
                    display_text: None,
                    supplemental_metrics_blocked: false,
                    inserted_control_text: false,
                    space_metric: crate::renderer::composer::SpaceMetric::Stored,
                    regenerated_font_metrics: false,
                    literal_figure_space: false,
                });
            }
        }
    }

    // 첫 번째 segment가 0이 아닌 경우, 앞 부분 처리
    if !segments.is_empty() && segments[0].0 > 0 {
        let style_id = find_active_char_shape(char_shapes, line_stream_start);
        let end_idx = segments[0].0.min(chars.len());
        let prefix_text: String = chars[..end_idx].iter().collect();
        if !prefix_text.is_empty() {
            runs.insert(
                0,
                ComposedTextRun {
                    text: prefix_text,
                    char_style_id: style_id,
                    lang_index: 0,
                    neutral_space_origin: None,
                    char_overlap: None,
                    footnote_marker: None,
                    display_text: None,
                    supplemental_metrics_blocked: false,
                    inserted_control_text: false,
                    space_metric: crate::renderer::composer::SpaceMetric::Stored,
                    regenerated_font_metrics: false,
                    literal_figure_space: false,
                },
            );
        }
    }

    if runs.is_empty() {
        let style_id = find_active_char_shape(char_shapes, line_stream_start);
        runs.push(ComposedTextRun {
            text: line_text.to_string(),
            char_style_id: style_id,
            lang_index: 0,
            neutral_space_origin: None,
            char_overlap: None,
            footnote_marker: None,
            display_text: None,
            supplemental_metrics_blocked: false,
            inserted_control_text: false,
            space_metric: crate::renderer::composer::SpaceMetric::Stored,
            regenerated_font_metrics: false,
            literal_figure_space: false,
        });
    }

    // 언어 카테고리별로 Run을 세분화
    split_runs_by_lang(runs)
}

/// 주어진 UTF-16 위치에서 활성화된 CharShapeRef의 char_shape_id를 찾는다.
///
/// [Task #884] 해석 B 적용으로 start_pos 는 visible char index 이므로 이 함수의
/// utf16_pos 인자는 의미가 모호해진다. 호출자가 char_offsets 통해 utf16 → visible
/// idx 변환 후 [`find_active_char_shape_visible`] 사용 권장. 본 함수는 호환성을
/// 위해 유지하나 향후 deprecate 예정.
pub(crate) fn find_active_char_shape(char_shapes: &[CharShapeRef], utf16_pos: u32) -> u32 {
    // utf16_pos 를 visible idx 로 직접 비교 (해석 B)
    find_active_char_shape_visible(char_shapes, utf16_pos as usize)
}

/// [Task #884] visible char index 로 활성 char_shape 찾기
pub(crate) fn find_active_char_shape_visible(
    char_shapes: &[CharShapeRef],
    visible_idx: usize,
) -> u32 {
    let mut active_id = char_shapes.first().map(|cs| cs.char_shape_id).unwrap_or(0);
    for cs in char_shapes {
        if (cs.start_pos as usize) <= visible_idx {
            active_id = cs.char_shape_id;
        } else {
            break;
        }
    }
    active_id
}

/// TextRun 목록을 언어 카테고리 경계에 따라 세분화한다.
///
/// 동일 CharShape 내에서도 한글→영문 전환 시 별도 Run으로 분리하여
/// 각 언어에 맞는 폰트를 적용할 수 있도록 한다.
///
/// 공백/구두점은 이전 문자의 언어를 따른다 (불필요한 Run 분할 방지).
pub(crate) fn split_runs_by_lang(runs: Vec<ComposedTextRun>) -> Vec<ComposedTextRun> {
    let mut result = Vec::new();

    for run in runs {
        let chars: Vec<char> = run.text.chars().collect();
        if chars.is_empty() {
            result.push(run);
            continue;
        }

        // 첫 번째 비중립 문자의 언어를 찾아 초기 언어로 설정
        let initial_lang = chars
            .iter()
            .map(|&c| detect_lang_category(c))
            .find(|&lang| lang != 0 || chars.iter().all(|&c| detect_lang_category(c) == 0))
            .unwrap_or(0);

        let mut current_lang = initial_lang;
        let mut current_start = 0;

        for (i, &ch) in chars.iter().enumerate() {
            let char_lang = detect_lang_category(ch);

            // 언어 중립 문자(공백/구두점 등 = 기본값 0)는 이전 언어를 따름
            // 단, detect_lang_category가 0을 반환하는 것은 한국어 또는 중립 두 가지 경우:
            //   - 한글 음절/자모: 명시적으로 0번 매치
            //   - 공백/구두점: _ => 0 폴백
            // 한글 음절은 확실한 한국어이므로 구분해야 함
            let is_neutral = is_lang_neutral(ch);

            if is_neutral {
                // 중립 문자: 현재 언어 유지
                continue;
            }

            if char_lang != current_lang {
                // 언어 전환: 이전 구간 확정
                if i > current_start {
                    let text: String = chars[current_start..i].iter().collect();
                    result.push(ComposedTextRun {
                        text,
                        char_style_id: run.char_style_id,
                        lang_index: current_lang,
                        neutral_space_origin: None,
                        char_overlap: run.char_overlap.clone(),
                        footnote_marker: None,
                        display_text: None,
                        supplemental_metrics_blocked: run.supplemental_metrics_blocked,
                        inserted_control_text: run.inserted_control_text,
                        space_metric: run.space_metric,
                        regenerated_font_metrics: run.regenerated_font_metrics,
                        literal_figure_space: run.literal_figure_space,
                    });
                }
                current_lang = char_lang;
                current_start = i;
            }
        }

        // 마지막 구간
        let text: String = chars[current_start..].iter().collect();
        if !text.is_empty() {
            result.push(ComposedTextRun {
                text,
                char_style_id: run.char_style_id,
                lang_index: current_lang,
                neutral_space_origin: None,
                char_overlap: run.char_overlap.clone(),
                footnote_marker: None,
                display_text: None,
                supplemental_metrics_blocked: run.supplemental_metrics_blocked,
                inserted_control_text: run.inserted_control_text,
                space_metric: run.space_metric,
                regenerated_font_metrics: run.regenerated_font_metrics,
                literal_figure_space: run.literal_figure_space,
            });
        }
    }

    result
}

/// Legacy Latin 구두점인가 — [`crate::renderer::TextStyle::ascii_punct_latin`] 가
/// 있는 run 에서 한/글은 이 글자를 영문 슬롯 글꼴로 잰다.
/// `76076` 한컴 2024 PDF의 가운뎃점·작은따옴표도 Palatino Linotype 슬롯이다.
/// 한글 face 자체의 가운뎃점 폭과 HFT 한점 리더는 이 슬롯 조건 밖이다.
pub(crate) fn is_latin_slot_punct(ch: char) -> bool {
    ch.is_ascii_punctuation()
}

/// Newly composed neutral punctuation shares one explicit slot between width and paint.
/// Stored runs retain their own inherited face; ASCII keeps its existing metric rule.
pub(crate) fn reflow_punctuation_slot(ch: char, inherited: usize, punct_latin: bool) -> usize {
    if crate::renderer::style_resolver::is_latin_slot_symbol(ch)
        || (punct_latin && !ch.is_ascii() && is_latin_slot_punct(ch))
    {
        1
    } else {
        inherited
    }
}

/// 글자의 언어 슬롯. 중립 문자는 앞 글자 언어(`carry`)를 따른다. `punct_latin` 인 글자
/// 모양의 ASCII 구두점은 영문(1)으로 재되 `carry` 는 바꾸지 않는다 — 뒤 공백이 앞 언어를 잇도록.
pub(crate) fn char_lang_slot(ch: char, carry: &mut usize, punct_latin: bool) -> usize {
    if punct_latin && is_latin_slot_punct(ch) {
        1
    } else if is_lang_neutral(ch) {
        *carry
    } else {
        let lang = detect_lang_category(ch);
        *carry = lang;
        lang
    }
}

/// 언어 중립 문자인지 판별한다 (공백, ASCII 구두점, 일반 기호 등).
/// 이 문자들은 Run 분할을 유발하지 않고 이전 문자의 언어를 따른다.
pub(crate) fn is_lang_neutral(ch: char) -> bool {
    let cp = ch as u32;
    matches!(cp,
        // 공백/제어문자
        0x0000..=0x0020 |
        // ASCII 구두점/기호 (영문자/숫자 제외)
        0x0021..=0x002F | 0x003A..=0x0040 | 0x005B..=0x0060 | 0x007B..=0x007F |
        // Latin-1 Supplement 구두점 (문자 제외)
        0x00A0..=0x00AF | 0x00B1..=0x00B6 | 0x00B8..=0x00BF
    )
}

/// 문단 내 인라인 컨트롤(표/도형)의 위치를 식별한다.
/// [#6706] 줄 끝 개체와 다음 줄 첫 글자는 같은 가시 위치로 투영될 수 있다.
/// 조합 결과가 현재 문단의 줄 경계·메트릭을 그대로 소비하고 충돌이 존재할 때
/// UTF-16 제어 슬롯으로 개체 소유 줄을 복원한다. 저장 줄과 편집 재조판 줄은
/// 각각의 출처 안에서만 적용하며 둘을 섞은 사다리는 수용하지 않는다.
/// 이 조회는 저장 캐시의 수용 조건을 변경하거나 옛 줄을 재조판에 적용하지 않는다.
pub(crate) fn composed_tac_line_assignment(
    para: &Paragraph,
    comp: &ComposedParagraph,
) -> Option<Vec<(usize, usize)>> {
    let computed_rows = para.line_segs.first()?.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0;
    if super::equation_tac_flow::uses_equation_only_flow(para, comp)
        // 글자 없는 문단도 증가하는 원시 제어 슬롯 경계가 각 줄을 증명한다.
        // 같은 시작을 반복한 안내 줄에는 이 배정을 적용하지 않는다.
        || (para.char_offsets.is_empty()
            && (!para.text.is_empty()
                || para
                    .line_segs
                    .windows(2)
                    .any(|rows| rows[0].text_start >= rows[1].text_start)))
        || comp.lines.len() != para.line_segs.len()
        || comp.lines.len() < 2
        || comp
            .lines
            .iter()
            .zip(&para.line_segs)
            .enumerate()
            .any(|(i, (line, seg))| {
                (seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0) != computed_rows
                    || line.line_height != stored_line_box_height(seg)
                    || line.segment_width != seg.segment_width
                    || line.char_start
                        != para
                            .char_offsets
                            .partition_point(|&offset| offset < para.line_seg_text_start(i))
            })
    {
        return None;
    }
    let assignments: Vec<(usize, usize)> = comp
        .tac_controls
        .iter()
        .map(|(_, _, ci)| {
            let owner = control_slot_line_index(para, *ci)?;
            Some((*ci, owner))
        })
        .collect::<Option<_>>()?;
    let collapsed_boundary =
        comp.tac_controls
            .iter()
            .zip(&assignments)
            .any(|((pos, _, _), (_, owner))| {
                comp.lines
                    .get(owner + 1)
                    .is_some_and(|next| *pos >= next.char_start)
            });
    collapsed_boundary.then_some(assignments)
}

/// 보이는 다음 글자와 확장 제어 슬롯이 같은 논리 위치여도 원시 줄 소속은 다를 수 있다.
/// 저장/현재 재조판 사다리의 UTF-16 경계로 조회하며 높이의 우연한 일치는 사용하지 않는다.
pub(crate) fn control_slot_line_index(para: &Paragraph, control_index: usize) -> Option<usize> {
    control_slot_line_index_with_starts(
        para,
        control_index,
        (0..para.line_segs.len()).map(|line| para.line_seg_text_start(line)),
    )
}

/// 저장 줄과 새로 발행할 줄은 같은 원시 제어 슬롯 경계로 소유를 판정한다.
pub(super) fn control_slot_line_index_with_starts(
    para: &Paragraph,
    control_index: usize,
    starts: impl Iterator<Item = u32>,
) -> Option<usize> {
    let start = *para.control_utf16_positions().get(control_index)?;
    starts
        .enumerate()
        .filter_map(|(line, pos)| (pos <= start).then_some(line))
        .last()
}
