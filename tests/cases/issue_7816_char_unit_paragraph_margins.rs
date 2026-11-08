//! #7816: 한컴 Print에서 확인한 문단 여백·들여쓰기·간격의 관계를 검사한다.
//!
//! 동일 문구의 기준 문단과 변형 문단을 비교하며 절대 px 원점·고정 영역으로 선택하지 않는다.
//! 짝수 HU 여백과 홀수 CHAR 여백의 비율, 바탕글 크기 변경의 배율, 정렬 불변식,
//! 남는 폭이 작은 들여쓰기의 취소를 독립 입력 조합으로 확인한다.
//! 원본 9종의 전체 Native/fresh WASM TSV와 확대 판독은 PR #7825 검토 기록에 연결한다.
//! 실제 glyph·미세 간격의 정확성은 독립 한컴 Print와 Visual Sweep에서 확인한다.

use rhwp::document_core::DocumentCore;
use std::collections::BTreeMap;

const SHORT: &str = "가나다라마바사아";
const LONG: &str = "한글문서의문단모양에는첫줄들여쓰기가있는데이값을정렬방식과함께바꾸면서저장된줄정보가없는문서를한글이어떻게다시나누는지살펴보려고이문장을충분히길게적어둔다.";

struct Fixture {
    name: &'static str,
    bytes: &'static [u8],
    paragraphs: &'static str,
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

const FIXTURES: &[Fixture] = &[
    Fixture {
        name: "indent_align_matrix",
        bytes: include_bytes!("../../samples/issue7816/indent_align_matrix.hwpx"),
        paragraphs: "SLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSL",
    },
    Fixture {
        name: "odd_indent_ladder",
        bytes: include_bytes!("../../samples/issue7816/odd_indent_ladder.hwpx"),
        paragraphs: "SLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSLSL",
    },
    Fixture {
        name: "odd_margin_left",
        bytes: include_bytes!("../../samples/issue7816/odd_margin_left.hwpx"),
        paragraphs: "SLSLSLSLSL",
    },
    Fixture {
        name: "odd_margin_right",
        bytes: include_bytes!("../../samples/issue7816/odd_margin_right.hwpx"),
        paragraphs: "SLSLSLSLSL",
    },
    Fixture {
        name: "odd_spacing_prev",
        bytes: include_bytes!("../../samples/issue7816/odd_spacing_prev.hwpx"),
        paragraphs: "SLSLSLSLSL",
    },
    Fixture {
        name: "odd_spacing_next",
        bytes: include_bytes!("../../samples/issue7816/odd_spacing_next.hwpx"),
        paragraphs: "SLSLSLSLSL",
    },
    Fixture {
        name: "base_charpr0_20pt",
        bytes: include_bytes!("../../samples/issue7816/base_charpr0_20pt.hwpx"),
        paragraphs: "SLSLSLSL",
    },
    Fixture {
        name: "base_style_charpr_20pt",
        bytes: include_bytes!("../../samples/issue7816/base_style_charpr_20pt.hwpx"),
        paragraphs: "SLSLSLSL",
    },
    Fixture {
        name: "indent_room_boundary",
        bytes: include_bytes!("../../samples/issue7816/indent_room_boundary.hwpx"),
        paragraphs: "SLSLSLSLSLSLSLSLSLSLSLSL",
    },
];

fn first_x(paragraphs: &[Vec<GlyphLine>], index: usize) -> f64 {
    paragraphs[index][0].2[0].0
}

fn second_x(paragraphs: &[Vec<GlyphLine>], index: usize) -> f64 {
    assert!(
        paragraphs[index].len() > 1,
        "문단 {index}의 이어지는 줄 보존"
    );
    paragraphs[index][1].2[0].0
}

fn gap(paragraphs: &[Vec<GlyphLine>], index: usize) -> f64 {
    let previous = paragraphs[index - 1].last().expect("앞 문단");
    let next = &paragraphs[index][0];
    assert_eq!(previous.0, next.0, "간격 대조 문단의 쪽 소속");
    assert!(next.1 > previous.1, "문단 순서와 기준선 전진");
    next.1 - previous.1
}

/// 같은 문구의 실제 글자 advance에 상대적인 반올림 범위다. 절대 px 허용치를 두지 않는다.
fn assert_relation(actual: f64, expected: f64, glyph_advance: f64, label: &str) {
    assert!(glyph_advance > 0.0);
    let rounding = glyph_advance / 32.0;
    assert!(
        (actual - expected).abs() <= rounding,
        "{label}: 관계값 {actual} ≠ {expected}, 글자 advance {glyph_advance}"
    );
}

#[test]
fn odd_paragraph_margins_are_char_units_and_overlong_indents_drop_like_hangul() {
    let mut all = BTreeMap::new();
    for fixture in FIXTURES {
        let core = DocumentCore::from_bytes(fixture.bytes)
            .unwrap_or_else(|e| panic!("{} 열기: {e:?}", fixture.name));
        let paragraphs = paragraph_lines(&glyph_lines(&core), fixture.paragraphs);
        assert_eq!(
            paragraphs.len(),
            fixture.paragraphs.len(),
            "{} 문단 보존",
            fixture.name
        );
        all.insert(fixture.name, paragraphs);
    }
    let matrix = &all["indent_align_matrix"];
    let first_line = &matrix[0][0].2;
    assert!(first_line.len() >= 2, "원점 대조의 두 글자");
    let advance = first_line[1].0 - first_line[0].0;
    let equal = |actual, expected, label| assert_relation(actual, expected, advance, label);
    let delta = |p: &[Vec<GlyphLine>], index, base| first_x(p, index) - first_x(p, base);
    let even_step = delta(matrix, 2, 0); // 짝수 저장2000: 1000 HU
    assert!(even_step > 0.0);
    for group in 0..4 {
        let base = group * 16;
        equal(
            first_x(matrix, base + 10),
            first_x(matrix, base),
            "폭을 넘는 CHAR 들여쓰기 취소",
        );
        equal(
            first_x(matrix, base + 14),
            first_x(matrix, base),
            "짧은 첫 줄의 내어쓰기 취소",
        );
        for index in [2, 4, 6, 8, 12] {
            match group {
                0 | 1 => equal(
                    delta(matrix, base + index, base),
                    delta(matrix, index, 0),
                    "양쪽/왼쪽 동일 들여쓰기",
                ),
                2 => equal(
                    delta(matrix, base + index, base) * 2.0,
                    delta(matrix, index, 0),
                    "가운데 원점 변화는 왼쪽의 절반",
                ),
                _ => equal(
                    first_x(matrix, base + index),
                    first_x(matrix, base),
                    "오른쪽 정렬 원점 보존",
                ),
            }
        }
    }
    let odd = &all["odd_indent_ladder"];
    for (index, factor) in [
        (4, 0.25),
        (6, 0.5),
        (8, 1.0),
        (10, 2.5),
        (12, 5.0),
        (14, 10.0),
        (16, 20.0),
        (18, 25.0),
    ] {
        equal(
            delta(odd, index, 0),
            even_step * factor,
            "CHAR 대 HU 단위 관계",
        );
        equal(
            delta(odd, index + 26, 26) * 2.0,
            delta(odd, index, 0),
            "CHAR 가운데 정렬 관계",
        );
    }
    for base in [0, 26] {
        equal(
            first_x(odd, base + 20),
            first_x(odd, base),
            "과도한 CHAR 들여쓰기 취소",
        );
        for index in [22, 24] {
            equal(
                first_x(odd, base + index),
                first_x(odd, base),
                "내어쓰기 첫 줄 보존",
            );
        }
        assert!(
            second_x(odd, base + 25) > second_x(odd, base + 23),
            "CHAR 내어쓰기 후속 줄 여백 증가"
        );
    }
    for (key, direction) in [("odd_margin_left", 1.0), ("odd_margin_right", -1.0)] {
        let p = &all[key];
        equal(delta(p, 2, 0) * direction, even_step, "좌우 짝수 여백");
        for (index, factor) in [(4, 2.5), (6, 5.0), (8, 10.0)] {
            equal(
                delta(p, index, 0) * direction,
                even_step * factor,
                "좌우 CHAR 여백 관계",
            );
        }
    }
    for key in ["odd_spacing_prev", "odd_spacing_next"] {
        let p = &all[key];
        let natural = gap(p, 1);
        let even = gap(p, 3) - natural;
        assert!(even > 0.0, "짝수 문단 간격 증가");
        for (index, factor) in [(5, 2.5), (7, 5.0), (9, 10.0)] {
            equal(
                gap(p, index) - natural,
                even * factor,
                "CHAR 문단 간격 관계",
            );
        }
        if key == "odd_spacing_prev" {
            for index in [2, 4, 6, 8] {
                equal(
                    gap(p, index),
                    gap(p, index + 1),
                    "위 간격은 현재 문단에 귀속",
                );
            }
        } else {
            equal(gap(p, 2), natural, "아래 간격은 다음 문단에 귀속");
            for index in [4, 6, 8] {
                equal(gap(p, index), gap(p, index - 1), "앞 문단 아래 간격 보존");
            }
        }
    }
    let ordinary = &all["base_charpr0_20pt"];
    let doubled = &all["base_style_charpr_20pt"];
    for (index, factor) in [(2, 2.5), (4, 5.0), (6, 10.0)] {
        equal(
            delta(ordinary, index, 0),
            even_step * factor,
            "비바탕 글자 모양은 CHAR 단위에 영향 없음",
        );
        equal(
            delta(doubled, index, 0),
            delta(ordinary, index, 0) * 2.0,
            "바탕글 크기에 비례하는 CHAR 여백",
        );
    }
    let room = &all["indent_room_boundary"];
    for base in [0, 12] {
        let accepted = first_x(room, base);
        assert!(
            first_x(room, base + 2) > accepted,
            "남는 폭이 충분한 양쪽/가운데 들여쓰기"
        );
        let dropped = first_x(room, base + 4);
        assert!(dropped < accepted, "남는 폭이 작은 들여쓰기 취소");
        for index in [6, 8, 10] {
            equal(
                first_x(room, base + index),
                dropped,
                "취소한 들여쓰기 원점 일치",
            );
        }
        equal(
            dropped,
            first_x(matrix, if base == 0 { 0 } else { 32 }),
            "취소 후 무들여쓰기 대조군 원점",
        );
    }
}
