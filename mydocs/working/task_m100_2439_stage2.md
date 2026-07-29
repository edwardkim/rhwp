# Task M100-2439 Stage 2 — page-local visible-float exclusion 교정

- 이슈: [#2439](https://github.com/edwardkim/rhwp/issues/2439)
- 브랜치: `fix/2439-split-table-flow`
- 작성일: 2026-07-19

## 1. 구현 내용

- fresh-page orphan 가드가 실제 페이지/단 advance를 수행하면 placement 전용
  `para_start_height`를 새 `current_height`로 재설정했다.
- 선행 co-anchored float가 있는 양수 offset 표는 저장 상단과 이미 소비한 flow 중 더
  아래를 실제 상단으로 사용해, 밀린 뒤에도 표 exclusion의 전체 높이를 보존했다.
- zero-offset 첫 표도 같은 host에 후행 co-anchored 표가 있을 때는 exclusion을 남긴다.
- 같은 owner의 표 뒤 post-text는 자기 exclusion을 소비한다. 표 앞 제목이 표 위에
  남는 #1549 동작은 유지한다.

## 2. 범위 제한

- `budget_para_start_height`와 RowBreak 분할 예산은 변경하지 않았다.
- 단독 zero-offset 표의 기존 flow 동작은 변경하지 않았다.
- 페이지 수를 10쪽으로 강제하는 휴리스틱은 넣지 않았다.

## 3. 신규 회귀 테스트

`cargo test --test issue_2439 -- --nocapture`: 2 passed, 0 failed.

- fresh page로 이월된 표가 양의 높이를 유지하고 후속 텍스트가 표 아래에 배치된다.
- zero-offset/positive-offset 표 쌍과 표 뒤 host 텍스트가 순서대로 배치된다.

## 4. 후속 기록

이 문서의 2/2는 Stage 2 당시 테스트 결과이며 최종 테스트 합계가 아니다. 페이지 수를
강제하는 휴리스틱은 끝까지 추가하지 않았고, [Stage 5](task_m100_2439_stage5.md)의 오라클
정정 뒤 저장 flow를 추가로 복원해 [Stage 6](task_m100_2439_stage6.md)에서 자연스럽게
10쪽을 만들었다.
