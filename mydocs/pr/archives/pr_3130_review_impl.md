# PR #3130 통합 적용 기록 — revision derived state

## 적용

| 항목 | 내용 |
| --- | --- |
| 기준 / 브랜치 | `upstream/devel@1b5950a95` / `integrate/postmelee-20260724` |
| 누적 위치 | #3125 뒤의 2/3 |
| 제외 | 원 PR 내부 devel merge `f4ac9cd` |
| 적용 SHA | `38df127, 0db4b6e, a09788a, 850cb69, 852277e, fc91d90, b211442, 599f643, b86723b, 4e0032b` |

## 조정 기록

체리픽 충돌은 render normalization과 deferred pagination의 공통 편집/조회 경로에서 발생했다.
메인터너는 source/overlay 선택, section revision invalidation, deferred job의 입력 생명주기,
profiler와 회귀 테스트를 모두 보존하도록 해결했다. 단순히 한쪽 파일을 선택하거나 원 PR을
rebase하지 않았으며, 통합 tree에서 `cargo check --lib`와 전체 선택 게이트로 확인했다.

## 후속 순서

1. 통합 PR 최신 head CI를 확인한다.
2. merge 승인 뒤 통합 PR만 merge한다. 원 #3130은 supersede 대상으로 남긴다.
3. #2308의 이슈가 아직 후속 작업을 갖는지 재확인한 뒤, 원 PR 감사/close 문안은 별도 승인으로 분리한다.
