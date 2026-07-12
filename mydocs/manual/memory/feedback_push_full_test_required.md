---
name: push-cargo-test-tests-fmt-check
description: cargo test --lib만으로는 통합 테스트(tests/issue_*.rs) 회귀를 못 잡음. push/머지 전 --tests 전체 + fmt --check 동시 검증 필수
metadata: 
  node_type: memory
  type: feedback
  originSessionId: da1865ca-614e-44a5-8c3e-ce3fe8956096
---

**Why:** PR #1020 머지 (2026-05-20) 시 자기 검증으로 `cargo test --release --lib` (1307 passed) 만 확인하고 `--test issue_826` 통합 테스트를 누락하여 CI 회귀 발생. `tests/issue_826.rs:52` 의 "기존 매핑된 PUA 정합 유지" 회귀 가드가 `U+F02B1 → ①` 기대로 남아 있었으나 PR #1020 이 매핑 entry 를 제거 (raw passthrough). 메인테이너 hotfix 1차 push 시 또 fmt 미검증으로 2차 hotfix 필요 + 사고로 다른 PR source 흡수.

**How to apply:**

PR cherry-pick / hotfix push 전 **세 가지 모두 실행**:

```bash
cargo test --release --lib       # 1차 빠른 unit 회귀
cargo test --release --tests     # 2차 통합 테스트 (tests/issue_*.rs 등)
cargo fmt --check                # 3차 fmt 검증
```

**`--lib` 만으로 충분하지 않은 이유:**
- 통합 테스트(`tests/issue_826.rs` 등)는 `--lib` 에 포함되지 않음
- 회귀 가드 테스트가 통합 영역에 많음 (특히 PUA 매핑, 한컴 호환)
- CI 가 `--all` 또는 `--tests` 로 실행하므로 본 환경과 차이

**관련 메모리:** [[feedback_v076_regression_origin]] (컨트리뷰터 환경 vs 작업지시자 환경 차이) + [[feedback_pr_supersede_chain]] (사고로 hotfix 적층) + [[feedback_release_manual_required]] (릴리즈 매뉴얼 정독)

**사고 사례:**
- 2026-05-20 PR #1020 머지 → CI #26135345296 실패 (issue_826)
- hotfix 1차 `1b58f12c` (단언 정정) → CI #26136717196 실패 (fmt)
- hotfix 2차 `3ed82975` (fmt 정정, **사고로 PR #1021 source 흡수**)
- PR #1021 cherry-pick `7f879ab7` 로 KTX golden 정합 + CI 회복 (`b5d38346`)
