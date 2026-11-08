//! [#7702] 휴먼명조 ASCII 전진폭과 문단 끝 condense 줄 판정 — 1342000 189쪽 `…실천한` / `다.`.
//!
//! 1342000 교육과정 지도 189쪽 칸 문단(휴먼명조 10pt, 자간 −10%, condense 10%, 양쪽 정렬,
//! 상자 26705 HU)을 한/글 2014 저장 줄도, 한/글 2020·2022·2024 PDF 도 `…실천한` / `다.` 로
//! 끊는다. 수정 전 rhwp 는 #7702 재조판이 `다.` 를 앞 줄에 붙였다.
//!
//! # 기대값의 출처 — 같은 문단을 폭만 바꿔 한/글 2024 에 직접 조판시킨 합성 입력
//!
//! 생성기와 입력·PDF 해시, 변환 job 은
//! `mydocs/tech/investigations/issue-7702/probes/humanmyeongjo-provenance.json` 에 있다.
//! 입력에는 저장 줄(`hp:linesegarray`)이 없어 한/글이 스스로 줄을 잡는다.
//!
//! - `humanmyeongjo_condense_end_*`: 189쪽 문단을 본문 폭 `26705 + d` HU 로 되풀이한다.
//!   한/글 2024 는 d ≤ 6 에서 `다.` 를 셋째 줄로 보내고 d ≥ 7 에서 붙인다(1 HU 간격 21문단).
//!   수정 전 rhwp 는 d ≥ −37 에서 붙였다.
//! - `humanmyeongjo_ascii_*`: 문자마다 `가c` × 20 을 왼쪽 정렬로 둔 사다리(94자 × 10pt 자간
//!   −10% / 10pt / 13pt). 한/글 PDF 글자 원점의 `가c` 주기에서 `가` 를 뺀 장치 단위(4 HU)가
//!   아래 표다. 휴먼명조 TrueType(`HMKMM.TTF`) hmtx 로 셈한 #7702 장치 전진폭과 94/94 같고,
//!   #2430 COM 래더 표는 37·58·70자가 다르다.
//!
//! 수정 전에는 `,` `.` `(` `)` 가 한 단위 좁아(10pt 자간 −10%: 59/69, 한/글 60/70) 189쪽
//! 줄이 16 HU 짧게 재졌다. 나머지 28 HU 는 문단 끝 글자 `.` 의 자간이다 — 한/글은 양쪽 정렬
//! 줄의 끝 글자를 자간 없는 폭으로 상자에 맞추는데(#5678), rhwp 는 #2470 근거로 문단 끝에서
//! 이를 끄고 있었다. 이제 자연폭으로 들어가는 문단 끝 줄만 그 예외를 쓰고, 공백을 줄여야
//! 들어가는 줄은 끝 글자 자간을 빼고 잰다.
//!
//! # 이 검사가 말하지 않는 것
//!
//! 휴먼명조 ASCII 밖(Latin-1 등)의 폭과 다른 크기·장평 조합은 이 사다리로 재지 않았다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::layout::{EmbeddedTextMeasurer, TextMeasurer};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn core(sample: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|error| panic!("parse: {error:?}"))
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

/// 문서 전체의 (문단 번호, 줄 글자) — 줄마다 한 항목.
fn lines(core: &DocumentCore) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("render page");
        let mut nodes = Vec::new();
        walk(&tree.root, &mut nodes);
        for node in nodes {
            if !matches!(node.node_type, RenderNodeType::TextLine(_)) {
                continue;
            }
            let mut para = None;
            let mut text = String::new();
            for child in &node.children {
                if let RenderNodeType::TextRun(run) = &child.node_type {
                    para = para.or(run.para_index);
                    text.push_str(&run.text);
                }
            }
            if let Some(para) = para {
                out.push((para, text));
            }
        }
    }
    out
}

/// 폭 `26705 + d` 문단마다 한/글처럼 `다.` 가 셋째 줄로 가는가.
fn split_by_width(sample: &str) -> Vec<(i32, bool)> {
    let lines = lines(&core(sample));
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let (para, text) = &lines[i];
        if let Some(d) = text.trim().strip_prefix("d=") {
            let d: i32 = d.parse().expect("폭 표지");
            let body: Vec<&str> = lines[i + 1..]
                .iter()
                .take_while(|(p, _)| *p == para + 1)
                .map(|(_, t)| t.as_str())
                .collect();
            assert!(!body.is_empty(), "d={d} 문단 줄");
            assert_eq!(
                body.concat()
                    .chars()
                    .filter(|c| !c.is_whitespace())
                    .collect::<String>(),
                "건강과생활환경(전염성·비전염성질병,기호품및약물의오·남용,영양등)의관계를이해하고,건강한생활습관을실천한다.",
                "d={d} 문단 글자 보존"
            );
            out.push((d, body.len() == 3));
            i += 1 + body.len();
        } else {
            i += 1;
        }
    }
    out
}

#[test]
fn condensed_paragraph_end_line_breaks_where_hancom_2024_does() {
    let fine = split_by_width("samples/issue7702/humanmyeongjo_condense_end_fine.hwpx");
    assert_eq!(fine.len(), 21, "d=0..20 문단");
    let coarse = split_by_width("samples/issue7702/humanmyeongjo_condense_end_coarse.hwpx");
    assert_eq!(coarse.len(), 20, "d=-120..420 문단");
    let mismatched: Vec<String> = fine
        .iter()
        .chain(coarse.iter())
        .filter(|(d, split)| *split != (*d <= 6))
        .map(|(d, split)| {
            format!(
                "d={d}: 한/글 {} / rhwp {}",
                if *d <= 6 {
                    "셋째 줄 `다.`"
                } else {
                    "두 줄"
                },
                if *split {
                    "셋째 줄 `다.`"
                } else {
                    "두 줄"
                }
            )
        })
        .collect();
    assert!(
        mismatched.is_empty(),
        "문단 끝 condense 줄이 한/글 2024 와 다르다:\n{}",
        mismatched.join("\n")
    );
}

/// 한/글 2024 PDF 의 `가c` 주기 − `가` (장치 단위), U+0021..U+007E 순서.
const HANCOM_10PT_M10: &str = "46 112 148 113 169 173 65 70 70 112 112 60 112 60 70 112 112 112 112 112 112 112 112 112 112 60 60 112 112 112 96 175 148 135 144 153 140 131 158 153 58 89 145 130 175 156 161 130 155 147 112 140 152 149 210 160 151 134 72 70 72 82 112 73 114 122 109 114 118 88 116 125 57 59 118 57 179 124 115 121 121 91 93 76 121 121 163 110 115 102 66 46 66 112";
const HANCOM_10PT: &str = "51 125 164 126 188 192 72 78 78 125 125 67 125 67 78 125 125 125 125 125 125 125 125 125 125 67 67 125 125 125 107 194 165 150 160 170 156 146 176 170 64 99 161 145 195 173 179 144 172 163 124 156 169 166 233 178 168 149 80 78 80 91 125 81 127 136 121 127 131 98 129 139 63 66 131 63 199 138 128 135 135 101 103 85 135 135 181 122 128 113 73 51 73 125";
const HANCOM_13PT: &str = "66 163 213 164 244 250 94 102 102 163 163 88 163 88 102 163 163 163 163 163 163 163 163 163 163 88 88 163 163 163 140 253 215 196 208 221 203 189 229 221 84 128 209 188 254 225 232 187 223 212 161 203 220 216 303 231 218 194 104 102 104 118 163 105 165 176 157 165 170 127 168 180 81 86 170 81 259 179 166 175 175 131 133 110 175 175 235 159 166 147 95 66 95 163";

fn ladder_mismatches(sample: &str, hancom: &str, hangul_units: i64) -> Vec<String> {
    let expected: Vec<i64> = hancom.split(' ').map(|v| v.parse().unwrap()).collect();
    assert_eq!(expected.len(), 94);
    let core = core(sample);
    let measurer = EmbeddedTextMeasurer;
    let mut measured = std::collections::BTreeMap::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("render page");
        let mut nodes = Vec::new();
        walk(&tree.root, &mut nodes);
        for node in nodes {
            if !matches!(node.node_type, RenderNodeType::TextLine(_)) {
                continue;
            }
            // 영문·숫자는 한글과 다른 run 으로 갈리므로 줄 안의 절대 원점(run 시작 + 측정 경계)으로 잰다.
            let mut glyphs = Vec::new();
            for child in &node.children {
                let RenderNodeType::TextRun(run) = &child.node_type else {
                    continue;
                };
                if run.text.is_empty() {
                    continue;
                }
                assert_eq!(
                    run.style
                        .font_family
                        .split(',')
                        .next()
                        .map(|f| f.trim_matches(['\'', '"', ' '])),
                    Some("휴먼명조"),
                    "사다리 글자는 휴먼명조: {:?}",
                    run.text
                );
                let positions = measurer.compute_char_positions(&run.text, &run.style);
                for (i, ch) in run.text.chars().enumerate() {
                    glyphs.push((child.bbox.x + positions[i], ch));
                }
            }
            let Some(&(_, probe)) = glyphs.iter().find(|(_, c)| *c != '가') else {
                continue;
            };
            let hangul: Vec<f64> = glyphs
                .iter()
                .filter(|(_, c)| *c == '가')
                .map(|(x, _)| *x)
                .collect();
            if hangul.len() < 5 || !(' '..='~').contains(&probe) {
                continue;
            }
            let period_px = (hangul[hangul.len() - 1] - hangul[0]) / (hangul.len() - 1) as f64;
            measured
                .entry(probe)
                .or_insert(((period_px * 75.0 / 4.0).round() as i64) - hangul_units);
        }
    }
    (0x21u8..=0x7E)
        .zip(expected)
        .filter_map(|(code, want)| {
            let ch = code as char;
            match measured.get(&ch) {
                Some(&got) if got == want => None,
                Some(&got) => Some(format!("{ch:?}: 한/글 {want} / rhwp {got}")),
                None => Some(format!("{ch:?}: 사다리 줄 없음")),
            }
        })
        .collect()
}

#[test]
fn humanmyeongjo_ascii_advances_match_hancom_2024_ladders() {
    let mut mismatched = Vec::new();
    for (sample, hancom, hangul_units, name) in [
        (
            "samples/issue7702/humanmyeongjo_ascii_10pt_m10.hwpx",
            HANCOM_10PT_M10,
            225,
            "10pt 자간 −10%",
        ),
        (
            "samples/issue7702/humanmyeongjo_ascii_10pt.hwpx",
            HANCOM_10PT,
            250,
            "10pt",
        ),
        (
            "samples/issue7702/humanmyeongjo_ascii_13pt.hwpx",
            HANCOM_13PT,
            325,
            "13pt",
        ),
    ] {
        for line in ladder_mismatches(sample, hancom, hangul_units) {
            mismatched.push(format!("{name} {line}"));
        }
    }
    assert!(
        mismatched.is_empty(),
        "휴먼명조 ASCII 전진폭이 한/글 2024 와 다르다:\n{}",
        mismatched.join("\n")
    );
}
