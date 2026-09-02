---
kind: pr-review
status: pending-ci-release-hold
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-12
---

# PR #4575 리뷰 - 개체 줄 누수 위양성 보강

| 항목 | 문서 작성 시점 참고값 |
| --- | --- |
| PR | [#4575](https://github.com/edwardkim/rhwp/pull/4575) |
| 작성자 | `planet6897` |
| base / 원 head | `devel` / `0b413cb05a2f703d865b0d32088b1e5e1c41d76e` |
| 원 변경 규모 | 1 file, +22/-2 |
| 통합 적용 | `56f0c8558` |
| 관련 이슈 | [#4533](https://github.com/edwardkim/rhwp/issues/4533) |

#4572와 같은 `verify_ladder_drift.py` 영역을 수정하므로 단순 중첩 적용하지 않았다. 통합 충돌 해소에서
`NODE_RE`가 실제 node 행의 `y=`를 요구하도록 #4575의 더 엄격한 조건을 유지하면서, #4572의
FOREIGN/분절/중앙값 가드도 모두 보존했다. 따라서 임의의 텍스트 줄을 node로 오인하지 않고 `기타` 같은
비영문 node label도 조상 추적에 포함한다.

Python compile·synthetic nested `기타` guard 및 통합 HEAD release-test/Clippy가 통과했다. 최신 통합 PR CI가
통과한 뒤에도 릴리스 준비 종료와 작업지시자 승인이 있기 전에는 merge 또는 원 PR close를 하지 않는다.
