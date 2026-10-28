# Task #3937 Stage 1 — 배분 간격과 browser glyph 폭 분리

- 이슈: [#3937](https://github.com/edwardkim/rhwp/issues/3937)
- 브랜치: stack/issue-3937-distribution-glyph-width
- 최신 기준: upstream/devel cf5d462dc
- code candidate: 4ca011cd5
- 작성일: 2026-08-04

## 원인과 수정

compute_char_positions의 cluster advance에는 다음 문자의 origin을 옮기는
extra_char_spacing이 포함된다. 기존 WebCanvas ASCII scaleX와 SVG textLength는 이 전체
advance를 glyph-fit 폭으로 사용해 배분 간격만큼 glyph 윤곽까지 확대했다.

TextStyle에 layout advance에서 양수 배분 간격만 제거하는 공통 계산을 추가했다.
Canvas2D와 SVG는 이 계산을 사용하고, 문자 origin과 layout advance는 바꾸지 않는다.
음수 간격은 #2189 셀 오버플로우 보정에서 사용하던 layout advance 기반 browser fit을
그대로 유지한다. 따라서 이번 변경은 양수 배분·셀 underflow 간격만 glyph 폭에서 제외하며,
음수 보정과 반각 CJK 인용부호의 기존 textLength 계약은 바꾸지 않는다.

## 최신 기준 검증

- cargo test spacing --lib: 42 / 42 통과
- cargo test renderer::svg::tests --lib: 41 / 41 통과
- cargo check --target wasm32-unknown-unknown --lib: 통과
- git diff --check: 통과

최종 검증 뒤 devel이 다시 전진해 ec1b21096로 rebase했다. 추가 변경은 이 레이어의
Canvas/SVG 제품 파일과 직접 겹치지 않았지만, typeset 쪽 경계 수정이 실제 문서의 쪽수에
영향을 줄 수 있어 spacing 42 / 42, SVG 41 / 41과 wasm32 library check를 다시 실행해
모두 통과했다.

PR 리뷰에서 음수 `extra_char_spacing`이 #2189 셀 오버플로우 보정의 주요 입력이라는
교차회귀 위험이 확인됐다. 양수만 glyph-fit에서 제외하도록 범위를 좁히고, 음수 Canvas
일반/ASCII fit과 SVG `textLength`, #2189 표적 회귀를 다시 검증했다.

최상단 stack revision에서 production WASM을 다시 만들고 HWP/HWPX 연속 IME→숫자 E2E
2 / 2를 재실행했다. 두 형식 모두 숫자 줄 전환 11 / 69, 최종 숫자 73, 최종 쪽수 116,
synchronous flush 0으로 GREEN이었다. 사용자 브라우저 시각 판정도 보존한다.

최신 CI 대기 중 devel이 중첩 표 배치 수정 #3949를 포함한 cf5d462dc로 전진했다. 공용 오늘할일
충돌은 양쪽 기록을 보존해 해소했고 제품 코드는 충돌하지 않았다. spacing 42 / 42,
`issue_2189_cell_text_clip` 1 / 1, composer 53 / 53과 production WASM HWP/HWPX 통합 E2E를
제한 재실행했다. 두 형식 모두 11 / 69, 숫자 73, 116쪽으로 GREEN이며 p95는 49.6 / 49.7ms였다.

## 범위 경계

- #3937: Canvas/SVG glyph 윤곽 폭
- #3822: 이전 break 뒤 긴 token의 반복 줄바꿈
- #3815: deferred pagination 시작 coalescing

세 변경은 제품 코드상 분리돼 있으며, 최상단 E2E가 실제 연속 입력 조합을 함께 검증한다.

Draft PR은 [#3944](https://github.com/edwardkim/rhwp/pull/3944)이며 GitHub Stack
#3947의 첫 레이어다.
