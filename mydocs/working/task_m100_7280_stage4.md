# Task #7280 Stage 4 — R2b 문단 구성의 관측·계산 경계

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2a](task_m100_7280_stage3.md), 시작 head `125df2eba`.
- 상태: 구현 후 검증 대기. R2 전체 완료나 제출 준비 완료가 아니다.
- 제품 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db` 유지.

## 1. 소유권과 입력 경계

기존 `TypesetEngine::format_paragraph_for_flow`의 계산 책임을 `paragraph/format.rs`로
옮겼다. 상위 메서드는 호환 위임만 남겨 본문/known-square-band/미주 호출자의 기존 flag와
호출 순서를 보존한다. public API, IR, 페이지 상태, 조건·상수·기대값은 변경하지 않는다.

- `ParagraphFormatContext`: dpi, 호환 profile, uniform-filler flag, float-carve 증거의 관측만 제공.
  내부 Cell/RefCell 참조는 sibling인 format 모듈에서 접근할 수 없는 private 필드다.
  생성자는 참조만 보관하고 get/borrow하지 않는다. 기존 호출 위치에서 getter가 읽으며
  float 증거의 읽기 Ref도 기존 frame 호출 문장 안에서 해제된다. set/borrow_mut는 제공하지 않는다.
- `recompose_for_flow`: 기존 입력 폭/스타일에 따른 frame 재구성 선택. `Option<ComposedParagraph>` 반환.
- `resolve_line_metrics`: 기존 구성된 줄/저장 줄/빈 문단 경로의 줄 높이·간격과 후속 보정.
  기존 순서 의존적인 이전 TAC 그림 높이도 이 호출 안의 지역 상태다.
- `format_paragraph_for_flow`: 원래 스타일 선택 → 재구성 → spacing 결정 → 줄 메트릭 →
  fit/총 높이·host line·tail 폭의 결과 조립 순서를 유지한다.

전체 TypesetEngine/TypesetState를 하위 구성 함수에 넘기지 않는다. 문서 IR을 복사하거나
범용 rule engine/trait도 추가하지 않았다. 환경 변수 실험 분기는 같은 시점에 조회한다.
현재 사용되지 않던 `hwp3_body_reflow` 인자도 이번에 정리하지 않고 기존 인터페이스를 보존한다.

## 2. 소비자와 남는 의존

결과 생산자는 새 format 모듈이다. `height_for_fit`의 문단 fit 판정, `line_heights/line_spacings`의
분할/전진, `computed_host_lines/tail_line_remaining_width`의 float host 배치 소비자는 기존
typeset 호출부 그대로다. 이들의 후속 원점/높이 보정이나 paint는 이번에 변경하지 않는다.

기존 parent helper 4개(`empty_paragraph_fallback_line_metrics`, `line_has_tac_control`,
`para_is_treat_as_char_picture_only`, `text_line_is_picture_lead_in`)는 명시적 import로 사용한다.
다른 소비자가 있는 이 helper들의 소유권 정리는 후속 항목이며 완전 단방향화로 보고하지 않는다.
`resolve_line_metrics` 내부의 세부 정책 분해, 문단 배치·TAC/float 경로 조정도 남아 있다.

기여자는 줄 구성 선택을 바꿀 때 recompose, 높이 분기를 바꿀 때 line metrics, fit/총 높이/host
결과를 바꿀 때 format의 조립부와 위 소비자를 함께 확인한다. 기존 heuristic을 옮겼다는 사실을
조판 규칙의 타당성을 새로 승인한 근거로 사용하지 않는다.

## 3. 동작 보존 검사와 실행 검증

`output/7280/stage4/verify-format.mjs`는 분리된 두 helper를 원래 위치에 인라인 복원한 뒤
관측 getter/경로 변경을 역치환하여 기존 함수와 비교한다. 문자열·문자 literal과 주석을 보존하고
formatter 공백/후행 쉼표 및 식 하나짜리 closure의 중괄호 차이만 정규화한다.
상위 파일은 wrapper 이외의 코드·테스트가 동일하며 wrapper 인자 순서도 검사한다.
정적 동일성은 통과했지만 출력 피델리티 판정을 대체하지 않는다.

source-side/통합 테스트, baseline/golden/ignore/pin/정책은 수정하지 않았다.
고정 제품 SHA의 review worktree에서 파생 suite를 준비하고 fmt·정책·회귀 및 lint를 실행한다.
실행 결과는 완료 후 추가하며 이전 절편의 PASS를 현재 결과로 대체하지 않는다.

최종 제출의 전체 lint 묶음, Native Skia, fresh Docker WASM/직접 시각 검증은 별도 통합 게이트다.
이번 절편의 통과를 #7280 전체 또는 PR 제출 준비 완료로 보고하지 않는다. 원격 쓰기는 하지 않는다.
