/// 새 LINE_SEG를 만들 때만 한컴 재조판 공백 metric을 반영한 텍스트 폭.
///
/// 저장본의 글꼴 고유 공백과 한컴이 새로 조판한 반각 공백은 다를 수 있다. 호출자는
/// 저장 LINE_SEG가 없는 문단이나 폭 변경 뒤 재조판한 문단만 `true`를 전달한다.
fn estimate_regenerated_line_text_width(
    text: &str,
    style: &TextStyle,
    regenerated_line_space_metric: bool,
) -> f64 {
    let measured = estimate_text_width_unrounded(text, style);
    if !regenerated_line_space_metric {
        return measured;
    }
    let Some(regenerated_space_width) = hancom_regenerated_space_width(style) else {
        return measured;
    };
    let stored_space_width = estimate_text_width_unrounded(" ", style);
    measured
        + text.chars().filter(|&ch| ch == ' ').count() as f64
            * (regenerated_space_width - stored_space_width)
}

/// literal-space 들여쓰기 재조판에서 사용할 반각 공백 advance.
///
/// 한컴 PDF의 #3128 문단은 글꼴 고유 U+0020 폭이 반각보다 넓어도
/// (한양중고딕 550/1024em) 선행 들여쓰기와 재조판된 내부 공백을 모두
/// 0.5em 칸으로 측정한다. 이 함수는 둘 이상의 literal 선행 공백과 동일
/// 글꼴 metric/구간별 자간을 확인한 좁은 fallback 경로에서만 사용한다.
fn regenerated_half_space_width(style: &TextStyle) -> f64 {
    let font_size = style.font_size.max(0.0);
    let ratio = if style.ratio > 0.0 { style.ratio } else { 1.0 };
    // [#7387] `use_font_space` run 은 반각이 아니라 영문 슬롯 글꼴의 공백폭을 쓴다.
    let base = font_size * style.font_space_em.unwrap_or(0.5) * ratio;
    let tracking = if font_size > 0.0 {
        style.letter_spacing * (base / font_size)
    } else {
        style.letter_spacing
    };
    let mut width = base + tracking + style.extra_char_spacing + style.extra_word_spacing;
    if style.letter_spacing + style.extra_char_spacing < 0.0 {
        width = width.max(base * 0.5);
    }
    width
}

/// [#2146] 저장 LINE_SEG 이 전혀 없고(NO_LS) 모든 문단이 1줄이며 각 줄이 셀
/// 폭을 여유 있게 쓰는 코너-라벨 셀 중, 선언 셀높이를 신뢰할 수 있는 두 경우:
///
/// - (a) **사선(대각선) 셀** — 셀 BF 또는 cellzone BF(#1623)에 사선. 한글은
///   사선 셀 문단("|직렬" 등)을 일반 텍스트 흐름으로 배치하지 않고 코너
///   라벨로 그리므로 행높이가 저장 선언 그대로다 (21761835 r0 c0).
/// - (b) **고정(Fixed) 줄간격 모순 셀** — 전 문단 Fixed ls 합이 선언 내부높이
///   초과 (21761835 r0 c1 "계급|직류": 37.76px×2 > 48.6px). 저장 스타일과
///   저장 지오메트리가 충돌하면 한글은 지오메트리(선언 행높이)를 유지한다.
///
/// 재합성 줄높이가 선언을 초과해도 선언높이를 신뢰한다 (#1763/#2097 계열).
///
/// 그 밖의 **사선 없는** 일반 라벨 셀은 제외한다 — 한글이 fresh 레이아웃으로
/// 선언 이상 키우는 문서(#1891 76076 규제영향분석서: 구분/장점/할인율 등
/// 클램프 시 82→79쪽 회귀 관측)가 존재하여 선언 신뢰가 성립하지 않는다.
/// 보조 가드:
/// - 폭 여유(85%): 한글 폰트 메트릭이 본 환경보다 넓어 한글에서만 2줄로
///   래핑되는 셀 배제.
/// - 선언 내부높이 ≥ 문단별 em 합: 한 줄 em 도 못 담는 스테일(생성기 기록)
///   선언높이 배제 — 한글은 최소 em 으로 행을 키운다 (#1842 em 원칙).
pub(crate) fn no_ls_short_label_cell(
    cell: &crate::model::table::Cell,
    table: &crate::model::table::Table,
    cell_inner_width: f64,
    cell_inner_height: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> bool {
    if cell.paragraphs.is_empty() || cell_inner_width <= 0.0 || cell_inner_height <= 0.0 {
        return false;
    }
    let bf_has_diagonal = |bf_id: u16| {
        bf_id != 0
            && styles
                .border_styles
                .get((bf_id as usize).saturating_sub(1))
                .is_some_and(crate::renderer::layout::border_style_has_diagonal)
    };
    // 사선은 셀 자체 BF 또는 셀을 덮는 cellzone BF(#1623)에 지정될 수 있다.
    let cell_has_diagonal = bf_has_diagonal(cell.border_fill_id)
        || table.zones.iter().any(|z| {
            z.start_row <= cell.row
                && cell.row <= z.end_row
                && z.start_col <= cell.col
                && cell.col <= z.end_col
                && bf_has_diagonal(z.border_fill_id)
        });
    // 저장 고정(Fixed) 줄간격의 합이 선언 내부높이를 초과하는 모순 셀
    // (21761835 r0 c1 "계급|직류": ps Fixed 37.76px ×2문단 > 선언 내부 48.6px).
    // 저장 스타일과 저장 지오메트리가 충돌할 때 한글은 지오메트리(선언 행높이)
    // 를 유지한다 — 선언 신뢰 가능한 국소 모순 신호.
    let fixed_ls_contradicts_declared = {
        let mut sum = 0.0f64;
        let all_fixed = cell.paragraphs.iter().all(|p| {
            styles
                .para_styles
                .get(p.para_shape_id as usize)
                .map(|ps| {
                    if ps.line_spacing_type == crate::model::style::LineSpacingType::Fixed {
                        sum += ps.line_spacing;
                        true
                    } else {
                        false
                    }
                })
                .unwrap_or(false)
        });
        all_fixed && sum > cell_inner_height
    };
    if !cell_has_diagonal && !fixed_ls_contradicts_declared {
        return false;
    }
    if !cell.paragraphs.iter().all(|p| p.line_segs.is_empty()) {
        return false;
    }
    let mut em_sum = 0.0f64;
    for p in &cell.paragraphs {
        let mut comp = compose_paragraph_in_context(p, styles);
        recompose_cell_lines_in_frame(
            &mut comp,
            p,
            // [#7422] 칸 내용 상자도 본문과 **같은** 문단 여백 계약을 쓴다. `#7410` 이
            // 편집 경로에서 통일한 계약이 이 재조합 경로에는 닿지 않아, 같은 문단이 어느
            // 경로로 왔느냐로 다른 상자를 받았다 — 칸 31 에서 프레임 폭이 한/글의 39208
            // 대신 40808 이 되어 줄마다 두 글자를 더 먹었다.
            ParagraphBox::content_for_style(
                cell_inner_width,
                styles.para_styles.get(p.para_shape_id as usize),
                dpi,
            ),
            styles,
            dpi,
            false,
        );
        if comp.lines.len() > 1 {
            return false;
        }
        if let Some(l) = comp.lines.first() {
            if estimate_composed_line_width(l, styles) > cell_inner_width * 0.85 {
                return false;
            }
            em_sum += l
                .runs
                .iter()
                .map(|r| {
                    styles
                        .char_styles
                        .get(r.char_style_id as usize)
                        .map(|cs| cs.font_size)
                        .unwrap_or(0.0)
                })
                .fold(0.0f64, f64::max);
        }
    }
    cell_inner_height >= em_sum
}

/// [#2291/#2287] 부실 저장 예외 — 기계생성 문서는 다줄 문단에도 저장 lineseg 를
/// 1개만 남기는 관례가 있어(연결맵 s5 244×10 r183 c8: 76자 문단 ls 1개 → 1줄
/// 렌더 + "…실천 계획 세" 절단), 셀 재래핑의 "저장 lineseg 신뢰" 가드가 이런
/// 문단의 텍스트를 segment_width 클립으로 절단한다. 저장 ls==1 이고 그 줄의
/// 추정 실폭이 셀 내폭을 명백히 초과(×1.05)하면 저장을 불신하고 fresh
/// 재래핑한다. **가로쓰기 셀 전용** — 세로쓰기 셀은 글자를 세로로 쌓아 가로
/// 실폭 판정이 무의미하므로 호출부(셀 방향을 아는 곳)에서 걸러야 한다
/// (task81 세로쓰기 회귀 실측). 정상 1줄(실폭 ≤ 내폭)은 불변.
///
/// [#5952] `※`/`☞` 유의사항 bullet의 저장 2~3줄이 한 줄로 합성되었고 각
/// `segment_width`가 셀 내폭과 같지만 합성 행이 ×1.10을 넘으면 분할을 복원한다.
/// [#6389] 이미 다중행인 결과에는 개입하지 않는다. 대체 글꼴의 추정 폭 차이를
/// 저장 줄 붕괴로 오인하면 정상 줄 경계를 파괴한다. 저장 정보의 유효성 판단은
/// 선행 프레임 경로, 보존한 줄의 폭 조정은 paragraph layout의 공통 경로가 맡는다.
pub fn recompose_stored_single_line_if_overflowing(
    composed: &mut ComposedParagraph,
    para: &Paragraph,
    cell_inner_width_px: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
) {
    recompose_stored_single_line_if_overflowing_cached(
        composed,
        para,
        cell_inner_width_px,
        styles,
        dpi,
        None,
    );
}

fn recompose_stored_single_line_if_overflowing_cached(
    composed: &mut ComposedParagraph,
    para: &Paragraph,
    cell_inner_width_px: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
    cache: Option<&SingleLineOverflowCache>,
) {
    match recompose_stored_single_line_if_overflowing_cached_with_storage(
        composed,
        para,
        cell_inner_width_px,
        styles,
        dpi,
        cache,
        &crate::storage::Unbounded,
    ) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}

pub(crate) fn recompose_stored_single_line_if_overflowing_cached_with_storage<
    S: crate::storage::Storage,
>(
    composed: &mut ComposedParagraph,
    para: &Paragraph,
    cell_inner_width_px: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
    cache: Option<&SingleLineOverflowCache>,
    storage: &S,
) -> Result<(), S::Error> {
    {
        if composed.lines.len() != 1 || cell_inner_width_px <= 0.0 {
            return Ok(());
        }
        let authentic_stored = !para.line_segs.is_empty()
            && para
                .line_segs
                .iter()
                .all(|seg| seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0);
        if authentic_stored && para.line_segs.len() >= 2 {
            let note_bullet = matches!(para.text.trim_start().chars().next(), Some('※' | '☞'));
            let short_note_cell = note_bullet
                && (2..=3).contains(&para.line_segs.len())
                && para.line_segs.iter().all(|seg| {
                    seg.segment_width > 0
                        && (crate::renderer::hwpunit_to_px(seg.segment_width, dpi)
                            - cell_inner_width_px)
                            .abs()
                            <= 1.0
                });
            if !short_note_cell {
                return Ok(());
            }
            let mut over = false;
            for line in &composed.lines {
                if estimate_composed_line_width_with_storage(line, styles, storage)?
                    > cell_inner_width_px * 1.10
                {
                    over = true;
                    break;
                }
            }
            if over {
                reflow_cell_line_ignoring_stored_segs_with_storage(
                    composed,
                    para,
                    cell_inner_width_px,
                    styles,
                    dpi,
                    storage,
                )?;
            }
            return Ok(());
        }
        let stored_single = para.line_segs.len() == 1 && authentic_stored;
        if !stored_single {
            return Ok(());
        }
        // [#2430] 발동 임계 ×1.05 는 측정(원패딩) vs 렌더(shrink패딩) 폭 발산(#2237)
        // 으로 살짝(1.05~1.35×) 초과한 정합 셀까지 거짓 재래핑해 줄수를 부풀리고
        // 쪽당 표 행 적재를 떨어뜨렸다(분할표 11건 과다분할 회귀). 본문 판
        // #2525 와 동일하게 ×1.8 로 좁혀 정당한 장평/자간·
        // 패딩 발산 범위(≤~1.5×)를 넘는 부실 저장만 재래핑한다. #2291 원 타깃
        // (76자 1-lineseg = ~7.6× 초과, 절단 해소)은 임계 위라 계속 재래핑.
        //
        // [#4149] 판정 memo — 같은 source paragraph와 셀 내폭이면 판정이 결정적
        // 인데, 페이지 트리 재빌드마다 estimate_composed_line_width 재측정이 반복돼
        // 거대 셀 문서의 캐럿 rect 질의당 ~30% 를 차지했다. 폭 키(f32 bits 패킹)로
        // 판정만 memo 하고(측정 생략), over=true 의 fresh 재래핑 자체는 매 빌드 그대로
        // 수행한다 — 재래핑 결과는 composed 에만 반영되고 저장 line_segs 는 안 바뀌므로
        // 재래핑을 생략하면 절단 렌더 회귀. cache는 renderer session이 소유하고
        // source/style mutation과 함께 session cache 전체를 비운다.
        let width_key = (cell_inner_width_px as f32).to_bits();
        let over = if let Some(value) = cache.and_then(|cache| cache.get(para, width_key)) {
            value
        } else {
            let measured = if let Some(line) = composed.lines.first() {
                estimate_composed_line_width_with_storage(line, styles, storage)?
                    > cell_inner_width_px * 1.8
            } else {
                false
            };
            if let Some(cache) = cache {
                cache.insert_with_storage(para, width_key, measured, storage)?;
            }
            measured
        };
        if std::env::var("RHWP_DIAG_CELLREWRAP").is_ok() && over {
            if let Some(l) = composed.lines.first() {
                for run in &l.runs {
                    let ts = run.try_text_style(styles, storage)?;
                    eprintln!(
                    "DIAG_CELLREWRAP inner={:.1} fs={:.1} lsp={:.2} font={:?} w={:.1} text={:?}",
                    cell_inner_width_px,
                    ts.font_size,
                    ts.letter_spacing,
                    ts.font_family.split(',').next().unwrap_or(""),
                    estimate_text_width(effective_text_for_metrics(run), &ts),
                    storage.chars(effective_text_for_metrics(run).chars().take(10))?,
                );
                }
            }
        }
        if !over {
            return Ok(());
        }
        // [#6802] 넘친 것이 점 채움(리더)뿐이면 저장 한 줄이 옳다 — 한/글도 자른다.
        if if let Some(line) = composed.lines.first() {
            line_overflow_is_leader_fill_with_storage(line, styles, cell_inner_width_px, storage)?
        } else {
            false
        } {
            return Ok(());
        }
        reflow_cell_line_ignoring_stored_segs_with_storage(
            composed,
            para,
            cell_inner_width_px,
            styles,
            dpi,
            storage,
        )?;
    };
    Ok(())
}

/// 차례 점 채움(리더)에 쓰이는 문자. 마침표·가운뎃점 계열만 본다.
fn is_leader_char(c: char) -> bool {
    matches!(
        c,
        '.' | '\u{00B7}' | '\u{2024}' | '\u{2025}' | '\u{2026}' | '\u{2027}' | '\u{22EF}'
    )
}

/// 채움으로 인정하는 최소 연속 길이 — 문장의 마침표·말줄임표를 채움으로 오인하지 않는다.
const MIN_LEADER_RUN: usize = 4;

/// 판정과 출력이 같은 문자 구간을 사용한다. 인덱스는 Unicode scalar 기준이다.
fn leader_fill_spans(chars: &[char]) -> Vec<std::ops::Range<usize>> {
    match leader_fill_spans_with_storage(chars, &crate::storage::Unbounded) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}

pub(crate) fn leader_fill_spans_with_storage<S: crate::storage::Storage>(
    chars: &[char],
    storage: &S,
) -> Result<Vec<std::ops::Range<usize>>, S::Error> {
    Ok({
        let mut spans = Vec::new();
        let mut start = 0;
        while start < chars.len() {
            if !is_leader_char(chars[start]) {
                start += 1;
                continue;
            }
            let mut end = start + 1;
            while end < chars.len() && is_leader_char(chars[end]) {
                end += 1;
            }
            if end - start >= MIN_LEADER_RUN {
                storage.push(&mut spans, start..end)?;
            }
            start = end;
        }
        spans
    })
}

/// [#6802] 저장 한 줄을 지킨 뒤, 상자를 넘는 **채움 글자만** 한/글처럼 잘라 낸다.
///
/// 한/글 2020 정본(`1400000-200600006` 2쪽)은 차례 줄을 한 줄로 두고 점을 상자 안에서
/// 끊는다 — 점이 쪽 번호 칸이나 용지 밖으로 이어지지 않는다. 자르는 것은 `display_text`
/// 뿐이라 `text`(문서 좌표·추출·편집)는 그대로 남는다. 채움이 아닌 글자는 건드리지 않는다.
fn trim_leader_fill_overflow(
    composed: &mut ComposedParagraph,
    inner_width_px: f64,
    styles: &ResolvedStyleSet,
) {
    match trim_leader_fill_overflow_with_storage(
        composed,
        inner_width_px,
        styles,
        &crate::storage::Unbounded,
    ) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}

pub(crate) fn trim_leader_fill_overflow_with_storage<S: crate::storage::Storage>(
    composed: &mut ComposedParagraph,
    inner_width_px: f64,
    styles: &ResolvedStyleSet,
    storage: &S,
) -> Result<(), S::Error> {
    let _: () = {
        if inner_width_px <= 0.0 {
            return Ok(());
        }
        for line in &mut composed.lines {
            let mut width = estimate_composed_line_width_with_storage(line, styles, storage)?;
            if width <= inner_width_px
                || !line_overflow_is_leader_fill_with_storage(
                    line,
                    styles,
                    inner_width_px,
                    storage,
                )?
            {
                continue;
            }
            for run_idx in (0..line.runs.len()).rev() {
                if width <= inner_width_px {
                    break;
                }
                let style = line.runs[run_idx].try_text_style(styles, storage)?;
                let run = &mut line.runs[run_idx];
                let mut chars: Vec<char> =
                    storage.collect(effective_text_for_metrics(run).chars())?;
                // 뒤쪽부터 처리하면 앞쪽 구간의 인덱스는 변하지 않는다.
                for span in leader_fill_spans_with_storage(&chars, storage)?
                    .into_iter()
                    .rev()
                {
                    if width <= inner_width_px {
                        break;
                    }
                    let current: String = storage.chars(chars.iter().copied())?;
                    let current_width = estimate_text_width(&current, &style);
                    let without = |drop: usize| -> Result<String, S::Error> {
                        storage.chars(
                            chars[..span.end - drop]
                                .iter()
                                .chain(&chars[span.end..])
                                .copied(),
                        )
                    };
                    // 제목/쪽번호/다른 run은 그대로 두고 필요한 최소 채움만 줄인다.
                    // 서로 다른 리더 글리프나 자간도 전체 run 재측정으로 반영한다.
                    let mut low = 0;
                    let mut high = span.len();
                    while low < high {
                        let middle = (low + high) / 2;
                        let candidate_width = estimate_text_width(&without(middle)?, &style);
                        if width - current_width + candidate_width <= inner_width_px {
                            high = middle;
                        } else {
                            low = middle + 1;
                        }
                    }
                    let kept = without(low)?;
                    width += estimate_text_width(&kept, &style) - current_width;
                    chars.drain(span.end - low..span.end);
                    run.display_text = Some(kept);
                }
            }
        }
    };
    Ok(())
}

/// [#6802] 점 채움(리더)만 넘치는 저장 한 줄은 **부실 저장이 아니다**.
///
/// 옛 차례 표는 제목 뒤를 점 문자로 직접 채워 한 줄을 만든다
/// (`1400000-200600006` 2쪽: `Ⅰ. 사업개요 ․․․․․…`, `․` = U+2024). 그 점 개수는
/// **한/글의 메트릭으로** 줄을 꽉 채우도록 정해져 있어서, 우리 추정 폭이 조금만 넓어도
/// 저장 한 줄이 셀 폭을 크게 넘는 것처럼 보인다. 그때 `#2291` 의 부실 저장 재래핑이
/// 발동하면 한 줄짜리 문단이 네 줄로 접히고, 뒤 문단들의 저장 `vertical_pos` 위에
/// 그대로 겹쳐 그려진다(그 문서: text-overlap 4건 · 표가 본문을 362.9px 초과).
///
/// 한/글은 그 줄을 한 줄로 두고 **넘치는 점을 자른다**. 채움 문자를 걷어낸 내용 폭이
/// 셀 안에 들어가면 저장 줄이 옳다고 보고 종전의 `segment_width` 클립에 맡긴다.
///
/// `#2291` 의 반례(`task2287` r183 c8: 점 없는 본문 76자가 저장 1줄)는 채움 런이 없어
/// 이 갈래를 타지 않는다 — 그쪽은 종전대로 재래핑한다.
pub(crate) fn line_overflow_is_leader_fill(
    line: &ComposedLine,
    styles: &ResolvedStyleSet,
    inner_width_px: f64,
) -> bool {
    match line_overflow_is_leader_fill_with_storage(
        line,
        styles,
        inner_width_px,
        &crate::storage::Unbounded,
    ) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}

pub(crate) fn line_overflow_is_leader_fill_with_storage<S: crate::storage::Storage>(
    line: &ComposedLine,
    styles: &ResolvedStyleSet,
    inner_width_px: f64,
    storage: &S,
) -> Result<bool, S::Error> {
    Ok({
        fn strip_leader_runs<S: crate::storage::Storage>(
            text: &str,
            storage: &S,
        ) -> Result<(String, bool), S::Error> {
            let mut chars = storage.collect(text.chars())?;
            let spans = leader_fill_spans_with_storage(&chars, storage)?;
            let found = !spans.is_empty();
            for span in spans.into_iter().rev() {
                chars.drain(span);
            }
            Ok((storage.chars(chars.iter().copied())?, found))
        }

        let mut found_leader = false;
        let mut content_width = 0.0f64;
        for run in &line.runs {
            let (kept, found) = strip_leader_runs(effective_text_for_metrics(run), storage)?;
            found_leader |= found;
            if !kept.is_empty() {
                content_width += estimate_text_width(&kept, &run.try_text_style(styles, storage)?);
            }
        }
        found_leader && content_width <= inner_width_px
    })
}

fn reflow_cell_line_ignoring_stored_segs(
    composed: &mut ComposedParagraph,
    para: &Paragraph,
    cell_inner_width_px: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
) {
    match reflow_cell_line_ignoring_stored_segs_with_storage(
        composed,
        para,
        cell_inner_width_px,
        styles,
        dpi,
        &crate::storage::Unbounded,
    ) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}

pub(crate) fn reflow_cell_line_ignoring_stored_segs_with_storage<S: crate::storage::Storage>(
    composed: &mut ComposedParagraph,
    para: &Paragraph,
    cell_inner_width_px: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
    storage: &S,
) -> Result<(), S::Error> {
    {
        // 저장 seg 를 일시적으로 무시하고 NO_LS 폴백과 동일 경로로 재분할한다.
        let mut para_no_ls = storage.clone_paragraph(para)?;
        para_no_ls.line_segs.clear();
        recompose_cell_lines_in_frame_with_storage(
            composed,
            &para_no_ls,
            // [#7422] 위와 같은 계약. 저장 seg 를 무시하는 경로도 문단 여백을 뺀다.
            ParagraphBox::content_for_style(
                cell_inner_width_px,
                styles.para_styles.get(para.para_shape_id as usize),
                dpi,
            ),
            styles,
            dpi,
            false,
            storage,
        )?;
    };
    Ok(())
}

/// 가로쓰기 셀의 렌더/측정 공통 재구성 경로.
///
/// `recompose_cell_lines_in_frame`만 적용하면 저장 다줄 문단이 실제로는 셀 폭을
/// 넘쳐 fresh 재래핑되는 경우(#5952)를 높이 계산이 놓칠 수 있다. 호출자는
/// 세로쓰기 셀을 이미 제외해야 한다.
pub(crate) fn recompose_horizontal_cell_lines_for_width(
    composed: &mut ComposedParagraph,
    para: &Paragraph,
    cell_inner_width_px: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
    stored_geometry_policy: impl Into<crate::model::provenance::StoredGeometryPolicy>,
    repair_stored_overflow: bool,
    overflow_cache: &SingleLineOverflowCache,
) {
    match recompose_horizontal_cell_lines_for_width_with_storage(
        composed,
        para,
        cell_inner_width_px,
        styles,
        dpi,
        stored_geometry_policy,
        repair_stored_overflow,
        overflow_cache,
        &crate::storage::Unbounded,
    ) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}

pub(crate) fn recompose_horizontal_cell_lines_for_width_with_storage<S: crate::storage::Storage>(
    composed: &mut ComposedParagraph,
    para: &Paragraph,
    cell_inner_width_px: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
    stored_geometry_policy: impl Into<crate::model::provenance::StoredGeometryPolicy>,
    repair_stored_overflow: bool,
    overflow_cache: &SingleLineOverflowCache,
    storage: &S,
) -> Result<(), S::Error> {
    let _: () = {
        recompose_cell_lines_in_frame_with_storage(
            composed,
            para,
            // [#7422] 위와 같은 계약. 렌더/측정 공통 경로도 문단 여백을 뺀다.
            ParagraphBox::content_for_style(
                cell_inner_width_px,
                styles.para_styles.get(para.para_shape_id as usize),
                dpi,
            ),
            styles,
            dpi,
            stored_geometry_policy,
            storage,
        )?;
        if repair_stored_overflow {
            recompose_stored_single_line_if_overflowing_cached_with_storage(
                composed,
                para,
                cell_inner_width_px,
                styles,
                dpi,
                Some(overflow_cache),
                storage,
            )?;
        }
    };
    Ok(())
}

/// [#2279] 저장 lineseg 분할의 실폭-과잉 판정 (본문 판, 줄수 무관).
///
/// 저장(비합성) 분할의 어떤 줄이든 추정 실폭이 단 내폭을 명백히(×1.05)
/// 초과하면 그 분할은 물리적으로 성립하지 않는 부실 저장이다 — 마스킹('*'
/// 치환) 결재문서는 원문 기준의 저장 분할을 남겨 실폭과 모순인 경우가 있고,
/// 한글은 항상 fresh 재계산하므로 더 많은 줄로 배치한다 (36392557 pi34
/// 실측: '*'×164 저장 2줄, 줄0 90자 ≈ 내폭 1.4× vs 한글 PDF 3줄 80/68/16).
/// [정밀화] 마스킹 문단('*' 비중 ≥ 50%) 한정 — 일반 텍스트 문단은 rhwp
/// 폭 추정 오차가 1.05×를 넘는 사례(prep_1790387/온새미로 실측 회귀)가
/// 있어 재래핑하지 않는다. 마스킹 치환은 원문과 글자폭이 달라지는 유일한
/// 물리적 근거가 있는 계열이다.
/// Whether stored rows are **stale** rather than merely unreproducible.
///
/// Mutation provenance is decisive: `Paragraph` marks its stored text
/// partition dirty whenever text or CharShapeRef inputs change. Geometry may
/// still match exactly after such an edit, but the old row boundaries cannot
/// be admitted. The width checks below remain a defensive import check for
/// malformed documents that arrived without local mutation provenance.
///
/// A stored row that cannot hold its own text is self-evidently wrong, and no
/// amount of authenticity rescues it: after a fill/replace the record still
/// describes the old text, and `#2525`'s `hwpx-02` p5 stores 135 characters on
/// one line statically. That is a different failure from a row we simply
/// cannot reproduce — missing 한양 metrics (#4779) — where the record is right
/// and our carve is wrong, and rebuilding makes things worse.
///
/// The test compares each row's measured text against **its own** stored
/// width, never against our carve, so it does not depend on reproducing HWP's
/// geometry and survives a wrong metric environment: a 4.5× overfill is not a
/// measurement disagreement.
///
/// One predicate, both consumers. `HeightMeasurer::measure_paragraph` passes a
/// composition **with runs**; the typeset path that reaches the frame can pass
/// one whose lines have `runs.len() == 0`, and `estimate_composed_line_width`
/// over no runs is ~0, so the 1.8× test cannot fire there. Splitting this into
/// two copies once disarmed the measurer silently while looking correct on the
/// frame side — the run-less compositions are empty paragraphs, which have no
/// text to overflow, so the asymmetry is invisible until a real overfull row
/// arrives at the wrong copy.
///
/// [#2525] 비마스킹 대형 과밀: 저장 lineseg 이 장평 반영 실폭
/// (`estimate_composed_line_width` 는 ts.ratio 를 자체 반영) 기준으로도 내폭을
/// 크게(≥1.8×) 초과하면, 정당한 장평/자간 압축 범위(최소 advance 클램프 0.5×
/// → 최대 ~2× 과밀)를 벗어난 부실 단일-저장 lineseg 다 (hwpx-02 p5: 135자 1줄
/// ≈4.5× 과밀). **이 경계는 추정이 아니라 압축 상한에서 나온 값이다** — 정당한
/// 장평 압축 문서는 ratio 반영 실폭이 내폭 이내라 오발동하지 않는다.
pub(crate) fn stored_rows_are_stale(
    composed: &ComposedParagraph,
    para: &Paragraph,
    inner_width_px: f64,
    styles: &ResolvedStyleSet,
) -> bool {
    match stored_rows_are_stale_with_storage(
        composed,
        para,
        inner_width_px,
        styles,
        &crate::storage::Unbounded,
    ) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}

pub(crate) fn stored_rows_are_stale_with_storage<S: crate::storage::Storage>(
    composed: &ComposedParagraph,
    para: &Paragraph,
    inner_width_px: f64,
    styles: &ResolvedStyleSet,
    storage: &S,
) -> Result<bool, S::Error> {
    Ok({
        if !para.line_segs.is_empty() && para.stored_text_partition_is_dirty() {
            return Ok(true);
        }
        let stored = !para.line_segs.is_empty()
            && para
                .line_segs
                .iter()
                .all(|seg| seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0);
        if !stored
            || composed.lines.is_empty()
            || inner_width_px <= 0.0
            || composed.lines.len() != para.line_segs.len()
        {
            return Ok(false);
        }
        // [#2525] 비마스킹 대형 과밀: 저장 lineseg 이 장평 반영 실폭
        // (estimate_composed_line_width 는 ts.ratio 를 자체 반영) 기준으로도 내폭을
        // 크게(≥1.8×) 초과하면, 정당한 장평/자간 압축 범위(최소 advance 클램프 0.5×
        // → 최대 ~2× 과밀)를 벗어난 부실 단일-저장 lineseg 다 (hwpx-02 p5: 135자
        // 1줄 ≈4.5× 과밀). **이 경계는 압축 상한에서 나온 값이지 맞춰 넣은 상수가
        // 아니다.** 빈 문단은 run 이 없어 이 판정이 발화하지 않는데, 폭을 넘길 텍스트
        // 자체가 없으므로 정상이다(전체 run-less 조합 8,345건 중 99.6%가 빈 문단).
        // [#6802] 넘친 것이 **점 채움(리더)** 뿐인 줄은 부실 저장이 아니다 — 옛 차례 표는
        // 제목 뒤를 점 문자로 직접 채워 한 줄을 만들고(`1400000-200600006` 2쪽:
        // `Ⅰ. 사업개요 ․․․․․…`), 그 점 개수는 한/글 메트릭 기준이라 우리 추정 폭에서는
        // 쉽게 1.8× 를 넘는다. 여기서 재래핑하면 저장이 한 줄로 둔 문단이 네 줄로 접혀
        // 뒤 문단들의 저장 `vertical_pos` 위에 겹쳐 그려진다(그 문서 2쪽 text-overlap 4건).
        // 한/글은 그 줄을 한 줄로 두고 넘치는 점을 자른다. 채움을 걷어낸 내용이 상자 안에
        // 들어갈 때만 저장을 믿는다 — `#2525`(hwpx-02 p5 135자)·`#2291`(task2287 r183c8
        // 76자)처럼 채움 없이 넘치는 줄은 종전대로 부실 저장이다.
        for l in &composed.lines {
            if estimate_composed_line_width_with_storage(l, styles, storage)? > inner_width_px * 1.8
                && !line_overflow_is_leader_fill_with_storage(l, styles, inner_width_px, storage)?
            {
                return Ok(true);
            }
        }
        if stored_rows_underfill_inside_token_with_storage(
            composed,
            para,
            inner_width_px,
            styles,
            storage,
        )? {
            return Ok(true);
        }
        // [#6102] **비말미** 저장 줄의 과밀: 이어지는 줄이 있는데도 자기 폭(저장
        // segment_width 와 내폭 둘 다)을 넘는 텍스트를 담았다고 주장하는 줄은
        // 물리적으로 성립하지 않는다 — 한글은 줄을 다 채우기 **전에** 끊는다.
        // 결재문서본문 계열(36360328 외 2건)의 저장 textpos 축이 파서 보정폭과
        // 어긋나 첫 줄이 6자 늦게 끊기고 본문 우단을 71~99px 넘던 결함의 관측식.
        // 0.5× advance 클램프 마스킹 코호트(#2525)는 **단일 줄**이라 비말미 조건에
        // 걸리지 않고, 말미 줄은 초과분이 다음 줄로 갈 수 없으므로 제외한다.
        // 여유는 6% + 12px — 정당한 Justify 줄은 말미 공백 overhang(≤반각 한 칸)
        // 까지 포함해도 이 밑에 있다.
        //
        // **범위는 자리차지 표 host 문단 한정** — 종전에 프레임 소유자가 아예 없던
        // 계보라 이 판정에 기대던 기존 핀이 없다. 일반 문단까지 넓히면 확정된
        // 쪽수 핀 5건(#2006/#3930/#3931/#2559/#5801)이 흔들린다(전량 게이트 실측).
        if !para.controls.iter().any(|c| {
            matches!(c, crate::model::control::Control::Table(t)
            if !t.common.treat_as_char
                && matches!(t.common.text_wrap, crate::model::shape::TextWrap::TopAndBottom))
        }) {
            return Ok(false);
        }
        let non_last_overfull = |line: &ComposedLine, seg: &crate::model::paragraph::LineSeg| {
            let est = estimate_composed_line_width(line, styles);
            let seg_width_px = crate::renderer::hwpunit_to_px(seg.segment_width, 96.0);
            est > inner_width_px * 1.06 + 12.0 && est > seg_width_px * 1.06 + 12.0
        };
        composed
            .lines
            .iter()
            .zip(para.line_segs.iter())
            .take(composed.lines.len().saturating_sub(1))
            .any(|(line, seg)| non_last_overfull(line, seg))
    })
}

/// [#7702] 저장 줄의 마지막 줄이 **한/글이 다시 나눠 앞 줄에 붙일 수 있는** 낱말 꼬리인가.
///
/// 한/글이 그 문단을 저장 줄 대신 새로 나누는 경우만 본다.
/// - condense 가 있는 문단: 한/글은 공백을 condense% 까지 줄여 이미 시작한 낱말의 글자를
///   앞 줄에 더 담는다(#7418, 한/글 2024 합성 문단 본문 208/208·칸 211/211 재현).
/// - [#8105] 문단 머리(번호·글머리표)를 그리는 문단: 한/글 2024 는 폭이 맞는 저장 줄도 쓰지 않고
///   다시 나눈다(«-»·«^1.» 번호 모두). 번호 없는 문단은 폭이 맞으면 저장 줄을 따른다
///   (`probes/make_numbered_stale_tail.py`). 1342000(한/글 2014 저장) 의 한/글 2020 Print 도
///   번호 문단 꼬리 39건을 붙이고 번호 없는 `…제|정]`·`…일|부개정]` 은 두 줄로 둔다.
///
/// 이 함수는 **모양**만 본다 — 늘려 채우는 양쪽 정렬 문단에서, 마지막 저장 줄이 앞 줄에서
/// 시작한 낱말의 나머지(공백 없음, 앞 줄 끝과 이 줄 첫 글자가 모두 한글 음절)뿐인가.
/// 실제로 붙는지는 호출자가 프레임의 새 줄 나눔으로 정한다(측정 근사를 따로 두지 않는다).
fn stored_last_row_is_rejoinable_word_tail(
    composed: &ComposedParagraph,
    para: &Paragraph,
    styles: &ResolvedStyleSet,
) -> bool {
    match stored_last_row_is_rejoinable_word_tail_with_storage(
        composed,
        para,
        styles,
        &crate::storage::Unbounded,
    ) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}

pub(crate) fn stored_last_row_is_rejoinable_word_tail_with_storage<S: crate::storage::Storage>(
    composed: &ComposedParagraph,
    para: &Paragraph,
    styles: &ResolvedStyleSet,
    storage: &S,
) -> Result<bool, S::Error> {
    Ok({
        use crate::model::style::Alignment;
        let n = composed.lines.len();
        if n < 2 || n != para.line_segs.len() {
            return Ok(false);
        }
        let Some(para_style) = styles.para_styles.get(para.para_shape_id as usize) else {
            return Ok(false);
        };
        let recomposed_by_hancom = para_style.condense_min_space != 0
            || para_style.head_type != crate::model::style::HeadType::None;
        if para_style.alignment != Alignment::Justify || !recomposed_by_hancom {
            return Ok(false);
        }
        let (prev, last) = (&composed.lines[n - 2], &composed.lines[n - 1]);
        if prev.has_line_break {
            return Ok(false);
        }
        let is_syllable = |c: char| ('\u{ac00}'..='\u{d7a3}').contains(&c);
        let prev_text: String = storage.chars(prev.runs.iter().flat_map(|run| run.text.chars()))?;
        let tail_text: String = storage.chars(last.runs.iter().flat_map(|run| run.text.chars()))?;
        let tail = tail_text.trim_end();
        !tail.is_empty()
            && !tail.chars().any(char::is_whitespace)
            && prev_text.chars().last().is_some_and(is_syllable)
            && tail.chars().next().is_some_and(is_syllable)
    })
}

/// [#7416] 저장 줄 사다리가 **제 선언 폭과 모순되는** 과소충전.
///
/// `#6102` 의 과밀(줄이 제 폭보다 많이 담았다)과 대칭인 쪽이다. 비말미 저장 줄이
///
/// 1. **어절 안**에서 끊겼고(앞 줄 끝 글자와 다음 줄 첫 글자가 모두 한글 음절),
/// 2. 줄에 **공백이 없고**,
/// 3. 문단이 **늘려 채우는 정렬**(양쪽·배분·나눔)이 아니며,
/// 4. 그 줄의 측정 폭 뒤에 **평균 글자 두 개 이상**이 더 들어갈 자리가 선언
///    `segment_width`(문단 들여쓰기/내어쓰기를 줄별로 뺀 값)에 남아 있으면,
///
/// 단, 선언 폭이 그 줄이 실제로 놓인 상자(`inner_width_px`)보다 넓으면 판정하지 않는다.
/// 그때 선언은 줄 나눔이 쓴 폭이 아니라서(좁은 칸 안의 줄에 쪽 폭을 적은 사다리)
/// «제 폭에 비해 남겼다» 는 비교 자체가 성립하지 않는다.
///
/// 그 사다리는 «다음 글자가 안 들어가서 끊었다» 는 줄 나눔의 유일한 이유와
/// 모순된다. 어절 줄바꿈이 아니므로 줄 나눔 규칙이 끊을 까닭이 없고, 늘릴 공백도
/// 정렬도 없으므로 자연 폭이 곧 배치 폭이다.
///
/// 비교 대상은 **그 줄 자신의 선언 폭**이지 우리 carve 가 아니다 — 위 과밀 판정과
/// 같은 이유로 우리 상자 재현에 기대지 않는다.
///
/// 문턱 2.0 은 맞춘 값이 아니라 `samples/**` 1,148건 전수에서 난 틈이다
/// (2026-09-25, `#7416` 코멘트). 조건 1~3 을 만족하는 한/글 저장본(`lastSavedWith`
/// 가 한컴인 1,044건)의 줄 316개는 여유가 최대 1.94글자이고, 2.0 이상은 한 줄도 없다.
/// 2.0 이상인 문서는 4건뿐이고 전부 저장 계보가 없다. 같은 원본을 한/글이 다시 저장한
/// 쌍둥이(`issue6639-hancom-160.hwpx`)는 같은 `horzsize` 로 포화(0.7글자 남김)라
/// 한 줄도 걸리지 않는다. 한/글은 이 원본의 사다리를 쓰지 않는다 — 사다리를 지운
/// 입력과 원본의 한/글 PDF 가 화소 단위로 같다.
///
/// 양쪽 정렬을 빼는 이유: 공백을 늘려 줄을 선언 폭까지 채우므로 자연 폭 비교가
/// 성립하지 않는다(종전 census 발화 줄의 99.8% 가 양쪽 정렬이었다).
fn stored_rows_underfill_inside_token(
    composed: &ComposedParagraph,
    para: &Paragraph,
    inner_width_px: f64,
    styles: &ResolvedStyleSet,
) -> bool {
    match stored_rows_underfill_inside_token_with_storage(
        composed,
        para,
        inner_width_px,
        styles,
        &crate::storage::Unbounded,
    ) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}

pub(crate) fn stored_rows_underfill_inside_token_with_storage<S: crate::storage::Storage>(
    composed: &ComposedParagraph,
    para: &Paragraph,
    inner_width_px: f64,
    styles: &ResolvedStyleSet,
    storage: &S,
) -> Result<bool, S::Error> {
    Ok({
        use crate::model::style::Alignment;
        const MIN_SPARE_CHARS: f64 = 2.0;

        if composed.lines.len() < 2 {
            return Ok(false);
        }
        let Some(para_style) = styles.para_styles.get(para.para_shape_id as usize) else {
            return Ok(false);
        };
        if matches!(
            para_style.alignment,
            Alignment::Justify | Alignment::Distribute | Alignment::Split
        ) {
            return Ok(false);
        }
        let is_syllable = |c: char| ('\u{ac00}'..='\u{d7a3}').contains(&c);
        let line_text =
            |line: &ComposedLine| storage.chars(line.runs.iter().flat_map(|run| run.text.chars()));
        // 글자 폭에 잡히지 않는 개체·각주·탭이 줄 안에 있으면 자연 폭을 잴 수 없다.
        let has_opaque_inline = |start: usize, end: usize| {
            composed
                .tac_controls
                .iter()
                .any(|&(pos, _, _)| (start..end).contains(&pos))
                || composed
                    .footnote_positions
                    .iter()
                    .any(|&(pos, _, _)| (start..end).contains(&pos))
        };
        let first_line_indent = para_style.indent.max(0.0);
        let hanging_indent = (-para_style.indent).max(0.0);

        for (idx, (pair, seg)) in composed
            .lines
            .windows(2)
            .zip(para.line_segs.iter())
            .enumerate()
        {
            let (line, next) = (&pair[0], &pair[1]);
            if line.has_line_break {
                continue;
            }
            let text = line_text(line)?;
            let next_text = line_text(next)?;
            let nchars = text.chars().count();
            if nchars == 0
                || text.chars().any(|c| c.is_whitespace())
                || !text.chars().last().is_some_and(is_syllable)
                || !next_text.chars().next().is_some_and(is_syllable)
                || has_opaque_inline(line.char_start, next.char_start)
            {
                continue;
            }
            let used = estimate_composed_line_width_with_storage(line, styles, storage)?;
            if used <= 0.0 {
                continue;
            }
            let indent = if idx == 0 {
                first_line_indent
            } else {
                hanging_indent
            };
            let seg_width_px = hwpunit_to_px(seg.segment_width, 96.0);
            // 선언 폭이 그 줄이 실제로 놓인 상자보다 넓으면 «제 폭» 이라는 전제가 서지
            // 않는다 — 그 값은 줄 나눔이 쓴 폭이 아니다(좁은 칸 안의 줄에 쪽 폭을 적은
            // 사다리가 있다). 상자와 어긋난 선언은 이 판정의 대상이 아니다.
            if seg_width_px > inner_width_px + 1.0 {
                continue;
            }
            let declared = seg_width_px - indent;
            let avg_char = used / nchars as f64;
            if declared - used >= avg_char * MIN_SPARE_CHARS {
                return Ok(true);
            }
        }
        false
    })
}

/// Resolve a paragraph's rows through the physical frame its own geometry
/// produces — on the stored route as well as the fresh one.
///
/// The caller states its coordinate system by handing a [`ParagraphBox`], and
/// that is what lets one owner serve both flows: a body paragraph arrives on
/// [`ParagraphBox::body`], the same expression the edit path builds in
/// `DocumentCore::reflow_paragraph`, and a cell paragraph arrives on
/// [`ParagraphBox::content_width_px`] via [`recompose_cell_lines_in_frame`]. One
/// paragraph must not get two boxes depending on which route reached it, and
/// sharing the constructors is what keeps that true — the body pair used to be
/// two copies of the same expression, and the cell had no box at all.
///
/// The frame models the column box and its declared exclusions and nothing
/// else — a float anchored in a different paragraph is not in `para.controls`
/// and so never reaches its exclusion list (see the exclusion-provenance note
/// in the report for #4755's deferred scope). Paragraphs whose controls have
/// their own layout owner keep the established #2279 owner below.
pub(crate) fn recompose_stored_lines_in_frame(
    composed: &ComposedParagraph,
    para: &Paragraph,
    paragraph_box: ParagraphBox,
    inner_width_px: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
    stored_geometry_policy: impl Into<crate::model::provenance::StoredGeometryPolicy>,
    miss_policy: line_breaking::StoredRowMissPolicy,
    // [#6175] 같은 세로 band의 용지/쪽 기준 어울림 개체 증거(HWPUNIT) — 저장 행이
    // 남긴 결손 폭과 위치를 함께 설명할 때만 재래핑하지 않는다. 측정·페인트가 같은
    // 함수를 타므로 두 경로가 갈리지 않는다.
    float_carve_evidence: &[crate::renderer::float_placement::FloatCarveEvidence],
) -> Option<ComposedParagraph> {
    match recompose_stored_lines_in_frame_with_storage(
        composed,
        para,
        paragraph_box,
        inner_width_px,
        styles,
        dpi,
        stored_geometry_policy,
        miss_policy,
        float_carve_evidence,
        &crate::storage::Unbounded,
    ) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}

pub(crate) fn recompose_stored_lines_in_frame_with_storage<S: crate::storage::Storage>(
    composed: &ComposedParagraph,
    para: &Paragraph,
    paragraph_box: ParagraphBox,
    inner_width_px: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
    stored_geometry_policy: impl Into<crate::model::provenance::StoredGeometryPolicy>,
    miss_policy: line_breaking::StoredRowMissPolicy,
    // [#6175] 같은 세로 band의 용지/쪽 기준 어울림 개체 증거(HWPUNIT) — 저장 행이
    // 남긴 결손 폭과 위치를 함께 설명할 때만 재래핑하지 않는다. 측정·페인트가 같은
    // 함수를 타므로 두 경로가 갈리지 않는다.
    float_carve_evidence: &[crate::renderer::float_placement::FloatCarveEvidence],
    storage: &S,
) -> Result<Option<ComposedParagraph>, S::Error> {
    Ok({
        recompose_stored_lines_in_frame_with_known_square_band_with_storage(
            composed,
            para,
            paragraph_box,
            inner_width_px,
            styles,
            dpi,
            stored_geometry_policy,
            miss_policy,
            float_carve_evidence,
            false,
            false,
            false,
            &[],
            storage,
        )?
    })
}
