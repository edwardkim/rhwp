//! Issue #3307 회귀 가드 — 정의 없는 개요 자동번호의 한컴 기본 모양 fallback.
//!
//! 국민참여입법센터 87774 첨부(별지 제1호서식 확인서)는 개요 번호를 쓰면서 개요
//! 모양 정의가 없다(`<hh:numbering>` 0개, `outlineShapeIDRef="0"`). 한컴 2020 은
//! 편집기 내장 기본 모양(전 수준 `^N` — 레벨 경로 + 후행 마침표)을 적용해 1.~4.
//! 를 렌더하지만, rhwp 는 정의 부재로 번호를 그리지 않았다(외부 리포트 #3307).
//! 기본 모양은 한컴 2020 MCP 수준 스윕 실측으로 확정했다(Stage 1 보고).
//!
//! 수정: 개요 문단이 유효한 numbering 정의에 도달하지 못하면
//! `default_outline_numbering()`(전 수준 `^N`) 으로 fallback. NUMBER/BULLET 불변.

use rhwp::wasm_api::HwpDocument;

const FIXTURE: &str = "samples/task3307/issue3307_outline_number.hwpx";

fn page_text(doc: &HwpDocument, page: u32) -> String {
    let svg = doc.render_page_svg(page).unwrap();
    let mut out = String::new();
    let mut rest = svg.as_str();
    while let Some(start) = rest.find("<text") {
        let Some(open_end) = rest[start..].find('>') else {
            break;
        };
        let after = &rest[start + open_end + 1..];
        let Some(close) = after.find("</text>") else {
            break;
        };
        out.push_str(&after[..close]);
        rest = &after[close..];
    }
    // SVG에서 꺾쇠는 XML 엔티티로 저장되므로 실제 문단 문자로 되돌린다.
    let text: String = out.chars().filter(|c| !c.is_whitespace()).collect();
    text.replace("&lt;", "<").replace("&gt;", ">")
}

#[test]
fn undefined_outline_gets_hancom_default_numbers() {
    let bytes = std::fs::read(FIXTURE).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();

    assert_eq!(
        doc.page_count(),
        9,
        "쪽수는 한컴 정답지와 같은 9가 유지되어야 한다"
    );

    let p7 = page_text(&doc, 6);

    // 한컴 2020 정답지 p7 — 개요 자동번호(1.~4.)와 리터럴(5.~6.) 전부 존재.
    for (needle, label) in [
        ("1.인적사항", "1. 인적사항 (개요 자동번호)"),
        ("2.비위유형", "2. 비위 유형 (개요 자동번호)"),
        ("3.징계부가금", "3. 징계부가금 (개요 자동번호)"),
        ("4.감경대상", "4. 감경 대상 (개요 자동번호)"),
        ("5.혐의자", "5. 혐의자… (리터럴 — 불변 확인)"),
        ("6.그밖의사항", "6. 그 밖의 사항 (리터럴 — 불변 확인)"),
    ] {
        assert!(p7.contains(needle), "{label} 이 p7 에 없다");
    }

    // 과발동 가드: 개요가 아닌 리터럴 번호 문단(p1 의 "1. 개정이유" 등)에 자동번호가
    // 중복으로 붙으면 "1.1." 류 이중 번호가 생긴다 — 없어야 한다.
    let p1 = page_text(&doc, 0);
    assert!(
        p1.contains("1.개정이유") && !p1.contains("1.1.개정이유"),
        "비개요 문단에 기본 개요 번호가 과발동했다: {}",
        &p1[..p1.len().min(120)]
    );

    // 한컴 2020 기준 출력은 3번 항목의 양쪽 셀을 함께 9쪽에서 시작한다.
    // 짧은 첫 유닛만 8쪽에 떼어 놓으면 왼쪽 <신 설>과 오른쪽 수당 본문이 갈린다.
    let p8 = page_text(&doc, 7);
    let p9 = page_text(&doc, 8);
    assert!(
        !p8.contains("3.<신설>"),
        "3번 항목의 왼쪽 셀이 8쪽에 남았다"
    );
    assert!(
        !p8.contains("공무원수당"),
        "3번 항목의 오른쪽 셀이 8쪽에 남았다"
    );
    assert!(p9.contains("3.<신설>"), "3번 항목의 왼쪽 셀이 9쪽에 없다");
    assert!(
        p9.contains("공무원수당"),
        "3번 항목의 오른쪽 셀이 9쪽에 없다"
    );
}
