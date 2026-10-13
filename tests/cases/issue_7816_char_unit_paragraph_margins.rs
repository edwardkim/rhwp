//! [#7816] 문단 여백의 홀수 저장값은 «글자» 단위이고, 들여쓴 뒤 남는 폭이 1,500 HU 미만이면
//! 한/글은 들여쓰기를 버린다.
//!
//! 입력은 `samples/issue7702/curly_quote_latin_slot.hwpx` 의 secPr 문단만 남기고 문단 모양 45 를
//! 복제해 정렬·여백 한 항목만 바꾼 합성 HWPX 다. 저장 줄(`linesegarray`)이 없어 한/글도 줄을 새로
//! 나눈다. 문단마다 짧은 글(`가나다라마바사아`, 1줄 — `S`)과 긴 글(3줄 — `L`)을 하나씩 둔다.
//! 시험 파일에는 rhwp 내부 표지 `META-INF/rhwp-hwp5-origin` 이 없다(한/글 출력은 표지 유무와 무관).
//!
//! 기대값은 같은 문서를 한/글 2020(MCP engine 2020)으로 출력한 `pdf/issue7816/<이름>-2020.pdf` 의
//! 글자 원점(px, 96dpi)이다. 홀수 저장값 `s` 는 HWPX HwpUnitChar `case` 에 `unit="CHAR"` ·
//! `(s − 1) / 2` 로 적히고, 한/글은 그 값에 바탕글 글자 크기 / 200 HU 를 곱한다.
//! 수정 전 rhwp 는 홀수도 `s / 2` HU 로 읽어 1/5 만 들여 쓰고(1001 → 6.7px, 한/글 33.3px),
//! 줄 상자를 넘는 들여쓰기도 그대로 얹었다(20445 → 136.3px, 한/글 0).
//!
//! 허용 오차는 PDF 좌표 격자와 글자 원점 반올림을 합친 0.5px 다.

use rhwp::document_core::DocumentCore;

const SHORT: &str = "가나다라마바사아";
const LONG: &str = "한글문서의문단모양에는첫줄들여쓰기가있는데이값을정렬방식과함께바꾸면서저장된줄정보가없는문서를한글이어떻게다시나누는지살펴보려고이문장을충분히길게적어둔다.";
const TOLERANCE_PX: f64 = 0.5;

#[derive(Clone, Copy, Debug)]
enum Probe {
    /// 문단 첫 줄 첫 글자 x
    FirstGlyphX,
    /// 문단 둘째 줄 첫 글자 x (내어쓰기)
    SecondLineFirstGlyphX,
    /// 앞 문단 마지막 줄 기준선 → 이 문단 첫 줄 기준선
    BaselineGapFromPrevious,
}

struct Fixture {
    name: &'static str,
    bytes: &'static [u8],
    /// 문단 순서의 종류 (`S` 짧은 글, `L` 긴 글)
    paragraphs: &'static str,
    /// (표지, 문단 번호, 탐침, 한/글 값 px)
    probes: &'static [(&'static str, usize, Probe, f64)],
}

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

/// 글자 줄: (쪽, 기준선 y, x 순 글자들)
type GlyphLine = (u32, f64, Vec<(f64, String)>);

fn glyph_lines(core: &DocumentCore) -> Vec<GlyphLine> {
    let mut all = Vec::new();
    for page in 0..core.page_count() {
        let svg = core.render_page_svg_native(page).expect("SVG");
        let mut lines: Vec<(f64, Vec<(f64, String)>)> = Vec::new();
        let mut rest = svg.as_str();
        while let Some(open) = rest.find("<text ") {
            rest = &rest[open..];
            let (Some(tag_end), Some(close)) = (rest.find('>'), rest.find("</text>")) else {
                break;
            };
            let text = rest[tag_end + 1..close].trim().to_string();
            if let (Some((x, y)), false) = (origin(&rest[..tag_end]), text.is_empty()) {
                let y = (y * 10.0).round() / 10.0;
                match lines.iter_mut().find(|(line_y, _)| *line_y == y) {
                    Some((_, line)) => line.push((x, text)),
                    None => lines.push((y, vec![(x, text)])),
                }
            }
            rest = &rest[close..];
        }
        lines.sort_by(|a, b| a.0.total_cmp(&b.0));
        for (y, mut line) in lines {
            line.sort_by(|a, b| a.0.total_cmp(&b.0));
            all.push((page, y, line));
        }
    }
    all
}

/// 문단 순서(`S`/`L`)대로 글자 줄을 묶는다. 글이 맞지 않는 줄(머리말 등)은 건너뛴다.
fn paragraph_lines(lines: &[GlyphLine], kinds: &str) -> Vec<Vec<GlyphLine>> {
    let text = |line: &GlyphLine| line.2.iter().map(|(_, t)| t.as_str()).collect::<String>();
    let mut out = Vec::new();
    let mut index = 0;
    for kind in kinds.chars() {
        let want = if kind == 'S' { SHORT } else { LONG };
        while index < lines.len() && !want.starts_with(text(&lines[index]).as_str()) {
            index += 1;
        }
        let mut acc = String::new();
        let mut group = Vec::new();
        while index < lines.len() && acc != want {
            let t = text(&lines[index]);
            if !want.starts_with(format!("{acc}{t}").as_str()) {
                break;
            }
            acc.push_str(&t);
            group.push(lines[index].clone());
            index += 1;
        }
        assert_eq!(acc, want, "{kind} 문단 #{} 의 글을 찾지 못했다", out.len());
        out.push(group);
    }
    out
}

fn measure(paragraphs: &[Vec<GlyphLine>], para: usize, probe: Probe) -> Option<f64> {
    let lines = &paragraphs[para];
    match probe {
        Probe::FirstGlyphX => lines.first()?.2.first().map(|g| g.0),
        Probe::SecondLineFirstGlyphX => lines.get(1)?.2.first().map(|g| g.0),
        Probe::BaselineGapFromPrevious => {
            let previous = paragraphs.get(para.checked_sub(1)?)?.last()?;
            let first = lines.first()?;
            (previous.0 == first.0).then_some(first.1 - previous.1)
        }
    }
}

const FIXTURES: &[Fixture] = &[
    Fixture {
        // 정렬 4종 × 첫 줄 들여쓰기 0·2000·6000·10000·20444·20445·30000·−6000. 20445(홀수, 글자 단위 681.5px)만 버린다.
        name: "indent_align_matrix",
        bytes: include_bytes!("../../samples/issue7816/indent_align_matrix.hwpx"),
        paragraphs: "SLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSL",
        probes: &[
            ("J0S", 0, Probe::FirstGlyphX, 94.51),
            ("J2000S", 2, Probe::FirstGlyphX, 107.78),
            ("J6000S", 4, Probe::FirstGlyphX, 134.49),
            ("J10000S", 6, Probe::FirstGlyphX, 161.19),
            ("J20444S", 8, Probe::FirstGlyphX, 230.75),
            ("J20445S", 10, Probe::FirstGlyphX, 94.51),
            ("J30000S", 12, Probe::FirstGlyphX, 294.40),
            ("J-6000S", 14, Probe::FirstGlyphX, 94.51),
            ("L0S", 16, Probe::FirstGlyphX, 94.51),
            ("L2000S", 18, Probe::FirstGlyphX, 107.78),
            ("L6000S", 20, Probe::FirstGlyphX, 134.49),
            ("L10000S", 22, Probe::FirstGlyphX, 161.19),
            ("L20444S", 24, Probe::FirstGlyphX, 230.75),
            ("L20445S", 26, Probe::FirstGlyphX, 94.51),
            ("L30000S", 28, Probe::FirstGlyphX, 294.40),
            ("L-6000S", 30, Probe::FirstGlyphX, 94.51),
            ("C0S", 32, Probe::FirstGlyphX, 343.49),
            ("C2000S", 34, Probe::FirstGlyphX, 350.05),
            ("C6000S", 36, Probe::FirstGlyphX, 363.48),
            ("C10000S", 38, Probe::FirstGlyphX, 376.76),
            ("C20444S", 40, Probe::FirstGlyphX, 411.46),
            ("C20445S", 42, Probe::FirstGlyphX, 343.49),
            ("C30000S", 44, Probe::FirstGlyphX, 443.44),
            ("C-6000S", 46, Probe::FirstGlyphX, 343.49),
            ("R0S", 48, Probe::FirstGlyphX, 592.32),
            ("R2000S", 50, Probe::FirstGlyphX, 592.32),
            ("R6000S", 52, Probe::FirstGlyphX, 592.32),
            ("R10000S", 54, Probe::FirstGlyphX, 592.32),
            ("R20444S", 56, Probe::FirstGlyphX, 592.32),
            ("R20445S", 58, Probe::FirstGlyphX, 592.32),
            ("R30000S", 60, Probe::FirstGlyphX, 592.32),
            ("R-6000S", 62, Probe::FirstGlyphX, 592.32),
        ],
    },
    Fixture {
        // 양쪽·가운데 × 홀수 들여쓰기 1~20001 · 내어쓰기 −1001·−2001. 20001(666.7px)은 버린다.
        name: "odd_indent_ladder",
        bytes: include_bytes!("../../samples/issue7816/odd_indent_ladder.hwpx"),
        paragraphs: "SLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSL",
        probes: &[
            ("J1S", 0, Probe::FirstGlyphX, 94.51),
            ("J3S", 2, Probe::FirstGlyphX, 94.51),
            ("J101S", 4, Probe::FirstGlyphX, 97.87),
            ("J201S", 6, Probe::FirstGlyphX, 101.23),
            ("J401S", 8, Probe::FirstGlyphX, 107.78),
            ("J1001S", 10, Probe::FirstGlyphX, 127.77),
            ("J2001S", 12, Probe::FirstGlyphX, 161.19),
            ("J4001S", 14, Probe::FirstGlyphX, 227.72),
            ("J8001S", 16, Probe::FirstGlyphX, 361.08),
            ("J10001S", 18, Probe::FirstGlyphX, 427.61),
            ("J20001S", 20, Probe::FirstGlyphX, 94.51),
            ("J-1001S", 22, Probe::FirstGlyphX, 94.51),
            ("J-1001L", 23, Probe::SecondLineFirstGlyphX, 127.93),
            ("J-2001S", 24, Probe::FirstGlyphX, 94.51),
            ("J-2001L", 25, Probe::SecondLineFirstGlyphX, 161.19),
            ("C1S", 26, Probe::FirstGlyphX, 343.49),
            ("C3S", 28, Probe::FirstGlyphX, 343.49),
            ("C101S", 30, Probe::FirstGlyphX, 345.09),
            ("C201S", 32, Probe::FirstGlyphX, 346.69),
            ("C401S", 34, Probe::FirstGlyphX, 350.05),
            ("C1001S", 36, Probe::FirstGlyphX, 360.12),
            ("C2001S", 38, Probe::FirstGlyphX, 376.76),
            ("C4001S", 40, Probe::FirstGlyphX, 410.02),
            ("C8001S", 42, Probe::FirstGlyphX, 476.70),
            ("C10001S", 44, Probe::FirstGlyphX, 509.96),
            ("C20001S", 46, Probe::FirstGlyphX, 343.49),
            ("C-1001S", 48, Probe::FirstGlyphX, 343.49),
            ("C-1001L", 49, Probe::SecondLineFirstGlyphX, 140.24),
            ("C-2001S", 50, Probe::FirstGlyphX, 343.49),
            ("C-2001L", 51, Probe::SecondLineFirstGlyphX, 173.51),
        ],
    },
    Fixture {
        // 왼쪽 여백 0·2000·1001·2001·4001.
        name: "odd_margin_left",
        bytes: include_bytes!("../../samples/issue7816/odd_margin_left.hwpx"),
        paragraphs: "SLSLSLSLSL",
        probes: &[
            ("J0S", 0, Probe::FirstGlyphX, 94.51),
            ("J2000S", 2, Probe::FirstGlyphX, 107.78),
            ("J1001S", 4, Probe::FirstGlyphX, 127.77),
            ("J2001S", 6, Probe::FirstGlyphX, 161.19),
            ("J4001S", 8, Probe::FirstGlyphX, 227.72),
        ],
    },
    Fixture {
        // 오른쪽 정렬 × 오른쪽 여백 0·2000·1001·2001·4001.
        name: "odd_margin_right",
        bytes: include_bytes!("../../samples/issue7816/odd_margin_right.hwpx"),
        paragraphs: "SLSLSLSLSL",
        probes: &[
            ("R0S", 0, Probe::FirstGlyphX, 592.32),
            ("R2000S", 2, Probe::FirstGlyphX, 579.05),
            ("R1001S", 4, Probe::FirstGlyphX, 559.06),
            ("R2001S", 6, Probe::FirstGlyphX, 525.64),
            ("R4001S", 8, Probe::FirstGlyphX, 459.11),
        ],
    },
    Fixture {
        // 문단 위 간격 0·2000·1001·2001·4001 (앞 문단 끝 줄 → 첫 줄 기준선).
        name: "odd_spacing_prev",
        bytes: include_bytes!("../../samples/issue7816/odd_spacing_prev.hwpx"),
        paragraphs: "SLSLSLSLSL",
        probes: &[
            ("J0L", 1, Probe::BaselineGapFromPrevious, 19.80),
            ("J2000S", 2, Probe::BaselineGapFromPrevious, 33.10),
            ("J2000L", 3, Probe::BaselineGapFromPrevious, 33.10),
            ("J1001S", 4, Probe::BaselineGapFromPrevious, 53.10),
            ("J1001L", 5, Probe::BaselineGapFromPrevious, 53.00),
            ("J2001S", 6, Probe::BaselineGapFromPrevious, 86.30),
            ("J2001L", 7, Probe::BaselineGapFromPrevious, 86.40),
            ("J4001S", 8, Probe::BaselineGapFromPrevious, 153.00),
            ("J4001L", 9, Probe::BaselineGapFromPrevious, 152.80),
        ],
    },
    Fixture {
        // 문단 아래 간격 0·2000·1001·2001·4001 (앞 문단 끝 줄 → 첫 줄 기준선).
        name: "odd_spacing_next",
        bytes: include_bytes!("../../samples/issue7816/odd_spacing_next.hwpx"),
        paragraphs: "SLSLSLSLSL",
        probes: &[
            ("J0L", 1, Probe::BaselineGapFromPrevious, 19.80),
            ("J2000S", 2, Probe::BaselineGapFromPrevious, 19.80),
            ("J2000L", 3, Probe::BaselineGapFromPrevious, 33.00),
            ("J1001S", 4, Probe::BaselineGapFromPrevious, 33.10),
            ("J1001L", 5, Probe::BaselineGapFromPrevious, 52.90),
            ("J2001S", 6, Probe::BaselineGapFromPrevious, 53.10),
            ("J2001L", 7, Probe::BaselineGapFromPrevious, 86.30),
            ("J4001S", 8, Probe::BaselineGapFromPrevious, 86.30),
            ("J4001L", 9, Probe::BaselineGapFromPrevious, 152.90),
        ],
    },
    Fixture {
        // 글자 모양 0 만 20pt — 글자 단위가 바뀌지 않는다.
        name: "base_charpr0_20pt",
        bytes: include_bytes!("../../samples/issue7816/base_charpr0_20pt.hwpx"),
        paragraphs: "SLSLSLSL",
        probes: &[
            ("J0S", 0, Probe::FirstGlyphX, 94.51),
            ("J1001S", 2, Probe::FirstGlyphX, 127.77),
            ("J2001S", 4, Probe::FirstGlyphX, 161.19),
            ("J4001S", 6, Probe::FirstGlyphX, 227.72),
        ],
    },
    Fixture {
        // 바탕글 스타일의 글자 모양(1)만 20pt — 글자 단위가 두 배가 된다.
        name: "base_style_charpr_20pt",
        bytes: include_bytes!("../../samples/issue7816/base_style_charpr_20pt.hwpx"),
        paragraphs: "SLSLSLSL",
        probes: &[
            ("J0S", 0, Probe::FirstGlyphX, 94.51),
            ("J1001S", 2, Probe::FirstGlyphX, 161.19),
            ("J2001S", 4, Probe::FirstGlyphX, 227.72),
            ("J4001S", 6, Probe::FirstGlyphX, 361.08),
        ],
    },
    Fixture {
        // 들여쓴 뒤 남는 폭 30·20·14·13·10·5px(짝수 저장값). 20px 이상만 적용한다.
        name: "indent_room_boundary",
        bytes: include_bytes!("../../samples/issue7816/indent_room_boundary.hwpx"),
        paragraphs: "SLSLSLSLSLSLSLSLSLSLSLSL",
        probes: &[
            ("J86212S", 0, Probe::FirstGlyphX, 668.92),
            ("J87712S", 2, Probe::FirstGlyphX, 678.99),
            ("J88612S", 4, Probe::FirstGlyphX, 94.51),
            ("J88762S", 6, Probe::FirstGlyphX, 94.51),
            ("J89212S", 8, Probe::FirstGlyphX, 94.51),
            ("J89962S", 10, Probe::FirstGlyphX, 94.51),
            ("C86212S", 12, Probe::FirstGlyphX, 670.68),
            ("C87712S", 14, Probe::FirstGlyphX, 682.35),
            ("C88612S", 16, Probe::FirstGlyphX, 343.49),
            ("C88762S", 18, Probe::FirstGlyphX, 343.49),
            ("C89212S", 20, Probe::FirstGlyphX, 343.49),
            ("C89962S", 22, Probe::FirstGlyphX, 343.49),
        ],
    },
];

#[test]
fn odd_paragraph_margins_are_char_units_and_overlong_indents_drop_like_hangul() {
    let mut failures = Vec::new();
    for fixture in FIXTURES {
        let core = DocumentCore::from_bytes(fixture.bytes)
            .unwrap_or_else(|e| panic!("{} 열기: {e:?}", fixture.name));
        let paragraphs = paragraph_lines(&glyph_lines(&core), fixture.paragraphs);
        assert!(!fixture.probes.is_empty(), "{} 탐침이 없다", fixture.name);
        for &(label, para, probe, hangul) in fixture.probes {
            match measure(&paragraphs, para, probe) {
                Some(got) if (got - hangul).abs() <= TOLERANCE_PX => {}
                got => failures.push(format!(
                    "{} {label} {probe:?}: rhwp {got:?} · 한/글 {hangul:.2}",
                    fixture.name
                )),
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{}개 불일치\n{}",
        failures.len(),
        failures.join("\n")
    );
}
