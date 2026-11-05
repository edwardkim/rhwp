//! 관계형 및 고정형 회귀: 한컴 덧말·본문의 표시 내용 보존. 실물 절대 배치 좌표를 고정하지 않는다. 같은 원본의 독립 한컴 Print 전체 1쪽과 대응 내용 보존을 함께 검사한다.
//! 주석 갱신: 2026-11-06T08:25:01+09:00 (Asia/Seoul).
//! 직전 파일 변경: 5172f2d55c73b2ed33d4d3657d81e769bc9004c2, 작성 2026-11-06T08:13:58+09:00, 커밋 2026-11-06T08:13:58+09:00.
//! 측정일: 2026-11-06 KST. 제품 source 86709ac16711820e61baa3b87e2c4ddaac23573e.
//! Print 프로필·96 DPI·PDFium·색상 경계 보정 전 2px raw. 독립 PDF와 전체 쪽수가 같다.
//! 대응 실문서·API 저장본의 전쪽 결과이며 입력 개입·합성 분기의 전수 시각 측정으로 주장하지 않는다.
//! hancom-mac-dutmal.hwp (8616-hancom-mac-dutmal-hwp-2022): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! hancom-mac-dutmal.hwpx (8616-hancom-mac-dutmal-hwpx-2022): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! 36389301_결재문서본문_직장훈련계획_덧말.hwpx (8616-opengov-ruby): 전 2쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! 원본·PDF SHA, 명령·원본별 전쪽 TSV·정밀 raw·대표 PNG:
//! mydocs/pr/assets/semanticist21_20261105/validation-r6/README.md 및 measurements.md.
//! 원본 형식 계약:
//! 한컴 Mac 12.30이 만든 덧말 문서를 그린다.
//!
//! `samples/dutmal/hancom-mac-dutmal.*`는 한컴이 만든 같은 문서의 HWP·HWPX와 HWP의 Print PDF다.
//! 본말(漢字·한·대한민국만세·글자·스타일)과 덧말이 모두 쪽에 그려져야 한다. 종전에는 둘 다 빠졌다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

fn page_text(path: &str) -> String {
    let path = format!("{}/{path}", env!("CARGO_MANIFEST_DIR"));
    let core = DocumentCore::from_bytes(&std::fs::read(&path).unwrap()).unwrap();
    text_in_first_page(&core)
}

fn text_in_first_page(core: &DocumentCore) -> String {
    let svg = core.render_page_svg_native(0).unwrap();
    let mut text = String::new();
    let mut rest = svg.as_str();
    while let Some(start) = rest.find("<text ") {
        rest = &rest[start..];
        let end = rest.find('>').unwrap();
        let close = rest.find("</text>").unwrap();
        text.push_str(&rest[end + 1..close]);
        rest = &rest[close + 7..];
    }
    text
}

#[test]
fn hancom_ruby_main_and_ruby_text_are_painted() {
    for ext in ["hwp", "hwpx"] {
        let path = format!(
            "{}/samples/dutmal/hancom-mac-dutmal.{ext}",
            env!("CARGO_MANIFEST_DIR")
        );
        let core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(core.page_count(), 1, "{ext}: 독立 한컴 Print 전체 쪽수");
        let text = text_in_first_page(&core);
        for word in [
            "漢字",
            "한자",
            "한국어사전",
            "대한민국만세",
            "대한",
            "글자",
            "덧",
            "스타일",
            "차례제목",
        ] {
            // 글자마다 따로 그리므로 글자 차례로 찾는다.
            let mut rest = text.as_str();
            for ch in word.chars() {
                let at = rest
                    .find(ch)
                    .unwrap_or_else(|| panic!("{ext}: {word}의 {ch} 없음"));
                rest = &rest[at + ch.len_utf8()..];
            }
        }
    }
}

#[test]
fn opengov_ruby_main_text_is_painted() {
    let text = page_text("samples/hwpx/opengov/36389301_결재문서본문_직장훈련계획_덧말.hwpx");
    // 본말 "팀단위 훈련"의 '위'와 덧말 "전술훈련 30% + 현지훈련 20%"의 '술'·'지'는 이 쪽의 다른 글에 없다.
    for ch in ['위', '술', '지'] {
        assert!(text.contains(ch), "{ch} 없음");
    }
}
