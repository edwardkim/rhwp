//! [#7702] 저장 줄이 없는 빈 host 문단의 줄은 첫 줄 들여쓰기가 양수면 자리차지 표 **아래**에 놓인다.
//!
//! 앵커에서 시작하는 문단 기준 자리차지(TopAndBottom) 띠는 줄 폭을 전부 막는다. 빈 host 줄이 요구하는
//! 폭은 첫 줄 들여쓰기다. 들여쓰기 ≤ 0 이면 폭 0 줄이 띠 시작에 흡수되고(한/글 저장형 `sw=0`),
//! 양수면 띠에 못 들어가 띠 아래에 놓인다(저장형 `sw`=본문 폭, 줄 vpos = 문단 시작 + 표 점유).
//! 다음 문단은 그 줄의 `lh + ls`(10pt·148% = 19.7px)만큼 더 내려간다. 수정 전 rhwp 는 저장 줄 없는
//! host 줄을 항상 띠에 흡수했다(`samples/issue7702/curly_quote_latin_slot.hwpx` 「발표 논문」 블록이
//! 한/글보다 19.6px 위).
//!
//! 입력은 같은 fixture 의 학력 표(3행, 비TAC·자리차지·vert=문단·오프셋 0)를 host 변형마다 복제하고
//! 표마다 뒤에 표지 문단(`표지NN <변형>`)을 둔 합성 HWPX 다. 저장 줄(`linesegarray`)이 없어 한/글도
//! 줄을 새로 나눈다. 변형은 host 문단 모양의 한 항목만 바꾼다.
//!
//! - `a`: 양쪽/가운데 정렬 × 들여쓰기 0/20445 × 표 뒤 빈 run 유무
//! - `b`: 들여쓰기 20445/20444 × 양쪽/가운데/왼쪽 정렬 × 빈 run (전부 띠 아래)
//! - `c`: 들여쓰기 2·20·200·2000(띠 아래) · -2000(흡수)
//! - `d`: 왼쪽 여백 2000·20000 · 오른쪽 여백 2000 · 내어쓰기 -20000(흡수) · 들여쓰기 6000(띠 아래)
//! - `g`: 빈 host 두 개가 연달아 오는 표 스택(2000+2000 · 0+2000 · 2000+0 · 0+0)
//!
//! 고정본에는 rhwp 내부 표지 `META-INF/rhwp-hwp5-origin` 이 있다(빈 줄 배열 경로). 표지를 지워
//! 로드 때 줄을 합성하는 경로는 표 바깥 여백 계상이 따로 어긋나(표마다 141HU) 이 시험에 넣지 않는다.
//!
//! 기대값은 같은 문서를 한/글 2020(MCP engine 2020)으로 출력한 `pdf/issue7702/empty_host_indent_*-2020.pdf`
//! 의 글자 기준선(PyMuPDF origin, 96dpi px)에서 얻은 줄 간격이다 — 표 첫 행(`1984.`) → 마지막 행(`1976.`)
//! → 다음 표 첫 행 또는 표지. 허용 오차 0.5px 는 PDF 좌표와 글자 원점 반올림을 합친 값이다.

use rhwp::document_core::DocumentCore;

const TOLERANCE_PX: f64 = 0.5;

struct Fixture {
    name: &'static str,
    bytes: &'static [u8],
    /// 한/글 2020 기준선 간격(px): `1984.` → `1976.` → (다음 `1984.` | `표지`) …
    gaps: &'static [f64],
}

const ABSORB: f64 = 25.89;

const FIXTURES: &[Fixture] = &[
    Fixture {
        name: "a",
        bytes: include_bytes!("../../samples/issue7702/empty_host_indent_a.hwpx"),
        gaps: &[
            53.54, ABSORB, 25.41, 53.54, 26.05, 25.42, 53.54, 45.55, 25.57, 53.54, 45.55, 25.41,
            53.55, 26.05, 25.41, 53.54, 25.89,
        ],
    },
    Fixture {
        name: "b",
        bytes: include_bytes!("../../samples/issue7702/empty_host_indent_b.hwpx"),
        gaps: &[
            53.54, 45.55, 25.57, 53.54, 45.55, 25.41, 53.55, 45.71, 25.41, 53.54, 45.71, 25.41,
            53.54, 45.71, 25.42, 53.54, 45.71,
        ],
    },
    Fixture {
        name: "c",
        bytes: include_bytes!("../../samples/issue7702/empty_host_indent_c.hwpx"),
        gaps: &[
            53.54, 45.55, 25.57, 53.54, 45.55, 25.41, 53.55, 45.71, 25.41, 53.54, 45.71, 25.41,
            53.54, ABSORB,
        ],
    },
    Fixture {
        name: "d",
        bytes: include_bytes!("../../samples/issue7702/empty_host_indent_d.hwpx"),
        gaps: &[
            53.54, ABSORB, 25.41, 53.54, 26.05, 25.42, 53.54, 25.89, 25.41, 53.54, 45.71, 25.42,
            53.54, 25.89,
        ],
    },
    Fixture {
        name: "g",
        bytes: include_bytes!("../../samples/issue7702/empty_host_indent_g.hwpx"),
        gaps: &[
            53.54, 49.38, 53.55, 45.71, 25.41, 53.54, 29.73, 53.54, 45.55, 25.41, 53.54, 49.55,
            53.54, 25.89, 25.41, 53.54, 29.73, 53.54, 25.9,
        ],
    },
];

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

/// 쪽 1 의 글자 줄: (기준선 y, x 순으로 이은 글)
fn text_lines(core: &DocumentCore) -> Vec<(f64, String)> {
    assert_eq!(core.page_count(), 1, "합성 문서는 한 쪽이다");
    let svg = core.render_page_svg_native(0).expect("SVG");
    let mut lines: Vec<(f64, Vec<(f64, String)>)> = Vec::new();
    let mut rest = svg.as_str();
    while let Some(open) = rest.find("<text ") {
        rest = &rest[open..];
        let (Some(tag_end), Some(close)) = (rest.find('>'), rest.find("</text>")) else {
            break;
        };
        let text = rest[tag_end + 1..close].trim().to_string();
        if let (Some((x, y)), false) = (origin(&rest[..tag_end]), text.is_empty()) {
            let y = (y * 100.0).round() / 100.0;
            match lines
                .iter_mut()
                .find(|(line_y, _)| (*line_y - y).abs() < 0.05)
            {
                Some((_, line)) => line.push((x, text)),
                None => lines.push((y, vec![(x, text)])),
            }
        }
        rest = &rest[close..];
    }
    let mut out: Vec<(f64, String)> = lines
        .into_iter()
        .map(|(y, mut glyphs)| {
            glyphs.sort_by(|a, b| a.0.total_cmp(&b.0));
            (y, glyphs.into_iter().map(|(_, t)| t).collect())
        })
        .collect();
    out.sort_by(|a, b| a.0.total_cmp(&b.0));
    out
}

#[test]
fn issue_7702_empty_host_indent_line_goes_below_topbottom_table() {
    let mut failures = Vec::new();
    for fixture in FIXTURES {
        let core = DocumentCore::from_bytes(fixture.bytes)
            .unwrap_or_else(|e| panic!("{} 열기: {e:?}", fixture.name));
        let probes: Vec<(f64, String)> = text_lines(&core)
            .into_iter()
            .filter(|(_, text)| {
                text.starts_with("1984.") || text.starts_with("1976.") || text.starts_with("표지")
            })
            .collect();
        assert_eq!(
            probes.len(),
            fixture.gaps.len() + 1,
            "{}: 탐침 줄 수 — {:?}",
            fixture.name,
            probes.iter().map(|(_, t)| t).collect::<Vec<_>>()
        );
        for (index, (pair, want)) in probes.windows(2).zip(fixture.gaps).enumerate() {
            let got = pair[1].0 - pair[0].0;
            if (got - want).abs() > TOLERANCE_PX {
                failures.push(format!(
                    "{} #{index} {:?} → {:?}: rhwp {got:.2}px, 한/글 {want:.2}px",
                    fixture.name, pair[0].1, pair[1].1
                ));
            }
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}
