---
kind: snapshot
status: active
canonical: mydocs/pr/archives/pr_7099_review.md
last_verified: 2026-09-14
---

# PR #7099 체리픽 충돌 보정 기록

- base: `410d22cdf77e3e9e45159999cab19719c621c025`.
- source: `465534ececc6a57702005aef50821f6e871f50eb`.
- 적용: `06a9322bcf7d6283f11979694cf9f3b81d6e4b85` (원 author와 -x 출처 유지).
- 충돌: `src/renderer/height_measurer.rs`, `tests/golden_svg/issue-267/ktx-toc-page.svg`.
- 해소: 두 파일은 기존 devel 구현을 유지했다. 이미 `0a6224e86`이 완전한 빈 문단을 처리하고 있었으며, 원 PR의 더 넓은 trim 판정과 golden 이동은 통합에서 제외했다.
- 실제 신규 diff: `tests/cases/issue_7097_cell_last_empty_line_trailing_ls.rs`의 원 기여 테스트 2개뿐이다.
- 최종 통합 code head: `b40c2953cf526b86319f2841c5b5584d7a83d735`.
- #7141·#7142 제외 전 branch는 `codex/non-draft-integration-20260914-before-exclusions`에 보존했다. 제외된 commit은 현재 통합 head의 조상이 아니다.
- native build, fresh WASM, 원본 CI와 전후 Visual Sweep 결과는 [검토 문서](pr_7099_review.md) 및 공통 증적을 따른다. 기존 PDF 차이를 해소했다고 주장하지 않는다.
- 로컬 단계 완료. 원격 push/통합 PR/CI/merge/원 PR close는 미실행이다.
