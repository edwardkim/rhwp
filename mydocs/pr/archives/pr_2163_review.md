# PR #2163 통합 재검토 - 함초롬바탕 라틴/구두점 폭 대체

- PR: https://github.com/edwardkim/rhwp/pull/2163
- 관련 이슈: #2156
- 작성자: `planet6897`
- reviewer: `jangster77` 지정 완료
- base/head: `devel` <- `pr/task2156` (원 PR), 현재는 통합 브랜치에서 재검토
- 원 PR 참고 head: `2f1cd31fae202efc4d16c66de7e5f8557fae3434`
- 통합 기준: `8d695deef` + PR #2163 cherry-pick `a4fbea951` + 메인터너 범위 교정
- 문서 작성 시점 참고 상태: 원 PR `CLEAN`, `MERGEABLE`; merge 전 최신 CI 재확인 필요
- 작성일: 2026-07-10

## 결론

**P1 해소.** 원 PR의 검증 범위 초과 적용을 메인터너가 통합 브랜치에서 보정했다.
Haansoft Batang 대체는 검증된 `함초롬바탕`/`HCR Batang` 정확한 별칭에만 적용하며,
`함초롬돋움`/`HCR Dotum`과 확장 별칭은 적용하지 않는다. 함초롬돋움의 별도 한컴
대체 메트릭은 후속 font-fidelity 축으로 남긴다.

## P1 해소 내역

### [P1] 함초롬돋움까지 한컴바탕 수치를 강제 적용함

원 PR의 `starts_with("함초롬") || starts_with("HCR ")` 조건은
`함초롬돋움`/`HCR Dotum`에도 `HAANSOFT_BATANG_ASCII`를 적용했다. 통합 브랜치의
`haansoft_latin_override`는 `matches!(primary_name, "함초롬바탕" | "HCR Batang")`로
정확한 별칭만 통과시킨다.

동일 10pt 입력의 `measure-width` 결과는 다음과 같다.

| 폰트 | `(` | `,` | `A` |
|---|---:|---:|---:|
| `함초롬바탕` / `HCR Batang` | 6.667px | 3.880px | 10.000px |
| `함초롬돋움` / `HCR Dotum` (범위 교정 후) | 4.173px | 3.440px | 8.720px |

`width_ladder.hwpx`와 HWP2020 PDF 증거는 `함초롬바탕`만 다룬다. 이에 맞춰 positive
회귀는 `함초롬바탕`과 `HCR Batang`의 동일 측정값을 확인하고, negative 회귀는
`함초롬돋움`, `HCR Dotum`, 두 확장 별칭이 바탕 대체를 타지 않음을 확인한다.

## 변경 범위

- `text_measurement.rs`: Haansoft Batang ASCII 95개 상수와 라틴/구두점 override.
- 진단 CLI `rhwp measure-width`, HWPX 문자폭 사다리 fixture, Windows COM용 probe 도구 추가.
- `form-002`, `issue-157` golden SVG 갱신.
- 계획/단계/보고서 문서 추가.

## 검증

- 원 PR 참고 head의 CI, CodeQL, Native Skia, Canvas visual diff: 통과.
- 통합 브랜치 `git diff --check`, `cargo fmt --check`: 통과.
- `cargo fmt --check`, `git diff --check`: 통과.
- `python3 -m py_compile tools/extract_haansoft_table.py tools/make_width_ladder.py tools/probe_width_ladder.py tools/hangul_row_heights2.py`: 통과.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --lib issue_2156_hcr_batang_latin_uses_haansoft_metrics`: 통과.
- #2158, #1749, #2093, #1891, #1842, #1623, #2146, `svg_snapshot` 통합 focused suite: 41 passed.
- `samples/task2156/width_ladder.hwpx`를 HWP2020 MCP Print 방식으로 PDF 변환:
  - [width-ladder-2020.pdf](../assets/pr_2163/width-ladder-2020.pdf), 1쪽, `validation: ok`,
    SHA-256 `699922ea07d0fd24bd796143258fc507bea281b413ee99aa35cac6818c8cd0d6`.
  - 통합 브랜치 visual sweep 자동 구조 후보: 0/1.
  - page 1 visual accuracy proxy: 9.73896. 폰트 raster 차이가 커서 이 값은 글꼴 픽셀 fidelity의
    통과 증명이 아니며, 바탕 계열 샘플의 줄 수/구조 참고값으로만 사용한다.
  - 임시 검토 이미지: `output/integration-2163-2165-2168-visual/integration-2163-width-ladder/review/review_001.png`.
  - 보존 review asset: [pr_2163_width_ladder_review_001.png](../assets/pr_2163/pr_2163_width_ladder_review_001.png).
- `wasm-pack build --target web --out-dir pkg`: 통과.
- Studio `http://localhost:7700`에서 `width_ladder.hwpx`를 직접 로드: 1쪽 렌더,
  console error/warn 없음. HWPX fixture의 빈 `lineseg` 경고는 기존 fixture 구조에 따른 안내다.
- 깨끗한 `target` 전체 사전 검증: `cargo build --release`, `cargo test --release --lib`
  (2191 passed), `cargo test --profile release-test --tests`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test --doc`, Studio `npx tsc --noEmit`/`npm test` (183 passed), WASM build 모두 통과.

## WASM 범위 참고

원 PR 본문은 WASM JS Canvas 경로를 후속이라고 설명하지만, 실제 `WasmTextMeasurer`의
`wasm_internals::measure_char_width_hwp`도 `measure_char_width_embedded`를 JS bridge보다
먼저 호출한다. 따라서 이 override는 브라우저에도 적용된다. 범위를 좁히거나 돋움 측정을
추가할 때에는 WASM/Studio 검증 결과와 PR 설명을 함께 갱신해야 한다.

## 잔여 및 최종 조건

1. 함초롬돋움/HCR Dotum의 한컴 기준 폭과 재현 자료는 별도 font-fidelity 과제로 남긴다.
2. 통합 PR의 최신 CI·CodeQL을 재확인한다.

## 옵션 1 기록

이 문서, 기준 PDF, 대표 review PNG와 오늘할일 `mydocs/orders/20260710.md`를 통합 PR에 함께 포함한다.

## Merge 결과

- 통합 PR [#2170](https://github.com/edwardkim/rhwp/pull/2170)은 2026-07-10에 merge commit
  `c95d8fd743ae4cfcbcbb0e26444ebef4e42b84ba`로 `devel`에 반영됐다.
- 최신 head CI, CodeQL, Render Diff는 모두 성공했다. 갱신 전 head `19abd763b`의 CI/CodeQL/Render Diff는
  force-cancel 후 `completed/cancelled`를 확인했다.
- #2156은 PR body의 closing keyword로 auto-close된 것을 확인했다.
