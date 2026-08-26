# PR #5014 검토 - `edit delete-col`

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#5014](https://github.com/edwardkim/rhwp/pull/5014) |
| 작성자 | `kevin9327` |
| source base / head | `devel` / `02ee1e8834ffa66a9273714c545177dc3daf3904` |
| 검토 브랜치 | `review/kevin9327-unincluded-5175-20260817` |
| 실제 적용 source commit | `064adad296c453af61c1c9607b2ec3cb6c0a8deb` |
| 누적 commit | `219a0d858e1502fd4457f2f86577d10d52b70f91` |
| 파생 산출물 보정 | `5f7caf48a` (공통) |
| source PR 상태 | `OPEN`, non-draft |

## 검토 경로

```text
base route: maintainer_general.md
modifiers: intake_and_review.md, local_validation.md, multi_pr_update_branch.md
loaded documents: pr_review_workflow.md, pr_review/README.md, maintainer_general.md, intake_and_review.md, local_validation.md, multi_pr_update_branch.md
current head: 02ee1e8834ffa66a9273714c545177dc3daf3904 (작성 시점 참고값)
```

## 검토 범위

stacked source head에서 본문 최상위 표의 열을 삭제하는 고유 commit만 적용했다. `hwp_delete_col` schema는 `path`, `table`, `col`을 필수로 선언하며 CLI의 필수 인수와 일치한다. 원본 계약은 표 열 삭제 결과와 MCP 선언을 확인한다.

## 검토 결과

차단·수정 필요 결함을 발견하지 못했다. 표 좌표를 `export-tables`의 최상위 표 index로 제한하고, dry-run과 출력 경로를 공통 edit 완료 경로로 전달한다.

## 누적 검증

`--prepare`를 review worktree에서 한 번 실행한 뒤, 전용 target의 전체 integration 회귀가 `6,643 passed, 38 skipped`로 통과했다. 파생 harness와 manifest는 검증 후 복원해 누적 commit에 포함하지 않았다.

## 권고

누적 통합 범위에서 승인한다. source PR의 선행 stacked 변경은 이 판단에 포함하지 않는다.
