---
kind: pr_review
status: accepted-for-integrated-merge
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-08
---

# PR #4249 검토 - units 지문 불변 시 eviction 생략

## 대상과 범위

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#4249](https://github.com/edwardkim/rhwp/pull/4249) / @humdrum00001010 |
| contributor 원 head | `d5a13e5713f75b0d4e5096b9eccd2fd34e1585fc` |
| base / 규모 | `devel`, 25개 파일, +2,832/-195 |
| 관련 이슈 | [#4167](https://github.com/edwardkim/rhwp/issues/4167) |
| 작성 시점 원격 상태 | `mergeable=CONFLICTING`, `mergeStateStatus=DIRTY`; #4248 후속 stack이므로 통합 PR에서 해소한다. |

deferred 셀 편집 전후의 units 입력 지문을 비교해, 레이아웃 단위가 불변이면 `cell_units` eviction을
생략한다. 재래핑·spacer 전이처럼 지문이 바뀌는 경우는 기존 eviction 경로를 유지한다.

## 검증과 판단

- 원 head의 `Build & Test`가 통과했다.
- 통합 후보에서 `issue4167` focused 검증과 전체 `release-test --tests`가 통과했다.
- #4248의 parity와 latency 검증도 함께 통과해 cache 재사용이 캐럿 rect 결과를 바꾸지 않았음을 확인했다.

**통합 수용 권고.** #4248 직후에만 적용한다. 단독 PR 충돌은 선행 fast-path stack의 merge 순서를
반영한 것이며 구현 결함으로 분류하지 않는다.
