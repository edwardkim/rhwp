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

/// 한글 run 안의 `-` 는 영문 슬롯 메트릭으로 전진한다 — run 은 쪼개지 않는다.
///
/// 공개 text-layout 의 `charX` 로 잰다. 한글 슬롯(휴먼명조 반각)으로 재면 9.63px, 영문 슬롯
/// (Palatino Linotype 0.336em × 20px − 자간 3%)이면 ≈6.1px 다. 정본 PDF 의 이 `-` 는 6.4px 전진.
#[test]
fn the_hyphen_in_a_hangul_run_advances_with_latin_slot_metrics() {
    let core = load();
    let layout: serde_json::Value = serde_json::from_str(
        &core
            .get_page_text_layout_native(PAGE)
            .expect("공개 text-layout"),
    )
    .expect("text-layout JSON");
    let run = layout["runs"]
        .as_array()
        .expect("runs")
        .iter()
        .find(|run| {
            run["text"]
                .as_str()
                .is_some_and(|text| text.contains("- 해당 사업자는"))
        })
        .expect("`- 해당 사업자는` 이 든 run — 구두점은 한글 run 안에 남는다");
    let text: Vec<char> = run["text"].as_str().unwrap().chars().collect();
    let hyphen = text.iter().position(|c| *c == '-').expect("`-`");
    let char_x: Vec<f64> = run["charX"]
        .as_array()
        .expect("charX")
        .iter()
        .map(|x| x.as_f64().expect("문자 경계"))
        .collect();
    let advance = char_x[hyphen + 1] - char_x[hyphen];
    assert!(
        (5.5..7.0).contains(&advance),
        "`-` 전진폭 {advance:.2}px — 영문 슬롯이면 ≈6.1px, 한글 슬롯(휴먼명조 반각)이면 9.63px"
    );
}
