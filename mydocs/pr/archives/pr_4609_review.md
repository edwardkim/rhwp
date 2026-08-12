---
kind: pr-review
status: pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-11
---

# PR #4609 리뷰 - 완료된 stale run 취소 오류 재조회

## 라우팅과 접수

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md
```

| 항목 | 문서 작성 시점 참고값 |
| --- | --- |
| PR | [#4609](https://github.com/edwardkim/rhwp/pull/4609) |
| 관련 이슈 | [#4608](https://github.com/edwardkim/rhwp/issues/4608) |
| 작성자 | `jangster77` |
| base | `devel` / `14cb775cdad4b013bbe00fc2872dcb8eba13806b` |
| code candidate | `81603f81d04cd83b0903396cabe3b64e58e0fae8` |
| trailing review head | 이 문서와 오늘할일을 포함한 후속 docs-only commit |
| 변경 범위 | stale run 취소 workflow, CI contract 배선, Python workflow contract test |
| reviewer request | 작성자와 self-review 담당이 같아 비어 있음 |

## 변경 판단

[PR #4602 stale cancel job](https://github.com/edwardkim/rhwp/actions/runs/31494843278/job/93789854722)은 이전
run을 실제로 취소했지만 GitHub REST endpoint가 같은 요청에 500을 반환해 housekeeping workflow가 실패했다.

이 PR은 오류 상태 코드를 성공으로 허용하지 않는다. `force-cancel` 오류 뒤 대상 workflow run을 다시 읽고,
실제 상태가 `completed`일 때만 정상 경과로 기록한다. 대상이 아직 active이거나 재조회 자체가 실패하면 원래
오류를 계속 전파하므로 실제 stale run 정리 실패는 숨기지 않는다.

정적 contract test가 기존 `409` 단독 예외를 금지하고, 재조회·active 상태 오류 전파·completed 기록 순서를
검증한다. 해당 test는 CI Lint job의 workflow contract 단계에 배선했다.

## 완료한 검증

- `python3 -m unittest scripts/tests/test_cancel_stale_pr_runs_workflow.py scripts/tests/test_workflow_contract_wiring.py`:
  4건 통과.
- `python3 -m unittest scripts/tests/test_ci_impact_workflow.py`: 27건 통과.
- `git diff --check`: 통과.

Rust, renderer, WASM 구현과 sample은 변경하지 않았다. 따라서 전체 Cargo 회귀와 시각 검증은 범위 밖이며,
workflow 변경의 실제 event·권한 경계 검증은 최신 PR head의 GitHub Actions가 담당한다.

## 최종 권고

**현재는 CI 대기.** 최신 head의 Full CI와 CodeQL이 통과하고 작업지시자가 승인하면 수용한다. merge 뒤에는
[#4608](https://github.com/edwardkim/rhwp/issues/4608)의 자동 close 상태와 integration branch 정리를
`post_merge.md` 절차로 확인한다.
