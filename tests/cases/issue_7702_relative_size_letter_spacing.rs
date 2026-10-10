//! [#7702] 자간은 상대 크기를 적용한 글자 크기의 백분율이다.
//!
//! 입력 `samples/issue7702/relative_size_letter_spacing.hwp` 는 한/글 2010 저장본
//! (`re-font-gulimche-hancom.hwp`)에 rhwp CLI 로 문단 27개를 넣고, 문단마다 기준 크기
//! 10pt 에 장평 {80, 100, 120}% × 상대 크기 {80, 100, 125}% × 자간 {−10, 0, +10}% 를 준
//! 것이다. 기대값은 같은 파일을 한/글 2020(11.0.0.9136, MCP engine 2020) Print 경로로 출력한
//! `pdf/issue7702/relative_size_letter_spacing-2020.pdf` 의 글자 원점에서 잰 평균 전진폭
//! (한글 10자·숫자 10자)이다.
//!
//! 비교하는 값은 **자간 몫** — 같은 장평·상대 크기에서 자간 ±10% 의 전진폭과 자간 0% 의
//! 전진폭 차 — 이다. 글꼴 메트릭 자체의 작은 차이는 이 차에서 지워진다. PDF 좌표 격자
//! (0.12pt) 때문에 허용 오차를 0.04px 로 둔다.
//!
//! 수정 전에는 자간을 상대 크기 적용 **전** 크기로 계산해 상대 크기 80·125% 에서 자간 몫이
//! 글자당 0.18~0.37px 어긋났다(100% 는 같다).

use rhwp::document_core::DocumentCore;

const FIXTURE: &[u8] = include_bytes!("../../samples/issue7702/relative_size_letter_spacing.hwp");

/// (장평 %, 상대 크기 %, 자간 %, 한/글 한글 전진 px, 한/글 숫자 전진 px) — 문단 순서.
const HANGUL_ADVANCES: [(u8, u8, i8, f64, f64); 27] = [
    (80, 80, -10, 7.410, 4.211),
    (80, 80, 0, 8.262, 4.691),
    (80, 80, 10, 9.115, 5.171),
    (80, 100, -10, 9.328, 5.277),
    (80, 100, 0, 10.341, 5.863),
    (80, 100, 10, 11.354, 6.450),
    (80, 125, -10, 11.620, 6.557),
    (80, 125, 0, 12.900, 7.302),
    (80, 125, 10, 14.179, 8.049),
    (100, 80, -10, 9.328, 5.277),
    (100, 80, 0, 10.341, 5.863),
    (100, 80, 10, 11.354, 6.450),
    (100, 100, -10, 11.673, 6.610),
    (100, 100, 0, 12.953, 7.356),
    (100, 100, 10, 14.232, 8.103),
    (100, 125, -10, 14.552, 8.262),
    (100, 125, 0, 16.152, 9.169),
    (100, 125, 10, 17.750, 10.075),
    (120, 80, -10, 11.141, 6.343),
    (120, 80, 0, 12.367, 7.036),
    (120, 80, 10, 13.593, 7.729),
    (120, 100, -10, 13.966, 7.889),
    (120, 100, 0, 15.511, 8.794),
    (120, 100, 10, 17.058, 9.701),
    (120, 125, -10, 17.430, 9.808),
    (120, 125, 0, 19.350, 10.927),
    (120, 125, 10, 21.268, 12.047),
];

const TOLERANCE_PX: f64 = 0.04;

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!(" {name}=\"");
    let start = tag.find(&key)? + key.len();
    let end = start + tag[start..].find('"')?;
    Some(&tag[start..end])
}

/// 글자 `<text>` 의 원점. 장평이 100% 가 아니면 `transform="translate(x,y) scale(..)"` 로 쓴다.
fn origin(tag: &str) -> Option<(f64, f64)> {
    if let (Some(x), Some(y)) = (attr(tag, "x"), attr(tag, "y")) {
        return Some((x.parse().ok()?, y.parse().ok()?));
    }
    let transform = attr(tag, "transform")?;
    let args = transform.strip_prefix("translate(")?;
    let args = &args[..args.find(')')?];
    let (x, y) = args.split_once(',')?;
    Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
}

/// 모든 쪽의 글자 줄을 위에서 아래로, 줄마다 (x, 글자) 를 x 순으로 모은다.
fn glyph_lines(core: &DocumentCore) -> Vec<Vec<(f64, String)>> {
    let mut out = Vec::new();
    for page in 0..core.page_count() {
        let svg = core.render_page_svg_native(page).expect("SVG");
        let mut lines: Vec<(f64, Vec<(f64, String)>)> = Vec::new();
        let mut rest = svg.as_str();
        while let Some(open) = rest.find("<text ") {
            rest = &rest[open..];
            let (Some(tag_end), Some(close)) = (rest.find('>'), rest.find("</text>")) else {
                break;
            };
            if let Some((x, y)) = origin(&rest[..tag_end]) {
                let y = (y * 10.0).round() / 10.0;
                let text = rest[tag_end + 1..close].to_string();
                match lines.iter_mut().find(|(line_y, _)| *line_y == y) {
                    Some((_, line)) => line.push((x, text)),
                    None => lines.push((y, vec![(x, text)])),
                }
            }
            rest = &rest[close..];
        }
        lines.sort_by(|a, b| a.0.total_cmp(&b.0));
        for (_, mut line) in lines {
            line.sort_by(|a, b| a.0.total_cmp(&b.0));
            out.push(line);
        }
    }
    out
}

fn line_text(line: &[(f64, String)]) -> String {
    line.iter().map(|(_, t)| t.as_str()).collect()
}

/// 시험 문단 줄마다 (한글 평균 전진, 숫자 평균 전진).
fn probe_line_advances(core: &DocumentCore) -> Vec<(f64, f64)> {
    glyph_lines(core)
        .into_iter()
        .filter(|line| line_text(line).starts_with("가나다라마바사아자차"))
        .map(|line| {
            let x_of = |s: &str| line.iter().find(|(_, t)| t == s).map(|(x, _)| *x);
            let digit = (x_of("9").expect("9") - x_of("0").expect("0")) / 9.0;
            ((line[9].0 - line[0].0) / 9.0, digit)
        })
        .collect()
}

/// `prefix` 로 시작하는 줄마다 `glyph` 원점 사이 평균 간격(px).
fn glyph_pitches(core: &DocumentCore, prefix: &str, glyph: &str) -> Vec<f64> {
    glyph_lines(core)
        .into_iter()
        .filter(|line| line_text(line).starts_with(prefix))
        .map(|line| {
            let xs: Vec<f64> = line
                .iter()
                .filter(|(_, t)| t == glyph)
                .map(|(x, _)| *x)
                .collect();
            (xs[xs.len() - 1] - xs[0]) / (xs.len() - 1) as f64
        })
        .collect()
}

#[test]
fn letter_spacing_share_matches_hangul_across_ratio_and_relative_size() {
    let core = DocumentCore::from_bytes(FIXTURE).expect("시험 문서 열기");
    let measured = probe_line_advances(&core);
    assert_eq!(
        measured.len(),
        HANGUL_ADVANCES.len(),
        "시험 문단 줄 수가 다르면 비교 대상이 어긋난다"
    );
    for group in 0..HANGUL_ADVANCES.len() / 3 {
        let zero = group * 3 + 1;
        let (ratio, rel, _, hangul_syllable_0, hangul_digit_0) = HANGUL_ADVANCES[zero];
        let (syllable_0, digit_0) = measured[zero];
        for idx in [zero - 1, zero + 1] {
            let (_, _, spacing, hangul_syllable, hangul_digit) = HANGUL_ADVANCES[idx];
            let (syllable, digit) = measured[idx];
            let share = syllable - syllable_0;
            let hangul_share = hangul_syllable - hangul_syllable_0;
            assert!(
                (share - hangul_share).abs() <= TOLERANCE_PX,
                "장평 {ratio}% · 상대 크기 {rel}% · 자간 {spacing}%: 한글 자간 몫 {share:.3}px, 한/글 {hangul_share:.3}px"
            );
            let share = digit - digit_0;
            let hangul_share = hangul_digit - hangul_digit_0;
            assert!(
                (share - hangul_share).abs() <= TOLERANCE_PX,
                "장평 {ratio}% · 상대 크기 {rel}% · 자간 {spacing}%: 숫자 자간 몫 {share:.3}px, 한/글 {hangul_share:.3}px"
            );
        }
    }
}

/// 장치 격자 시험의 허용 오차. 남는 차는 PDF 좌표 배율(약 1800/1801)에서 오는
/// 일정한 치우침(0.013px 이하)이다. 수정 전 계산은 같은 표에서 최대 0.058px 어긋났다.
const GRID_TOLERANCE_PX: f64 = 0.02;

const RATIO_STEPS: &[u8] = include_bytes!("../../samples/issue7702/ratio_steps.hwp");
const SIZE_RATIO_RELSIZE: &[u8] = include_bytes!("../../samples/issue7702/size_ratio_relsize.hwp");
const HALF_SPACE_TIES: &[u8] = include_bytes!("../../samples/issue7702/half_space_ties.hwp");

/// [#7702] 한/글은 글자 전진폭을 장치 격자(4 HWPUNIT)의 정수로 놓는다.
///
/// 입력은 위 시험과 같은 한/글 2010 저장본에 굴림체 10pt 음절 38자 문단 17개를 넣고 장평만
/// 50~150% 로 바꾼 것이다. 기대값은 `pdf/issue7702/ratio_steps-2020.pdf`(한/글 2020 Print)의
/// 줄 첫 글자부터 끝 글자까지 원점 간격 평균이다. 한/글 전진폭은 장평에 비례하지 않고
/// ⌊em비 × ppem × 장평⌋ 단위로 계단을 이룬다(장평 99% 959.5 · 98% 947.5 · 97% 939.4 HU).
#[test]
fn glyph_advance_steps_with_ratio_like_hangul() {
    const HANGUL: [(u8, f64); 17] = [
        (100, 12.9529),
        (99, 12.7937),
        (98, 12.6327),
        (97, 12.5250),
        (96, 12.3647),
        (95, 12.2613),
        (94, 12.1011),
        (92, 11.8858),
        (90, 11.6220),
        (85, 10.9822),
        (80, 10.3424),
        (75, 9.6467),
        (60, 7.7275),
        (50, 6.4484),
        (110, 14.1808),
        (120, 15.5114),
        (150, 19.3490),
    ];
    let core = DocumentCore::from_bytes(RATIO_STEPS).expect("시험 문서 열기");
    let measured: Vec<f64> = glyph_lines(&core)
        .into_iter()
        .filter(|line| line_text(line).starts_with("가나다라마바"))
        .map(|line| (line[line.len() - 1].0 - line[0].0) / (line.len() - 1) as f64)
        .collect();
    assert_eq!(measured.len(), HANGUL.len(), "시험 문단 줄 수");
    for ((ratio, hangul), advance) in HANGUL.iter().zip(measured) {
        assert!(
            (advance - hangul).abs() <= GRID_TOLERANCE_PX,
            "장평 {ratio}%: 음절 전진 {advance:.4}px, 한/글 {hangul:.4}px"
        );
    }
}

/// 같은 계단이 글자 크기·상대 크기에도 걸린다. 전진폭은 상대 크기를 적용한 크기의 ppem 에서
/// 정해지고, 상대 크기 95% 와 장평 95% 는 다른 경로로 같은 값(10pt 에서 919.3 HU)이 된다.
/// 기대값은 `pdf/issue7702/size_ratio_relsize-2020.pdf`.
#[test]
fn glyph_advance_steps_with_size_and_relative_size_like_hangul() {
    const HANGUL: [(u16, u8, u8, f64); 28] = [
        (800, 100, 100, 10.3383),
        (800, 95, 100, 9.8105),
        (800, 90, 100, 9.2752),
        (900, 100, 100, 11.6178),
        (900, 95, 100, 11.0345),
        (900, 90, 100, 10.4503),
        (1000, 100, 100, 12.9529),
        (1000, 95, 100, 12.2576),
        (1000, 90, 100, 11.6182),
        (1100, 100, 100, 14.2324),
        (1100, 95, 100, 13.4881),
        (1100, 90, 100, 12.7930),
        (1200, 100, 100, 15.5116),
        (1200, 95, 100, 14.7113),
        (1200, 90, 100, 13.9122),
        (1300, 100, 100, 16.7905),
        (1300, 95, 100, 15.9362),
        (1300, 90, 100, 15.0874),
        (1400, 100, 100, 18.1259),
        (1400, 95, 100, 17.1660),
        (1400, 90, 100, 16.2547),
        (2000, 100, 100, 25.8509),
        (2000, 95, 100, 24.5178),
        (2000, 90, 100, 23.2434),
        (1000, 100, 95, 12.2577),
        (1000, 100, 90, 11.6178),
        (1300, 100, 95, 15.9362),
        (1300, 100, 90, 15.0877),
    ];
    let core = DocumentCore::from_bytes(SIZE_RATIO_RELSIZE).expect("시험 문서 열기");
    let measured: Vec<f64> = glyph_lines(&core)
        .into_iter()
        .filter(|line| line_text(line).starts_with("가나다라마바"))
        .map(|line| (line[line.len() - 1].0 - line[0].0) / (line.len() - 1) as f64)
        .collect();
    assert_eq!(measured.len(), HANGUL.len(), "시험 문단 줄 수");
    for ((size, ratio, rel, hangul), advance) in HANGUL.iter().zip(measured) {
        assert!(
            (advance - hangul).abs() <= GRID_TOLERANCE_PX,
            "{size} HU · 장평 {ratio}% · 상대 크기 {rel}%: 음절 전진 {advance:.4}px, 한/글 {hangul:.4}px"
        );
    }
}

/// 반각 공백이 정확히 ½ 단위에 걸리는 크기(ppem 175·225·275·325)에서 한/글은 공백을 내린다.
/// 입력은 「가 」 21쌍 문단을 7·9·11·13pt 로 둔 것이고, 기대값은
/// `pdf/issue7702/half_space_ties-2020.pdf` 의 「가」 원점 간격(음절 + 공백)이다.
#[test]
fn half_space_ties_round_down_like_hangul() {
    const HANGUL: [(u16, f64); 4] = [
        (700, 13.6966),
        (900, 17.5905),
        (1100, 21.5323),
        (1300, 25.4262),
    ];
    let core = DocumentCore::from_bytes(HALF_SPACE_TIES).expect("시험 문서 열기");
    let measured = glyph_pitches(&core, "가가", "가");
    assert!(
        measured.len() >= HANGUL.len(),
        "시험 문단 줄 수 {}",
        measured.len()
    );
    for ((size, hangul), pitch) in HANGUL.iter().zip(measured) {
        assert!(
            (pitch - hangul).abs() <= GRID_TOLERANCE_PX,
            "{size} HU: 「가 」 간격 {pitch:.4}px, 한/글 {hangul:.4}px"
        );
    }
}
