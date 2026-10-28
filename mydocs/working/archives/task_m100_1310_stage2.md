# Task #1310 Stage 2 - TAC 수식-only 자동 줄바꿈 구현 검증

## 1. 목적

Stage 1에서 확인한 가설 A를 최소 구현으로 대입했다.

핵심 검증 질문:

- 한 줄에 TAC 수식이 연달아 있을 때 다음 TAC 수식까지 현재 줄 폭에 들어가는지 판단하는가?
- 자동 줄바꿈으로 생긴 virtual row 높이가 렌더링과 높이 계산에 같이 반영되는가?
- 기존 #1308 강제 줄넘김/내어쓰기/커서 이동 회귀와 기존 미주 페이지네이션 회귀 테스트를 깨지 않는가?

## 2. 구현 요약

추가 파일:

- `src/renderer/equation_tac_flow.rs`

주요 동작:

- 모든 `ComposedLine.runs`가 실제로 비어 있는 TAC 수식-only 문단만 대상으로 한다.
- 대상 TAC가 모두 `Control::Equation`인 경우에만 처리한다.
- 기존 line assignment를 유지하되, 같은 logical line 안에서 `row_width + next_tac_width > available_width`이면 다음 virtual row로 넘긴다.
- 렌더링 경로와 높이 측정 경로가 같은 helper 결과를 사용한다.
- 셀 내부 수식은 기존 alignment 동작을 유지하기 위해 `available_width = infinity`로 두어 wrapping하지 않는다.

## 3. vpos 보정 범위

초기 구현에서는 미주 LINE_SEG vpos를 누적 y와 항상 `max()` 처리했다. 이 경우 목표 샘플은 겹침이 사라졌지만, 다른 미주 회귀 샘플에서 기존 PDF 정합 위치가 밀리는 문제가 발생했다.

수정 후 규칙:

- 저장된 vpos는 기본적으로 존중한다.
- 단, 현재 줄에서 TAC 자동 줄바꿈이 발생했거나 그 여파가 다음 줄의 저장 vpos보다 아래까지 이어지는 동안만 누적 y를 우선한다.
- 여파가 끝나면 다시 저장 vpos 흐름으로 복귀한다.

이 범위 제한 후 `tests/issue_1139_inline_picture_duplicate.rs` 전체가 통과했다.

## 4. 기존 테스트 판정 기준 조정

`issue_1256_2022_sep_page10_question12_keeps_between_notes_gap`의 기존 단언에는 다음 조건이 있었다.

```text
question13_y <= 724.0
```

이 값은 문12 수식-only 블록이 자동 줄바꿈되지 않던 기존 상태를 전제로 한 상한이다. #1310의 한컴 기준은 수식-only 흐름이 여러 visual row로 줄바꿈되는 것이므로, 문13 y의 절대 상한만 유지하면 새 요구사항과 충돌한다.

변경한 판정:

- 문12 제목/본문/between-notes gap은 기존 범위를 유지한다.
- 문12 첫 수식 x는 기존처럼 오른쪽 단 시작 근처를 유지한다.
- 문12 수식 bbox의 오른쪽 끝은 column right를 넘지 않아야 한다.
- 첫 줄에서 넘치던 세 번째 TAC 수식은 다음 visual row로 넘어가야 한다.
- 문13은 wrapping된 문12 수식 블록의 실제 bottom 뒤에 자연스러운 gap으로 이어져야 한다.

즉 테스트를 완화한 것이 아니라, 새 기능의 실제 정답 조건으로 교체했다.

## 5. 산출물

시각 판정용:

- `output/poc/task1310/stage2_fixed/3-09월_교육_통합_2022_010.svg`
- `output/poc/task1310/stage2_fixed_tree/render_tree_010.json`

render-tree 수치:

| 항목 | 값 |
|---|---:|
| 오른쪽 단 right | 759.7 |
| 첫 수식 row right | 582.9 |
| 줄바꿈된 세 번째 TAC right | 585.3 |
| 다음 수식 row right | 632.7 |
| 마지막 수식 row right | 564.1 |
| 문13 제목 y | 758.9 |

## 6. 검증

통과:

```bash
cargo check
cargo test --test issue_1308_forced_break_hanging_indent -- --nocapture
cargo test --test issue_1139_inline_picture_duplicate -- --nocapture
```

결과:

- #1308 강제 줄넘김/내어쓰기/커서 이동 테스트 8개 통과
- `issue_1139_inline_picture_duplicate` 전체 68개 통과
- 목표 페이지 SVG/render-tree 생성 성공

## 7. 현재 판정

현재 증거 기준으로는 가설 A를 범위 제한 조건과 함께 채택할 수 있다.

다만 기존 테스트 판정 기준 자체가 잘못되었을 가능성이 있는 영역이므로, 최종 완료 전에는 메인테이너 시각 판정이 필요하다.

요청 판정 대상:

- `output/poc/task1310/stage2_fixed/3-09월_교육_통합_2022_010.svg`
