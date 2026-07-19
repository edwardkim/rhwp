---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-17
name: feedback-release-test-profile
description: push 전 통합 테스트는 release-test 프로필 사용 권고 — Linux(WSL2)에서도 dev 프로필 대비 ~2.4배 빠름
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 2560b31a-9f1c-4764-bbf1-7ba5fc27c7ce
---

작업지시자 권고 (2026-06-12, #1388 마무리 중): push/머지 전 통합 테스트 검증을
`cargo test --tests`(dev 프로필) 대신 **`cargo test --profile release-test --tests`**로
실행한다. macOS뿐 아니라 이 컴퓨터(Linux WSL2)에서도 적용.

**Why:** `[profile.release-test]`(Cargo.toml — inherits release, lto=false,
codegen-units=16)는 통합 테스트 바이너리별 LTO 링크 비용을 제거한다. Linux WSL2 실측
(2026-06-12): dev warm 262s vs release-test warm 108s (~2.4배), cold ~133s.
macOS 실측은 `mydocs/manual/dev_environment_guide.md` 참조 (886s→149s).

**How to apply:** push 전 CI급 검증([[feedback-push-full-test-required]]) 시
`cargo test --profile release-test --tests` + `cargo fmt --check` 사용. 릴리즈 산출물
빌드는 계속 `[profile.release]`(LTO) 사용 — release-test는 검증 속도 전용.
