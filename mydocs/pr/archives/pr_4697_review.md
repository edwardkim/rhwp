---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-13
---

# PR #4697 검토 - 단일 단 전면 개체의 저장 `vpos` reset

| 항목 | 기록 |
| --- | --- |
| PR | [#4697](https://github.com/edwardkim/rhwp/pull/4697) |
| 작성자 / 원 head | @planet6897 / `11cdee18c55c80fa661c370b43118a79c705bef3` |
| 적용 commit | `6e2165eee` |
| 통합 후보 | `c7cfaefb9` |

단일 단 전면 개체 줄의 저장 `vpos`가 0으로 재시작될 때, 앞 조각 좌표를 계속 누적해 페이지 밖으로
적재하던 경로를 바로잡는다.

## 완료한 검증

- 비공개 fixture에서 control 970의 조각이 191~199쪽에 모두 분배되는 것을 확인했다.
- 전체 페이지 수 260쪽과 한컴 기준 235쪽의 차이는 이 수정으로 해결되지 않는 별도 fidelity 범위이며
  [#4092](https://github.com/edwardkim/rhwp/issues/4092)에 남긴다.
- 누적 후보 전체 `nextest`는 5,923건 통과, 37건 제외, 실패 0건이었다.

**통합 수용 대상이다.**
