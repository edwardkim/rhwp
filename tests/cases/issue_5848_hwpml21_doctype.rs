//! [Issue #5848] DOCTYPE 이 붙은 HWPML 을 "알 수 없는 파일 형식"으로 거부하고
//! HWPML 2.1 도 버전 게이트에 막힌다 — 법제처 국가법령정보센터 배포본(08462).
//!
//! 두 겹의 차단: ① `has_hwpml_root` 가 `Event::DocType` 에서 즉시 false →
//! UNSUPPORTED_FILE_FORMAT 오도, ② 버전 게이트가 2.9/2.91 만 수용.
//!
//! 수정: 감지는 DOCTYPE 을 건너뛰고, DOCTYPE 은 **안전한 내부 서브셋**(문자
//! 참조만 담은 일반 엔티티, 외부 식별자·파라미터 엔티티·중첩 참조 거절)만
//! 수용해 본문 `&nbsp;` 를 해석한다. 버전 게이트에 2.1 추가(어휘 2.91 동일).
//! 적대적 DOCTYPE 회귀 가드는 tests/hml_parser.rs 의
//! `rejects_hostile_hml_doctype`.
//!
//! 픽스처는 법제처 배포 원본(126KB, 한글 2022 실측 4쪽 7,428자).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5848/hwpml21_doctype_lawinfo.hwp";

#[test]
fn issue_5848_hwpml21_with_doctype_opens_and_resolves_entities() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample"))
        .expect("DOCTYPE 이 붙은 HWPML 2.1 이 열려야 한다");

    // 한글 2022 실측 4쪽.
    assert_eq!(core.page_count(), 4, "한글 2022 는 4쪽으로 연다");

    // 본문 텍스트가 실제로 실려 있고, &nbsp; 13곳이 U+00A0 으로 해석된다.
    let doc = core.document();
    let mut text = String::new();
    for section in &doc.sections {
        for para in &section.paragraphs {
            text.push_str(&para.text);
        }
    }
    assert!(
        text.contains("재난안전제품"),
        "본문 텍스트가 소실되면 안 된다"
    );
    // 본문 최상위 문단에 7곳 (원본 &nbsp; 13곳 중 나머지 6곳은 표 셀 문단 —
    // export-text 4쪽 합산 13곳 실측).
    let nbsp = text.matches('\u{00A0}').count();
    assert_eq!(nbsp, 7, "내부 엔티티 &nbsp; 가 U+00A0 으로 해석되어야 한다");
    assert!(!text.contains("&nbsp;"), "엔티티가 리터럴로 남으면 안 된다");
}
