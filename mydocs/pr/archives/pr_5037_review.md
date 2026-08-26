# PR #5037 검토 - `edit insert-header-footer`

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#5037](https://github.com/edwardkim/rhwp/pull/5037) |
| 작성자 | `kevin9327` |
| source base / head | `devel` / `4ec3ff51de03bae43095007d62aaa641564ef3c0` |
| 검토 브랜치 | `review/kevin9327-unincluded-5175-20260817` |
| 실제 적용 source commit | `a94168ff826f15c2d2e2196ba0c9d8b038fa56f0` |
| 누적 commit | `a2a264cb5e5a47682cc0f380d264b6b6a31c4709` |
| 파생 산출물 보정 | `ca50ba86f` |
| source PR 상태 | `OPEN`, non-draft |

## 검토 경로

```text
base route: maintainer_general.md
modifiers: intake_and_review.md, local_validation.md, multi_pr_update_branch.md
loaded documents: pr_review_workflow.md, pr_review/README.md, maintainer_general.md, intake_and_review.md, local_validation.md, multi_pr_update_branch.md
current head: 4ec3ff51de03bae43095007d62aaa641564ef3c0 (작성 시점 참고값)
```

## 검토 범위

source PR의 stacked head 전체가 아니라, 머리말·꼬리말 생성 기능의 고유 커밋만 적용했다.

- `rhwp edit insert-header-footer`의 CLI, capabilities, MCP 선언
- `tests/cases/insert_header_footer_contract.rs`와 provenance 계약 보강
- `tests/suites/unit-test-tiers.json`의 tier 입력

생성 harness와 manifest는 #5177 정책에 따라 누적 commit에서 제외했다.

## 발견 사항과 메인터너 보정

### P2 - MCP 스키마가 생성 종류를 필수로 요구하지 않음

`hwp_insert_header_footer`의 입력 스키마는 `path`만 `required`로 선언해 생성 종류가 없는 요청을 허용했다. CLI는 `--header` 또는 `--footer` 중 정확히 하나를 요구한다. 메인터너 보정 `3cd523ab1`이 하나의 Boolean만 `true`여야 하는 두 `oneOf` branch를 추가하고, capabilities MCP JSON에서 두 branch의 required·const 계약을 확인하도록 보강했다. 보정 commit 기준의 focused Rust contract 검증과 누적 통합 회귀를 모두 통과했다.

## 누적 검증

보정 뒤 전용 target에서 전체 integration 회귀 `6,643 passed, 38 skipped`를 확인했다. `insert_header_footer_contract::mcp_declared`를 포함한 focused MCP contract도 통과했다. generated suite 산출물은 검증 뒤 복원했다.

## 권고

**승인.** 메인터너 보정 `3cd523ab1` 뒤 focused MCP contract와 누적 통합 회귀가 통과했다. source PR의 나머지 stacked 변경은 이 검토의 범위 밖이다.
