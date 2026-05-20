# 최종 결과 보고서 — Skia/WASM 개요번호 정렬 어긋남 정정 (M100 #977)

- 이슈: edwardkim/rhwp#977
- 브랜치: `local/task977` (← `upstream/devel`)
- 마일스톤: M100 (v1.0.0)
- 정정 방향: (C) 폴백 로직 공용화

## 1. 문제

목차(개요번호 포함) 페이지를 rhwp-studio(WASM)로 열면, 선두 공백 글자의
CharShape가 인접 문단과 다른 경우 일부 개요번호가 ~9px 어긋났다.
`export-svg`(네이티브) 출력은 정상이라 WASM 전용 문제였다.

## 2. 원인

`src/renderer/layout/text_measurement.rs`의 `compute_char_positions`가
플랫폼별 이중 구현:

- 네이티브: `EmbeddedTextMeasurer` — 미등록 폰트 문자 폭을 휴리스틱
  (`font_size*0.5` 등)으로 산출
- WASM: `WasmTextMeasurer` — 미등록 폰트를 브라우저 `measureText` 실측

미등록 폰트의 공백 폭이 WASM에서만 폰트별로 달라져, 선두 공백 폰트가
다른 인접 목차 문단의 개요번호가 어긋났다. 네이티브 휴리스틱은 한컴
PDF 정합 기준값이므로, WASM 실측이 정답지(PDF)에서 이탈한 것이다.

## 3. 정정

`src/renderer/layout/text_measurement.rs` 단일 파일:

1. 공용 함수 `base_char_width` 신설 — 내장 메트릭 → 휴리스틱 폴백
2. `EmbeddedTextMeasurer` 3개소 인라인 if-else → `base_char_width`
   (순수 리팩터, 네이티브 동작 불변)
3. `WasmTextMeasurer` 2개소 — `measure_char_width_hwp`(JS 실측) →
   `base_char_width`(휴리스틱)로 통일
4. 사용처 없어진 `wasm_internals::measure_char_width_hwp` 제거
5. 옛한글 합성 클러스터 경로는 회귀 방지로 종전 유지

→ 미등록 폰트 문자 폭이 네이티브·WASM 동일 규칙으로 산출.

## 4. 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✅ |
| `cargo check --target wasm32-unknown-unknown` | ✅ |
| `cargo test --release --lib` | ✅ 1297 passed, 0 failed |
| `export-svg` 회귀 | ✅ 무회귀 (네이티브 동작 불변) |
| WASM 빌드 (Docker) | ✅ |
| rhwp-studio 정렬 회복 | ✅ 개요번호 캔버스 x좌표가 SVG와 일치 |
| 회귀 (표/수능형 샘플) | ✅ 정상 렌더 |

`cargo clippy` 2건 error는 `src/diagnostics/hwp5_contract_probe.rs`의
pre-existing 이슈로 본 타스크 무관 (해당 파일 미수정). `text_measurement.rs`
clippy 무경고.

## 5. 한계 및 후속

- 옛한글 합성 클러스터(`cluster_len>1`)의 WASM 측정은 종전 `'가'` 대리
  측정값을 유지했다. 본 버그와 무관하며 회귀 위험 회피 목적. 필요 시
  별도 타스크로 통일 검토 가능.
- `WasmTextMeasurer`에서 단일 문자 JS 실측을 제거했으므로, 내장 메트릭
  DB 미등록 폰트는 휴리스틱 폭을 쓴다. 네이티브와 동일 동작이며 한컴
  PDF 정합 기준이나, 향후 메트릭 DB 확충으로 정밀도를 높일 수 있다.

## 6. 결론

WASM 전용 개요번호 정렬 어긋남 해소. 단계 1~3 완료.
