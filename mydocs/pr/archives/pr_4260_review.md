---
kind: pr_review
status: accepted-for-integrated-merge
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-08
---

# PR #4260 검토 - 저장 시점 캐럿 메타데이터

## 대상과 범위

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#4260](https://github.com/edwardkim/rhwp/pull/4260) / @humdrum00001010 |
| contributor 원 head | `0d2bf6042ef44dbb30d0e5eb1522a3518d73989d` |
| base / 규모 | `devel`, 17개 파일, +854/-39 |
| 관련 이슈 | [#4180](https://github.com/edwardkim/rhwp/issues/4180) |
| 작성 시점 원격 상태 | `mergeable=MERGEABLE`, `mergeStateStatus=CLEAN` |

마지막 편집 위치가 아닌 실제 저장 시점의 캐럿을 메타데이터에 기록해, 재열기 때 캐럿이 다른 쪽으로
복원되는 문제를 막는다.

## 검증과 판단

- 원 head의 `Build & Test`가 통과했다.
- `issue_4180_caret_stamp_roundtrip`과 전체 `release-test --tests`가 통과했다.
- HWP/HWPX 저장·Studio 입력 경로가 함께 바뀌므로 #4265 최신 Full CI와 package gate를 merge 전에
  다시 확인한다.

**통합 수용 권고.** #4258·#4259 이후, #4261 이전의 pagination·저장 stack으로 누적했다.
