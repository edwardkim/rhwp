---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-25
---

# PR #7388 메인터너 보정·통합 순서

1. 원 PR head `b19a3ecd0075417cf4e05d2b000b511bd592ca26`를 base `b3e3d4e2170a43ca449e3d832440a9274e4e8ee4`에 `-x`로 적용한 commit은 `dc5369f942baec5a1a7d6c577c075a40bf706aa3`이다. 실제 저장 0 도형 확대 후 undo가 200이 되는 FAIL을 확인했다.
2. `f4ddd1f12945783f51a4124c4fa9727848f6cff9`는 실제 높이 4 가로선 대조군과 저장 0 복원 의도 전달을 추가했다. 양수→0 일반 편집의 최소 크기 보호는 유지했다. 이 보정은 contributor 원 head를 재작성하지 않는다.
3. 통합 code head `459d5e581eac979e8a3c69a72e71fb7cc3c9ea88`에서 집중 검사, 전체 release-test 10,229 PASS, 필수 lint, Studio 1,770 PASS, fresh WASM 브라우저 0→400→0→400을 확인했다. 로그는 `output/pr-review/planet6897-20260924/diagnostics/`에만 둔다.
4. 같은 통합 PR의 archive review를 보존한다. 원격 push·PR 생성은 별도 승인 뒤 수행하고, 최신 PR head CI·mergeability를 확인한다. 실패 시 #7388 보정 commit과 다른 PR 변경을 구분해 원인을 수정하며 contributor branch를 force-push하지 않는다. 실제 merge 뒤에만 원 PR 후속 comment와 소유 output 정리를 진행한다.
