---
kind: pr_review
status: accepted-for-integrated-merge
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-08
---

# PR #4247 검토 - 단일줄 과밀 판정 memo

## 대상과 범위

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#4247](https://github.com/edwardkim/rhwp/pull/4247) / @humdrum00001010 |
| contributor 원 head | `376b10611a275607701e8eaf61d6608af42db241` |
| base / 규모 | `devel`, 16개 파일, +853/-7 |
| 관련 이슈 | [#4169](https://github.com/edwardkim/rhwp/issues/4169) |
| 작성 시점 원격 상태 | `mergeable=MERGEABLE`, `mergeStateStatus=CLEAN` |

저장된 단일줄 문단의 과밀 여부를 `AtomicU64` memo로 보관해 입력이 불변인 재측정을 피한다. 텍스트,
문자 모양, 셀 폭을 바꾸는 모든 편집 경로에서 무효화하고, 이미 과밀인 문단의 fresh 재래핑은 유지한다.

## 검증과 판단

- 원 head의 `Build & Test`가 통과했다.
- 통합 후보에서 `overflow_cell_baseline`은 275.56초에, 전체 `release-test --tests`는 종료 코드 0으로
  통과했다. 과밀 판단 memo가 overflow-cell 기준선을 숨기지 않았음을 확인했다.
- renderer/layout 영향의 Native Skia 3종은 #4265 최신 Full CI의 결과를 merge 전에 다시 확인한다.

**통합 수용 권고.** #4246의 메트릭 색인 뒤에 적용하며, #4248/#4249의 셀 캐럿 fast path와 같은
문단 측정·무효화 계약을 공유한다.
