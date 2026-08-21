---
kind: pr-review
status: review-complete-pending-merge
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-21
---

# PR #5835 검토 - review-only green 후보 실행 재사용

## 접수 메타데이터

| 항목 | 검토 기록 |
| --- | --- |
| PR / 작성자 | [#5835](https://github.com/edwardkim/rhwp/pull/5835) / `jangster77` self-review |
| 관련 issue | [#5834](https://github.com/edwardkim/rhwp/issues/5834) |
| base / code candidate | `devel` / `7ed4296b581374477637a82c12d739bff99c93ef` |
| source branch | `fix/5834-review-tail-candidate` |
| 변경 규모 | 3 files, +210 / -58 |
| 라우팅 | `collaborator_self_merge` + `intake_and_review` + `local_validation` |
| GitHub 상태 | 작성 시점 Open, non-draft, `MERGEABLE`; workflow 변경 CI 실행 중이며 merge 전 최신 head 재확인 필요 |

## 변경 범위와 판정

[PR #5832](https://github.com/edwardkim/rhwp/pull/5832)에서는 Full CI가 성공한 `af9ea69`가 review 문서와
신규 PDF 증적을 함께 담았다는 이유만으로 trailing tail에 소비됐다. 그 결과 Adapter inter-diff와
Proptest roundtrip은 실제 green head 대신 이전 `47251a1`의 run을 찾았고,
`review-tail-candidate-run-unavailable`으로 worker를 다시 실행했다.

두 worker preflight를 CI·CodeQL·Render Diff와 같은 후보 탐색 방식으로 맞췄다. 최신 review-only commit부터
차례로 같은 PR branch, source repository, PR 생성 이후 실행 조건을 확인한다. 현재 head의 run이 진행 중이거나
없으면 더 이전 후보를 계속 확인하고, 완료 실패·identity 불일치는 즉시 fail-closed로 worker를 실행한다.
후보 tail은 single-parent 20개 이하와 parent chain을 계속 요구한다.

renderer, HWP/HWPX fixture, Rust source, 기준 PDF는 바꾸지 않았다. 시각 검증은 적용 대상이 아니다.

## 로컬 검증

- `python3 -m unittest discover -s scripts/tests -p 'test_*workflow*.py'`: **171 passed**.
- `cargo fmt --all -- --check`: 통과.
- `git diff --check`: 통과.
- Python compile: `test_review_only_fast_pass_workflows.py`, `test_proptest_roundtrip_workflow.py`,
  `test_adapter_diff_workflow.py` 통과.

추가 계약은 #5832와 동일한 graph를 고정한다. green review-only head 뒤의 오늘할일 commit은 worker를 skip하며,
그 green 후보가 실패한 경우에는 더 오래된 성공 run으로 우회하지 않고 worker를 실행한다. 기존 PDF/mydocs
허용 경로, fork repository identity, modified PDF 거부도 함께 통과했다.

## 최종 권고

**수용 권고, 최신 GitHub Actions 승인 대기.** CI workflow를 바꾸므로 이 PR의 trailing review·오늘할일 commit도
fast-pass 대상이 아니며, 최신 head의 Full CI·CodeQL·필요 check 성공과 작업지시자 승인을 확인한 뒤 merge한다.
