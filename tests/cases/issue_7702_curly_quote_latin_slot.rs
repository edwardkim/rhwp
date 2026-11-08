//! [#7702] 곡선 따옴표 ‘’“” 는 영문 슬롯 글꼴의 글리프 폭으로 전진한다.
//!
//! 입력 `samples/issue7702/curly_quote_latin_slot.hwpx` 는 `issue6190/center_align_first_line_indent.hwp`
//! 에 rhwp CLI 로 문단 넷을 넣은 합성 문서다. 문단마다 `가“가”가‘가’가"가'가“가”가‘가’가` 를
//! 10pt 로 두고, 한글 슬롯은 휴먼명조(TTF)로 고정한 채 영문 슬롯만 {휴먼명조, Times New Roman}
//! × 정렬 {왼쪽, 양쪽}으로 바꿨다. 저장 줄 정보(`linesegarray`)는 지워 한/글이 줄을 다시
//! 나누게 했다. 기대값은 같은 파일을 한/글 2020(MCP engine 2020) Print 경로로 출력한
//! `pdf/issue7702/curly_quote_latin_slot-2020.pdf` 의 글자 원점 간격(px)이다.
//!
//! 한/글은 앞 글자가 한글이어도 이 따옴표를 영문 슬롯 글꼴로 잰다 — Times 면 Times 글리프
//! (`“` 0.444em · `‘` 0.333em), 휴먼명조면 그 TrueType 글리프(512/512 = 1.0em)다.
//! 수정 전 rhwp 는 따옴표를 한글 슬롯으로 분류하고 전각 글리프를 0.5em·0.3em 으로 눌러
//! 영문 슬롯과 무관하게 `“` 6.667px · `‘` 4.000px 을 냈다.
//!
//! PDF 좌표 격자(0.12pt = 0.16px)와 두 번 나온 글자의 평균이라 허용 오차는 0.2px 다.

use rhwp::document_core::DocumentCore;

const FIXTURE: &[u8] = include_bytes!("../../samples/issue7702/curly_quote_latin_slot.hwpx");

const PROBE_PREFIX: &str = "가“가”가‘가’가";

/// (문단, `“`, `”`, `‘`, `’`) 한/글 전진폭 px — 문단 순서.
const HANGUL_QUOTE_ADVANCES: [(&str, [f64; 4]); 4] = [
    ("영문 휴먼명조 · 왼쪽", [13.433, 13.273, 13.273, 13.433]),
    ("영문 Times · 왼쪽", [5.917, 5.917, 4.478, 4.478]),
    ("영문 휴먼명조 · 양쪽", [13.433, 13.273, 13.273, 13.433]),
    ("영문 Times · 양쪽", [5.917, 5.917, 4.478, 4.478]),
];

const QUOTES: [&str; 4] = ["“", "”", "‘", "’"];

const TOLERANCE_PX: f64 = 0.2;

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!(" {name}=\"");
    let start = tag.find(&key)? + key.len();
    let end = start + tag[start..].find('"')?;
    Some(&tag[start..end])
}

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

/// 첫 쪽의 글자 줄을 위에서 아래로, 줄마다 (x, 글자) 를 x 순으로 모은다.
fn glyph_lines(core: &DocumentCore) -> Vec<Vec<(f64, String)>> {
    let svg = core.render_page_svg_native(0).expect("SVG");
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
    lines
        .into_iter()
        .map(|(_, mut line)| {
            line.sort_by(|a, b| a.0.total_cmp(&b.0));
            line
        })
        .collect()
}

/// 시험 줄마다 따옴표 넷의 평균 전진폭(다음 글자 원점까지).
fn quote_advances(core: &DocumentCore) -> Vec<[f64; 4]> {
    glyph_lines(core)
        .into_iter()
        .filter(|line| {
            line.iter()
                .map(|(_, t)| t.as_str())
                .collect::<String>()
                .starts_with(PROBE_PREFIX)
        })
        .map(|line| {
            QUOTES.map(|quote| {
                let gaps: Vec<f64> = line
                    .windows(2)
                    .filter(|pair| pair[0].1 == quote)
                    .map(|pair| pair[1].0 - pair[0].0)
                    .collect();
                assert!(!gaps.is_empty(), "{quote} 가 줄에 없다");
                gaps.iter().sum::<f64>() / gaps.len() as f64
            })
        })
        .collect()
}

#[test]
fn curly_quotes_advance_by_latin_slot_glyph_like_hangul() {
    let core = DocumentCore::from_bytes(FIXTURE).expect("시험 문서 열기");
    let measured = quote_advances(&core);
    assert_eq!(
        measured.len(),
        HANGUL_QUOTE_ADVANCES.len(),
        "시험 줄 수가 문단 수와 다르다"
    );
    let mut failures = Vec::new();
    for ((label, expected), got) in HANGUL_QUOTE_ADVANCES.iter().zip(&measured) {
        for ((quote, want), have) in QUOTES.iter().zip(expected).zip(got) {
            if (have - want).abs() > TOLERANCE_PX {
                failures.push(format!(
                    "{label} {quote}: rhwp {have:.3}px · 한/글 {want:.3}px"
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
