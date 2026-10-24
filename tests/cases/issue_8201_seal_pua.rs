//! [#8201] 한컴 서명·날인 기호 U+F012B 는 1em 전진의 **타원 안 '인' 한 글자**다.
//!
//! 독립 기준: `samples/issue8201/seal-delegation-form-2020.pdf`(Hwp 2020)와
//! `pdf/복학원서-2022.pdf`(Hwp 2022)는 이 기호를 모두 내장 Haansoft Batang GID 3 한 글리프
//! (`/W` 1000)로 그린다. 이슈 서식의 같은 줄에서 기호 전진은 57, 한글 전진은 58 device unit 으로
//! 같은 1em·자간 −31% 를 받는다. 모양은 타원 테두리 안의 작은 '인'이다.
//!
//! 종전 `(인)` 세 글자 표시 투영은 기호 하나를 약 두 글자 폭으로 늘려 같은 칸 끝 글자를
//! 셀 경계로 밀었다. 이 검사는 화면 절대 좌표가 아니라 **같은 run 안의 전진 관계**와
//! **seal 전진 상자 안의 도형·글자 소속**을 본다.
//!
//! HWP5 원시 `0xA12B` 의 평면 15 정규화는 `issue_5800_hwp5_pua_plane15` 가 지킨다. 그 합성
//! fixture 는 독립 한컴 PDF 가 없어 시각 회귀 신규 추가의 선행 조건을 만족하지 못하므로
//! 여기서 렌더링 회귀로 고정하지 않는다.

use rhwp::renderer::composer::{
    char_overlap_display_text, expand_pua_render_text, pua_to_display_text, pua_to_text_surface,
};
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;
use std::fs;
use std::path::Path;

const ISSUE_FORM: &str = "samples/issue8201/seal-delegation-form.hwp";
const BOKHAK: &str = "samples/복학원서.hwp";

const SEAL: char = '\u{F012B}';
const HWP5_SEAL: char = '\u{A12B}';

fn load(relative: &str) -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    let bytes = fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    HwpDocument::from_bytes(&bytes).unwrap_or_else(|error| panic!("load {relative}: {error:?}"))
}

/// 페이지 좌표계의 텍스트 run 한 개.
struct TextRun {
    x: f64,
    y: f64,
    height: f64,
    chars: Vec<char>,
    positions: Vec<f64>,
}

impl TextRun {
    fn advance(&self, index: usize) -> f64 {
        self.positions[index + 1] - self.positions[index]
    }

    fn index_of(&self, ch: char) -> usize {
        self.chars
            .iter()
            .position(|candidate| *candidate == ch)
            .unwrap_or_else(|| panic!("run 에 {ch:?} 가 없음: {:?}", self.chars))
    }

    /// `index` 글자의 전진 상자 `(left, right, top, bottom)`.
    fn advance_box(&self, index: usize) -> (f64, f64, f64, f64) {
        (
            self.x + self.positions[index],
            self.x + self.positions[index + 1],
            self.y,
            self.y + self.height,
        )
    }
}

fn collect_text_runs(value: &Value, out: &mut Vec<TextRun>) {
    match value {
        Value::Object(op) => {
            if op.get("type").and_then(Value::as_str) == Some("textRun") {
                let bbox = &op["bbox"];
                let number = |value: &Value| value.as_f64().expect("bbox number");
                let positions: Vec<f64> = op["positions"]
                    .as_array()
                    .expect("textRun positions")
                    .iter()
                    .map(number)
                    .collect();
                out.push(TextRun {
                    x: number(&bbox["x"]),
                    y: number(&bbox["y"]),
                    height: number(&bbox["height"]),
                    chars: op["text"].as_str().expect("textRun text").chars().collect(),
                    positions,
                });
            }
            for child in op.values() {
                collect_text_runs(child, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_text_runs(item, out);
            }
        }
        _ => {}
    }
}

fn page_text_runs(doc: &HwpDocument, page: u32) -> Vec<TextRun> {
    let json = doc
        .get_page_layer_tree_native(page)
        .expect("page layer tree");
    let root: Value = serde_json::from_str(&json).expect("layer tree JSON");
    let mut runs = Vec::new();
    collect_text_runs(&root, &mut runs);
    runs
}

fn run_starting_with(doc: &HwpDocument, page: u32, prefix: &str) -> TextRun {
    page_text_runs(doc, page)
        .into_iter()
        .find(|run| run.chars.iter().collect::<String>().starts_with(prefix))
        .unwrap_or_else(|| panic!("{prefix:?} 로 시작하는 textRun 이 없음"))
}

fn runs_containing(doc: &HwpDocument, page: u32, ch: char) -> Vec<TextRun> {
    let runs: Vec<TextRun> = page_text_runs(doc, page)
        .into_iter()
        .filter(|run| run.chars.contains(&ch))
        .collect();
    for run in &runs {
        assert_eq!(
            run.positions.len(),
            run.chars.len() + 1,
            "positions 는 원문 글자 기준이어야 함"
        );
    }
    runs
}

fn run_containing(doc: &HwpDocument, page: u32, ch: char) -> TextRun {
    let mut runs = runs_containing(doc, page, ch);
    assert_eq!(runs.len(), 1, "{ch:?} 를 가진 textRun 은 한 개여야 함");
    runs.remove(0)
}

/// SVG 의 그리기 요소. 좌표는 페이지 px 이다.
#[derive(Debug)]
enum SvgMark {
    Ellipse { cx: f64, cy: f64 },
    Text { x: f64, y: f64, content: String },
}

fn attribute(element: &quick_xml::events::BytesStart<'_>, name: &str) -> Option<String> {
    element
        .attributes()
        .flatten()
        .find(|attribute| attribute.key.as_ref() == name)
        .map(|attribute| attribute.value.to_string())
}

fn number_attribute(element: &quick_xml::events::BytesStart<'_>, name: &str) -> f64 {
    attribute(element, name)
        .and_then(|value| value.parse().ok())
        .unwrap_or(0.0)
}

/// `<text transform="matrix(a b c d e f)" x y>` 의 실제 원점.
fn text_origin(element: &quick_xml::events::BytesStart<'_>) -> (f64, f64) {
    let x = number_attribute(element, "x");
    let y = number_attribute(element, "y");
    let Some(matrix) = attribute(element, "transform")
        .as_deref()
        .and_then(|value| value.strip_prefix("matrix("))
        .and_then(|value| value.strip_suffix(')'))
        .map(|value| {
            value
                .split([' ', ','])
                .filter(|part| !part.is_empty())
                .map(|part| part.parse::<f64>().expect("matrix number"))
                .collect::<Vec<_>>()
        })
    else {
        return (x, y);
    };
    let [a, b, c, d, e, f] = matrix[..] else {
        panic!("matrix 는 6개 값이어야 함: {matrix:?}");
    };
    (a * x + c * y + e, b * x + d * y + f)
}

fn svg_marks(svg: &str) -> Vec<SvgMark> {
    let mut reader = quick_xml::Reader::from_str(svg);
    let mut marks = Vec::new();
    let mut open_text: Option<(f64, f64, String)> = None;
    loop {
        match reader.read_event().expect("parse SVG") {
            quick_xml::events::Event::Empty(element) | quick_xml::events::Event::Start(element)
                if element.name().as_ref() == "ellipse" =>
            {
                marks.push(SvgMark::Ellipse {
                    cx: number_attribute(&element, "cx"),
                    cy: number_attribute(&element, "cy"),
                });
            }
            quick_xml::events::Event::Start(element) if element.name().as_ref() == "text" => {
                let (x, y) = text_origin(&element);
                open_text = Some((x, y, String::new()));
            }
            quick_xml::events::Event::Text(text) => {
                if let Some((_, _, content)) = open_text.as_mut() {
                    content.push_str(text.as_ref());
                }
            }
            quick_xml::events::Event::End(element) if element.name().as_ref() == "text" => {
                if let Some((x, y, content)) = open_text.take() {
                    marks.push(SvgMark::Text { x, y, content });
                }
            }
            quick_xml::events::Event::Eof => break,
            _ => {}
        }
    }
    marks
}

/// seal 전진 상자 안의 도형·글자가 한컴 글리프(타원 + 안쪽 '인')와 같은 구성인지 본다.
fn assert_seal_box_is_circled_in(doc: &HwpDocument, page: u32, run: &TextRun, index: usize) {
    let svg = doc.render_page_svg_native(page).expect("page SVG");
    assert!(
        !svg.contains(SEAL) && !svg.contains(HWP5_SEAL),
        "SVG 에 원 PUA 코드포인트가 글자로 남으면 안 됨"
    );
    let (left, right, top, bottom) = run.advance_box(index);
    let inside = |x: f64, y: f64| x >= left && x <= right && y >= top && y <= bottom;
    let marks = svg_marks(&svg);
    let ellipses: Vec<_> = marks
        .iter()
        .filter(|mark| matches!(mark, SvgMark::Ellipse { cx, cy } if inside(*cx, *cy)))
        .collect();
    let texts: Vec<&str> = marks
        .iter()
        .filter_map(|mark| match mark {
            SvgMark::Text { x, y, content } if inside(*x, *y) => Some(content.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(
        ellipses.len(),
        1,
        "seal 전진 상자 [{left:.2}, {right:.2}] 안에 타원 테두리 한 개가 있어야 함. texts={texts:?}"
    );
    assert_eq!(
        texts,
        ["인"],
        "seal 전진 상자 안에는 안쪽 '인' 한 글자만 있어야 함(괄호 없음)"
    );
}

#[test]
fn issue_form_seal_advances_like_the_hangul_beside_it() {
    let doc = load(ISSUE_FORM);
    let run = run_containing(&doc, 0, SEAL);
    assert_eq!(
        run.chars.iter().collect::<String>(),
        "인감 \u{F012B} 또는 서명",
        "이슈가 지목한 동의 칸 줄이어야 함"
    );
    let seal = run.advance(run.index_of(SEAL));
    let hangul = run.advance(run.index_of('감'));
    assert!(hangul > 0.0, "한글 전진은 양수여야 함");
    assert!(
        (seal - hangul).abs() < 0.01,
        "seal 은 같은 글자모양의 한글과 같은 1em 전진을 받아야 함: seal={seal:.3} 한글={hangul:.3}"
    );
}

#[test]
fn issue_form_seal_is_painted_as_circled_in() {
    let doc = load(ISSUE_FORM);
    let run = run_containing(&doc, 0, SEAL);
    assert_seal_box_is_circled_in(&doc, 0, &run, run.index_of(SEAL));
}

#[test]
fn bokhakwonseo_signature_seal_is_painted_as_circled_in() {
    let doc = load(BOKHAK);
    // 서명란 기호와, 접수증 위 F081C 채움줄에 심긴 기호로 만든 날인선 두 곳이다(#4384).
    let mut runs = runs_containing(&doc, 0, SEAL);
    assert_eq!(
        runs.len(),
        2,
        "서명란과 접수증 날인선에 기호가 하나씩 있어야 함"
    );
    let receipt_index = runs
        .iter()
        .position(|run| run.chars.contains(&'-'))
        .expect("접수증 날인선 run");
    let receipt = runs.remove(receipt_index);
    let run = runs.remove(0);
    assert_eq!(run.chars, [SEAL], "서명란 기호는 단독 run 이어야 함");
    let index = run.index_of(SEAL);
    // `(` 는 다른 글자모양의 별도 run 이다(한컴 PDF 에서도 다른 글꼴로 찍힌다).
    let signature = run_starting_with(&doc, 0, "Signature)");
    let (_, seal_right, _, _) = run.advance_box(index);
    assert!(
        (signature.y - run.y).abs() < 1.0 && signature.x >= seal_right - 0.01,
        "서명란 기호 뒤 같은 줄에 `(Signature)` 가 겹치지 않고 이어져야 함"
    );
    assert_seal_box_is_circled_in(&doc, 0, &run, index);
    // 한컴 PDF 는 날인선 가운데에도 같은 타원 안 '인' 글리프를 찍는다.
    assert_seal_box_is_circled_in(&doc, 0, &receipt, receipt.index_of(SEAL));
}

/// 반례: 텍스트 표면과 글자겹침 안쪽 표시는 종전 읽기 문자열을 유지하고,
/// 인접 PUA 는 합성 대상이 아니다.
#[test]
fn reading_surfaces_keep_the_seal_text_and_neighbours_stay_raw() {
    assert_eq!(pua_to_text_surface("서명 \u{F012B}"), "서명 (인)");
    assert_eq!(pua_to_display_text(SEAL).as_deref(), Some("(인)"));
    assert_eq!(pua_to_display_text(HWP5_SEAL).as_deref(), Some("(인)"));
    assert_eq!(char_overlap_display_text(SEAL, true), "(인)");
    assert_eq!(
        expand_pua_render_text("\u{F012A}\u{F012C}"),
        "\u{F012A}\u{F012C}",
        "근거 없는 인접 PUA 를 같은 기호군으로 추정하면 안 됨"
    );
}
