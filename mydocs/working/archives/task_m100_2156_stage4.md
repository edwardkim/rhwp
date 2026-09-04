# Task M100 #2156 - Stage 4 메인터너 범위 교정 완료

**날짜**: 2026-07-10  
**대상**: PR #2163 통합 검증 브랜치

## 배경

PR #2163의 `haansoft_latin_override`는 `함초롬*`/`HCR *` 접두 전체에
Haansoft Batang ASCII 메트릭을 적용한다. 그러나 기존 문자폭 사다리와 HWP2020
PDF 검증은 `함초롬바탕`만 대상으로 하며, `함초롬돋움`/`HCR Dotum`은 별도
Haansoft Dotum 가능성을 후속 확인으로 남겼다.

## 변경 계획

1. 오버라이드 대상을 검증된 `함초롬바탕`과 `HCR Batang` 정확한 별칭으로 한정한다.
2. `함초롬돋움`, `HCR Dotum`, 확장 계열이 Haansoft Batang 대체를 타지 않는
   negative regression을 추가한다.
3. 계획/보고서/fixture README의 "함초롬 계열" 표현을 검증된 바탕 계열로 정정한다.
4. native focused test, WASM build, HWP2020 MCP PDF 및 visual sweep으로 통합 결과를
   재검증한다.

## 판정 기준

- `함초롬바탕`의 괄호/쉼표/숫자/라틴/중점 기대값은 유지한다.
- `함초롬돋움`/`HCR Dotum`은 Haansoft Batang 상수를 사용하지 않는다.
- sample16 HWPX 64쪽, issue_1891/1842/1623/2146 및 SVG snapshot 회귀가 없다.
- WASM 빌드가 통과하고 브라우저 영향 범위를 리뷰 문서와 PR 설명에 기록한다.

## 결과

1. `haansoft_latin_override`를 `함초롬바탕`/`HCR Batang` 정확한 별칭으로 제한했다.
   `함초롬돋움`, `HCR Dotum`, 두 확장 별칭은 Haansoft Batang override를 사용하지 않는
   negative regression으로 고정했다.
2. `HCR Batang` 별칭이 `함초롬바탕`과 같은 괄호 폭을 반환하는 positive regression을 추가했다.
3. 문서·fixture README의 "함초롬 계열" 표현을 문자폭 사다리로 검증한 바탕 계열로 정정했다.
4. `width_ladder.hwpx` HWP2020 MCP PDF visual sweep은 1/1쪽 자동 구조 후보 0건이다.
   픽셀 차이는 글꼴 raster fidelity로 남기고, 줄 수·경계 구조만 이 변경의 판정 근거로 사용했다.
5. 깨끗한 `target`에서 `cargo build --release`, `cargo test --release --lib` (2191 passed),
   `cargo test --profile release-test --tests`, `cargo fmt --check`,
   `cargo clippy --all-targets -- -D warnings`, `cargo test --doc`, Studio `tsc`/`npm test`,
   `wasm-pack build --target web --out-dir pkg`를 모두 통과했다.

## 결론

PR #2163의 P1은 범위 교정과 alias 회귀로 해소했다. 함초롬돋움/HCR Dotum의 실제 한컴
대체 메트릭은 재현 HWP/HWPX와 기준 PDF를 갖춘 별도 font-fidelity 과제로 남긴다.
