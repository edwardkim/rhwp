fn try_identify_inline_controls<S: crate::storage::Storage>(
    para: &Paragraph,
    storage: &S,
) -> Result<Vec<InlineControl>, S::Error> {
    let mut result = storage.vec(0)?;

    for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
        let control_type = match ctrl {
            Control::Table(t) if t.common.treat_as_char => InlineControlType::Table,
            Control::Shape(shape) if shape.common().treat_as_char => InlineControlType::Shape,
            Control::Picture(pic) if pic.common.treat_as_char => InlineControlType::Shape,
            Control::Equation(eq) if eq.common.treat_as_char => InlineControlType::Shape,
            Control::Form(form) if form.common.treat_as_char => InlineControlType::Shape,
            Control::SectionDef(_) | Control::ColumnDef(_) => InlineControlType::Other,
            _ => continue,
        };

        // 이 컨트롤이 어느 줄에 속하는지 결정
        // 컨트롤은 문단의 controls 배열에 순서대로 저장됨
        // 정확한 줄 위치는 텍스트 내 제어 문자 위치로 결정해야 하지만,
        // 현재는 첫 번째 줄에 배치 (향후 정확한 위치 계산 가능)
        let line_index = 0;

        storage.push(
            &mut result,
            InlineControl {
                line_index,
                control_index: ctrl_idx,
                control_type,
            },
        )?;
    }

    Ok(result)
}

fn is_render_inline_control(ctrl: &Control) -> bool {
    match ctrl {
        Control::Picture(pic) => pic.common.treat_as_char,
        Control::Shape(shape) => shape.common().treat_as_char,
        Control::Table(table) => table.common.treat_as_char,
        Control::Equation(eq) => eq.common.treat_as_char,
        // [#6266] 양식 개체도 자기 배치를 갖는다 — 비-TAC 은 인라인이 아니다.
        Control::Form(form) => form.common.treat_as_char,
        _ => false,
    }
}

/// [#6300] 다음 줄의 머리 공백 뒤에 treat_as_char 인라인 개체가 있는지.
fn try_tac_inline_object_starts_at<S: crate::storage::Storage>(
    para: &Paragraph,
    pos: usize,
    storage: &S,
) -> Result<bool, S::Error> {
    let chars: Vec<_> = storage.collect(para.text.chars())?;
    Ok(para
        .controls
        .iter()
        .zip(para.try_control_text_positions(storage)?)
        .any(|(ctrl, ctrl_pos)| {
            is_render_inline_control(ctrl)
                && pos <= ctrl_pos
                && ctrl_pos <= chars.len()
                && chars[pos..ctrl_pos]
                    .iter()
                    .all(|ch| ch.is_whitespace() && !matches!(ch, '\n' | '\r'))
        }))
}

pub(crate) fn find_render_inline_control_positions(para: &Paragraph) -> Vec<usize> {
    match try_find_render_inline_control_positions(para, &crate::storage::Unbounded) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}

fn try_find_render_inline_control_positions<S: crate::storage::Storage>(
    para: &Paragraph,
    storage: &S,
) -> Result<Vec<usize>, S::Error> {
    if para.text.is_empty() && para.char_offsets.is_empty() {
        let mut inline_seen = 0usize;
        let mut positions = storage.vec(para.controls.len())?;
        for ctrl in &para.controls {
            positions.push(inline_seen);
            if is_render_inline_control(ctrl) {
                inline_seen += 1;
            }
        }
        return Ok(positions);
    }

    para.try_control_text_positions(storage)
}

/// CharOverlap 컨트롤의 글자를 조합된 텍스트에 올바른 위치로 삽입한다.
///
/// char_offsets 갭 분석으로 각 CharOverlap의 원래 텍스트 위치를 복원하고,
/// 해당 위치의 composed line에서 기존 텍스트 런을 분할하여 CharOverlap 런을 삽입한다.
fn try_inject_char_overlap_text<S: crate::storage::Storage>(
    composed: &mut ComposedParagraph,
    para: &Paragraph,
    styles: Option<&ResolvedStyleSet>,
    storage: &S,
) -> Result<(), S::Error> {
    // CharOverlap 컨트롤과 인덱스 수집
    let char_overlap_indices: Vec<(usize, &crate::model::control::CharOverlap)> =
        storage.collect(para.controls.iter().enumerate().filter_map(|(i, c)| {
            if let Control::CharOverlap(co) = c {
                Some((i, co))
            } else {
                None
            }
        }))?;

    if char_overlap_indices.is_empty() {
        return Ok(());
    }

    // 모든 컨트롤의 텍스트 위치 결정
    let control_positions = para.try_control_text_positions(storage)?;
    let stream_positions = para.control_utf16_positions_with_storage(storage)?;

    // CharOverlap별 (텍스트위치, 런) 수집
    let mut insertions: Vec<(usize, ComposedTextRun)> = storage.vec(0)?;
    for (ctrl_idx, co) in &char_overlap_indices {
        let text = char_overlap_flow_text(&co.chars, storage)?;
        let stored_char_style_id = co
            .char_shape_ids
            .iter()
            .find(|&&id| id != 0xFFFFFFFF)
            .copied()
            .unwrap_or(0);
        let text_pos = control_positions.get(*ctrl_idx).copied().unwrap_or(0);
        // 제어 슬롯 뒤 글자의 스타일이 아니라, 원문 슬롯 자체를 소유한 run을 찾는다.
        let stream_pos = stream_positions.get(*ctrl_idx).copied().unwrap_or(0);
        let host_style_id = para
            .char_shapes
            .iter()
            .filter(|r| r.start_pos <= stream_pos)
            .max_by_key(|r| r.start_pos)
            .or_else(|| para.char_shapes.first())
            .map_or(0, |r| r.char_shape_id);
        let composition = styles.and_then(|styles| {
            super::char_overlap::Composition::from_model(co, styles, host_style_id)
        });
        let char_style_id = if composition.is_some() {
            host_style_id
        } else {
            stored_char_style_id
        };
        storage.push(&mut insertions, (
            text_pos,
            ComposedTextRun {
                text,
                char_style_id,
                lang_index: 0,
                neutral_space_origin: None,
                char_overlap: Some(CharOverlapInfo {
                    border_type: co.border_type,
                    inner_char_size: styles
                        .map_or(co.inner_char_size, |s| s.char_overlap_inner_size(co.inner_char_size)),
                    composition,
                    // HWP의 프레임 문자와 HWPX의 명시적 전각 공백은 다르다.
                    empty_ink: co.border_type == 0
                        && (co.chars.is_empty()
                            || (co.hwp_wire_chars && co.chars.as_slice() == ['\u{3000}'])),
                }),
                footnote_marker: None,
                display_text: None,
                supplemental_metrics_blocked: false,
                inserted_control_text: true,
                space_metric: crate::renderer::composer::SpaceMetric::Stored,
                regenerated_font_metrics: text_metric_origin::regenerated_row(para, 0),
                literal_figure_space: false,
            },
        ))?;
    }

    if insertions.is_empty() {
        return Ok(());
    }

    if composed.lines.is_empty() {
        // 빈 문단: line_segs에서 줄 정보를 가져와 새 줄 생성
        let (lh, bd, ls) = para
            .line_segs
            .first()
            .map(|s| (s.line_height, s.baseline_distance, s.line_spacing))
            .unwrap_or((400, 340, 0));
        storage.push(
            &mut composed.lines,
            ComposedLine {
                runs: storage.collect(insertions.into_iter().map(|(_, run)| run))?,
                line_height: lh,
                baseline_distance: bd,
                segment_width: 0,
                column_start: 0,
                line_spacing: ls,
                has_line_break: false,
                char_start: 0,
            },
        )?;
        return Ok(());
    }

    // 역순으로 삽입하여 이전 인덱스가 무효화되지 않도록
    storage.bytes(
        insertions
            .len()
            .saturating_mul(std::mem::size_of::<(usize, ComposedTextRun)>()),
    )?;
    insertions.sort_by_key(|(pos, _)| std::cmp::Reverse(*pos));

    for (text_pos, overlap_run) in insertions {
        try_insert_control_text_run(composed, text_pos, overlap_run, storage)?;
    }
    Ok(())
}

/// 조합된 라인들에서 text_pos 위치에 CharOverlap 런을 삽입한다.
/// 기존 텍스트 런을 필요시 분할한다.
pub(super) fn insert_control_text_run(
    composed: &mut ComposedParagraph,
    text_pos: usize,
    overlap_run: ComposedTextRun,
) {
    match try_insert_control_text_run(composed, text_pos, overlap_run, &crate::storage::Unbounded) {
        Ok(()) => (),
        Err(never) => match never {},
    }
}

fn try_insert_control_text_run<S: crate::storage::Storage>(
    composed: &mut ComposedParagraph,
    text_pos: usize,
    mut overlap_run: ComposedTextRun,
    storage: &S,
) -> Result<(), S::Error> {
    let mut char_offset = 0usize;

    for line in composed.lines.iter_mut() {
        let line_char_count: usize = line
            .runs
            .iter()
            .filter(|r| r.char_overlap.is_none() && !r.inserted_control_text)
            .map(|r| r.text.chars().count())
            .sum();

        if text_pos < char_offset + line_char_count || text_pos == char_offset {
            // 이 라인에 삽입
            let local_pos = text_pos - char_offset;
            let mut run_offset = 0usize;

            for run_idx in 0..line.runs.len() {
                // CharOverlap 런은 건너뜀 (이미 삽입된 것)
                if line.runs[run_idx].inserted_control_text {
                    continue;
                }

                let run_chars = line.runs[run_idx].text.chars().count();
                overlap_run.regenerated_font_metrics = line.runs[run_idx].regenerated_font_metrics;

                if local_pos == run_offset {
                    // 런 앞에 삽입
                    storage.reserve(&mut line.runs, 1)?;
                    line.runs.insert(run_idx, overlap_run);
                    return Ok(());
                } else if local_pos > run_offset && local_pos < run_offset + run_chars {
                    // 런 중간에 삽입: 런을 분할
                    let split_at = local_pos - run_offset;
                    let original_text = &line.runs[run_idx].text;
                    let before = storage.chars(original_text.chars().take(split_at))?;
                    let after = storage.chars(original_text.chars().skip(split_at))?;
                    storage.reserve(&mut line.runs, 2)?;

                    let style_id = line.runs[run_idx].char_style_id;
                    let lang_idx = line.runs[run_idx].lang_index;

                    // 기존 런을 before로 교체
                    line.runs[run_idx].text = before;

                    // after 런 생성
                    let after_run = ComposedTextRun {
                        text: after,
                        char_style_id: style_id,
                        lang_index: lang_idx,
                        neutral_space_origin: None,
                        char_overlap: None,
                        footnote_marker: None,
                        display_text: None,
                        supplemental_metrics_blocked: line.runs[run_idx]
                            .supplemental_metrics_blocked,
                        inserted_control_text: line.runs[run_idx].inserted_control_text,
                        space_metric: line.runs[run_idx].space_metric,
                        regenerated_font_metrics: line.runs[run_idx].regenerated_font_metrics,
                        literal_figure_space: line.runs[run_idx].literal_figure_space,
                    };

                    // overlap_run과 after_run을 삽입
                    line.runs.insert(run_idx + 1, after_run);
                    line.runs.insert(run_idx + 1, overlap_run);
                    return Ok(());
                }

                run_offset += run_chars;
            }

            // 삽입한 표시도 이 물리 줄이 선택한 측정 출처를 따른다.
            if let Some(last) = line.runs.last() {
                overlap_run.regenerated_font_metrics = last.regenerated_font_metrics;
            }
            storage.push(&mut line.runs, overlap_run)?;
            return Ok(());
        }

        char_offset += line_char_count;
    }

    // 어느 라인에도 해당하지 않으면 마지막 라인에 추가
    if let Some(last_line) = composed.lines.last_mut() {
        storage.push(&mut last_line.runs, overlap_run)?;
    }
    Ok(())
}

/// ComposedLine의 폭을 언어 인식 측정으로 계산한다.
///
/// 각 run별로 해당 언어의 폰트/자간/장평을 적용하여 측정한다.
/// 진단 API에서 저장된 segment_width와 비교하는 데 사용한다.
pub fn estimate_composed_line_width(line: &ComposedLine, styles: &ResolvedStyleSet) -> f64 {
    match estimate_composed_line_width_with_storage(line, styles, &crate::storage::Unbounded) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}

pub(crate) fn estimate_composed_line_width_with_storage<S: crate::storage::Storage>(
    line: &ComposedLine,
    styles: &ResolvedStyleSet,
    storage: &S,
) -> Result<f64, S::Error> {
    // Retain the original sum, stopping immediately at the first storage error.
    let mut error = None;
    let width = line
        .runs
        .iter()
        .scan(&mut error, |error, run| {
            match run.try_text_style(styles, storage) {
                Ok(ts) => Some(estimate_text_width(effective_text_for_metrics(run), &ts)),
                Err(e) => {
                    **error = Some(e);
                    None
                }
            }
        })
        .sum();
    match error {
        Some(error) => Err(error),
        None => Ok(width),
    }
}
