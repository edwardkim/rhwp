# Task M100 #2277 단계별 완료보고서 — 2단계: stock 2종 (파서 + render_stock)

- 이슈: #2277 (C2a stock HLC 렌더 + 2D fidelity 정합)
- 브랜치: `local/task2277`
- 단계: 2/5
- 작성일: 2026-07-15

## 핵심 성과

**코퍼스 28종 export 기준 "차트 (미지원)" placeholder 0건 달성** — stock 2종
(고가저가종가=HLC, 시가고가저가종가=OHLC)이 코퍼스 마지막 미지원 종류였음
(#1431 Track C 완료기준의 첫 축).

## 구현 내용

### `src/ooxml_chart/mod.rs` (모델)

- `OoxmlChartType::Stock` variant (label "주식형").
- `OoxmlChart`: `has_hi_low_lines`(고저선)·`has_up_down_bars`(캔들)·
  `up_down_gap_width`(캔들 폭, 미지정 시 렌더러 150 폴백) 추가.
- `SeriesMarker` enum {NotSpecified/None/Auto/Named} + `OoxmlSeries.marker_symbol` —
  계열 내부 `<c:marker>` 상태 보존 (stock 종가 판별용, plot 레벨 `line_markers`와 별개).
- 모듈 doc 지원 범위에 stockChart 추가, 범위 외에서 stock 제거.

### `src/ooxml_chart/parser.rs`

- `b"stockChart"` arm (기존 plot arm 미러) + handle_end plot 종료 리스트에 추가
  (axis_ids 복사).
- `b"hiLowLines"`/`b"upDownBars"`/`b"gapWidth"` arm — 전부 **Stock plot 게이트**
  (hiLowLines는 lineChart에도 올 수 있는 요소, gapWidth는 barChart의 막대 간격과 동명).
- `b"marker"` 핸들러 재구성: 계열 내부(`cur_series.is_some()`) → `marker_symbol=Auto`
  (래퍼만 존재 시), plot 레벨(Line) → 기존 `line_markers` 로직 불변.
- `b"symbol"` arm 신설: `val="none"` → `SeriesMarker::None`(시/고/저 실측),
  그 외 → `Named(val)`.

### `src/ooxml_chart/renderer.rs`

- 라우팅 match에 `Stock => render_stock`.
- `render_stock` 신설:
  - **계열 역할 = XML 순서 규약**: 3계열=고0/저1/종2, 4계열=시0/고1/저2/종3.
    그 외 계열 수 → `render_line` 폴백(placeholder 재발 방지).
  - **축**: `nice_axis_no_headroom + 1 step` **무조건 헤드룸** — 정답지 실측
    max 59 → 0~80 step 20 (`nice_axis` 경계 조건부 +1로는 0~60이라 부족.
    3D 누적세로 "+1 step"과 동형 패턴, 기존 축 헬퍼 재사용 — 새 기계장치 없음).
  - **고저선**: 카테고리 슬롯 중앙에 검정 1px 세로선, class `hwp-stock-hilow`.
  - **캔들**(OHLC): 시가↔종가 rect, 폭 = `cat_span/(1+gapWidth/100)`(실측 150→40%).
    하락(종<시)=진회색 채움(#404040 근사 — 5단계 시각판정에서 픽셀 실측 확정),
    상승·동률=흰 채움+검정 테두리(동률 미실측 — 상승 처리 고정, 주석 명기).
    class `hwp-stock-candle`.
  - **종가 마커**: `marker_symbol ∈ {Auto, Named}` 계열만 `push_marker`(1단계 인프라) —
    HLC 종가 si=2 → ▲ 회색(팔레트3), OHLC si=3 → × 노랑(팔레트4).
    **마커 사이클·팔레트 폴백이 정답지 색·형상을 자동 결정** (신규 색 상수 0개).

### 설계 결정 기록

`SeriesData` 합타입(C1b #1660 보고서 예약) **도입하지 않음** — stock 계열 역할은
순서 규약으로 충분(코퍼스 `c:order` 실측 일치)하고, 합타입은 파서 상태기계·콤보·
scatter 경로 전반 ~200줄+ 파급 대비 코퍼스 이득이 없음 (수행계획서 §4 확정).

## 테스트 (RED→GREEN)

| 테스트 | 단언 | 종별 |
|--------|------|------|
| `test_parse_stock_hlc` | Stock 타입·hiLowLines·3계열·시/고/저 `None`·종가 `Auto`·카테고리 | 신규(파서) |
| `test_parse_stock_ohlc_up_down_bars` | upDownBars·gapWidth 150·4계열 | 신규(파서) |
| `test_parse_stock_no_line_marker_cross_talk` | 계열 내부 marker가 `line_markers` 무오염 | 신규(파서) |
| `test_parse_bar_gap_width_not_captured` | barChart gapWidth는 캔들 폭과 무관(게이트) | 신규(파서) |
| `test_stock_axis_unconditional_headroom` | `>80<`·`>20<` 존재, `>100<` 부재 | 신규(렌더러) |
| `test_stock_hilow_lines_per_category` | 고저선 4, HLC 캔들 0 | 신규(렌더러) |
| `test_stock_ohlc_candles` | 캔들 4 — 하락 1(#404040)·상승 3(흰+검정 테두리) | 신규(렌더러) |
| `test_stock_close_marker_only` | 마커 4개만 — HLC ▲(MLLZ)/OHLC ×(MLML) | 신규(렌더러) |
| `test_stock_unusual_series_count_line_fallback` | 2계열 → 라인 폴백, fallback 클래스 부재 | 신규(렌더러) |
| `tests/issue_2277_stock.rs` (2 테스트) | 2종×hwp/hwpx 4파일: placeholder 부재·`>80<`·고저선 4·마커 4·OHLC만 캔들 4(+하락 진회색) | 신규(통합) |

## TDD 절차 준수

RED (구현 전) — 모델 필드만 추가한 상태에서 7건이 정확한 사유로 실패:

```
test_parse_stock_hlc / _ohlc_up_down_bars       FAILED (stockChart 미인식 → Unknown)
test_stock_axis / _hilow / _ohlc_candles /
_close_marker_only / _unusual_series_count      FAILED (render_stock 부재 → 빈 플롯)
→ 75 passed; 7 failed
```

(통합 issue_2277_stock도 이 시점엔 placeholder 존재로 실패 상태 — 파서/렌더러
구현 후 GREEN.)

GREEN (구현 후):

```
$ cargo test --lib ooxml_chart
test result: ok. 82 passed; 0 failed
$ cargo test --test issue_2277_stock
test result: ok. 2 passed; 0 failed
```

## 검증 결과

- [x] `cargo test --lib ooxml_chart` 82/82
- [x] `tests/issue_2277_stock.rs` 2/2 (2종×hwp/hwpx 4파일)
- [x] 차트 통합 무회귀: issue_2129 / issue_1431_scatter / issue_1882 / issue_1453 전부 통과
- [x] **전체 `cargo test` 전수 (클린 로그 실행): 264스위트 전부 ok — 3,171 passed /
  0 failed, cargo exit 0**
- [x] `cargo clippy --all-targets -- -D warnings` 무경고
- [x] fmt: 수정 4파일 rustfmt 적용(rustfmt.toml 준수), `cargo fmt --check` 지적 0건.
  mod.rs/parser.rs 워킹카피도 LF 정규화(git 저장 내용 무영향 — i/lf 불변)

## 비고

- 빌드 중 "bin `rhwp`와 lib `rhwp` 출력 파일명 동일" 경고는 저장소 기존 구조의
  선재 경고 — 본 작업 무관.
- stock 범례는 현재 색 사각형 스와치(기존 경로) — **4단계(SwatchKind)에서 시/고/저
  빈 스와치 + 종가 글리프로 정합** 예정 (구현계획서 순서).

## 다음 단계

3단계 — 범례 순서: 28 PDF 전수 실측표 작성(보고서 수록) → `legend_order_reversed`
단일 결정 함수 + `render_bars` 가로묶음 슬롯 반전(동일 커밋) +
`tests/issue_2277_legend_order.rs`.
