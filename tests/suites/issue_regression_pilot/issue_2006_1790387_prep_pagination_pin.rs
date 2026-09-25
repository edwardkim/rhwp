//! Issue #2006 — 1790387 HIV PrEP 최종결과보고서 페이지네이션 드리프트 핀.
//!
//! `samples/issue2006/1790387_prep_final_report.hwpx` — 빈 문단에 전면급
//! tac(treat_as_char) 이미지 여러 장이 스택된 프레임 페이지가 많은 정책연구
//! 최종결과보고서. PR #2082(전면 tac 이미지 스택 라인 경계 강제분할)로
//! 130쪽 → 141쪽 (스택 문단 h>1500px 잔여 0).
//!
//! 권위 정답지는 `pdf/issue2006/1790387_prep_final_report-hwp2020-20260814.pdf`
//! 140쪽이다. 이 문서는 header 에서 `KoPub돋움체` 계열을 선언하고, 이 PDF 는
//! **그 글꼴이 설치된 환경의 한/글 2020 출력**이다(`KoPubDotumLight`·
//! `KoPubDotumBold` 내장). Producer 는 `cairo 1.18.0` 인데 이는 PDF writer 일 뿐
//! 조판 엔진이 아니다 — 아래 대조군이 그것을 보인다.
//!
//! 같은 저장소의 `2025 행정업무운영 편람(최종)` 은 엔진과 글꼴 환경을 따로 가른
//! 짝을 갖고 있고, **쪽수는 Producer 가 아니라 글꼴 환경으로 갈린다**(#7394).
//!
//! ```text
//!   Hancom PDF 1.3.0.550 / Hwp 2020 · KoPub 내장    383쪽
//!   cairo 1.18.0                     · KoPub 내장    383쪽   ← 같다
//!   Hancom PDF 1.3.0.550 / Hwp 2022 · KoPub 미내장   384쪽
//!   Hancom PDF 1.3.0.404 (2010)     · KoPub 내장    388쪽
//!   Hancom PDF 1.3.0.404 (2010)     · KoPub 미내장   389쪽
//! ```
//!
//! `pdf/issue2006/1790387_prep_final_report-2022.pdf` 146쪽(Hancom PDF 1.3.0.550 ·
//! Hwp 2022)은 **KoPub 미설치 환경의 대조군**이다. 선언 글꼴이 하나도 안 들어 있고
//! 한글 전진폭 중앙값이 10.200pt 로 140쪽 본(9.000pt)보다 13.3% 넓다 — KoPubDotum
//! 한글 전각 `0.872em` 이 치환 글꼴 `1.0em` 으로 바뀐 폭이다. 본문이 그만큼 넓어져
//! 줄과 쪽이 늘어난 것이므로 비교 기준으로 쓰지 않는다.
//!
//! rhwp 는 시스템 설치와 무관하게 KoPub 전용 폭 표를 `find_metric` 보다 앞세워
//! 조판한다(`src/renderer/font_metrics_data.rs`). 곧 rhwp 가 모델링하는 환경이
//! KoPub 설치 환경이므로 대조군도 140쪽 본이어야 한다.
//!
//! ⚠ 미검증 — 140쪽 본을 만든 변환 경로가 한/글 조판을 그대로 보존한다는 **절차적**
//! 증거는 없다. 위 판정은 전부 산출물 쪽 증거(내장 글꼴·전진폭·대조군 쪽수)다.

use std::fs;
use std::path::Path;

fn page_count_of(rel: &str) -> u32 {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {:?}", rel, e));
    doc.page_count()
}

#[test]
fn prep_1790387_page_count_pin() {
    let pages = page_count_of("samples/issue2006/1790387_prep_final_report.hwpx");
    assert_eq!(
        pages, 140,
        "issue2006 1790387 HWP 2020 MCP 정본 140쪽과 달라짐: 실제 {}쪽",
        pages
    );
}
