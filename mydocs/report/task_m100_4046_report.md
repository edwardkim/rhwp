---
kind: report
status: active
canonical: mydocs/report/task_m100_4046_report.md
last_verified: 2026-08-05
---

# native↔WASM SVG 패리티 — 측정기 통일과 패리티 하네스

Issue: #4046

## 요약

rhwp-studio(WASM)와 CLI(`export-svg`)가 같은 문서에서 만들어내는 SVG가 달랐다. 원인은
단 하나 — `default_measurer()`의 wasm32 분기가 native(`EmbeddedTextMeasurer`)와 폴백
사다리가 다른 `WasmTextMeasurer`를 쓰는 것. WASM도 `EmbeddedTextMeasurer`를 쓰도록
통일하고, 이제 완전히 죽은 `WasmTextMeasurer`·`wasm_internals` 모듈(JS Canvas 브릿지
`cached_js_measure` 포함, #977 이후 이미 미호출)을 제거했다.

검증용으로 native↔WASM SVG byte 비교 하네스 `scripts/svg_native_wasm_diff.mjs`를
추가했다 — `pkg/rhwp.js`를 Node에서 직접 로드하므로 puppeteer 없이 대량 스윕이 가능하다.
사용법·해석은 [`svg_native_wasm_parity.md`](../manual/verification/svg_native_wasm_parity.md).

## 증상과 원인

- 회귀 기본 샘플 7종(exam 4종·synam-001·aift·2010-01-06, 167페이지) **전 페이지** SVG 불일치.
  글자 x 좌표 미세 이동(예 0.1667px), 본문 클립 너비 차이(897.0133 vs 897.5634…),
  정렬 오프셋 이동.
- SVG 생성은 native/WASM이 동일 Rust 함수(`render_page_svg_native`)를 공유. 측정 소스도
  양쪽 모두 `font_metrics_data.rs` 정적 테이블로 이미 수렴(#977 계열). 남은 차이는
  `WasmTextMeasurer` 폴백 사다리의 비대칭뿐 — U+318D 분기 부재, 다중 코드포인트 클러스터
  처리('가' 폭 대리), narrow-punct↔CJK 검사 순서.

## 변경

- `src/renderer/layout/text_measurement.rs`
  - `default_measurer()` wasm32 분기를 `EmbeddedTextMeasurer`로 통일(cfg 이중화 해소)
  - `WasmTextMeasurer`(struct+impl), `wasm_internals` 모듈(JS 브릿지·LRU 캐시·
    `measure_char_width_hwp`·`measure_hangul_width_hwp`) 제거 — 통일 후 전부 미도달 코드
  - 관련 주석 3곳 갱신 (#2430 테스트 주석 포함)
- `scripts/svg_native_wasm_diff.mjs` 신규 — 패리티 하네스
- `mydocs/manual/verification/svg_native_wasm_parity.md` 신규 — 하네스 가이드

studio TS 쪽 `installMeasureTextWidth`(`globalThis.measureTextWidth`)는 `input-handler.ts`가
계속 사용하므로 유지. 화면 뷰포트는 canvas2d replay라 이 변경의 영향 경로가 아니다
(SVG는 인쇄·embed API·e2e에서 사용).

## 검증

- 하네스: 회귀 샘플 7종 167페이지 + 10k 코퍼스 무작위 40건 **전부 byte 일치**
  (native 축은 RHWP_* env 제거 실행 — WASM은 env 미관측이므로 공정 조건).
  데드코드 제거 후 동일 셋 재실행 47/47 일치 재확인.
- **10k 코퍼스 전수 스윕**: 10,000건을 50건×200청크·12워커 병렬로 전수 실행.
  렌더 대상 9,948건 **전부 byte 일치**(75,500+페이지), **발산 0건**
  (diff/page-count-mismatch/wasm-error 없음). 나머지 52건은 렌더 이전 단계 실패 —
  미지원 포맷 47(HWP 2.x 등), 암호 문서 5. 스윕 중 발견된 하네스 결함 2건
  (255바이트 파일명 한계: 라벨 디렉터리 생성, native export 출력 파일명)은
  라벨 절단 + 짧은 심링크 우회로 수정.
- cargo fmt / clippy --lib 통과
- release-test 전체: 통과 — lib 3252 passed 포함 전 테스트 바이너리 0 failed
- Native Skia 3종: 통과 (skia --lib 58, issue_2225 2, render_p37 4 passed).
  이 머신에는 freetype/fontconfig dev 심링크가 없어 `~/.local/lib-shim`에
  `libfreetype.so→so.6`, `libfontconfig.so→so.1` 심링크를 만들고
  `LIBRARY_PATH`로 링크했다(환경 문제, 코드와 무관)
- wasm-pack build: 통과 (경고 0) — 제거 코드가 wasm32 전용이므로 실질 컴파일 검증
- studio e2e: `e2e:render-diff:ci` PASS (KTX 0.01054%·biz_plan 0%·tac-case-001 0%),
  `e2e:renderer-contract` PASS — fresh WASM 기준
- 문서 링크 검사(check_markdown_links.py): 이상 없음
