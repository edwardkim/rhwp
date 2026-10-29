/// 구역의 문단 목록을 구성한다.
pub fn compose_section(section: &Section) -> Vec<ComposedParagraph> {
    section
        .paragraphs
        .iter()
        .map(|para| {
            let mut composed = compose_paragraph(para);
            auto_number_projection::project(&mut composed, para);
            composed
        })
        .collect()
}

/// Exact-font contexts가 준비된 DocumentCore 파생-state 경계만 사용하는 opt-in
/// composition path. 기존 공개 composition 경로의 결과는 그대로 유지한다.
pub(crate) fn compose_section_with_horizontal_shaping(
    section: &Section,
    styles: &ResolvedStyleSet,
) -> Vec<ComposedParagraph> {
    match try_compose_section_with_horizontal_shaping(section, styles, &crate::storage::Unbounded) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}

pub(crate) fn try_compose_section_with_horizontal_shaping<S: crate::storage::Storage>(
    section: &Section,
    styles: &ResolvedStyleSet,
    storage: &S,
) -> Result<Vec<ComposedParagraph>, S::Error> {
    let mut paragraphs = storage.vec(section.paragraphs.len())?;
    for paragraph in &section.paragraphs {
        paragraphs.push(try_compose_paragraph_with_horizontal_shaping(
            paragraph, styles, storage,
        )?);
    }
    Ok(paragraphs)
}

/// [Task #991] HWP5 parser 가 extended ctrl (1-3, 11-12, 14-18, 21-23) 의
/// inline visible marker (\u{FFFC}) 를 text 에 push 하지 않아서 발생하는 layout
/// 어긋남 보정. HWP3 parser 는 마커를 push 하므로 HWP3/HWPX 동일 IR 보장.
///
/// 검사: para.text 의 \u{FFFC} count < extended inline-visible ctrl count.
/// 부족하면 char_offsets gap (8 wchar 단위) 에 마커 삽입한 synth paragraph 반환.
///
/// 영향 범위: composer 내부만 (rendering pipeline). para 원본 (editor) 영향 없음.
fn try_synthesize_marker_paragraph<S: crate::storage::Storage>(
    para: &Paragraph,
    storage: &S,
) -> Result<Option<Paragraph>, S::Error> {
    fn needs_synthesized_inline_marker(ctrl: &Control) -> bool {
        is_render_inline_control(ctrl)
    }

    // 렌더에 실제 자리를 차지하는 TAC/개체 컨트롤 수 계산.
    // Field/ColumnDef/SectionDef 같은 비가시 컨트롤은 char_offsets gap에 있어도
    // 본문 텍스트 char_start를 밀면 안 된다.
    let inline_ctrl_count = para
        .controls
        .iter()
        .filter(|ctrl| needs_synthesized_inline_marker(ctrl))
        .count();

    if inline_ctrl_count == 0 {
        return Ok(None);
    }

    // HWP3 정답지의 미주 수식 문단은 본문 텍스트 없이 줄바꿈/탭과
    // TAC 수식만으로 LINE_SEG를 구성한다. 이 경우 char_offsets와 line_seg
    // text_start가 이미 컨트롤 줄 위치를 표현하므로 HWP5 누락 마커 보정을 적용하면
    // 수식이 뒤 줄로 밀린다.
    let text_has_only_layout_space = para
        .text
        .chars()
        .all(|ch| matches!(ch, '\n' | '\r' | '\t' | ' ' | '\u{2007}'));
    let controls_are_tac_objects = para.controls.iter().all(|ctrl| {
        matches!(
            ctrl,
            Control::Equation(eq) if eq.common.treat_as_char
        ) || matches!(
            ctrl,
            Control::Picture(_) | Control::Shape(_) | Control::Table(_) | Control::Form(_)
        )
    });
    if text_has_only_layout_space && controls_are_tac_objects {
        return Ok(None);
    }

    let existing_markers = para.text.chars().filter(|c| *c == '\u{FFFC}').count();
    if existing_markers >= inline_ctrl_count {
        // HWP3 path — 이미 마커 충분
        return Ok(None);
    }

    // [Task #991 좁힘] 단일 control 또는 단일 leading ctrl 경우는 fix 미적용.
    // 본 fix 의 root cause 는 "여러 TAC controls 가 한 paragraph 의 char_offsets
    // gap 으로 인해 모두 position 0 으로 분석되는" 특정 case (sample16 pi=394).
    // 일반 case (1-2 TAC + 텍스트) 는 기존 control_text_positions 의 marker 보조
    // 분기 또는 inline rendering 로 처리 — F2 적용 시 \u{FFFC} 가 text run 에
    // 추가되어 다른 sample (exam_eng p8 puko box 등) 위치 shift 발생.
    //
    // 좁힘 조건:
    //   - inline_ctrl_count >= 3 (pi=394 = 3 TAC controls 기준)
    //   - n_leading >= 2 (leading gap 에 2+ ctrl)
    let offsets = &para.char_offsets;
    let first_off = offsets.first().copied().unwrap_or(0) as usize;
    let n_leading = first_off / 8;
    if n_leading < 2 || inline_ctrl_count < 3 {
        return Ok(None);
    }

    // 원본 char_offsets 갭 분석이 이미 컨트롤을 텍스트 중간/뒤 위치로
    // 분산해 주는 문단은 합성 마커를 만들지 않는다. 예: 수식 TAC 여러 개와
    // 쉼표/고정탭/일반 글자가 한 줄에 섞인 문단은 [0,0,2,2,4] 같은 raw
    // position 자체가 편집자가 입력한 순서다. 여기에 \u{FFFC}를 재합성하면
    // TAC가 쉼표/탭 뒤로 밀려 순서가 깨진다.
    let raw_positions = para.try_control_text_positions(storage)?;
    let raw_inline_positions: Vec<usize> = storage.collect(
        para.controls
            .iter()
            .enumerate()
            .filter(|(_, ctrl)| needs_synthesized_inline_marker(ctrl))
            .filter_map(|(i, _)| raw_positions.get(i).copied()),
    )?;
    if raw_inline_positions.iter().any(|pos| *pos > 0) {
        return Ok(None);
    }
    // 좁힘 조건 (n_leading >= 2) 통과 ⇒ offsets 비어있지 않음.
    // 따라서 빈 paragraph (offsets/chars empty) 경로는 본 좁힘 하에 도달 불가 —
    // 별도 분기 두지 않음 (검토 PR #995 §3.3 b).

    // HWP5 path — char_offsets gap 분석으로 누락된 마커 위치 합성
    let chars: Vec<char> = storage.collect(para.text.chars())?;
    let mut new_text = storage.string("")?;
    let mut new_offsets: Vec<u32> = storage.vec(
        para.char_offsets
            .len()
            .saturating_add(inline_ctrl_count - existing_markers),
    )?;

    let mut literal_figure_spaces = Vec::new();

    // 첫 visible char 전 leading gap (좁힘 가드에서 계산한 n_leading 재사용)
    for i in 0..n_leading {
        storage.push(&mut new_offsets, (i * 8) as u32)?;
        storage.push_char(&mut new_text, '\u{FFFC}')?;
    }

    // visible chars 사이 / 후행
    for (i, &off) in offsets.iter().enumerate() {
        let ch = chars.get(i).copied().unwrap_or(' ');
        if para.literal_figure_spaces.binary_search(&i).is_ok() {
            storage.push(&mut literal_figure_spaces, new_offsets.len())?;
        }
        storage.push(&mut new_offsets, off)?;
        storage.push_char(&mut new_text, ch)?;

        // 다음 char 까지의 gap 분석
        let char_width: u32 = if (ch as u32) > 0xFFFF { 2 } else { 1 };
        let next_off = if i + 1 < offsets.len() {
            offsets[i + 1] as usize
        } else {
            // 마지막 char 후행 controls — trailing gap 추정 불가능하면 종료
            // (line_segs 분석 등 더 정교한 방법 가능하지만 본 fix 의 좁은 범위 유지)
            continue;
        };
        let gap = next_off
            .saturating_sub(off as usize)
            .saturating_sub(char_width as usize);
        let n_ctrls_between = gap / 8;
        for k in 0..n_ctrls_between {
            storage.push(
                &mut new_offsets,
                (off as usize + char_width as usize + k * 8) as u32,
            )?;
            storage.push_char(&mut new_text, '\u{FFFC}')?;
        }
    }

    // 모든 후행 controls 처리 — line_segs.ts 마지막 + 8 단위로 추정
    let added_so_far = new_text.chars().filter(|c| *c == '\u{FFFC}').count();
    let still_needed = inline_ctrl_count.saturating_sub(added_so_far);
    if still_needed > 0 {
        // 마지막 char_offsets 의 stream pos + char_width 부터 8 단위씩
        let last_off = offsets.last().copied().unwrap_or(0) as usize;
        let last_ch = chars.last().copied().unwrap_or(' ');
        let last_w: usize = if (last_ch as u32) > 0xFFFF { 2 } else { 1 };
        let mut next_pos = last_off + last_w;
        for _ in 0..still_needed {
            storage.push(&mut new_offsets, next_pos as u32)?;
            storage.push_char(&mut new_text, '\u{FFFC}')?;
            next_pos += 8;
        }
    }

    let mut synth = storage.clone_paragraph(para)?;
    synth.text = new_text;
    synth.char_offsets = new_offsets;
    synth.literal_figure_spaces = literal_figure_spaces;
    Ok(Some(synth))
}

/// 문단을 줄별 텍스트 런으로 분할한다.
pub fn compose_paragraph(para: &Paragraph) -> ComposedParagraph {
    compose_paragraph_scoped(para, false, None)
}

/// Supplemental width contexts may subdivide a nominal run; portable composition must not.
pub(crate) fn compose_paragraph_in_context(
    para: &Paragraph,
    styles: &ResolvedStyleSet,
) -> ComposedParagraph {
    compose_paragraph_scoped(para, styles.supplemental_metrics.is_some(), Some(styles))
}

pub(crate) fn compose_paragraph_in_context_with_storage<S: crate::storage::Storage>(
    para: &Paragraph,
    styles: &ResolvedStyleSet,
    storage: &S,
) -> Result<ComposedParagraph, S::Error> {
    try_compose_paragraph_scoped(
        para,
        styles.supplemental_metrics.is_some(),
        Some(styles),
        storage,
    )
}
pub(crate) fn compose_paragraph_for_metric_requests(para: &Paragraph) -> ComposedParagraph {
    compose_paragraph_scoped(para, true, None)
}

fn compose_paragraph_scoped(
    para: &Paragraph,
    protect_metrics: bool,
    metric_styles: Option<&ResolvedStyleSet>,
) -> ComposedParagraph {
    match try_compose_paragraph_scoped(
        para,
        protect_metrics,
        metric_styles,
        &crate::storage::Unbounded,
    ) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}

fn try_compose_paragraph_scoped<S: crate::storage::Storage>(
    para: &Paragraph,
    protect_metrics: bool,
    metric_styles: Option<&ResolvedStyleSet>,
    storage: &S,
) -> Result<ComposedParagraph, S::Error> {
    // [Task #991] HWP5 parser 의 inline marker 누락 보정 (rendering 전용)
    let synth_para = try_synthesize_marker_paragraph(para, storage)?;
    let para = synth_para.as_ref().unwrap_or(para);

    let mut lines = try_compose_lines(para, storage)?;
    if let Some(styles) = metric_styles {
        punctuation_slots::try_resolve_run_slots(
            &mut lines,
            styles,
            crate::renderer::para_has_no_stored_line_segs(para),
            storage,
        )?;
    }
    let inline_controls = try_identify_inline_controls(para, storage)?;

    // treat_as_char 컨트롤의 텍스트 위치와 HWPUNIT 너비 수집
    let tac_positions = try_find_render_inline_control_positions(para, storage)?;
    let seg_width = para.line_segs.first().map(|s| s.segment_width).unwrap_or(0);
    let tac_controls: Vec<(usize, i32, usize)> =
        storage.collect(para.controls.iter().enumerate().filter_map(|(i, ctrl)| {
            let pos = *tac_positions.get(i)?;
            match ctrl {
                Control::Picture(p) if p.common.treat_as_char => {
                    Some((pos, inline_picture_size_hu(p).0, i))
                }
                Control::Shape(s) if s.common().treat_as_char => {
                    Some((pos, s.common().width as i32, i))
                }
                Control::Equation(eq) if eq.common.treat_as_char => {
                    // HWP 저장값을 사용 — 한컴 편집기가 실제 폰트로 계산한 정확한 너비
                    Some((
                        pos,
                        crate::renderer::equation::inline_advance_width_hwp(eq),
                        i,
                    ))
                }
                Control::Form(f) if f.common.treat_as_char => Some((pos, f.width as i32, i)),
                Control::Table(t)
                    if t.common.treat_as_char
                        && super::height_measurer::is_tac_table_inline_in_para(
                            t, seg_width, para,
                        ) =>
                {
                    // [#5785 후속] 선언 폭 우선.
                    let table_width: u32 = t.flow_width_hu();
                    Some((pos, table_width as i32, i))
                }
                _ => None,
            }
        }))?;

    // 각주/미주 위치 수집
    let footnote_positions: Vec<(usize, u16, usize)> =
        storage.collect(para.controls.iter().enumerate().filter_map(|(i, ctrl)| {
            let pos = *tac_positions.get(i)?;
            match ctrl {
                Control::Footnote(fn_) => Some((pos, fn_.number, i)),
                Control::Endnote(en) => Some((pos, en.number, i)),
                _ => None,
            }
        }))?;

    // 각주 마커는 paragraph_layout에서 FootnoteMarker 노드로 처리 (텍스트에 삽입하지 않음)

    // [#6101] 블록(비인라인) TAC 표와 뒤따르는 본문 텍스트가 저장 lineseg **한
    // 줄**에 함께 담긴 문단: 블록 표(폭 ≥ 줄폭 90%)와 텍스트는 물리적으로 한
    // 줄에 공존할 수 없고, 한글은 표를 줄 머리에 두고 텍스트를 다음 줄로
    // 내린다(36361137 7쪽 오라클 실측: 표 x 69.7, 텍스트는 표 아래 y 517.6).
    // 분리하지 않으면 ① 레이아웃 TAC 폴백이 line0 전체 폭(=뒤 텍스트 558px)을
    // leading 으로 오산해 표가 텍스트 폭만큼 우측으로 밀려 쪽 밖으로 잘리고
    // ② 조판이 텍스트 줄을 계상·발행하지 않아 본문이 통째로 소실된다.
    // 표 줄(빈 runs — TAC \n 경로의 "표 줄" 선례와 동형)과 텍스트 줄로 분리
    // 한다. 텍스트 줄 메트릭은 저장 줄높이(=표 높이)를 물려받으면 표가 이중
    // 계상되므로 lineseg-부재 합성 폴백과 같은 소형 값(400/320HU)으로 두어
    // 소비자(조판 max_fs 재산출·레이아웃 corrected_line_height)가 스타일로
    // 재산출하게 한다. 스트림상 텍스트가 표 앞이든 뒤든 같은 서명이다 —
    // 36501883(텍스트→표 순서)도 한글은 표를 줄 머리(x 76.8)에, 텍스트를 표
    // 아래(y 766.8)에 둔다. 저장 사다리가 이미 표에 자기 줄을 준 다중 줄
    // 문단(#842 선행 텍스트 축)은 표 줄에 가시 글자가 없어 여기 걸리지 않는다.
    let block_tac_split_pos = para
        .controls
        .iter()
        .enumerate()
        .find_map(|(i, ctrl)| match ctrl {
            Control::Table(t)
                if t.common.treat_as_char
                    && !tac_controls.iter().any(|(_, _, ci)| *ci == i)
                    && !super::height_measurer::is_tac_table_inline_in_para(t, seg_width, para) =>
            {
                tac_positions.get(i).copied()
            }
            _ => None,
        });
    if let Some(ctrl_pos) = block_tac_split_pos {
        let total_chars = para.text.chars().count();
        let chars: Vec<char> = storage.collect(para.text.chars())?;
        let line_end = |lines: &[ComposedLine], idx: usize| {
            lines
                .get(idx + 1)
                .map(|l| l.char_start)
                .unwrap_or(total_chars)
        };
        let host_line = (0..lines.len()).find(|&i| {
            let end = line_end(&lines, i).max(lines[i].char_start + 1);
            // 마지막 줄은 끝 경계 포함 — 텍스트→표 순서(36501883)는 컨트롤
            // 위치가 줄 끝(== 텍스트 길이)에 앵커된다.
            let end_inclusive = if i + 1 == lines.len() { end + 1 } else { end };
            lines[i].char_start <= ctrl_pos && ctrl_pos < end_inclusive
        });
        if let Some(li) = host_line {
            let start = lines[li].char_start.min(total_chars);
            let end = line_end(&lines, li).min(total_chars);
            // 가시 글자 판정: 공백·제어문자·오브젝트마커(U+FFFC)·한컴 PUA 필러
            // (BMP U+E000~F8FF, 보충평면 U+F0000~ — U+F081C 등)는 제외한다.
            // is_alphanumeric 으로 좁히면 별표(`*`)만으로 채운 마스킹 줄
            // (36501883)을 놓친다.
            let is_visible_glyph = |ch: char| {
                !ch.is_whitespace()
                    && ch > '\u{001F}'
                    && ch != '\u{FFFC}'
                    && !('\u{E000}'..='\u{F8FF}').contains(&ch)
                    && (ch as u32) < 0xF0000
            };
            let line_has_visible_text = chars[start..end].iter().any(|&c| is_visible_glyph(c));
            if line_has_visible_text {
                let table_line = ComposedLine {
                    runs: Vec::new(),
                    line_height: lines[li].line_height,
                    baseline_distance: lines[li].baseline_distance,
                    segment_width: lines[li].segment_width,
                    column_start: lines[li].column_start,
                    line_spacing: lines[li].line_spacing,
                    has_line_break: true,
                    char_start: lines[li].char_start,
                };
                lines[li].line_height = 400;
                lines[li].baseline_distance = 320;
                storage.reserve(&mut lines, 1)?;
                lines.insert(li, table_line);
            }
        }
    }

    let mut composed = ComposedParagraph {
        lines,
        para_style_id: para.para_shape_id,
        inline_controls,
        numbering_text: None,
        tac_controls,
        footnote_positions,
        // 탭의 저장 전진폭은 저장 줄에 속한다. 줄이 없거나 합성 줄이면 현재
        // 커서와 문단 TabDef로 다시 배치해야 인라인 개체 폭을 이중 반영하지 않는다.
        tab_extended: if super::para_has_no_stored_line_segs(para) {
            Vec::new()
        } else {
            storage.copy_slice(&para.tab_extended)?
        },
        horizontal_shaping: None,
        frame_geometry: None,
    };

    if let Some(styles) = metric_styles {
        equation_line_metrics::try_refresh_equation_rows(&mut composed, para, styles, storage)?;
    empty_line_metrics::refresh_with_storage(&mut composed, para, styles, storage)?;
    }

    supplemental_clusters::preserve_literal_spaces_with_storage(&mut composed, &para.literal_figure_spaces, storage)?;

    // CharOverlap 글자를 조합된 텍스트에 삽입
    try_inject_char_overlap_text(&mut composed, para, metric_styles, storage)?;

    // PUA 테두리 숫자(사각형/원형 안의 숫자) → CharOverlap 런으로 변환
    try_convert_pua_enclosed_numbers(&mut composed, storage)?;

    // Hanyang-PUA 옛한글 / 한컴 PUA와 legacy 제품명 표시 문자열 변환 (렌더링·측정용)
    // 제어 표기의 소프트 하이픈만 줄 끝에 표시한다. 모델 글자와 offset은 유지한다.
    let soft_hyphen_control = para.control_mask & (1u32 << 0x0018) != 0;
    try_convert_pua_display_text(&mut composed, soft_hyphen_control, metric_styles, storage)?;

    // Keep existing display projection intact; inserted control text has no source span.
    if protect_metrics {
        supplemental_clusters::preserve_boundaries(&mut composed, &para.text, metric_styles);
    }

    mapped_symbol_fonts::try_split_mapped_symbol_font_runs(&mut composed, storage)?;
    resolved_fields::try_apply(para, &mut composed, storage)?;
    if let Some(styles) = metric_styles {
        conjoining_jamo::split_runs_with_storage(&mut composed, styles, storage)?;
    }

    storage.bytes(0)?;
    Ok(composed)
}

/// Legacy composition을 먼저 완성한 뒤 Q2-C shadow transaction의 qualified
/// 결과만 같은 paragraph owner에 붙인다. 실패·상한·range mismatch는 기존
/// `compose_paragraph()` 결과를 그대로 반환한다.
pub(crate) fn compose_paragraph_with_horizontal_shaping(
    para: &Paragraph,
    styles: &ResolvedStyleSet,
) -> ComposedParagraph {
    match try_compose_paragraph_with_horizontal_shaping(para, styles, &crate::storage::Unbounded) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}

fn try_compose_paragraph_with_horizontal_shaping<S: crate::storage::Storage>(
    para: &Paragraph,
    styles: &ResolvedStyleSet,
    storage: &S,
) -> Result<ComposedParagraph, S::Error> {
    let mut composed = try_compose_paragraph_scoped(
        para,
        styles.supplemental_metrics.is_some(),
        Some(styles),
        storage,
    )?;
    // 본문 최초 composition과 편집 후 재조판이 같은 표제 투영을 소비한다.
    auto_number_projection::project(&mut composed, para);
    composed.horizontal_shaping =
        line_breaking::compose_horizontal_shaping_handoff(para, &composed, styles);
    storage.bytes(0)?;
    Ok(composed)
}

/// 컴포즈드 줄 목록에서 첫 "텍스트 포함 줄"(런에 텍스트 성격 문자가 있는 줄)의 인덱스.
/// 실제 텍스트가 있는 문단은 leading 컨트롤-전용 줄(수식 객체마커 ￼ 등)을 건너뛰고 이
/// 줄부터 그린다 — `LayoutEngine::layout_column_item`(실제 렌더)과
/// `TypesetEngine::measure_endnote_para_advance`(측정 전용)가 이 판정을 공유한다.
/// 종전엔 각자 재구현해 sep20/20(pi=936, 측정 127.7px vs 렌더 101.3px)에서 갈라졌다(#4312).
pub(crate) fn first_text_line(composed: &ComposedParagraph) -> Option<usize> {
    composed.lines.iter().position(|line| {
        line.runs
            .iter()
            .any(|r| r.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}'))
    })
}

/// Height of the physical LineSeg that owns a splittable TAC table.
///
/// A stored one-row table keeps the ordinary saved-ladder cap because its box
/// may overlap the following saved row. A current reflow row, or a multi-row
/// RowBreak table, uses the owning LineSeg as its vertical frame when that row
/// covers the declared object.
pub(crate) fn owned_rowbreak_tac_height(para: &Paragraph, control_index: usize) -> Option<i32> {
    let Control::Table(table) = para.controls.get(control_index)? else {
        return None;
    };
    if !table.common.treat_as_char
        || !matches!(
            table.page_break,
            crate::model::table::TablePageBreak::RowBreak
        )
    {
        return None;
    }
    let seg = para
        .line_segs
        .get(control_line_seg_index(para, control_index)?)?;
    let is_current_row = seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0;
    if table.row_count <= 1 && !is_current_row {
        return None;
    }
    (i64::from(seg.line_height) >= i64::from(table.common.height)).then_some(seg.line_height)
}

/// TAC 표를 소유한 줄이 실제 흐름에 남기는 양의 후행 간격.
/// 저장 HWP5의 표 소유 줄과 현재 재조판한 단일 RowBreak 줄은
/// 개체 프레임 뒤의 간격 절반을 물리 흐름에 남긴다.
pub(crate) fn tac_host_trailing_spacing(
    para: &Paragraph,
    control_index: usize,
    seg: &LineSeg,
    hwpx_stored_layout: bool,
    native_stored_layout: bool,
    dpi: f64,
) -> f64 {
    let spacing = crate::renderer::hwpunit_to_px(seg.line_spacing, dpi);
    // 압축 간격은 글자 높이에 대한 전진 감소량이다. 양수 저장 간격의
    // 절반 분배 규칙을 적용하거나 0으로 올리면 다음 문단이 내려간다.
    if spacing <= 0.0 {
        return spacing;
    }
    let current_owned_row = hwpx_stored_layout
        && para.line_segs.len() == 1
        && seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        && owned_rowbreak_tac_height(para, control_index).is_some();
    // 같은 문단의 뒤 글줄이 표 줄의 전체 간격 끝에서 시작하면 그 간격을
    // 줄 흐름과 표 배치에 모두 보존한다. 문단 경계의 별도 계약은 유지한다.
    let stored_successor_owns_full_gap = native_stored_layout
        && !para.stored_text_partition_is_dirty()
        && seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
        && para.line_segs.windows(2).any(|pair| {
            std::ptr::eq(&pair[0], seg)
                && pair[1].tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                && pair[1].text_start > seg.text_start
                && seg
                    .vertical_pos
                    .checked_add(seg.line_height)
                    .and_then(|end| end.checked_add(seg.line_spacing))
                    == Some(pair[1].vertical_pos)
        });
    if stored_successor_owns_full_gap {
        spacing
    } else if current_owned_row
        || (native_stored_layout
            && seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
            && !para.stored_text_partition_is_dirty())
    {
        spacing / 2.0
    } else {
        spacing
    }
}

/// 다음 문단을 현재 프레임에서 재조판하면 저장 vpos가 간격의 나머지를
/// 복원하지 않는다. 마지막 저장 TAC 줄은 후행 간격 전량을 흐름에 남긴다.
fn successor_lacks_stored_text_axis(para: &Paragraph) -> bool {
    para.stored_text_partition_is_dirty()
        || crate::renderer::para_has_no_stored_line_segs(para)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn tac_host_trailing_spacing_at_boundary(
    para: &Paragraph,
    next_para: Option<&Paragraph>,
    control_index: usize,
    seg: &LineSeg,
    hwpx_stored_layout: bool,
    native_stored_layout: bool,
    dpi: f64,
) -> f64 {
    if native_stored_layout
        && !para.stored_text_partition_is_dirty()
        && seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
        && para
            .line_segs
            .last()
            .is_some_and(|last| std::ptr::eq(last, seg))
        && next_para.is_some_and(successor_lacks_stored_text_axis)
    {
        crate::renderer::hwpunit_to_px(seg.line_spacing, dpi)
    } else {
        tac_host_trailing_spacing(
            para,
            control_index,
            seg,
            hwpx_stored_layout,
            native_stored_layout,
            dpi,
        )
    }
}

/// 뒤 본문을 재조판하면 그 본문의 저장 원점이 표 뒤 간격을 복원할 수 없다.
/// 마지막 권위 있는 표 소유 줄의 간격을 실제 흐름에 전량 남긴다.
pub(crate) fn tac_successor_uses_reflow_spacing(
    para: &Paragraph,
    next_para: Option<&Paragraph>,
    seg: &LineSeg,
    profile: crate::model::provenance::LayoutCompatibilityProfile,
) -> bool {
    profile.hwp5_stored_pagination_layout()
        && !profile.session_edited()
        && !para.stored_text_partition_is_dirty()
        && seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
        && para
            .line_segs
            .last()
            .is_some_and(|last| std::ptr::eq(last, seg))
        && next_para.is_some_and(|next| {
            next.controls.is_empty() && successor_lacks_stored_text_axis(next)
        })
}

/// 재구성한 표 줄의 옛 vpos와 독립 후속 줄의 새 로컬 축을 구분한다.
/// 합성 줄을 저장 사다리로 승격하지 않는다. 현재 개체의 전체 바깥 상자와
/// 다음 원본 줄이 함께 닫은 전진 끝만 측정 상한과 paint가 공유한다.
pub(crate) fn current_tac_local_successor_advance_hu(
    para: &Paragraph,
    next_para: Option<&Paragraph>,
    current_spacing_after_px: f64,
    next_spacing_before_px: f64,
    seg: &LineSeg,
    profile: crate::model::provenance::LayoutCompatibilityProfile,
    dpi: f64,
) -> Option<i32> {
    if !profile.hwpx_stored_layout()
        || profile.session_edited()
        || para.stored_text_partition_is_dirty()
        || para.cell_format_vpos_dirty
        || seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
        || seg.line_spacing < 0
        || para
            .text
            .chars()
            .any(|ch| ch > '\u{001f}' && ch != '\u{fffc}' && !ch.is_whitespace())
    {
        return None;
    }
    let [line] = para.line_segs.as_slice() else {
        return None;
    };
    let [Control::Table(table)] = para.controls.as_slice() else {
        return None;
    };
    if !std::ptr::eq(line, seg)
        || table.caption.is_some()
        || owned_rowbreak_tac_height(para, 0) != Some(seg.line_height)
        || i64::from(seg.line_height)
            != i64::from(table.common.height)
                + i64::from(table.outer_margin_top)
                + i64::from(table.outer_margin_bottom)
    {
        return None;
    }
    let next = next_para?;
    if next.stored_text_partition_is_dirty()
        || next.cell_format_vpos_dirty
        || !next.controls.is_empty()
        || next.raw_break_type != 0
        || next.column_type != crate::model::paragraph::ColumnBreakType::None
    {
        return None;
    }
    let first = next.line_segs.first()?;
    if first.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || first.vertical_pos < 0
        || first.vertical_pos >= seg.vertical_pos
    {
        return None;
    }
    let unit = hwpunit_to_px(1, dpi);
    let after = (current_spacing_after_px / unit).round() as i32;
    let before = (next_spacing_before_px / unit).round() as i32;
    let end = seg
        .line_height
        .checked_add(seg.line_spacing)?
        .checked_add(after)?;
    (end.checked_add(before)? == first.vertical_pos).then_some(end)
}

/// 실제 표 소유 줄의 끝과 다음 저장 줄이 닫히면 후행 간격 전량을 적용한다.
/// 같은 문단의 내부 줄에는 문단 앞/뒤 간격을 넣지 않고, 마지막 줄은 다음 문단의
/// 스타일 간격까지 대조한다. 분할 예산의 표 점유는 별도로 센다.
pub(crate) fn tac_next_line_full_spacing(
    para: &Paragraph,
    next_para: Option<&Paragraph>,
    current_spacing_after_px: f64,
    next_spacing_before_px: f64,
    seg: &LineSeg,
    profile: crate::model::provenance::LayoutCompatibilityProfile,
    dpi: f64,
) -> bool {
    if tac_successor_uses_reflow_spacing(para, next_para, seg, profile) {
        return true;
    }
    let next_spacing_before_hu = (next_spacing_before_px / hwpunit_to_px(1, dpi)).round() as i32;
    let current_spacing_after_hu =
        (current_spacing_after_px / hwpunit_to_px(1, dpi)).round() as i32;
    // 원본 저장 줄과 현재 개체 프레임을 소유한 재조판 줄을 구분한다.
    // 후자는 합성 태그를 가진 현재 사다리를 사용하므로 저장 원본으로 취급하지 않는다.
    let current_owned_row = profile.hwpx_stored_layout()
        && para.line_segs.len() == 1
        && seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        && para
            .controls
            .iter()
            .enumerate()
            .any(|(ci, _)| owned_rowbreak_tac_height(para, ci).is_some());
    // 저장 줄의 닫힌 사다리는 컨테이너와 다른 문단의 편집 여부에 무관하다.
    // 이 줄과 후속 줄의 원점·점유 높이·문단 간격이 유효하면 fit과 paint가
    // 같은 끝을 쓴다. 해당 줄의 텍스트·셀 서식이 바뀌었으면 수용하지 않는다.
    let stored_host = (profile.hwp5_stored_pagination_layout() || profile.hwpx_stored_layout())
        && !para.stored_text_partition_is_dirty()
        && !para.cell_format_vpos_dirty
        && seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0;
    // 다음 흐름 개체가 도형이어도 같은 문단의 저장 줄 간격을 소유한다.
    // 표 개수만 세어 마지막 표로 판단하면 두 줄 사이의 거리를 절반으로 줄인다.
    // 같은 vpos의 수평 조각과 편집/합성 사다리는 이 닫힘의 근거가 아니다.
    if stored_host
        && !para.cell_format_vpos_dirty
        && seg.vertical_pos >= 0
        && seg.line_height > 0
        && seg.line_spacing > 0
        && para.line_segs.windows(2).any(|pair| {
            std::ptr::eq(&pair[0], seg)
                && pair[1].tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                && pair[1].line_height > 0
                && i64::from(seg.vertical_pos)
                    + i64::from(seg.line_height)
                    + i64::from(seg.line_spacing)
                    == i64::from(pair[1].vertical_pos)
        })
    {
        return true;
    }
    (stored_host || current_owned_row)
        && para
            .line_segs
            .last()
            .is_some_and(|last| std::ptr::eq(last, seg))
        && next_para.is_some_and(|next| {
            (current_owned_row || !next.stored_text_partition_is_dirty())
                && (next.line_segs.is_empty()
                    || next.line_segs.first().is_some_and(|first| {
                        (current_owned_row
                            || (first.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                                && !next.cell_format_vpos_dirty))
                            && (stored_tac_successor_delta_hu(
                                para,
                                seg,
                                first,
                                current_spacing_after_hu.saturating_add(next_spacing_before_hu),
                            )
                            .is_some_and(|delta| {
                                seg.line_height
                                    .checked_add(seg.line_spacing)
                                    .and_then(|advance| {
                                        advance.checked_add(current_spacing_after_hu)
                                    })
                                    .and_then(|advance| advance.checked_add(next_spacing_before_hu))
                                    == Some(delta)
                            })
                                || crate::renderer::float_placement::restarted_tac::restarted_tac_line_advance(
                                    para,
                                    next,
                                    current_spacing_after_px,
                                    next_spacing_before_px,
                                    dpi,
                                ).is_some())
                    }))
        })
}

/// 다음 저장 원점과 표 소유 줄의 거리. 쪽으로 통째 이월된 단일 TAC 줄은
/// 호스트에 이전 쪽 vpos가 남아도 후속 문단이 새 쪽의 0 원점에서 상자를 닫는다.
/// 줄 높이가 바깥여백 포함 개체 상자와 같고 후속 원점이 정확히 닫힐 때만 그
/// 좌표계 전환을 사용한다. 측정과 실제 배치가 같은 거리 계약을 소비한다.
pub(crate) fn stored_tac_successor_delta_hu(
    para: &Paragraph,
    seg: &LineSeg,
    successor: &LineSeg,
    paragraph_spacing_hu: i32,
) -> Option<i32> {
    let closing_advance = seg
        .line_height
        .checked_add(seg.line_spacing)?
        .checked_add(paragraph_spacing_hu)?;
    let reset_closes_owned_line = successor.vertical_pos < seg.vertical_pos
        && successor.vertical_pos == closing_advance
        && para.line_segs.len() == 1
        && !para.cell_format_vpos_dirty
        && successor.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
        && stored_first_tac_line(para).is_some_and(|first| std::ptr::eq(first, seg))
        && para
            .controls
            .iter()
            .filter(|control| matches!(control, Control::Table(_)))
            .count()
            == 1
        && !para.controls.iter().any(|control| {
            matches!(
                control,
                Control::Picture(_) | Control::Shape(_) | Control::Equation(_)
            )
        });
    if reset_closes_owned_line {
        Some(successor.vertical_pos)
    } else {
        successor.vertical_pos.checked_sub(seg.vertical_pos)
    }
}
