# PR #3461 검토 기록 — Studio PDF 안내 E2E 멱등화

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3461](https://github.com/edwardkim/rhwp/pull/3461) — `test(studio/#3450): PDF 안내 모달 E2E 멱등화` |
| 작성자·검토자 | `@planet6897` (external contributor) · `@jangster77` (collaborator) |
| base / source head | `devel` / `b8d1ab08c1e7fd8edfc1d71668984e12b9a30985` |
| 통합 검토 | `review/planet6897-20260727`; 적용 `b8d1ab08…` → `2d2163526` |
| 작성 시점 source 상태 | `MERGEABLE` / `BEHIND`, source CI 전체 성공 |
| 라우팅 | `collaborator_external_pr` + `intake_and_review`, `local_validation`, `multi_pr_update_branch` |

## 판정 및 검증

시험이 시작 전 PDF 안내 preference를 설정한 뒤 `loadApp()`을 호출하고 종료 시 기존 상태를 복원한다.
따라서 앞선 E2E가 남긴 localStorage 상태에 따라 모달이 사라지는 flake를 제거하며, 제품 동작은 바꾸지
않는다. 새 Vite/Chrome headless에서 `print-pdf-issue3126.test.mjs --mode=headless`가 성공했다.
통합 Studio `npm test` 670/0, TypeScript no-emit 및 production build도 성공했다.

## 최종 권고

**기술적으로 수용 가능**. 최신 통합 PR CI·mergeable과 작업지시자 승인을 최종 조건으로 둔다.
