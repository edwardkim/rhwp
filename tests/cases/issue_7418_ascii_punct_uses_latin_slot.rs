//! 전체 피델리티 미달의 차단 검사만 #7445(comment5981655880)로 이관했습니다. 나머지 검사는 유지합니다.
//! [#7418] 영문 슬롯이 옛 한컴 영문 글꼴이면 ASCII 구두점도 한/글이 **영문 슬롯** 글꼴로 재고
//! 그린다.
//!
//! # 무엇이 깨져 있었나
//!
//! rhwp 는 ASCII 구두점을 언어 중립으로 보고 앞 글자 언어를 물려받았다. 한글 뒤 `-` 는 한글
//! 슬롯의 휴먼명조로 재여 9.63px(722 HWPUNIT)가 됐다. 한/글 PDF 에서 같은 `-` 는 영문 슬롯의
//! Palatino Linotype(영문 슬롯 `HCI Poppy` 의 치환)으로 그려지고 전진폭은 ≈6.4px 다.
//!
//! `78494-virtual-convergence-industry-decree` 8쪽 `pi=73` 첫 줄 `  - 해당 사업자는 … 수반되나,
//! 사업` 이 그 3.2px 때문에 `업` 을 213 HWPUNIT 넘겨 `사`/`업` 으로 갈렸다. 문단이 정본 4줄 대신
//! 5줄이 되어 +29px, 그 뒤 빈 문단 `pi=86` 이 혼자 9쪽을 열어 75쪽(정본 74)이 됐다.
//!
//! # 근거
//!
//! 말뭉치 `pdf/` 1,359개에서 한글 뒤 ASCII 구두점의 글꼴을 쟀다. 한글 글꼴이 휴먼명조인 줄에서
//! `(` 2278:17 · `,` 1733:6 · `)` 819:8 · `.` 525:10 · `:` 252:15 · `-` 77:0 (영문 글꼴:한글 글꼴)
//! — 99% 가 영문 슬롯이다. 구두점 뒤 공백은 88% 가 한글 글꼴이라 앞 글자 언어를 그대로 잇는다.
//! 한글 글꼴이 바탕·맑은 고딕이면 대부분 한글 글꼴 그대로다(2404:329 · 622:70) — 그래서 규칙은
//! 영문 슬롯이 옛 한컴 영문 글꼴(`LegacyLatin` 치환 — 이 문서는 휴먼명조 + `HCI Poppy`)일 때만
//! 건다. 반례는 마지막 검사다.
//!
//! # 이 검사가 말하지 않는 것
//!
//! 저장 줄(`LINE_SEG`)을 쓰는 문단의 줄 나눔은 바뀌지 않는다. `samples/` 1,176개 중 쪽 경계가
//! 바뀐 문서는 이 문서 하나다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

const DOC: &str = "samples/issue6776/78494-virtual-convergence-industry-decree.hwpx";
/// 0-based. 한/글 출력 PDF 의 8쪽.
const PAGE: u32 = 7;

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(DOC);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{DOC} 읽기: {e}"));
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

/// 원문 하이픈은 영문 슬롯으로 전진하고, 한글과 런이 나뉘어도 원문 소속을 보존한다.
///
/// 공개 text-layout의 원문 주소와 `charX`를 함께 검사한다. 독립 Print의 하이픈 전진은
/// 글자 크기의 약 0.32배이고, 한글 슬롯의 반각 전진인 약 0.48배보다 작다.
#[test]
fn the_hyphen_in_a_hangul_run_advances_with_latin_slot_metrics() {
    let core = load();
    let layout: serde_json::Value = serde_json::from_str(
        &core
            .get_page_text_layout_native(PAGE)
            .expect("공개 text-layout"),
    )
    .expect("text-layout JSON");
    let runs = layout["runs"].as_array().expect("runs");
    let paragraph: Vec<_> = runs
        .iter()
        .filter(|run| run["secIdx"] == 0 && run["paraIdx"] == 73)
        .collect();
    let source = &core.document().sections[0].paragraphs[73];
    let mut next = 0;
    let mut actual_text = String::new();
    for run in &paragraph {
        assert_eq!(run["charStart"].as_u64(), Some(next), "원문 순서·누락·중복");
        let text = run["text"].as_str().expect("소유 런 내용");
        next += text.encode_utf16().count() as u64;
        actual_text.push_str(text);
    }
    assert_eq!(actual_text, source.text, "원문 문단의 전체 내용 보존");
    let matches: Vec<_> = paragraph
        .iter()
        .filter(|run| run["charStart"] == 2)
        .collect();
    assert_eq!(matches.len(), 1, "원문 하이픈의 소유 런은 유일하다");
    let run = matches[0];
    assert_eq!(run["text"], "-");
    assert_eq!(
        run["fontFamily"], "Palatino Linotype",
        "독립 Print의 영문 슬롯"
    );
    let text: Vec<char> = run["text"].as_str().unwrap().chars().collect();
    let hyphen = text.iter().position(|c| *c == '-').expect("`-`");
    assert_eq!(run["charStart"].as_u64().unwrap() + hyphen as u64, 2);
    assert_eq!(
        run["fontFamily"], "Palatino Linotype",
        "독립 Print의 구두점 face"
    );
    let char_x: Vec<f64> = run["charX"]
        .as_array()
        .expect("charX")
        .iter()
        .map(|x| x.as_f64().expect("문자 경계"))
        .collect();
    let advance = char_x[hyphen + 1] - char_x[hyphen];
    let font_size = run["fontSize"].as_f64().expect("원문 글자 크기");
    let advance_em = advance / font_size;
    assert!(
        (0.275..0.35).contains(&advance_em),
        "`-` 전진/글자 크기 {advance_em:.3} — 독립 Print의 약 0.32em, 한글 반각 약 0.48em"
    );
    let following = paragraph
        .iter()
        .find(|following| following["charStart"] == 3)
        .expect("하이픈 다음의 한글 소유 런");
    assert!(following["text"]
        .as_str()
        .unwrap()
        .starts_with(" 해당 사업자는"));
    assert_eq!(following["fontFamily"], "휴먼명조");
    assert_eq!(
        following["charShapeId"], run["charShapeId"],
        "같은 원문 글자모양"
    );
    assert_eq!(following["y"], run["y"], "하이픈과 뒤 본문은 같은 줄");
    assert!(
        (following["x"].as_f64().unwrap() - run["x"].as_f64().unwrap() - advance).abs() / font_size
            < 0.01,
        "영문 전진 끝에서 한글 본문을 이어 그린다"
    );
}
