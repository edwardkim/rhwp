---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-08
---

# Task #3820 Stage 54 — 전체 회귀·Clippy·native Skia

## 시작 기준

- 시작 commit: `186b8a9f8`
- 브랜치: `task/3820-3821-fidelity`
- integration target: `target/task-3820-3821-fidelity-rebase`
- Stage 53 focused 회귀와 675개 fixture `overflow_cell_baseline`: 통과
- issue1891 overflow 합계: 기준 상한 34줄 유지
- #3637: 31쪽 유지

## 검증 순서

1. 최종 commit에서 `cargo test --profile release-test --tests`를 종료 summary와 exit
   code까지 기다린다.
2. 전체 integration 통과 뒤 native Skia library, missing-picture, direct-PDF 회귀를
   각각 실행한다.
3. full Clippy를 `-D warnings`로 실행한다.
4. 최종 `cargo fmt --all -- --check`와 `git diff --check`를 확인한다.

## 결과

### 전체 integration

```text
CARGO_INCREMENTAL=0
CARGO_TARGET_DIR=target/task-3820-3821-fidelity-rebase
cargo test --profile release-test --tests
exit: 0
```

- library: 3315 passed, 0 failed, 10 ignored
- 모든 integration test binary: 실패 0
- `issue_1891`: 4 passed
- `issue_1921_59043_pagination_pin`: 5 passed
- `issue_2007_nested_cell_pagination`: 15 passed
- #3637 집중 binary 3종: 각 1 passed
- `overflow_cell_baseline`: 1 passed, 0 failed, 81.10s

Stage 53의 별도 `--nocapture` 전수 결과도 675 fixture(스킵 3), 비영점 17종,
총 691줄로 기준 증가가 없었다.

### Native Skia

공식 회귀 3종을 `target/task-3820-3821-fidelity-skia`에서 실행했다.

- `cargo test --profile release-test --features native-skia skia --lib`:
  58 passed, 0 failed
- `cargo test --profile release-test --features native-skia --test
  issue_2225_missing_picture_placeholder`: 2 passed, 0 failed
- `cargo test --profile release-test --features native-skia --test
  render_p37_direct_pdf_export`: 4 passed, 0 failed

### 정적 검사

- `CARGO_TARGET_DIR=target/task-3820-3821-fidelity-clippy cargo clippy
  --all-targets -- -D warnings`: 통과, 57.39s
- `cargo fmt --all -- --check`: 통과
- `git diff --check`: 통과

전체 release-test, Native Skia 3종, full Clippy에 실패가 없다. WASM build는 이 Stage의
중복 실행 범위가 아니며 사용자 수동 검증 결과를 유지한다.
