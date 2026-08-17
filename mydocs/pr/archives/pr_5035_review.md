# PR #5035 검토 - `edit delete-table`

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#5035](https://github.com/edwardkim/rhwp/pull/5035) |
| 작성자 | `kevin9327` |
| source base / head | `devel` / `1a9c3587d629609e037fd44fc9b4a0efa6f619e5` |
| 검토 브랜치 | `review/kevin9327-unincluded-5175-20260817` |
| 실제 적용 source commit | `7bdffea5c5e4a3677255b4a78fdf15f4c279d9bd` |
| 누적 commit | `8c4347c5267f5c16c8ac2912ed3ac7ac287b51f4` |
| 파생 산출물 보정 | `bbed06487` |
| source PR 상태 | `OPEN`, non-draft |

## 검토 경로

```text
base route: maintainer_general.md
modifiers: intake_and_review.md, local_validation.md, multi_pr_update_branch.md
loaded documents: pr_review_workflow.md, pr_review/README.md, maintainer_general.md, intake_and_review.md, local_validation.md, multi_pr_update_branch.md
current head: 1a9c3587d629609e037fd44fc9b4a0efa6f619e5 (작성 시점 참고값)
```

## 검토 범위

source PR은 선행 CLI 기능을 누적한 stacked head다. 이 통합에서는 본문 최상위 표 한 건을 삭제하는 `delete-table` 고유 커밋만 적용했다.

- `rhwp edit delete-table --table N`의 CLI, capabilities, MCP 선언
- `tests/cases/delete_table_contract.rs`의 원본 계약
- `tests/suites/unit-test-tiers.json`의 tier 입력

`tests/generated/regression_suite_026.rs`와 `tests/suites/manifest.json`은 #5177 정책에 따라 누적 commit에서 제외했다.

## 검토 결과

발견한 차단·수정 필요 결함은 없다. 계약은 `extract_tables`와 같은 본문 최상위 표 index를 사용해 삭제 전후 개수가 정확히 하나 감소함을 확인하고, dry-run 및 MCP 선언도 대조한다.

## 누적 검증

전용 target `target/pr-review-kevin9327-unincluded-5175-20260817`에서 전체 integration 회귀를 실행해
`6,643 passed, 38 skipped`를 확인했다. suite 생성은 검토 시작 시 한 번만 수행했으며, 생성 산출물은 검증 뒤 기준 상태로 복원했다.

## 권고

누적 통합 범위에서 승인한다. source PR의 나머지 stacked 변경은 별도 고유 증분 검토 대상으로 남긴다.
