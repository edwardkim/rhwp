# PR #3304 검토 기록

| 항목 | 내용 |
|---|---|
| 통합 PR | [#3304](https://github.com/edwardkim/rhwp/pull/3304) |
| 작성자 / base | `jangster77` / `devel` |
| head branch | `review/kevin9327-cli-json-20260725-v2` |
| code candidate | `bcff621521c4af56eb6a7d68952d4bf1853c0dec` (문서 trailing commit 전) |
| 통합 대상 | #3258, #3262, #3264, #3276, #3280, #3282, #3285, #3288; #3036은 보류로 제외 |
| 규모 | +4,935/-49, 38 files (문서 trailing commit 전) |
| 판단 | 최신 code candidate full CI 통과 뒤 merge 후보 |

## 범위와 보정

- 기여자 PR 8건의 JSON 조회·검증 CLI 기능을 최신 `devel` 위에 누적 체리픽했다.
- 메인터너 보정은 통합 branch에만 포함했다: 단일 입력 계약, table control/containerPath 역참조, 글상자 주소와
  Unicode offset, JSON 실패 stdout 0-byte, command/MCP capability inventory, `rhwp_bin()` 런타임 경로 호환.
- 원 contributor PR head에는 보정 commit을 남기지 않는다. 원 PR은 #3304 merge 뒤 통합 완료 사유로 close한다.

## 검증과 렌더 판정

- 로컬: `cargo test --profile release-test --tests --quiet`, focused JSON contract tests, `cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings`, `git diff --check` 성공.
- renderer·layout·sample·PDF/golden 변경이 없으므로 visual sweep과 IR field-sweep baseline은 대상이 아니다.
- GitHub Actions는 `bcff62152`의 full CI를 성공했다(CI Build & Test, CodeQL, Render Diff 포함). 이 문서의
  trailing docs commit push 뒤에는 fast-pass preflight와 최신 aggregate·mergeability를 다시 확인한다.

## merge 뒤 후속 처리

- review·review_impl·오늘할일은 현재 PR diff에 포함한다. merge 뒤 별도 문서 PR은 만들지 않는다.
- #3237, #3238, #3261, #3263, #3274, #3278, #3281, #3283, #3287의 close 상태를 확인하고 OPEN이면 #3304 merge
  근거와 검증 요약을 남겨 수동 close한다.
- 원 PR 8건에는 #3304 통합 완료·CI·로컬 검증·후속 없음의 comment를 남기고 close한다.
