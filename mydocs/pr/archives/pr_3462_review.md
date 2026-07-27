# PR #3462 검토 기록 — 대형 Studio 문서의 idle pagination flush 제한

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3462](https://github.com/edwardkim/rhwp/pull/3462) — `fix(studio/#3412): idle 전체 pagination flush` |
| 작성자·검토자 | `@planet6897` (external contributor) · `@jangster77` (collaborator) |
| base / source head | `devel` / `0672b98267abcd9e4d8cf752c7f92418d8728d5e` |
| 통합 검토 | `review/planet6897-20260727`; `6a66bc…` → `e3839344b`, `0672b982…` → `242ec43fd` |
| 작성 시점 source 상태 | `MERGEABLE` / `BEHIND`, source CI 전체 성공 |
| 라우팅 | `collaborator_external_pr` + `intake_and_review`, `local_validation`, `multi_pr_update_branch` |

## 판정

idle full flush는 30쪽 이하이고 deferred runner가 비활성일 때만 예약한다. 115쪽 문서에서 idle 타이머가
재개형 runner를 취소하고 약 839ms main-thread stall을 만드는 경로를 막는다. undo/redo/navigation/blur,
저장·인쇄의 명시 boundary flush는 유지하므로 결과 완결성도 보존한다. static guard는 limit과 runner active
guard를 고정한다.

## 검증

- HWP/HWPX 각각 115 steps의 실제 headless E2E 성공: `flush=0`, HWP boundary `108.60ms`, HWPX
  boundary `107.60ms`; raw/IME·delete·save/export·print barrier 모두 green.
- Studio focused 32/0, 전체 `npm test` 670/0, TypeScript no-emit 및 production build 성공.
- 통합 Rust release-test 전체, Native Skia 3종, fmt·clippy·WASM lib check 성공.

## 최종 권고

**기술적으로 수용 가능**. 최신 통합 PR CI·mergeable과 작업지시자 승인을 최종 조건으로 둔다.
