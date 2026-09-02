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
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
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

// ── [#6053] 계열 역할은 위치가 아니라 그리기 장치가 정한다 ──────────────────
//
// 예전 `render_stock` 은 3계열=고/저/종, 4계열=시/고/저/종으로 역할을 순서에 고정하고
// 그 밖의 계열 수는 `render_line` 으로 폴백했다. 한컴 실측이 그 규약을 반증한다 —
// 5계열에서도 캔들·고저선이 남고, 고저선은 **추가 계열의 낮은 값까지** 내려온다.
//
// 정답지(이미 커밋된 자산):
//   문서  samples/issue6037/engine/시가고가저가종가-중간계열추가.{hwp,hwpx}
//   한컴  pdf/issue6037/engine/시가고가저가종가-중간계열추가{,-hwpx}.pdf
//   원장  samples/issue6037/MANIFEST-engine.json — verdict "반영",
//         "끝 계열(종가) 유지 — 캔들이 살아 있고 추가계열이 마커로 붙음"
//
// 계열: 시가[44,22,21,33] 고가[55,57,57,59] 저가[11,12,13,21]
//       추가계열[11,11,11,11] 종가[32,35,34,35]   + hiLowLines + upDownBars
const FIVE_SERIES_STEM: &str = "samples/issue6037/engine/시가고가저가종가-중간계열추가";

#[test]
fn stock_five_series_keeps_candles_and_hilow() {
    for ext in ["hwpx", "hwp"] {
        let rel = format!("{FIVE_SERIES_STEM}.{ext}");
        let svg = render_page0_svg(&rel);

        assert!(
            !svg.contains("차트 (미지원)"),
            "{rel}: placeholder 로 떨어지면 안 된다",
        );
        assert_eq!(
            svg.matches("hwp-stock-hilow").count(),
            4,
            "{rel}: 고저선 카테고리당 1 — 선형 폴백이면 0 이다",
        );
        assert_eq!(
            svg.matches("hwp-stock-candle").count(),
            4,
            "{rel}: 캔들은 첫(시가)↔끝(종가) 이므로 계열이 5개여도 선다",
        );
        assert!(
            svg.contains("#404040"),
            "{rel}: 1월은 시가 44 → 종가 32 하락이라 진회색 채움",
        );
        // 추가계열·종가 둘 다 `c:symbol` 미지정(Auto) → 카테고리당 하나씩, 시/고/저는 무마커.
        assert_eq!(
            svg.matches("hwp-chart-marker").count(),
            8,
            "{rel}: 종가 4 + 추가계열 4",
        );
        // [#6053] 정체 경로 산출 — 추가계열만 기본 스타일(`c:spPr` 없음 = 기본 선)이고
        // 나머지 4계열은 코퍼스의 `a:ln > a:noFill` 을 지킨다. 그래서 계열 선은 추가계열
        // 하나뿐이다 — 한컴 편집기가 더한 계열이 선+마커로 보이는 것과 같은 모양이다.
        // (위치 기반 시절의 산출은 전건 noFill 상속이라 선 0 이었다 — 원장 재판정 대기.)
        assert_eq!(
            svg.matches(r#"stroke-width="2""#).count(),
            1,
            "{rel}: 기본 스타일은 추가계열 하나 — 계열 선 1",
        );
    }
}

#[test]
fn stock_corpus_series_draw_no_lines() {
    // [#6053] 코퍼스 주식형은 계열 전건이 `c:spPr > a:ln > a:noFill` 이라 선이 없다.
    // 선 표시 필드의 기본값이 뒤집히면(표기 없음 = 선 있음) 여기서 바로 드러난다.
    for stem in [HLC_STEM, OHLC_STEM] {
        for ext in ["hwpx", "hwp"] {
            let rel = format!("samples/chart/{stem}.{ext}");
            let svg = render_page0_svg(&rel);
            assert_eq!(
                svg.matches(r#"stroke-width="2""#).count(),
                0,
                "{rel}: 고저선·캔들이 그림을 만들고 계열 선은 없다",
            );
        }
    }
}

#[test]
fn stock_hilow_spans_all_series_not_positional_pair() {
    // [#6053] 고저선의 아래 끝이 「전 계열 최소」인지 값 좌표로 못 박는다.
    //
    // `고가저가종가-꼬리계열추가` = [고가 55·57·57·59, 저가 11·12·13·21, 종가 32·35·34·35,
    // 계열4 11·14·13·12] (hiLowLines 만). 계열이 4개라 **옛 규약도 폴백하지 않고** `(hi=1, lo=2)`
    // = 저가↔종가를 집어 1월을 11↔32 로 그렸다. 전 계열 최소↔최대는 11↔55 다.
    let rel = "samples/issue6037/고가저가종가-꼬리계열추가.hwpx";
    let svg = render_page0_svg(rel);

    assert_eq!(
        svg.matches("hwp-stock-hilow").count(),
        4,
        "{rel}: 고저선 카테고리당 1",
    );
    // 축·플롯 기하에 기대지 않는다 — 값축은 선형이므로 **카테고리별 고저선 길이의 비율**이
    // 곧 (max-min) 의 비율이다. 두 규칙이 내는 비율이 뚜렷이 다르므로 이것으로 가른다.
    //
    //   전 계열 min↔max : [11↔55, 12↔57, 13↔57, 12↔59] = 44, 45, 44, 47
    //   옛 (hi=1, lo=2) : 저가↔종가                     = 21, 23, 21, 14
    let attr = |src: &str, key: &str| -> f64 {
        let pat = format!("{key}=\"");
        let at = src
            .find(&pat)
            .unwrap_or_else(|| panic!("{rel}: {key} 없음"));
        let rest = &src[at + pat.len()..];
        rest[..rest.find('"').unwrap()].parse().unwrap()
    };
    let spans: Vec<f64> = svg
        .split("hwp-stock-hilow")
        .skip(1)
        .map(|seg| (attr(seg, "y1") - attr(seg, "y2")).abs())
        .collect();
    assert_eq!(spans.len(), 4, "{rel}: 고저선 4개");

    let unit = spans[0] / 44.0; // 픽셀/값 — 첫 칸을 전 계열 규칙으로 읽어 눈금을 세운다
    for (ci, want) in [44.0_f64, 45.0, 44.0, 47.0].into_iter().enumerate() {
        assert!(
            (spans[ci] - want * unit).abs() < 0.05,
            "{rel}: cat{ci} 고저선이 전 계열 최소↔최대({want}단위)가 아니다 — \
             실측 {:.2}단위. 옛 위치 매핑(저가↔종가)이면 [21,23,21,14] 비율이 나온다",
            spans[ci] / unit,
        );
    }
    // 위 루프만으로도 옛 규약은 cat3 에서 갈리지만(47 vs 14), 비율 척도 자체가 뒤집히는 것을
    // 한 번 더 못 박는다 — 전 계열 규칙은 cat3 이 가장 길고, 옛 규약은 가장 짧다.
    assert!(
        spans[3] > spans[0],
        "{rel}: cat3(12↔59)이 cat0(11↔55)보다 길어야 한다 — {:.2} vs {:.2}",
        spans[3],
        spans[0],
    );
}
