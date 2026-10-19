
fn identify_inline_controls(para: &Paragraph) -> Vec<InlineControl> {
    let mut result = Vec::new();

    for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
        let control_type = match ctrl {
            Control::Table(t) if t.common.treat_as_char => InlineControlType::Table,
            Control::Shape(shape) if shape.common().treat_as_char => InlineControlType::Shape,
            Control::Picture(pic) if pic.common.treat_as_char => InlineControlType::Shape,
            Control::Equation(eq) if eq.common.treat_as_char => InlineControlType::Shape,
            Control::SectionDef(_) | Control::ColumnDef(_) => InlineControlType::Other,
            _ => continue,
        };

        // 이 컨트롤이 어느 줄에 속하는지 결정
        // 컨트롤은 문단의 controls 배열에 순서대로 저장됨
        // 정확한 줄 위치는 텍스트 내 제어 문자 위치로 결정해야 하지만,
        // 현재는 첫 번째 줄에 배치 (향후 정확한 위치 계산 가능)
        let line_index = 0;

        result.push(InlineControl {
            line_index,
            control_index: ctrl_idx,
            control_type,
        });
    }

    result
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
fn tac_inline_object_starts_at(para: &Paragraph, pos: usize) -> bool {
    let chars: Vec<_> = para.text.chars().collect();
    para.controls
        .iter()
        .zip(para.control_text_positions())
        .any(|(ctrl, ctrl_pos)| {
            is_render_inline_control(ctrl)
                && pos <= ctrl_pos
                && ctrl_pos <= chars.len()
                && chars[pos..ctrl_pos]
                    .iter()
                    .all(|ch| ch.is_whitespace() && !matches!(ch, '\n' | '\r'))
        })
}

pub(crate) fn find_render_inline_control_positions(para: &Paragraph) -> Vec<usize> {
    if para.text.is_empty() && para.char_offsets.is_empty() {
        let mut inline_seen = 0usize;
        let mut positions = Vec::with_capacity(para.controls.len());
        for ctrl in &para.controls {
            positions.push(inline_seen);
            if is_render_inline_control(ctrl) {
                inline_seen += 1;
            }
        }
        return positions;
    }

    para.control_text_positions()
}

/// CharOverlap 컨트롤의 글자를 조합된 텍스트에 올바른 위치로 삽입한다.
///
/// char_offsets 갭 분석으로 각 CharOverlap의 원래 텍스트 위치를 복원하고,
/// 해당 위치의 composed line에서 기존 텍스트 런을 분할하여 CharOverlap 런을 삽입한다.
fn inject_char_overlap_text(composed: &mut ComposedParagraph, para: &Paragraph) {
    // CharOverlap 컨트롤과 인덱스 수집
    let char_overlap_indices: Vec<(usize, &crate::model::control::CharOverlap)> = para
        .controls
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            if let Control::CharOverlap(co) = c {
                Some((i, co))
            } else {
                None
            }
        })
        .collect();

    if char_overlap_indices.is_empty() {
        return;
    }

    // 모든 컨트롤의 텍스트 위치 결정
    let control_positions = para.control_text_positions();

    // CharOverlap별 (텍스트위치, 런) 수집
    let mut insertions: Vec<(usize, ComposedTextRun)> = Vec::new();
    for (ctrl_idx, co) in &char_overlap_indices {
        let text: String = co.chars.iter().collect();
        if text.is_empty() {
            continue;
        }
        let char_style_id = co
            .char_shape_ids
            .iter()
            .find(|&&id| id != 0xFFFFFFFF)
            .copied()
            .unwrap_or(0);
        let text_pos = control_positions.get(*ctrl_idx).copied().unwrap_or(0);
        insertions.push((
            text_pos,
            ComposedTextRun {
                text,
                char_style_id,
                lang_index: 0,
                neutral_space_origin: None,
                char_overlap: Some(CharOverlapInfo {
                    border_type: co.border_type,
                    inner_char_size: co.inner_char_size,
                }),
                footnote_marker: None,
                display_text: None,
                supplemental_metrics_blocked: false,
                inserted_control_text: true,
                space_metric: crate::renderer::composer::SpaceMetric::Stored,
            },
        ));
    }

    if insertions.is_empty() {
        return;
    }

    if composed.lines.is_empty() {
        // 빈 문단: line_segs에서 줄 정보를 가져와 새 줄 생성
        let (lh, bd, ls) = para
            .line_segs
            .first()
            .map(|s| (s.line_height, s.baseline_distance, s.line_spacing))
            .unwrap_or((400, 340, 0));
        composed.lines.push(ComposedLine {
            runs: insertions.into_iter().map(|(_, run)| run).collect(),
            line_height: lh,
            baseline_distance: bd,
            segment_width: 0,
            column_start: 0,
            line_spacing: ls,
            has_line_break: false,
            char_start: 0,
        });
        return;
    }

    // 역순으로 삽입하여 이전 인덱스가 무효화되지 않도록
    insertions.sort_by_key(|(pos, _)| std::cmp::Reverse(*pos));

    for (text_pos, overlap_run) in insertions {
        insert_control_text_run(composed, text_pos, overlap_run);
    }
}

/// 조합된 라인들에서 text_pos 위치에 CharOverlap 런을 삽입한다.
/// 기존 텍스트 런을 필요시 분할한다.
pub(super) fn insert_control_text_run(
    composed: &mut ComposedParagraph,
    text_pos: usize,
    overlap_run: ComposedTextRun,
) {
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

                if local_pos == run_offset {
                    // 런 앞에 삽입
                    line.runs.insert(run_idx, overlap_run);
                    return;
                } else if local_pos > run_offset && local_pos < run_offset + run_chars {
                    // 런 중간에 삽입: 런을 분할
                    let split_at = local_pos - run_offset;
                    let original_text: String = line.runs[run_idx].text.chars().collect();
                    let before: String = original_text.chars().take(split_at).collect();
                    let after: String = original_text.chars().skip(split_at).collect();

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
                    };

                    // overlap_run과 after_run을 삽입
                    line.runs.insert(run_idx + 1, after_run);
                    line.runs.insert(run_idx + 1, overlap_run);
                    return;
                }

                run_offset += run_chars;
            }

            // 라인 끝에 삽입
            line.runs.push(overlap_run);
            return;
        }

        char_offset += line_char_count;
    }

    // 어느 라인에도 해당하지 않으면 마지막 라인에 추가
    if let Some(last_line) = composed.lines.last_mut() {
        last_line.runs.push(overlap_run);
    }
}

/// ComposedLine의 폭을 언어 인식 측정으로 계산한다.
///
/// 각 run별로 해당 언어의 폰트/자간/장평을 적용하여 측정한다.
/// 진단 API에서 저장된 segment_width와 비교하는 데 사용한다.
pub fn estimate_composed_line_width(line: &ComposedLine, styles: &ResolvedStyleSet) -> f64 {
    line.runs
        .iter()
        .map(|run| {
            let ts = run.text_style(styles);
            estimate_text_width(effective_text_for_metrics(run), &ts)
        })
        .sum()
}
