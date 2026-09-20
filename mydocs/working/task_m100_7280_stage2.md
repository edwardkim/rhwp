# Task #7280 Stage 2 — R1 구성된 줄 조회의 책임 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 승인 기록: `1b6038d46`. 제품 baseline은 `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`와 동일하다.
- 상태: R1 구현, 검증 진행 중. R2 미착수. PR/push 준비 완료를 뜻하지 않는다.

## 1. 변경과 비변경

`typeset.rs`의 다음 네 함수를 `typeset/paragraph/line_queries.rs`로 이동했다.

- `composed_line_char_end`
- `line_has_strict_tac_control`
- `line_has_visible_text`
- `line_has_text_span`

입력은 기존 `&ComposedParagraph`와 줄 인덱스다. 줄 재구성·높이 판단·원본 IR 변경·페이지
상태 변경은 하지 않는다. 기존 조건·반환값·본문은 그대로이며 조판 문제 수정은 없다.
기존 호출부는 typeset의 명시적 import로 연결하고 함수 가시성은 typeset 하위로 제한했다.
`paragraph.rs`는 현재 소유 범위가 줄 조회뿐임을 명시한다. 문단 전체 책임 분리는 아직 아니다.

함수 이름의 `visible_text`를 새 의미로 해석하지 않았다. 기존 구현은 일반 공백도 true이며,
줄의 점유 높이나 실제 잉크 가시성의 대용 조건이 아니다. 이 차이를 주석에 남겼다.

## 2. 호출자와 테스트 경계

- 줄 끝 위치는 TAC/equation 줄 소속 판단과 문단 inline 계획에 사용된다.
- 텍스트/범위 조회는 기존 미주 준비·배치와 equation/TAC 분기에서도 사용된다.
  이번에는 호출 위치·순서 및 이 분기들의 조건을 바꾸지 않았다.
- source-side의 기존 네 테스트 모듈은 typeset.rs에 그대로 둔다. 새 cfg(test) 모듈,
  테스트 공개 API, baseline/ignore/pin 변경은 없다.
- 향후 이 조회를 수정하면 문단뿐 아니라 위 미주·TAC 소비자도 검토해야 한다.
  삭제할 때는 네 함수 사이의 호출과 typeset의 import/호출부까지 확인한다.

## 3. 현재 검증 증거

- Git HEAD의 이동 전 함수와 새 파일의 함수 본문 `{ ... }`를 추출하여 4건 모두 문자열 동일 확인.
  이전 파일에 동일 함수 선언이 남아 있지 않음도 확인했다.
- `node scripts/rust-unit-test-tiers.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db` 통과:
  4,205 tests / 298 modules / ready 0 / support 87 / white-box 4,114 / cfg support items 28.
- `git diff --check` 통과. Rust 포맷과 정확한 source commit의 컴파일·focused 실행 결과는 아래에 이어 기록한다.

위 정적 동일성 검사는 한컴 피델리티 또는 전체 renderer 검증을 뜻하지 않는다.
제출 전 native/WASM/workspace lint, 전체 회귀, Native Skia 및 fresh WASM·시각 검증은
구현계획의 통합 게이트로 남아 있다. 이전 baseline의 10,096건 PASS를 변경본 결과로 재사용하지 않는다.
