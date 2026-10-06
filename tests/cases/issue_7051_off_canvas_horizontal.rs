//! [#7051 진단 축] off-canvas 가 가로 축을 보지 않아 용지 밖 글자가 어느 지표에도 안 잡힌다.
//!
//! ## 무엇이 문제였나
//!
//! `deep_vertical_union_bbox` 는 자손까지 합치되 **세로만** 합치고 가로는 자기 bbox 를
//! 그대로 뒀다. 그런데 `TextLine` 의 bbox 폭은 **단 폭**이고, 넘치는 것은 그 자식
//! `TextRun` 이다. 그래서 줄이 단을 238px 넘겨 글자가 용지 밖 124.7px 까지 나가는
//! 문서가 763쪽 내내 `offCanvas = 0` 이었다.
//!
//! ## 가로를 버린 근거와 그 반쪽
//!
//! 종전 주석은 *"`TextRun` 의 말미 공백 advance 등 측정 폭이 쪽 우측을 스치는 무해한
//! 초과가 흔해(표본 100문서 62건 위양성), 가로까지 합치면 검출기의 신호가 잠긴다"* 고
//! 적었다. 그 관찰은 사실이다 — 다만 그 위양성의 정체는 **공백 전진폭**이고, 이 파일은
//! 그걸 걷어내는 기계를 이미 갖고 있다(`glyph_band_bbox`, text-overlap 이 같은 이유로
//! 쓴다). 그 잉크 상자로 합치면 위양성을 피하면서 가로 축이 살아난다.
//!
//! ## 무엇이 드러났나
//!
//! 이 변경은 **진단 전용**이다(`src/diagnostics/` 밖을 건드리지 않는다). 그러니 새로 잡히는
//! 것은 전부 **이전부터 있었는데 안 보이던** 결함이다. `samples` 전수에서 7문서 113건이
//! 드러난다(`off_canvas_baseline.tsv` 델타).
//!
//! ```text
//!   hwp3-sample10-hwp5.hwp / -hwpx.hwpx            각 49건  최대 우측 초과 124.7px
//!   issue6280/156742029_prosecutor_transfer_list    8건
//!   task1725/text_footnote_tail_overpagination     15 -> 16
//!   issue2559/1341000_research_report_footnotes     2건  최대 113.6px
//!   issue6778/156757920-animal-welfare-...          2건  최대 103.9px
//!   hwp3-table-cell-overlap.hwp                     2건
//! ```
//!
//! SVG 글리프로 교차 확인했다 — `156757920` 1쪽은 용지 793.7px 인데 글자가 `x=879.1`
//! 까지 그려진다(**용지 밖 +85.4px**, 그런 글자 10개).
//!
//! ## 표본이 바뀐 이유 (`#7051` 조판 축 수정 뒤)
//!
//! 종전 표본은 `samples/hwp3-sample10-hwp5.hwp` 였고, 이 파일의 비범위 절은 그 문서의
//! 저장 사다리를 *"ASCII 반각(0.508 em)을 가정했지만 우리 메트릭(0.737 em)으로는 재현되지
//! 않는 **부실 저장**(`#2279` 계열)"* 으로 판정했다. **그 판정이 틀렸다.**
//!
//! 한컴 정본이 그 문서를 0.502 em 으로 그린다(MCP engine 2024 / 13.0.0.3901 변환 763쪽,
//! 489쪽 실측 — `issue_7051_hft_hangul_latin_halfwidth.rs` 의 표). 저장 사다리가 맞았고
//! 우리 메트릭이 틀렸다. 조판 축을 고치자(HFT 한글 face 의 ASCII = `em/2`) 그 문서의
//! 가로 초과 49건은 **0건**이 됐다.
//!
//! 그래서 검출 보증의 표본을 같은 델타 목록의 다른 문서로 옮긴다 — 이쪽은 HFT 치환 축이
//! 아니라서 지금도 용지 밖 103.9px 까지 나간다. 검출기 자체의 계약은 그대로다.
//!
//! ## 비범위
//!
//! 새 표본이 왜 넘치는지(그쪽 조판 원인)는 이 파일이 다루지 않는다. 여기서 지키는 것은
//! **가로 초과가 지표에 나타난다**는 검출기 계약 하나다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::diagnostics::layout_anomaly::{scan_document, AnomalyOptions};
use rhwp::document_core::DocumentCore;

/// 가로 초과가 남아 있는 문서 — 1쪽 글자가 용지(793.7px) 밖 103.9px 까지 나간다.
/// 종전 표본(`hwp3-sample10-hwp5.hwp`)은 조판 축 수정으로 49건 → 0건이 됐다(위 절).
const SAMPLE: &str = "samples/issue6778/156757920-animal-welfare-husbandry-guidelines.hwp";

/// 가로 초과가 off-canvas 로 잡힌다.
///
/// 수정 전에는 763쪽 전체가 `offCanvas = 0` 이었다.
#[test]
fn off_canvas_sees_horizontal_overflow() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core =
        DocumentCore::from_bytes(&std::fs::read(&path).expect("정식 원본")).expect("문서 로드");
    let anomalies = scan_document(&core, &AnomalyOptions::default()).expect("layout-anomaly");

    let right: Vec<_> = anomalies
        .pages
        .iter()
        .flat_map(|page| page.off_canvas.iter())
        .filter(|a| a.over_right > 1.0)
        .collect();
    assert!(
        !right.is_empty(),
        "가로(우측) 초과가 하나도 안 잡혔다 — 수정 전 상태다. off-canvas 총 {}건",
        anomalies.off_canvas_count()
    );

    let worst = right.iter().map(|a| a.over_right).fold(0.0_f64, f64::max);
    assert!(
        worst > 100.0,
        "이 문서의 최대 우측 초과는 103.9px 다(SVG 글리프 실측). got {worst:.1}px"
    );
}

/// 세로 축은 종전대로다 — 가로를 켰다고 세로 판정이 달라지면 안 된다.
#[test]
fn vertical_axis_is_unchanged() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core =
        DocumentCore::from_bytes(&std::fs::read(&path).expect("정식 원본")).expect("문서 로드");
    let anomalies = scan_document(&core, &AnomalyOptions::default()).expect("layout-anomaly");
    // 이 문서의 off-canvas 는 전부 가로다 — 세로 초과는 0 이다.
    let vertical: Vec<_> = anomalies
        .pages
        .iter()
        .flat_map(|page| page.off_canvas.iter())
        .filter(|a| a.over_top > 1.0 || a.over_bottom > 1.0)
        .collect();
    assert!(
        vertical.is_empty(),
        "이 문서에 세로 초과는 없어야 한다(가로 전용 결함). got {}건",
        vertical.len()
    );
}
