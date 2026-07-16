//! Issue #2277 (C2a, #1431 Track C): stock(주식형) 2종 렌더 커버리지 + HLC 표현 회귀 가드.
//!
//! stock 2종(고가저가종가=HLC, 시가고가저가종가=OHLC)은 파서가 `c:stockChart`를
//! 인식하지 못해 `chart_type=Unknown` → "차트 (미지원)" placeholder로 렌더되던
//! 코퍼스 마지막 미지원 종류. 파서(`stockChart`/`hiLowLines`/`upDownBars`/계열 내부
//! `c:marker`/`c:symbol`)와 렌더러(`render_stock`: 고저선/캔들/종가 마커/전용 +1 step
//! 축)를 추가해 정답지(`pdf/chart/기타/*-2022.pdf`)와 정합하게 그리도록 한 회귀 가드.
//!
//! 검증: 2종 × (hwp, hwpx) = 4파일 각각 page 0 SVG가
//!   - "차트 (미지원)" placeholder **미포함** + `hwp-ooxml-chart"` 포함
//!   - 축 `>80<` (데이터 max 59 → stock 전용 무조건 +1 step 헤드룸, 정답지 0~80 step 20)
//!   - 고저선 `hwp-stock-hilow` 4개 (카테고리당 1)
//!   - 종가 마커 `hwp-chart-marker` 4개 (시/고/저는 `c:symbol val="none"` → 무마커)
//!   - OHLC만 캔들 `hwp-stock-candle` 4개 (하락 1 = 진회색 채움)

use std::fs;
use std::path::Path;

/// stock 2종 (samples/chart 하위 상대경로, 확장자 제외)
const HLC_STEM: &str = "기타/고가저가종가";
const OHLC_STEM: &str = "기타/시가고가저가종가";

fn render_page0_svg(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    let mut doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {:?}", rel, e));
    doc.render_page_svg(0)
        .unwrap_or_else(|e| panic!("render {}: {:?}", rel, e))
}

#[test]
fn stock_charts_render_without_placeholder() {
    for stem in [HLC_STEM, OHLC_STEM] {
        for ext in ["hwpx", "hwp"] {
            let rel = format!("samples/chart/{stem}.{ext}");
            let svg = render_page0_svg(&rel);

            assert!(
                !svg.contains("차트 (미지원)"),
                "{rel}: '차트 (미지원)' placeholder가 남아있음 (stock 렌더 누락)",
            );
            assert!(
                svg.contains("hwp-ooxml-chart\""),
                "{rel}: 정상 차트(hwp-ooxml-chart) 미렌더",
            );
            assert!(
                !svg.contains("hwp-ooxml-chart-fallback"),
                "{rel}: fallback 차트가 렌더됨",
            );
            assert!(
                svg.contains(">80<"),
                "{rel}: stock 축 max 80 미생성 (전용 +1 step 헤드룸 — 정답지 0~80)",
            );
            assert_eq!(
                svg.matches("hwp-stock-hilow").count(),
                4,
                "{rel}: 고저선은 카테고리당 1 (4개)",
            );
            assert_eq!(
                svg.matches("hwp-chart-marker").count(),
                4,
                "{rel}: 종가 마커만 4개 (시/고/저 무마커)",
            );
        }
    }
}

#[test]
fn stock_legend_swatches_blank_except_close_glyph() {
    // 정답지 실측: 시/고/저 라벨은 스와치 없음(빈 칸), 종가만 마커 글리프(HLC ▲/OHLC ×)
    // — stage4 SwatchKind(Blank/GlyphOnly). 글리프는 별도 클래스 hwp-legend-glyph.
    for stem in [HLC_STEM, OHLC_STEM] {
        for ext in ["hwpx", "hwp"] {
            let rel = format!("samples/chart/{stem}.{ext}");
            let svg = render_page0_svg(&rel);
            assert_eq!(
                svg.matches("hwp-legend-glyph").count(),
                1,
                "{rel}: 종가만 범례 글리프",
            );
        }
    }
}

#[test]
fn ohlc_renders_candles_hlc_does_not() {
    for ext in ["hwpx", "hwp"] {
        let hlc = render_page0_svg(&format!("samples/chart/{HLC_STEM}.{ext}"));
        assert_eq!(
            hlc.matches("hwp-stock-candle").count(),
            0,
            "HLC({ext})는 캔들 없음 (hiLowLines만)",
        );

        let ohlc = render_page0_svg(&format!("samples/chart/{OHLC_STEM}.{ext}"));
        assert_eq!(
            ohlc.matches("hwp-stock-candle").count(),
            4,
            "OHLC({ext}): 시가↔종가 캔들 카테고리당 1",
        );
        assert!(
            ohlc.contains("#404040"),
            "OHLC({ext}): 하락 캔들 진회색 채움 (정답지 1월 하락)",
        );
    }
}
