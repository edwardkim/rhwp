/// [Task #555] 폰트 매트릭스 (글자폭/줄간격) 계산용 effective text 반환.
///
/// PUA 옛한글 변환 (Task #528) 후 `run.display_text` 가 자모 시퀀스를 보유하면
/// 본 함수는 그 자모 시퀀스를 반환한다. 그렇지 않으면 `run.text` (PUA char 1글자
/// 또는 일반 텍스트) 를 그대로 반환.
///
/// 사용처: `estimate_text_width` / `estimate_composed_line_width` 등 폰트 매트릭스
/// 측정 함수의 caller. visual 출력 (svg/web_canvas) 은 이미 `display_text` 사용.
///
/// 단일 룰 (분기/허용오차 없음): 비-PUA 텍스트는 fallback 으로 동일 동작.
pub fn effective_text_for_metrics(run: &ComposedTextRun) -> &str {
    // [#7017] U+F081C 는 `expand_pua_display_text` 가 **지우는**(continue) 글자라,
    // `display_text` 로 측정하면 글자 수가 줄어 폭이 모자란다. 원문을 유지해 글자
    // 수를 보존한다 — 폭은 `text_measurement` 가 다른 `hancom_pua` 괘선 조각과 같이
    // 폴백 0.5em 으로 잰다(렌더가 `┈` 를 그리는 전진폭과 같다).
    //
    // 종전 주석은 "0폭 규칙을 우회하지 않으려고" 라고 적었는데, 그 0폭 규칙 자체가
    // #7017 에서 한/글 정본과 어긋남이 확인돼 사라졌다.
    if run.text.contains('\u{F081C}') {
        return &run.text;
    }
    display_projection_metric_text(&run.text, run.display_text.as_deref().unwrap_or(&run.text))
}

/// 일대일 표시 투영은 source 프로그램의 메트릭을 유지한다. 필드 값·옛한글
/// 확장처럼 별도 표시 문자열인 경우는 표시 문자열 자체의 폭을 사용한다.
pub(crate) fn display_projection_metric_text<'a>(source: &'a str, display: &'a str) -> &'a str {
    if source
        .chars()
        .map(single_scalar_display_char)
        .eq(display.chars())
    {
        source
    } else {
        display
    }
}

/// PUA Supplementary 영역(U+F0000~) 문자가 테두리 숫자인지 판별한다.
///
/// HWP 특수문자표에서 표준 Unicode가 없는 테두리 숫자를 PUA로 인코딩한다.
/// - U+F02B1~U+F02C4: map_pua_bullet_char 에서 ①~⑳ 으로 매핑 (CharOverlap 제외)
/// - U+F02CE~U+F02E1: 반전 사각형 안의 숫자 1~20 (border_type=4)
///
/// 반환: Some(border_type) 또는 None
/// PUA 문자 자체는 변환하지 않고, 렌더러(draw_char_overlap)에서 표시 문자열로 변환한다.
/// 이렇게 하면 PUA 문자가 항상 1글자로 유지되어 font_size 기반 폭 계산이 정확하다.
/// PUA 글자겹침용 숫자 컴포넌트 디코딩
///
/// HWP tcps 컨트롤의 2~3자리 숫자는 자릿수별 PUA 코드포인트로 저장된다.
/// 각 PUA 문자를 (자릿수_그룹, 숫자값) 쌍으로 디코딩한다.
///
/// 2자리 블록 (U+F0288 base):
///   십의자리: F0289~F0291 (1-9)
///   일의자리: F0292~F029B (0-9)
///
/// 3자리 블록 (U+F0490 base):
///   백의자리: F0491~F0499 (1-9)
///   십의자리: F049A~F04A3 (0-9)
///   일의자리: F04A4~F04AD (0-9)
fn pua_overlap_digit(ch: char) -> Option<(u8, u8)> {
    let cp = ch as u32;
    // 2자리 블록
    if (0xF0289..=0xF0291).contains(&cp) {
        return Some((0, (cp - 0xF0288) as u8));
    } // tens 1-9
    if (0xF0292..=0xF029B).contains(&cp) {
        return Some((1, (cp - 0xF0292) as u8));
    } // ones 0-9
      // 3자리 블록
    if (0xF0491..=0xF0499).contains(&cp) {
        return Some((0, (cp - 0xF0490) as u8));
    } // hundreds 1-9
    if (0xF049A..=0xF04A3).contains(&cp) {
        return Some((1, (cp - 0xF049A) as u8));
    } // tens 0-9
    if (0xF04A4..=0xF04AD).contains(&cp) {
        return Some((2, (cp - 0xF04A4) as u8));
    } // ones 0-9
    None
}

/// CharOverlap의 PUA 문자 배열을 숫자 문자열로 디코딩한다.
///
/// 모든 문자가 PUA 겹침용 숫자인 경우에만 디코딩 성공 (Some).
/// 그룹 번호(0=최상위자리, 1=중간, 2=최하위)로 정렬하여 올바른 자릿수 순서를 보장한다.
pub fn decode_pua_overlap_number(chars: &[char]) -> Option<String> {
    if chars.is_empty() {
        return None;
    }
    let mut groups: Vec<(u8, u8)> = Vec::with_capacity(chars.len());
    for &ch in chars {
        groups.push(pua_overlap_digit(ch)?);
    }
    // 그룹 번호 순 정렬 (최상위 자리 → 최하위 자리)
    groups.sort_by_key(|(g, _)| *g);
    let s: String = groups.iter().map(|(_, d)| char::from(b'0' + d)).collect();
    Some(s)
}

/// CharOverlap controls occupy one text-flow position even when their payload
/// contains multiple glyph components.
///
/// Hancom uses this for overlapped two-digit markers such as the boxed 10/11/12
/// in `table-vpos-01.hwp`: the control stores two PUA glyph components, but
/// caret movement and line measurement advance by one character box.
pub fn char_overlap_advance_units(chars: &[char]) -> usize {
    usize::from(!chars.is_empty())
}

/// 글자겹침(CharOverlap) 내부 글자의 크기 비율.
///
/// `charSz` 는 OWPML 상 **"테두리 내부 글자의 크기 비율. 단위 %"**
/// (`mydocs/manual/OWPML SCHEMA/ParaList XML schema.xml:571`) 다. 따라서 테두리를
/// 그리지 않는 겹침에는 적용하지 않는다 — 축소할 "테두리 내부"가 없다.
///
/// `effective_border` 는 raw `border_type` 이 아니라 **실제로 테두리를 그리는지** 다.
/// PUA 다자리 숫자는 `border_type=0` 이어도 원형 테두리로 승격되므로(각 렌더 경로의
/// combined 분기) 그 경우는 축소가 정당하다.
///
/// 한컴 실측 두 건이 이 규칙을 함께 만족한다 (#4085):
/// - `samples/hwpx/k-water-rfp.hwpx` p13 — 반전 사각형(4), `charSz=-2` → 0.80 (PR #1101)
/// - 관세청 월간 수출입 현황 p1 — 테두리 없음(0), `charSz=-4` → 축소 없음. 한컴 PDF
///   content stream 에서 마커와 본문이 같은 `101 Tf`, 같은 baseline 으로 나온다.
///
/// 음수 영역의 10% step 해석은 PR #1101 의 실측 가설을 그대로 둔다.
pub fn char_overlap_size_ratio(effective_border: u8, inner_char_size: i8) -> f64 {
    if effective_border == 0 {
        return 1.0;
    }
    if inner_char_size > 0 {
        // 양수 → percent ratio (HWPX 양수 case 보존: 50 = 0.5)
        inner_char_size as f64 / 100.0
    } else if inner_char_size < 0 {
        // 음수 → 10% step 축소 (한컴 정합: charSz=-3 → 1.0 + (-3)×0.10 = 0.70)
        1.0 + inner_char_size as f64 * 0.10
    } else {
        1.0
    }
}

/// 글자겹침(CharOverlap) 한 글자의 **표시 문자열**을 정한다.
///
/// `border_drawn` 은 그 겹침을 그릴 때 렌더러가 **실제로 원/사각 테두리를 따로 그리는지**다.
///
/// - 테두리를 그리는 경우: 원문자 `①`~`⑳`(U+2460~U+2473)는 안쪽 숫자로 풀어 쓴다.
///   테두리를 그려 놓고 원문자 글리프까지 찍으면 동그라미가 이중으로 나온다.
/// - 테두리를 안 그리는 경우(`circleType="CHAR"` → `border_type=0`): `composeText` 를
///   **그대로** 그린다. 이때 숫자로 풀면 그릴 동그라미가 아무 데도 없어 `①` 이 맨 `1` 로
///   나간다 (#5790).
///
/// 한컴은 `circleType="CHAR"` + `composeText="①"` 을 전각 한 칸에 `①` 한 글자로 찍는다.
pub fn char_overlap_display_text(ch: char, border_drawn: bool) -> String {
    let cp = ch as u32;
    if border_drawn && (0x2460..=0x2473).contains(&cp) {
        return (cp - 0x2460 + 1).to_string();
    }
    if let Some(display) = pua_to_display_text(ch) {
        return display;
    }
    ch.to_string()
}

fn pua_enclosed_border_type(ch: char) -> Option<u8> {
    let cp = ch as u32;
    // U+F02B1~F02C4 (①~⑳): map_pua_bullet_char 에서 표준 원문자로 매핑 — CharOverlap 제외
    // 반전 사각형 안의 숫자: U+F02CE(1) ~ U+F02E1(20)
    if (0xF02CE..=0xF02E1).contains(&cp) {
        return Some(4); // border_type=4: 반전 사각형
    }
    None
}

fn pua_plain_text_display(ch: char) -> Option<&'static str> {
    super::hancom_pua::verified_hancom_pua_display(ch)
}

/// TAC 흐름에서만 채움 기호를 빈 표시로 분류한다. 실제 paint와 구분한다.
fn pua_flow_text_display(ch: char) -> Option<&'static str> {
    if ch == '\u{F081C}' {
        Some("")
    } else {
        pua_plain_text_display(ch)
    }
}

/// 원본 한 문자와 표시 한 문자의 대응만 replay 경계에 공유한다.
/// 여러 글자 확장·미등록 PUA의 기존 계약은 바꾸지 않는다.
fn verified_pua_replay_scalar(ch: char) -> Option<char> {
    let display = pua_plain_text_display(hancom_symbol_display_key(ch))?;
    let mut scalars = display.chars();
    let scalar = scalars.next()?;
    (scalars.next().is_none() && scalar != ch).then_some(scalar)
}

/// 원문 scalar 수를 유지하는 표시 투영만 문자 경계 조회에 사용한다.
pub(crate) fn one_to_one_pua_replay_text(text: &str) -> std::borrow::Cow<'_, str> {
    if !text
        .chars()
        .any(|ch| verified_pua_replay_scalar(ch).is_some())
    {
        return std::borrow::Cow::Borrowed(text);
    }
    std::borrow::Cow::Owned(
        text.chars()
            .map(|ch| verified_pua_replay_scalar(ch).unwrap_or(ch))
            .collect(),
    )
}

/// [#5800] HWP5 원시 한컴 사용자 기호를 **표시 매핑 조회용 평면-15 키**로 정규화한다.
///
/// 같은 글자를 HWP5 는 BMP 단일 유닛 `0xA000 | X` 로, HWPX 는 평면 15 보충 PUA
/// `U+F0000 | X` 로 싣는다(`parser::tags::HANCOM_SYMBOL_BMP_TO_PLANE15` — 한글 실측
/// 값 집합). 표시 매핑표(`hancom_pua` · `map_pua_bullet_char`)는 평면 15 키만 갖고
/// 있어 HWP5 경로가 표를 못 타고 원시 코드포인트를 그대로 그렸다 — `0xA832` 가
/// 유니코드 U+A832(실로티 나그리 `꠲`)로, `0xA12B` 가 `ꄫ` 로 나오는 식이다.
///
/// **정규화는 표에 값이 있을 때만 한다.** 값이 없으면 원문을 그대로 둬서, 미등록 PUA
/// 축(#5599)이 관측하는 표면을 이 변경이 흔들지 않게 한다.
pub(crate) fn hancom_symbol_display_key(ch: char) -> char {
    let Ok(unit) = u16::try_from(ch as u32) else {
        return ch;
    };
    let Some(plane15) =
        crate::parser::tags::hancom_symbol_to_plane15(unit).and_then(char::from_u32)
    else {
        return ch;
    };
    let has_mapping = pua_plain_text_display(plane15).is_some()
        || super::layout::map_pua_bullet_char(plane15) != plane15;
    if has_mapping {
        plane15
    } else {
        ch
    }
}

/// 이미 확정한 단일 표시 기호를 원문 폭 측정에도 전달한다.
/// 다문자 확장과 글자겹침 숫자는 각 구성 owner의 계약을 유지한다.
pub(crate) fn single_scalar_display_char(ch: char) -> char {
    let key = hancom_symbol_display_key(ch);
    if let Some(display) = pua_plain_text_display(key) {
        let mut chars = display.chars();
        if let Some(first) = chars.next() {
            if chars.next().is_none() {
                return first;
            }
        }
        return ch;
    }
    super::layout::map_pua_bullet_char(key)
}

/// 한글 방점(U+302E/U+302F)을 렌더용 spacing 가운데 점 글리프로 치환한다. (Task #1735)
///
/// U+302E/U+302F 는 유니코드 결합문자(combining mark)라, 유효한 base 없이
/// (줄 시작·공백 뒤) 셰이핑되면 브라우저/엔진이 dotted-circle(U+25CC)
/// placeholder 를 삽입하고 톤 점을 그 위에 쌓아 한컴과 다르게 표기된다.
/// 한컴은 방점을 독립 spacing 점으로 렌더하므로, 렌더 경로에서 결합 성질이
/// 없는 spacing 점 글리프로 치환한다. IR 텍스트는 불변(측정/캐럿/텍스트추출
/// 보존)이며, 측정 폭 정합은 text_measurement 의 전각 분류로 맞춘다.
fn tone_mark_display(ch: char) -> Option<char> {
    match ch {
        '\u{302E}' => Some('\u{00B7}'), // 방점 → · MIDDLE DOT
        '\u{302F}' => Some('\u{205A}'), // 쌍방점 → ⁚ TWO DOT PUNCTUATION (세로 두 점)
        _ => None,
    }
}

/// TAC 흐름 판정용 표시 문자열. 채움 기호가 흐름의 가시 본문이 되지 않는다.
///
/// 원문·측정 폭은 별도로 보존하며 실제 출력은 expand_pua_render_text를 쓴다.
///
/// Hanyang-PUA 옛한글은 KS X 1026-1:2007 자모 시퀀스로 확장한다.
///
/// CharOverlap 전용 숫자(`U+F02CE..=U+F02E1`)는 여기서 확장하지 않는다.
/// 해당 문자는 `pua_to_display_text()`가 글자겹침 렌더러에서만 처리한다.
pub fn expand_pua_display_text(text: &str) -> String {
    expand_pua_surface(text, None, true)
}

pub fn expand_pua_render_text_in_style(text: &str, style: &TextStyle) -> String {
    expand_pua_surface(text, style.native_pua_metrics.as_deref(), false)
}

fn expand_pua_surface(
    text: &str,
    program: Option<&super::native_pua_fonts::NativePuaMetrics>,
    flow: bool,
) -> String {
    use super::pua_oldhangul::map_pua_old_hangul;

    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        // [#5800] HWP5 원시 기호값을 먼저 평면-15 키로 정규화한다.
        let ch = hancom_symbol_display_key(ch);
        if program.is_some_and(|program| program.advance_em(ch).is_some()) {
            out.push(ch);
        } else if let Some(dot) = tone_mark_display(ch) {
            out.push(dot);
        } else if let Some(replacement) = if flow {
            pua_flow_text_display(ch)
        } else {
            pua_plain_text_display(ch)
        } {
            out.push_str(replacement);
        } else if let Some(jamos) = map_pua_old_hangul(ch) {
            out.extend(jamos.iter().copied());
        } else {
            out.push(super::layout::map_pua_bullet_char(ch));
        }
    }
    out
}

/// 실제 paint는 원본 점선 U+F081C도 표시한다. 흐름이 비어도 잉크는 남는다.
pub fn expand_pua_render_text(text: &str) -> String {
    expand_pua_surface(text, None, false)
}

/// PUA 테두리 숫자와 한컴 PUA 기호를 표시 문자열로 변환한다. (렌더러 전용)
///
/// draw_char_overlap()에서 호출하여, 실제 렌더링 시에만 변환한다.
pub fn pua_to_display_text(ch: char) -> Option<String> {
    // [#5800] HWP5 원시 기호값을 먼저 평면-15 키로 정규화한다.
    let ch = hancom_symbol_display_key(ch);
    let cp = ch as u32;
    if let Some(replacement) = pua_plain_text_display(ch) {
        return Some(replacement.to_string());
    }
    // U+F02B1~F02C4 는 렌더러의 boxed_pua_char_overlap_semantics 가 먼저 처리한다 (#4158).
    // 반전 사각형 안의 숫자: U+F02CE(1) ~ U+F02E1(20)
    if (0xF02CE..=0xF02E1).contains(&cp) {
        let num = cp - 0xF02CD;
        return Some(format!("{}", num));
    }
    None
}

/// [#3385] **텍스트 추출 전용** PUA 표시 변환.
///
/// IR은 U+F02B1~F02C4(사각 안 숫자) 원문을 보존하고, 렌더러는 폰트 글리프 대신 결정적인
/// 사각형+숫자를 합성한다(#4158). 표준 ①~⑳ 로 직접 렌더하면 1순위 폰트의 *원 안* 글리프가
/// 잡혀 한컴 정답지의 *사각 안* 의미와 달라지므로 렌더 표시 문자열로는 사용하지 않는다.
///
/// 그러나 **텍스트 표면은 사정이 다르다.** 추출 결과는 폰트가 없는 소비자(RAG·LLM·grep)
/// 에게 가므로 원문 PUA 는 읽을 수 없는 코드포인트일 뿐이다. 그래서 렌더 결정은 그대로
/// 두고 텍스트 표면에서만 읽을 수 있는 문자로 바꾼다.
///
/// 의미를 모르는 PUA 는 **매핑을 지어내지 않고 그대로 둔다.**
pub fn pua_to_text_surface(text: &str) -> std::borrow::Cow<'_, str> {
    if !text
        .chars()
        .any(|ch| text_surface_replacement(ch).is_some())
    {
        return std::borrow::Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match text_surface_replacement(ch) {
            Some(rep) => out.push_str(&rep),
            None => out.push(ch),
        }
    }
    std::borrow::Cow::Owned(out)
}

fn text_surface_replacement(ch: char) -> Option<String> {
    let cp = ch as u32;
    // 사각 안 숫자 1~20 — IR은 원문을 유지하고 렌더는 사각형+숫자를 합성하지만, 텍스트
    // 표면에서는 둘러싸인 숫자라는 뜻이 전달되면 충분하다.
    if (0xF02B1..=0xF02C4).contains(&cp) {
        let n = cp - 0xF02B1; // 0-based
        return char::from_u32(0x2460 + n).map(|c| c.to_string());
    }
    // [#6127] U+F02B0 = 네모 안 0 (2599643 신청 번호란 "②⓪⓪") — 0 은 U+2460
    // 연속열 밖이라 ⓪(U+24EA) 를 따로 짝짓는다.
    if cp == 0xF02B0 {
        return Some('\u{24EA}'.to_string());
    }
    // [#5599] U+F02C5 는 연속 구간의 21이 아니라 **네모 12** 다 — 한글 2022 오라클
    // 실측(mel-001 p18 국정과제 bullet, PDF 좌표 절단 판정). 렌더는 위 대역과 같은
    // 원문 유지 계약이고, 텍스트 표면만 ⑫ 로 읽을 수 있게 바꾼다.
    if cp == 0xF02C5 {
        return Some('\u{246B}'.to_string());
    }
    // 렌더가 이미 표시 문자열을 갖고 있는 대역은 같은 답을 쓴다.
    if let Some(replacement) = pua_to_display_text(ch) {
        return Some(replacement);
    }
    // 렌더의 글리프 치환 표(`map_pua_bullet_char`)를 텍스트 표면에도 적용한다.
    //
    // 새 매핑을 지어내지 않고 **렌더가 이미 쓰는 표를 재사용**한다 — 근거(한컴 정답지
    // 실측)가 그 표에 붙어 있다. 사각 안 숫자(U+F02B1~F02C4)는 그 표와 별도로 위 분기가
    // 계속 담당한다.
    //
    // 규모: 저장소 샘플 346건 중 50건이 추출 텍스트에 PUA 를 흘렸고, 그중 U+F080F
    // (굵은 가로선 ━)만 155,709자다. hwp3-sample11.hwp 는 한 쪽 1,398자 중 181자가
    // 이 문자이고 최장 96자 연속 — 머리말/꼬리말 가로선이 본문 텍스트로 나갔다.
    let mapped = super::layout::map_pua_bullet_char(ch);
    if mapped != ch {
        return Some(mapped.to_string());
    }
    None
}
/// 조합된 텍스트 런에서 PUA 테두리 숫자 문자를 찾아 CharOverlap 런으로 변환한다.
///
/// PUA 문자는 원본 그대로 유지하되 CharOverlapInfo만 부착한다.
/// 이렇게 하면 PUA 문자가 항상 1글자로 유지되어:
/// - reflow_line_segs()의 텍스트 측정과 레이아웃 폭 계산이 일치
/// - 두 자리 숫자(10~20)도 1글자 = 1박스 = font_size 폭
///   실제 표시 문자열(PUA → "1", "10" 등) 변환은 draw_char_overlap()에서 수행한다.
fn convert_pua_enclosed_numbers(composed: &mut ComposedParagraph) {
    for line in composed.lines.iter_mut() {
        let mut new_runs: Vec<ComposedTextRun> = Vec::new();
        let mut changed = false;

        for run in line.runs.iter() {
            // 이미 CharOverlap인 런은 그대로 유지
            if run.char_overlap.is_some() {
                new_runs.push(run.clone());
                continue;
            }

            // PUA 테두리 숫자 문자가 있는지 확인
            let has_pua = run
                .text
                .chars()
                .any(|ch| pua_enclosed_border_type(ch).is_some());
            if !has_pua {
                new_runs.push(run.clone());
                continue;
            }

            changed = true;
            let mut buf = String::new();

            for ch in run.text.chars() {
                if let Some(border_type) = pua_enclosed_border_type(ch) {
                    // buf에 쌓인 일반 텍스트를 먼저 런으로 추가
                    if !buf.is_empty() {
                        new_runs.push(ComposedTextRun {
                            text: buf.clone(),
                            char_style_id: run.char_style_id,
                            lang_index: run.lang_index,
                            neutral_space_origin: None,
                            char_overlap: None,
                            footnote_marker: None,
                            display_text: None,
                            supplemental_metrics_blocked: run.supplemental_metrics_blocked,
                            inserted_control_text: run.inserted_control_text,
                            space_metric: run.space_metric,
                            regenerated_font_metrics: run.regenerated_font_metrics,
                            literal_figure_space: run.literal_figure_space,
                        });
                        buf.clear();
                    }
                    // PUA 문자 그대로 유지 + CharOverlapInfo 부착
                    new_runs.push(ComposedTextRun {
                        text: ch.to_string(),
                        char_style_id: run.char_style_id,
                        lang_index: run.lang_index,
                        neutral_space_origin: None,
                        char_overlap: Some(CharOverlapInfo {
                            border_type,
                            inner_char_size: 0,
                        }),
                        footnote_marker: None,
                        display_text: None,
                        supplemental_metrics_blocked: run.supplemental_metrics_blocked,
                        inserted_control_text: run.inserted_control_text,
                        space_metric: run.space_metric,
                        regenerated_font_metrics: run.regenerated_font_metrics,
                        literal_figure_space: run.literal_figure_space,
                    });
                } else {
                    buf.push(ch);
                }
            }

            // 남은 일반 텍스트
            if !buf.is_empty() {
                new_runs.push(ComposedTextRun {
                    text: buf,
                    char_style_id: run.char_style_id,
                    lang_index: run.lang_index,
                    neutral_space_origin: None,
                    char_overlap: None,
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

        if changed {
            line.runs = new_runs;
        }
    }
}
