# 단계 1 완료 보고서 — 진단 및 분기점 확정 (M100 #977)

- 이슈: edwardkim/rhwp#977
- 브랜치: `local/task977`
- 단계: 1/3 (진단)

## 결론 요약

분기점은 **`compute_char_positions`의 플랫폼별 이중 구현**이다.

- 네이티브: `EmbeddedTextMeasurer` (내장 메트릭 + 휴리스틱)
- WASM: `WasmTextMeasurer` (브라우저 `measureText` 폴백)

두 측정기가 **미등록 폰트의 공백 글자 폭**을 다르게 산출하여, 인접 목차 문단의 선두 공백 폰트가 다를 때 WASM 경로에서만 개요번호가 어긋난다. **소스 정정은 하지 않았다** (진단 전용 단계).

## 진단 과정

### 1. 렌더 경로 확정

- rhwp-studio는 `renderPageToCanvas` → `WebCanvasRenderer`(`src/renderer/web_canvas.rs`)로 렌더
- (`getCanvasKitReplayPlan`/`text_replay.rs`는 `native-skia` `export-png` 전용 — studio 경로 아님)

### 2. `bbox.x`는 정상

`RHWP_RENDER_PATH=layer-svg`로 PageLayerTree를 SVG 출력 → 개요번호 정렬 정상 (legacy SVG와 동일 좌표).
→ `PageLayerTree`의 TextRun `bbox.x`는 올바름. `LayerBuilder`·render tree 구성은 무결.
`WebCanvasRenderer`와 `SvgLayerRenderer`는 동일 `PaintOp::TextRun { bbox }`를 소비 (web_canvas.rs:858, svg_layer.rs:129) → `node.bbox.x` 동일.

### 3. 캔버스 실측 (rhwp-studio, 줌 110%)

`fillText` 후킹으로 캔버스 사용자 좌표(x) 측정:

| 문단 | 선두 공백 CharShape | 캔버스 digit x | SVG digit x |
|------|--------------------|----------------|-------------|
| `1. 업무현황` | id=946 (나눔바른고딕, 100%) | **105.6** | 115.6 |
| `2. 시스템 현황` | id=949 (맑은 고딕, 95%) | 114.6 | 114.6 |
| `3. 당면과제…` | id=949 (맑은 고딕, 95%) | 114.6 | 114.6 |
| 그 외 Ⅰ·Ⅲ·Ⅳ 하위 항목 | id=946 류 | 105.6 | 115.6 |

→ 실제로는 **id=946(나눔바른고딕) 선두 공백 문단의 개요번호가 ~9px 좌측으로 그려진다.** id=949 문단은 SVG와 일치(정상). 다수가 좌측으로 밀려, 소수의 정상 항목(`2.`,`3.`)이 우측으로 튀어나온 것처럼 보인다.

캔버스 텍스트는 글자마다 advance가 SVG보다 좁아 누적 압축됨 (선두 공백이 같은 run에 포함된 문단에서 첫 가시 글자가 좌측으로 밀림).

### 4. 분기점 확정 — `compute_char_positions` 이중 구현

`src/renderer/layout/text_measurement.rs`:

```
993  #[cfg(target_arch = "wasm32")]      fn default_measurer() -> WasmTextMeasurer
996  #[cfg(not(target_arch = "wasm32"))] fn default_measurer() -> EmbeddedTextMeasurer
```

공백 글자 폭 산출:

- **EmbeddedTextMeasurer** (native, line 357~): `measure_char_width_embedded` 미스 시 → `font_size * 0.5` (폰트 무관 상수)
- **WasmTextMeasurer** (WASM, line 846~): `measure_char_width_hwp` → 내장 메트릭 미스 시 → `cached_js_measure` = 브라우저 `measureText` 실측 (폰트 의존)

나눔바른고딕·맑은 고딕의 공백 글자가 내장 메트릭 DB에 없어 폴백 경로로 빠진다.

- 네이티브: 두 폰트 공백 모두 `font_size*0.5` → **동일** → 정렬 (PDF 정합)
- WASM: 폰트별 브라우저 실측 → 나눔바른고딕 공백이 맑은 고딕보다 ~좁게 측정 → 선두 공백 폭 불일치 → **개요번호 어긋남**

한컴 PDF(정답지)는 정렬 → 네이티브(상수 폭) 동작이 정답에 부합, WASM 실측이 이탈.

## 단계 2 정정 방향 (검토 필요)

핵심: WASM `compute_char_positions`의 미등록 폰트 공백 폭이 네이티브와 일치해야 한다.

후보 (단계 2 착수 전 작업지시자 판단 요청):

- **(A)** `WasmTextMeasurer`의 공백(또는 미등록 폰트 폴백)을 `EmbeddedTextMeasurer`와 동일 규칙(`font_size*0.5` 등)으로 통일
- **(B)** 공백 글자에 한해 JS 실측을 건너뛰고 상수 폭 적용
- **(C)** 두 측정기의 폴백 로직을 공용 함수로 추출해 구조적으로 일치 보장

회귀 위험: 측정기 변경은 줄바꿈·탭·정렬 전반에 영향. 광범위 샘플 + 한컴 정답지 검증 필수 ([[feedback_essential_fix_regression_risk]], [[feedback_rule_not_heuristic]]).

## 승인 요청

단계 1(진단) 완료. 단계 2 정정 방향((A)/(B)/(C)) 선택과 함께 승인을 요청합니다.
