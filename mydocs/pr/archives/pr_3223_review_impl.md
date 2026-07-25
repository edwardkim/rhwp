---
kind: reference
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-24
---

# PR #3223 통합 실행 계획 — square-OLE 문단 메타 복원

원 기능 커밋 `05b4dd807`의 `ParaMeta` 계약을 최신 `devel` 위에 적용한다. 체리픽 중
`src/wasm_api.rs`의 import 충돌은 deferred pagination import와 `parse_removed_para_meta` import를
함께 보존해 해소했다. 이후 maintainer 보정 `0cf1f98f6`은 square-OLE wrap의 새 문단 생성 분기에도
`restore_meta`를 적용한다.

`samples/한셀OLE.hwp`의 Enter → Backspace merge → undo split을 이용해 7개 문단 메타 필드를 모두
검증한다. 회귀 시 보정과 원 기능 커밋을 역순으로 revert하며, 통합 PR CI 성공과 작업지시자 승인 뒤에만
원 PR #3223의 supersede 후속을 처리한다.
