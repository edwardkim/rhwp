# task_m100_3695 Stage 3 완료보고서 — 최신 devel 통합과 PR 전 전체 검증

- **Issue**: #3695
- **상위 이슈**: #1528
- **브랜치**: `codex/issue-3695-export-structure-auto`
- **승인된 범위**: 최신 `devel` 통합, focused·full 검증, 단계 보고
- **채택 WIP**: `8343c98c6`
- **통합 기준**: `upstream/devel` `fe9749d542f46643e408c23878229c326e341363`
- **완료일**: 2026-08-02
- **상태**: 로컬 PR 준비 완료, 후속 PR 생성·base drift 동기화는 Stage 4에서 추적

## 1. 통합 방법과 결과

기존 WIP와 절차 복구 커밋을 재작성하지 않고 `upstream/devel`을 merge했다. `structure.rs`는 자동
통합되어 #3715의 marker 원문 보존·clause 문맥 게이트와 #3695의 `select_auto_mode()`가 모두 남았다.

충돌은 `mydocs/orders/20260801.md` 한 곳뿐이었다. `todo` 규약에 따라 날짜가 지난 보드를 현재 사실로
덮어쓰지 않고, 8월 1일 종료 시점의 #3693 PR 검토 상태와 #3695 Stage 2 승인 상태를 함께 보존했다.
8월 2일 진행 상태는 별도 `mydocs/orders/20260802.md`에 기록했다.

## 2. 최신 devel 대비 독립 대조

- net diff는 #3695의 원래 9개 파일, 593 insertions / 19 deletions와 일치했다.
- source 변경은 `src/document_core/queries/structure.rs`의 auto selector에 한정됐다.
- explicit `outline|clause`, JSON 필드·봉투·exit code는 변경되지 않았다.
- #3715가 추가한 모든 단위·실문서 clause 회귀 테스트를 유지했다.
- 파서·렌더러·레이아웃·직렬화 변경이 없어 시각 검증은 비대상이다.

## 3. 검증 결과

모든 Cargo 명령은 `CARGO_INCREMENTAL=0`으로 같은 checkout에서 순차 실행했다.

| 명령 | 결과 |
| --- | --- |
| `cargo test --lib document_core::queries::structure -- --nocapture` | 6 passed |
| `cargo test --test issue_3695_structure_auto_policy -- --nocapture` | 8 passed |
| `cargo test --test issue_3693_structure_clause_context -- --nocapture` | 3 passed |
| `cargo test --test cli_json_contract export_structure_ -- --nocapture` | 4 passed |
| `cargo test --profile release-test --tests` | 406 binaries, 4,480 passed / 0 failed / 26 ignored |
| `cargo fmt --check` | 통과 |
| `git diff --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |

test list 집계는 4,506건이며 ignored-only list는 26건이다. 전체 실행이 실패 없이 끝났으므로 실제
통과 수는 4,480건이다.

## 4. 판정과 다음 게이트

#3695 구현은 최신 #3715 결과와 양립하며 PR 준비 기준을 충족한다. 로컬 merge commit으로 이번 단계를
고정한 뒤 원격 push와 `devel` 대상 PR 생성은 작업지시자 별도 승인 전까지 수행하지 않는다. 이후
승인을 받아 draft PR #3749를 생성했으며, PR 생성 뒤 발생한 base drift 처리는 Stage 4 보고서에
분리한다. #3695 통합 뒤에는 #3744를 진행하고, 두 이슈가 모두 `devel`에 포함된 뒤 #1528 최종
통합 검증으로 넘어간다.
