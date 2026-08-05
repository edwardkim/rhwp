---
kind: guide
status: active
canonical: mydocs/manual/verification/visual_verification_governance.md
last_verified: 2026-08-05
---

# native↔WASM SVG 패리티 하네스 — `scripts/svg_native_wasm_diff.mjs`

CLI(`export-svg`)와 rhwp-studio(WASM)가 같은 문서에서 만들어내는 SVG 문자열을 byte 단위로
직접 비교한다. 두 축 모두 동일한 Rust 함수(`render_page_svg_native` /
`render_page_svg_layer_with_profile_native`)를 타므로, 차이가 나면 원인은 다음 중 하나로
좁혀진다:

1. **폰트 측정기 분기** — native `EmbeddedTextMeasurer` vs WASM `WasmTextMeasurer`의
   폴백 사다리 비대칭 (`src/renderer/layout/text_measurement.rs`)
2. **환경변수 분기** — `wasm32`에서 `std::env::var`는 항상 `Err`이므로 `RHWP_*` 게이트가
   전부 기본값이 된다 (하네스는 이를 공정하게 만들기 위해 native 축도 `RHWP_*`를 제거하고 실행)
3. **`cfg(target_arch = "wasm32")` 분기** — 렌더러/코어의 조건부 컴파일 차이

studio 화면(canvas2d/CanvasKit) 픽셀 비교는 이 하네스의 범위가 아니다 — 그쪽은
`scripts/renderer_baseline.py` + `e2e/renderer-baseline-native-diff.mjs`를 쓴다.
이 하네스는 그 앞 단계인 **SVG 문자열 생성의 결정성**을 고정한다.

## 사전 조건

같은 커밋에서 두 산출물을 빌드한다. 버전 문자열이 다르면 하네스가 경고를 낸다.

```bash
cargo build --release
wasm-pack build --target web --out-dir pkg
```

## 사용법

```bash
# 단일 문서 전체 페이지
node scripts/svg_native_wasm_diff.mjs 문서.hwpx

# 디렉터리 스윕 (예: 10k 코퍼스에서 100건)
node scripts/svg_native_wasm_diff.mjs /home/planet/hwpdocs_10k_share --limit 100

# layer 경로 비교 (studio 인쇄 = renderPageSvgWithProfile('print'))
node scripts/svg_native_wasm_diff.mjs 문서.hwpx --profile print

# 특정 페이지만
node scripts/svg_native_wasm_diff.mjs 문서.hwpx --pages 0,3,7
```

WASM 축은 puppeteer 없이 Node에서 `pkg/rhwp.js`를 직접 로드한다(web target이지만
`module_or_path`에 bytes를 넘기면 Node에서도 초기화된다).

## 출력

- `output/svg-native-wasm-diff/report.json` — 문서별 상태
  (`match` / `diff` / `page-count-mismatch` / `native-error` / `wasm-error`),
  불일치 페이지의 diff 머리부(`diffHead`), 버전·git HEAD 스탬프
- 불일치 페이지의 native/wasm SVG 파일 쌍 (일치 페이지는 기본 삭제, `--keep-match`로 보존)
- 종료 코드 0 = 전부 일치, 1 = 불일치 존재

## 해석 지침

- `diff` — `diffHead`에서 첫 발산 지점을 본다. `<text>`의 `x` 좌표 차이면 측정기
  사다리 발산이 유력하다(ㆍ U+318D, 옛한글 클러스터, narrow punctuation 순서가 알려진
  비대칭). `<style>` 블록 유무 차이면 폰트 임베딩 경로 차이다.
- `page-count-mismatch` — 측정 폭 차이가 줄바꿈→페이지네이션까지 번진 경우.
  겹치는 페이지 구간은 계속 비교되므로 최초 발산 페이지를 찾을 수 있다.
- 목표 상태는 **전 문서 byte 일치**다. 측정기 사다리를 통일하면 도달 가능하다.

## 2026-08-05 진단 결과

`default_measurer()`의 wasm32 분기를 `WasmTextMeasurer` → `EmbeddedTextMeasurer`로 바꾸는
한 줄 변경으로, 회귀 기본 샘플 7종(167페이지) + 10k 코퍼스 40건이 **전부 byte 일치**를
달성했다. 즉 native↔WASM SVG 발산의 원인은 측정기 사다리 비대칭 하나로 수렴하며,
`cfg(wasm32)` 분기나 환경변수는 (RHWP_* 제거 조건에서) 추가 발산을 만들지 않는다.
통일 후 `WasmTextMeasurer`·`cached_js_measure`는 데드코드가 되므로 정리 대상이다.
