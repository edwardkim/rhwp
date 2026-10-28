---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-08
---

# Task #3820 Stage 52 — 전체 회귀와 native gate

## 시작 기준

- 시작 commit: `5eecffce2`
- 브랜치: `task/3820-3821-fidelity`
- 전용 target: `target/task-3820-3821-fidelity-rebase`
- [Stage 51](task_m100_3820_stage51_59043_hwpx_projection_repair.md) 집중 회귀:
  59043, issue2007, #2279, #2430, #3637, #1891, #2308 모두 통과
- `cargo fmt --all -- --check`, `git diff --check`: 통과

## 검증 순서

1. `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`를 최종 summary와
   exit code까지 기다린다.
2. 실패하면 전체 gate를 즉시 반복하지 않고 최초 실패 test를 focused 분석한다.
3. 전체 integration이 통과하면 동일 HEAD에서 full Clippy를 실행한다.
4. 마지막으로 native Skia lib, missing-picture, direct-PDF 세 회귀를 실행한다.

## 결과

전체 integration binary는 마지막 `overflow_cell_baseline` 전까지 모두 통과했다. 특히
issue2007 15건, #3637 3건, 59043 5건과 HWP3/HWP5/HWPX 실물 fixture 회귀는 통과했다.

마지막 전수 스윕 결과는 다음과 같다.

```text
overflow-cell 스윕: 샘플 675건(스킵 3) / 0 아닌 문서 17종 / 총 713줄
증가: issue1891_external_bindata_link.hwpx — 34 → 56줄
test result: FAILED. 0 passed; 1 failed; finished in 76.01s
cargo exit: 101
```

이전 전체 실패의 `76076`, `86712`, `issue3637` 증가는 해소됐고, 남은 실패는 direct
HWPX `issue1891_external_bindata_link.hwpx` 한 건이다. 전체 스위트를 반복하지 않고
[Stage 53](task_m100_3820_stage53_issue1891_direct_hwpx_scope.md)에서 이 fixture만
집중 분석한다.
