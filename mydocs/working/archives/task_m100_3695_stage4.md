# task_m100_3695 Stage 4 완료보고서 — draft PR 생성과 base drift 재동기화

- **Issue**: #3695
- **상위 이슈**: #1528
- **Draft PR**: [#3749](https://github.com/edwardkim/rhwp/pull/3749)
- **브랜치**: `codex/issue-3695-export-structure-auto`
- **PR 최초 head**: `4c0a511fde84c8d762cccb715a149f7dac2667b2`
- **최신 통합 기준**: `upstream/devel` `cc3829116`
- **완료일**: 2026-08-02
- **상태**: 최신-base head push와 PR CI 대기

## 1. PR 생성과 drift 확인

작업지시자 승인 뒤 fork 브랜치를 push하고 `devel` 대상 draft PR #3749를 생성했다. PR의 최초 head,
base, 본문을 확인하는 사이 `devel`이 `fe9749d54`에서 PR #3742 merge commit `cc3829116`까지
56개 커밋 전진해 GitHub가 `DIRTY`·`CONFLICTING`으로 판정했다.

## 2. 재통합 결과

최신 `upstream/devel`을 다시 merge했다. source와 `mydocs/manual/cli_commands.md`는 자동 통합됐고,
충돌은 양쪽이 새로 만든 `mydocs/orders/20260802.md`의 add/add 한 건뿐이었다. `todo` 규약에 따라
upstream의 Kevin·planet6897 통합 검토 기록을 그대로 보존하고, 뒤에 #1528·#3695·#3744 M100 표를
추가했다.

최신 devel 대비 #3695 net diff는 auto selector, 회귀 테스트, 관련 매뉴얼·계획·보고 문서로 한정된다.
새 devel의 renderer·MCP·CLI 변경은 merge base에 포함되고 #3695 자체 변경으로 나타나지 않는다.

## 3. 최신 결합 트리 검증

모든 Cargo 명령은 `CARGO_INCREMENTAL=0`으로 순차 실행했다.

| 명령 | 결과 |
| --- | --- |
| `cargo test --lib document_core::queries::structure -- --nocapture` | 6 passed |
| `cargo test --test issue_3695_structure_auto_policy -- --nocapture` | 8 passed |
| `cargo test --test issue_3693_structure_clause_context -- --nocapture` | 3 passed |
| `cargo test --test cli_json_contract export_structure_ -- --nocapture` | 4 passed |
| `cargo fmt --check` | 통과 |
| `git diff --check`, `git diff --cached --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |

전체 release-test 4,480 passed / 0 failed / 26 ignored는 직전 `fe9749d54` 통합 트리에서 수행했다.
새 base `cc3829116`은 PR #3742의 full CI를 통과했고, 두 변경의 최종 결합 전체 검증은 PR #3749의
최신 head CI가 담당한다. #3695 net diff는 query-only이므로 별도 시각 검증은 비대상이다.

## 4. 다음 게이트

재통합 merge commit을 fork branch에 push한 뒤 PR #3749가 `CLEAN`·`MERGEABLE`인지 확인한다. 이후
latest head CI와 리뷰를 근거로 draft 해제·merge 승인 단계로 넘어간다.
