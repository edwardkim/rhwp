//! [Issue #5757] 비-TAC 표를 선언 높이보다 12.4px 크게 재 쪽나눔이 나고 2쪽이 제목만
//! 남는다 — 전 문서 +1쪽 (rhwp 58쪽 vs 한글 57쪽, 156739836).
//!
//! 일러두기 3×3 표: Σ셀선언(cellSz) 981.8px > 표 선언높이(hp:sz) 966.2px. 한글 2022
//! 오라클 PDF 괘선 실측은 두 행 모두 ×0.984 균일 축소로 표 전체 965.2px — 표 선언이
//! 권위다. rhwp 는 셀선언 합 + 빈 간격 셀 성장분 983.7px 로 본문 칸(971.3px)을 12.4px
//! 넘겨 불필요한 쪽나눔이 났다.
//!
//! 수정(height_measurer): 비-TAC 표에 **선언끼리 모순 축소** 분기 — Σ셀선언 > 표선언
//! 이고 측정 성장분이 축소 임계(2%) 이하일 때만 표선언으로 비례 축소. 콘텐츠가 행을
//! 실제로 키운 표(#5714 축)는 성장분 조건이 걸러 불변.
//!
//! 픽스처는 원본 1~3쪽 추출 + 대형 BinData 1×1 스텁(65KB, marker-HWPX).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5757/nontac_declared_height_shrink.hwpx";

/// SVG 는 글자 단위 `<text>` 로 방출된다 — 순서대로 이어 붙여 문구를 찾는다.
fn svg_text_concat(svg: &str) -> String {
    let mut out = String::new();
    for cap in svg.split("</text>") {
        if let Some(i) = cap.rfind('>') {
            out.push_str(&cap[i + 1..]);
        }
    }
    out
}

#[test]
fn issue_5757_ilreodugi_table_fits_one_page_at_declared_height() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    // 결함 시 표가 2/3쪽으로 쪼개져 전체가 5→6쪽이 된다 (원문서는 58쪽 vs 한글 57쪽).
    assert_eq!(core.page_count(), 5, "일러두기 표가 쪼개지면 6쪽이 된다");

    // 2쪽: 제목("일 러 두 기")과 본문 첫 줄이 같은 쪽에 있어야 한다.
    let page2 = svg_text_concat(&core.render_page_svg_native(1).expect("page 2 svg"));
    assert!(
        page2.contains("일") && page2.contains("러"),
        "2쪽에 일러두기 제목이 있어야 한다"
    );
    assert!(
        page2.contains("공공부문"),
        "2쪽에 일러두기 본문(공공부문 …)이 함께 있어야 한다 — 결함 시 3쪽으로 밀림"
    );
}
