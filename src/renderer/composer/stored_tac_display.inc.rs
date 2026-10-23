/// 저장된 서로 다른 물리 줄을 소유한 빈/공백 캐리어 TAC 표의 흐름.
/// top/end는 첫 저장 줄 원점 기준 HU이며, 테두리가 아닌 바깥여백 포함 pen이다.
#[derive(Debug, Clone, Copy)]
pub(crate) struct StoredTacLine {
    pub control: usize,
    pub top: i32,
    pub end: i32,
    pub occupied_end: i32,
}

/// 가시 텍스트 없이 첫 저장 줄 전체를 소유한 TAC 표의 줄 상자다.
/// 다중 줄의 페인트 단축과 달리 단일 표도 수용하며, 합성/편집 줄은 원본 근거가 아니다.
pub(crate) fn stored_first_tac_line(para: &Paragraph) -> Option<&LineSeg> {
    let first = para.line_segs.first()?;
    if para.stored_text_partition_dirty
        || para
            .line_segs
            .iter()
            .any(|line| line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0)
        || para
            .text
            .chars()
            .any(|c| !c.is_whitespace() && c > '\u{001f}' && c != '\u{fffc}')
    {
        return None;
    }
    para.controls
        .iter()
        .enumerate()
        .any(|(ci, control)| {
            let Control::Table(table) = control else {
                return false;
            };
            table.common.treat_as_char
                && table.caption.is_none()
                && control_line_seg_index(para, ci) == Some(0)
                && i64::from(first.line_height)
                    == i64::from(table.common.height)
                        + i64::from(table.outer_margin_top)
                        + i64::from(table.outer_margin_bottom)
        })
        .then_some(first)
}

/// Ordinary whitespace rows before an object in the same stored fragment.
/// A vpos reset has a different continuation owner; its prefix cannot be
/// emitted again beside the object after that owner advances the page.
pub(crate) fn ordinary_tac_prefix_rows(para: &Paragraph, owner: usize) -> bool {
    !para.text.is_empty()
        && para.text.chars().all(char::is_whitespace)
        && owner > 0
        && para.line_segs.get(owner).is_some_and(|row| {
            row.vertical_pos >= para.line_segs[0].vertical_pos
                && para.line_segs[..=owner]
                    .windows(2)
                    .all(|pair| pair[1].vertical_pos >= pair[0].vertical_pos)
                && para.line_segs[..owner]
                    .iter()
                    .all(|prefix| prefix.line_height == prefix.text_height)
        })
}

pub(crate) fn stored_tac_lines(para: &Paragraph) -> Option<Vec<StoredTacLine>> {
    // 공백도 자기 저장 줄을 가질 수 있다. 표 앞 공백 줄의 line_height에는
    // 문단의 최대 개체 높이가 반복 저장되므로 text_height와 다음 원점을 확인한다.
    let whitespace_carrier = !para.text.is_empty() && para.text.chars().all(char::is_whitespace);
    if !whitespace_carrier {
        para.empty_control_stream_position(0)?;
    }
    if para.stored_text_partition_dirty
        || para
            .line_segs
            .iter()
            .any(|s| s.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0)
    {
        return None;
    }
    let origin = para.line_segs.first()?.vertical_pos;
    let mut lines = Vec::new();
    let mut previous_owner = None;
    for (ci, control) in para.controls.iter().enumerate() {
        let table = match control {
            Control::Table(table) if table.common.treat_as_char => table,
            // [#7344] 누름틀 시작 마커는 원시 스트림의 8유닛 칸만 차지하고 줄 상자를
            // 만들지 않는다(결재문서 `본문` 누름틀이 표 문단 첫머리에 놓인다). 같은 문단에서
            // 닫히는 필드는 위 `empty_control_stream_position`이 이미 거른다.
            Control::SectionDef(_)
            | Control::ColumnDef(_)
            | Control::Header(_)
            | Control::Footer(_)
            | Control::PageNumberPos(_)
            // 감추기는 쪽 표시 설정이며 본문 글줄을 점유하지 않는다.
            | Control::PageHide(_)
            | Control::Field(_) => continue,
            _ => return None,
        };
        let owner = control_line_seg_index(para, ci)?;
        let seg = para.line_segs.get(owner)?;
        // 저장 줄은 아래 캡션까지 포함한 온전한 객체 상자를 소유한다.
        // 실제 원본 줄이 있는 캡션만 인증하며 위·옆 캡션은 별도 경로에 둔다.
        let caption_extent = match &table.caption {
            None => 0,
            Some(caption) => {
                if caption.direction != crate::model::shape::CaptionDirection::Bottom
                    || caption.paragraphs.is_empty()
                    || caption.paragraphs.iter().any(|paragraph| {
                        paragraph.stored_text_partition_is_dirty()
                            || paragraph.cell_format_vpos_dirty
                            || paragraph.line_segs.is_empty()
                            || paragraph.line_segs.iter().any(|line| {
                                line.line_height <= 0
                                    || line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                            })
                    })
                {
                    return None;
                }
                i64::from(crate::renderer::px_to_hwpunit(
                    caption_height_px(&table.caption, crate::renderer::DEFAULT_DPI),
                    crate::renderer::DEFAULT_DPI,
                )) + i64::from(caption.spacing)
            }
        };
        let outer_height = i64::from(table.common.height)
            + i64::from(table.outer_margin_top)
            + i64::from(table.outer_margin_bottom)
            + caption_extent;
        // 여기서 높이가 같다는 것은 baseline 정렬된 객체의 전체 점유 줄을 뜻한다.
        // 텍스트와 같은 줄, 더 큰 이웃 객체, 쪽 리셋은 기존 분할/재조판이 처리한다.
        if previous_owner.is_some_and(|previous| previous >= owner)
            || i64::from(seg.line_height) != outer_height
            || seg.vertical_pos < origin
        {
            return None;
        }
        // Ordinary prefix rows own their text/empty-line boxes and indentation.
        // A table-only placement cannot replace those rows with an anonymous
        // offset. Let the common composed paragraph route emit and measure them.
        // Carriers whose lh repeats the object maximum still need this band's
        // th-based physical-space contract.
        if ordinary_tac_prefix_rows(para, owner) {
            return None;
        }
        // 빈 컨트롤 캐리어도 표 앞에 짧은 저장 줄을 가질 수 있다.
        // 표 높이가 반복된 line_height 대신 실제 text_height와 다음 원점을
        // 대조하여 빈 줄의 물리 점유를 보존한다.
        if (whitespace_carrier || (lines.is_empty() && owner > 0))
            && (table.caption.is_some()
                || owner == 0
                || para.line_segs[..owner].iter().any(|blank| {
                    // 음수 줄간격도 실제 줄 전진량이 유효하면 저장 줄의 일부다.
                    blank.text_height <= 0
                        || i64::from(blank.text_height) + i64::from(blank.line_spacing) < 0
                })
                || i64::from(seg.text_height) != outer_height
                || para.line_segs[..owner].windows(2).any(|pair| {
                    i64::from(pair[0].vertical_pos)
                        + i64::from(pair[0].text_height)
                        + i64::from(pair[0].line_spacing)
                        != i64::from(pair[1].vertical_pos)
                })
                || i64::from(para.line_segs[owner - 1].vertical_pos)
                    + i64::from(para.line_segs[owner - 1].text_height)
                    + i64::from(para.line_segs[owner - 1].line_spacing)
                    != i64::from(seg.vertical_pos))
        {
            return None;
        }
        let top = seg.vertical_pos.checked_sub(origin)?;
        if lines
            .last()
            .is_some_and(|line: &StoredTacLine| line.top >= top)
        {
            return None;
        }
        let occupied_end = top.checked_add(seg.line_height)?;
        let end = occupied_end.checked_add(seg.line_spacing)?;
        if end < top {
            return None;
        }
        lines.push(StoredTacLine {
            control: ci,
            top,
            occupied_end,
            end,
        });
        previous_owner = Some(owner);
    }
    let single_prefix_carrier = lines.len() == 1
        && previous_owner.is_some_and(|owner| owner > 0 && owner == para.line_segs.len() - 1);
    if lines.len() < 2 && !whitespace_carrier && !single_prefix_carrier {
        return None;
    }
    // 글자가 있는 일반 문단이나 같은 줄의 여러 개체는 재조판 경로가 처리한다.
    // 공백 캐리어의 단축은 앞 공백 줄과 마지막 표 줄이 분리된 단일 표만 수용한다.
    if whitespace_carrier && (lines.len() != 1 || previous_owner != Some(para.line_segs.len() - 1))
    {
        return None;
    }
    // 다음 표의 실제 소유 줄로 이동한다. 중간의 header/footer용 LineSeg는 표 줄이 아니다.
    for i in 0..lines.len() - 1 {
        lines[i].end = lines[i + 1].top;
    }
    Some(lines)
}

/// 캡션(문단 목록)의 총 높이를 px 로 계산한다.
///
/// 렌더(`calculate_caption_height`)와 측정(`measure_caption`)이 각자 재구현하며 갈라졌던
/// 산식을 통일한 것이다(#4320). 저장된 `line_segs` 는 한컴이 실제로 배치한 레이아웃 값이므로
/// `compose_paragraph` 재계산 높이의 하한으로 쓴다 — `line_segs`가 있어도 폰트 대체 등으로
/// 재계산 높이가 더 작게 나오면 실제보다 낮게 예약되어 다음 요소가 겹친다
/// (`2d973021c`가 `samples/rowbreak-problem-pages.hwpx`에서 고친 오버랩이 이 경우다).
/// `line_segs`가 비어 있으면(레이아웃 전/미저장 캡션) `compose_paragraph`로만 계산한다.
pub fn caption_height_px(caption: &Option<Caption>, dpi: f64) -> f64 {
    caption_height_px_in_context(caption, None, dpi)
}

/// [`caption_height_px`] 에 글자 모양을 더한 판. 저장 줄이 없는 **빈** 캡션 문단의 줄 높이를
/// 그 문단 글자 크기로 잡는다 — 캡션을 실제로 그리는 `layout_caption` 이 그 크기의 줄 하나를
/// 그리므로 측정과 배치가 같은 높이를 쓴다. 글자 모양이 없으면 종전 기본값(400 HU)이다.
///
/// 78494 10쪽: 표 위 캡션은 빈 문단 하나(글자 2pt = 200 HU, 저장 줄 없음)다. 기본값 400 HU 로
/// 재면 표가 2.67px 아래로 밀린다(rhwp 245.93, 한/글 2020 243.09; 200 HU 면 243.27).
pub(crate) fn caption_height_px_in_context(
    caption: &Option<Caption>,
    styles: Option<&ResolvedStyleSet>,
    dpi: f64,
) -> f64 {
    let caption = match caption {
        Some(c) => c,
        None => return 0.0,
    };

    if caption.paragraphs.is_empty() {
        return 0.0;
    }

    // line_segs가 비어 컴포즈로 대체할 때 쓰는 기본 줄 높이(HWPUNIT).
    const DEFAULT_LINE_HEIGHT_HWPUNIT: i32 = 400;

    let mut line_seg_height = 0.0f64;
    let mut composed_height = 0.0f64;
    for para in &caption.paragraphs {
        if let (Some(first), Some(last)) = (para.line_segs.first(), para.line_segs.last()) {
            let para_top = first.vertical_pos.min(0);
            let para_bottom = last.vertical_pos.saturating_add(last.line_height);
            line_seg_height = line_seg_height.max(hwpunit_to_px(para_bottom - para_top, dpi));
        }

        let composed = compose_paragraph(para);
        if composed.lines.is_empty() {
            let own_font_size = styles
                .filter(|_| para.line_segs.is_empty())
                .and_then(|styles| {
                    let id = para
                        .char_shape_id_at(0)
                        .or_else(|| para.char_shapes.first().map(|shape| shape.char_shape_id))?;
                    styles.char_styles.get(id as usize)
                })
                .map(|style| style.font_size)
                .filter(|size| *size > 0.0);
            composed_height +=
                own_font_size.unwrap_or_else(|| hwpunit_to_px(DEFAULT_LINE_HEIGHT_HWPUNIT, dpi));
        // 기본 줄 높이
        } else {
            for (i, line) in composed.lines.iter().enumerate() {
                let line_h = hwpunit_to_px(line.line_height, dpi);
                let spacing = if i < composed.lines.len() - 1 {
                    hwpunit_to_px(line.line_spacing, dpi)
                } else {
                    0.0 // 마지막 줄은 line_spacing 제외
                };
                composed_height += line_h + spacing;
            }
        }
    }

    line_seg_height.max(composed_height)
}

/// Hanyang-PUA 옛한글 코드포인트·한컴 PUA와 legacy 제품명을 렌더링용 텍스트로 변환한다.
///
/// 한컴 자체 폰트 (함초롬바탕 LVT 등) 는 PUA 영역에 옛한글 글리프를 직접
/// 보유하나, OFL 폰트 (Noto Serif KR / Source Han Serif K 등) 는 KS X 1026-1
/// 자모 영역만 지원하므로 PUA → 자모 변환 후 합자 렌더링이 필요.
///
/// `U+F012B` 같은 한컴 전용 PUA 기호는 표준 Unicode 단일 문자 대응이 없어서
/// 표시 문자열(`(인)`)로 확장한다. 본 함수는 `run.text` 를 변경하지 않고
/// `run.display_text` 에만 변환 결과를 저장한다. 이는 `char_offsets`,
/// `line.char_start`, `line_chars` 등 인덱싱 불변성을 유지하기 위함이다
/// (PUA 1 char = display N chars).
///
/// 1990년대 한컴 제품 설명서는 `ᄒᆞᆫ글`·`ᄒᆞᆫ메일`처럼 제품명을 옛한글 자모로
/// 저장했지만, 한컴 PDF는 이를 각각 `한글`·`한메일`로 인쇄한다. 이것은 일반
/// 옛한글 정규화가 아니다. 아래의 닫힌 제품명 어휘만 display projection으로
/// 바꾸며, 원문 IR·검색·캐럿 offset은 그대로 보존한다.
///
/// 매핑 표: KTUG HanyangPuaTableProject (Public Domain).
fn convert_pua_display_text(
    composed: &mut ComposedParagraph,
    soft_hyphen_control: bool,
    styles: Option<&ResolvedStyleSet>,
) {
    use super::pua_oldhangul::map_pua_old_hangul;

    let product_prefix_starts = legacy_hancom_product_prefix_starts(composed);
    let mut run_char_start = 0usize;
    for line in composed.lines.iter_mut() {
        // [#7993] 제어 표기 소프트 하이픈은 줄 끝에서만 `-` 로 보이고, 줄 중간에서는
        // 보이지도 폭을 갖지도 않는다(한/글 2020 Print: task1725 162쪽 `<hp:hypen/>` 16개가
        // 줄 첫머리에 아무 자국도 남기지 않는다). 모델 글자는 offset 공간에 남기고
        // 표시 문자열만 바꾼다 — 줄 끝 판정은 이 줄의 마지막 모델 글자다.
        let line_last_run = line.runs.iter().rposition(|run| !run.text.is_empty());
        for (run_index, run) in line.runs.iter_mut().enumerate() {
            let program = styles.and_then(|styles| run.text_style(styles).native_pua_metrics);
            let chars: Vec<char> = run.text.chars().collect();
            let run_char_end = run_char_start + chars.len();
            let has_product_projection = product_prefix_starts
                .iter()
                .any(|start| *start < run_char_end && start.saturating_add(3) > run_char_start);
            let has_pua_display = chars.iter().any(|ch| {
                pua_plain_text_display(*ch).is_some()
                    || map_pua_old_hangul(*ch).is_some()
                    || map_pua_bullet_char(*ch) != *ch
                    || (soft_hyphen_control && *ch == '\u{00AD}')
            });
            let line_end_char = if Some(run_index) == line_last_run {
                chars.len().checked_sub(1)
            } else {
                None
            };
            if !has_product_projection && !has_pua_display {
                run_char_start = run_char_end;
                continue;
            }
            let mut display = String::with_capacity(run.text.len() * 3);
            let mut changed = false;
            for (index, ch) in chars.iter().copied().enumerate() {
                let char_position = run_char_start + index;
                if product_prefix_starts.contains(&char_position) {
                    display.push('한');
                    changed = true;
                } else if product_prefix_starts
                    .iter()
                    .any(|start| (start + 1..start + 3).contains(&char_position))
                {
                    // `ᄒᆞᆫ` 세 자모가 style/line 경계를 넘더라도 첫 위치에만
                    // `한`을 투영한다. 뒤 두 model char는 offset 공간에만 남긴다.
                    changed = true;
                } else if soft_hyphen_control && ch == '\u{00AD}' {
                    if line_end_char == Some(index) {
                        display.push('-');
                    }
                    changed = true;
                } else if program
                    .as_ref()
                    .is_some_and(|program| program.advance_em(ch).is_some())
                {
                    // 실제 공급한 원 glyph는 Unicode 표시 대체보다 우선한다.
                    // 측정과 paint가 같은 프로그램의 전진·표시 문자를 소비한다.
                    let source = hancom_symbol_display_key(ch);
                    display.push(source);
                    changed |= source != ch;
                } else if let Some(replacement) = pua_plain_text_display(ch) {
                    display.push_str(replacement);
                    changed = true;
                } else if let Some(jamos) = map_pua_old_hangul(ch) {
                    display.extend(jamos.iter().copied());
                    changed = true;
                } else {
                    // paint 경로(`expand_pua_display_text`)와 같이 한컴 PUA 책괄호
                    // (U+F0854/F0855 → 《》) 등을 측정 문자열에도 올린다. 원문 PUA 는
                    // CJK가 아니라 0.5em 휴리스틱이 되어 run bbox 가 전각 글리프보다
                    // 짧고, 다음 라틴 run 이 한글 위에 겹친다 (#6057).
                    let mapped = map_pua_bullet_char(ch);
                    display.push(mapped);
                    if mapped != ch {
                        changed = true;
                    }
                }
            }
            run_char_start = run_char_end;
            // 글꼴 공급 뒤 재구성할 때 앞선 일반 compose의 대체 문자열도 폐기한다.
            // 원 PUA를 그대로 쓰는 결과는 changed=false여도 이전 display와 다르다.
            run.display_text = changed.then_some(display);
        }
    }
}

const LEGACY_HANCOM_PRODUCT_WORDS: [(&str, &str); 4] = [
    ("ᄒᆞᆫ글", "한글"),
    ("ᄒᆞᆫ메일", "한메일"),
    ("ᄒᆞᆫ팩스", "한팩스"),
    ("ᄒᆞᆫ소프트", "한소프트"),
];

/// 한컴 PDF가 현대 글리프로 인쇄하는 레거시 제품명만 화면 문자열로 투영한다.
///
/// 이 함수는 모델 문자열을 정규화하지 않는다. 표 셀처럼 composer를 우회해
/// `TextRunNode`를 직접 만드는 레이아웃 경로에도 같은 제한된 표시 계약을
/// 적용하기 위해 render-tree 최종화 단계에서 재사용한다.
pub(crate) fn legacy_hancom_product_display_text(text: &str) -> Option<String> {
    let mut display = text.to_owned();
    for (legacy, modern) in LEGACY_HANCOM_PRODUCT_WORDS {
        display = display.replace(legacy, modern);
    }
    (display != text).then_some(display)
}

/// `ᄒᆞᆫ`이 legacy 한컴 제품명으로 쓰인 model-character 시작 위치를 찾는다.
///
/// `ComposedParagraph`의 run은 줄·글자모양 경계에서 나뉠 수 있으므로, 먼저 모든
/// run을 이어 검사한 뒤 model char 좌표를 돌려준다. 그 뒤 display projection은
/// run별로 적용해도 줄 경계를 넘어선 제품명을 놓치지 않는다.
fn legacy_hancom_product_prefix_starts(composed: &ComposedParagraph) -> Vec<usize> {
    let logical_text: String = composed
        .lines
        .iter()
        .flat_map(|line| line.runs.iter())
        .map(|run| run.text.as_str())
        .collect();
    let mut starts = Vec::new();
    for (char_index, (byte_index, _)) in logical_text.char_indices().enumerate() {
        if LEGACY_HANCOM_PRODUCT_WORDS
            .iter()
            .any(|(legacy, _)| logical_text[byte_index..].starts_with(legacy))
        {
            starts.push(char_index);
        }
    }
    starts
}

/// 각주 마커를 해당 텍스트 위치의 런에 인라인 삽입
/// 각주 위치에서 기존 런을 분할하고 마커 런("1)" 등)을 사이에 삽입
fn inject_footnote_markers(lines: &mut [ComposedLine], positions: &[(usize, u16)]) {
    for &(char_pos, number) in positions {
        let marker_text = format!("{})", number);
        // char_pos에 해당하는 줄과 런 찾기
        for line in lines.iter_mut() {
            let line_start = line.char_start;
            let line_end = line_start
                + line
                    .runs
                    .iter()
                    .map(|r| r.text.chars().count())
                    .sum::<usize>();
            if char_pos < line_start || char_pos > line_end {
                continue;
            }

            // 이 줄 내에서 char_pos에 해당하는 런 찾기
            let mut run_char = line_start;
            let mut target_run_idx = None;
            let mut offset_in_run = 0;
            for (ri, run) in line.runs.iter().enumerate() {
                let run_len = run.text.chars().count();
                if char_pos >= run_char && char_pos <= run_char + run_len {
                    target_run_idx = Some(ri);
                    offset_in_run = char_pos - run_char;
                    break;
                }
                run_char += run_len;
            }

            if let Some(ri) = target_run_idx {
                let orig_run = &line.runs[ri];
                let cs_id = orig_run.char_style_id;
                let lang = orig_run.lang_index;

                // 런을 분할: [앞부분] [마커] [뒷부분]
                let orig_text: Vec<char> = orig_run.text.chars().collect();
                let before: String = orig_text[..offset_in_run].iter().collect();
                let after: String = orig_text[offset_in_run..].iter().collect();

                let marker_run = ComposedTextRun {
                    text: marker_text.clone(),
                    char_style_id: cs_id,
                    lang_index: lang,
                    neutral_space_origin: None,
                    char_overlap: None,
                    footnote_marker: Some(number),
                    display_text: None,
                    supplemental_metrics_blocked: false,
                    inserted_control_text: false,
                    space_metric: crate::renderer::composer::SpaceMetric::Stored,
                    regenerated_font_metrics: line.runs[ri].regenerated_font_metrics,
                };

                let mut new_runs = Vec::new();
                // 앞부분에서 기존 런 교체
                for (i, run) in line.runs.iter().enumerate() {
                    if i == ri {
                        if !before.is_empty() {
                            new_runs.push(ComposedTextRun {
                                text: before.clone(),
                                char_style_id: cs_id,
                                lang_index: lang,
                                neutral_space_origin: None,
                                char_overlap: run.char_overlap.clone(),
                                footnote_marker: None,
                                display_text: None,
                                supplemental_metrics_blocked: run.supplemental_metrics_blocked,
                                inserted_control_text: run.inserted_control_text,
                                space_metric: run.space_metric,
                                regenerated_font_metrics: run.regenerated_font_metrics,
                            });
                        }
                        new_runs.push(marker_run.clone());
                        if !after.is_empty() {
                            new_runs.push(ComposedTextRun {
                                text: after.clone(),
                                char_style_id: cs_id,
                                lang_index: lang,
                                neutral_space_origin: None,
                                char_overlap: run.char_overlap.clone(),
                                footnote_marker: None,
                                display_text: None,
                                supplemental_metrics_blocked: run.supplemental_metrics_blocked,
                                inserted_control_text: run.inserted_control_text,
                                space_metric: run.space_metric,
                                regenerated_font_metrics: run.regenerated_font_metrics,
                            });
                        }
                    } else {
                        new_runs.push(run.clone());
                    }
                }
                line.runs = new_runs;
                break; // 이 각주 처리 완료
            }
        }
    }
}

// 저장 줄의 실제 점유 높이. 줄 구성과 저장 줄 소속 재사용 판정이 같은
// 메트릭을 비교해야 글자 테두리로 높아진 정상 줄을 재조판 줄로 오인하지 않는다.
fn stored_line_box_height(seg: &LineSeg) -> i32 {
    seg.line_height.max(seg.text_height)
}
