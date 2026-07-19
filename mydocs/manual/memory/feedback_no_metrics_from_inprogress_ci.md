---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-17
name: feedback_no_metrics_from_inprogress_ci
description: CI 진행 중(in_progress) run 에서 step 시간/성능 수치를 측정·보고하지 말 것. 완료 후 측정
metadata: 
  node_type: memory
  type: feedback
  originSessionId: da1865ca-614e-44a5-8c3e-ce3fe8956096
---

CI 성능 수치(step 시간, 단축율 등)는 **run 이 completed 된 뒤에만** 측정·보고한다. in_progress 상태의 `gh api .../jobs` step 데이터는 아직 실행 안 된 무거운 step(예: `cargo test` ~356s, `Clippy`)이 빠져 있어 총계가 크게 왜곡된다.

**Why:** Task #1192(CI 시간 단축)에서 CI 진행 중(Run tests 실행 전) 부분 데이터로 "Build & Test 712s→212s(−70%)" 라 보고했으나, 완료 후 실제는 712s→647s(−9%) 였다. 보고서·orders·이슈 코멘트를 전부 정정해야 했다. "report outcomes faithfully" 원칙 위반.

**How to apply:** (1) 수치 측정 전 `gh run list/view` 로 `status==completed` 확인. (2) 미완료면 ScheduleWakeup 으로 대기 후 재확인. (3) 가장 무거운 step(여기선 cargo test 6분)이 결과에 반영됐는지 step 목록으로 검증. (4) 캐시 효과(B 같은 항목)는 "이번 run cold, 차기 PR warm" 처럼 발생 시점을 정확히 구분해 보고. 관련 [[feedback_push_full_test_required]].
