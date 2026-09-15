---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-14
---

# PR #7107 메인터너 충돌 해결 및 검토 실행 기록

## source 보존과 완료 단계

- 원 contributor head `7a805793476ad8361d4eee87119ff1dcec43b329`,
  source `LJYeon12/rhwp:fix/inline-table-outer-margins-ci`를 유지했다.
- 사용자에게 승인받은 conflict 해결은 최신 `upstream/devel`의
  `d2fc85a9df92bd6a7e2c367bd703d8fad3ef489b`를 병합하는 방식으로 수행했다.
  contributor commit rebase/amend/force-push는 수행하지 않았다.
- `6e5dc6f52b3b5d5c14333a734d95e628cbd77f7c`:
  `merge: PR 7107에 최신 devel을 반영하고 조판 조건 충돌 해소`.
  `paragraph_layout.rs`의 `uses_stored_segment_geometry` 충돌에서 devel의
  `physical_frame_rows` 우선순위와 원 PR의 문단 여백 중복 차감 방지를 함께 보존했다.
  이는 소스 branch 직접 보정에 대한 기존 작업지시 범위이며 새 통합 PR은 만들지 않았다.
- LFS 대상 0 확인 → ref dry-run → 위 source branch 실제 push → GitHub head 일치를 확인했다.
- 해당 코드의 로컬 lint/focused 및 최신 Full CI가 성공했다. 이후 사용자 지시에 따라
  정식 review에서는 CI를 재사용하고 회귀테스트를 반복하지 않았다.
- 가시성 branch `review/ljyeon12-pr7107-20260914`에서 추가 코드 수정 없이
  독립 PDF 좌표·새 WASM Visual Sweep 검토를 완료했다. [review](pr_7107_review.md)의 판정은 승인이다.

## 최종 기록과 남은 순서

1. 완료: `pr_7107_review.md`, 이 실행 기록, `mydocs/orders/20260914.md`, PNG/JSON/log 증적을
   같은 branch의 trailing 문서 commit으로 준비한다. code/test/fixture/baseline 변경은 섞지 않는다.
2. 원격 반영 시: PR/fork의 head가 여전히 `6e5dc6f52`인지 확인하고, 새 변경이 있으면 먼저 판독한다.
   LFS 사전 판독과 dry-run 뒤 승인 범위에서만 기록을 push한다. contributor history를 덮어쓰지 않는다.
3. GitHub review 게시와 merge는 작업지시자 승인 후 수행한다. trailing head에 실제로 발행된
   preflight/aggregate 결과를 확인한다. 문서 commit이라고 자동 fast-pass를 단정하지 않는다.
4. 승인된 merge 직전 exact final head, MERGEABLE/CLEAN, 최신 CI를 재조회한다.
5. merge 뒤 확정 SHA와 이미지 devel 반영 확인 → review의 contributor comment 계획 실행 →
   관련 issue 상태 확인 → 로컬 devel 동기화 → 허용된 duration refresh만 실행되는지 확인한다.
6. 이번 소유 local branch/target만 정리한다. contributor fork branch는 유지한다.
   `target/pr7107-conflict-20260914` 정리는 실행 중 Cargo/Rust가 없고 종료된 작업 소유임을
   확인한 뒤 [post_merge 7.7.1](../../manual/pr_review/post_merge.md)에 따른다.

## 복구 범위

머지 전에는 문서 commit을 로컬에서 유지하며 source branch를 강제로 되돌리지 않는다.
추가 수정이 필요하면 같은 가시성 branch의 별도 commit으로 적용하고 변경된 코드에 맞게 검증을 갱신한다.
공유 devel·contributor commit을 reset하거나 다른 작업 산출물을 삭제하지 않는다.
