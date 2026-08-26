# PR #5030 검토 - `edit add-bookmark`

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#5030](https://github.com/edwardkim/rhwp/pull/5030) |
| 작성자 | `kevin9327` |
| source base / head | `devel` / `3c1a0b36a06e5a00f01bb5d208043f2bb58ef83e` |
| 검토 브랜치 | `review/kevin9327-unincluded-5175-20260817` |
| 실제 적용 source commit | `7a69d04a3759b0ac97c3d6d18854e62f7857a515` |
| 누적 commit | `707aa59093ea1f1f959769eab760c29c0dd1f845` |
| 파생 산출물 보정 | `5f7caf48a` (공통) |
| source PR 상태 | `OPEN`, non-draft |

## 검토 경로

```text
base route: maintainer_general.md
modifiers: intake_and_review.md, local_validation.md, multi_pr_update_branch.md
loaded documents: pr_review_workflow.md, pr_review/README.md, maintainer_general.md, intake_and_review.md, local_validation.md, multi_pr_update_branch.md
current head: 3c1a0b36a06e5a00f01bb5d208043f2bb58ef83e (작성 시점 참고값)
```

## 검토 범위

문단 좌표에 이름을 가진 책갈피를 추가하는 고유 commit만 적용했다. 원본 계약은 추가 후 목록 조회와 dry-run을 확인하며, MCP tool 등록도 포함한다.

## 발견 사항과 메인터너 보정

### P2 - MCP schema가 빈·공백 책갈피 이름을 허용함

`hwp_add_bookmark`는 `name`을 required string으로만 선언해 빈·공백 문자열을 허용했다. 반면 CLI는 `name.trim().is_empty()`를 usage error로 거부한다. 메인터너 보정 `3cd523ab1`이 schema에 `.*\\S.*` pattern을 추가하고, capabilities MCP JSON에서 이 제약을 확인하는 계약을 추가했다. 보정 commit 기준의 focused Rust contract 검증과 누적 통합 회귀를 모두 통과했다.

## 누적 검증

보정 뒤 전용 target에서 전체 integration 회귀 `6,643 passed, 38 skipped`를 확인했다. `add_bookmark_contract::mcp_declared`를 포함한 focused MCP contract도 통과했다. generated suite 산출물은 검증 뒤 복원했다.

## 권고

**승인.** 메인터너 보정 `3cd523ab1` 뒤 focused MCP contract와 누적 통합 회귀가 통과했다. source PR의 나머지 stacked 변경은 이 검토의 범위 밖이다.
