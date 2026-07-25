---
kind: reference
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-24
---

# PR #3231 통합 실행 계획 — 선택 삭제 snapshot 전환

#3231은 Studio 명령 경로만 바꾸며 #3228·#3240의 Rust 머리말 변경과 직접 충돌하지 않는다. 다만 세
원 PR을 같은 contributor 통합 PR로 반영하므로, `cf00036ce` 적용 뒤 `tsc`, `npm test`, WASM build와
전체 Rust test를 누적 tree에서 다시 실행한다. snapshot 예산은 #3230에서 별도 판단하며 이 통합에서
숫자·eviction 정책을 바꾸지 않는다.

롤백은 `cf00036ce` 하나로 가능하다. 최종 PR에는 [검토 기록](pr_3231_review.md)과 오늘할일을 포함하고,
원 PR의 `BEHIND`는 직접 update/merge가 아닌 최신 `devel` 대상 통합 PR로 대체한다.
