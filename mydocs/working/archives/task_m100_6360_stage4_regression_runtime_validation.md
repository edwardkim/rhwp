---
kind: working
status: active
issue: 6360
---

# #6360 Stage 4: 회귀 테스트 단축 변경 검증과 보정

## 시작 기준

- 이전 stage 문서: `mydocs/working/task_m100_6360_stage3_regression_runtime_impl_plan.md`
- checkpoint commit: `3fbc5e5c4`
- branch: `fix/pdf-reference-fast-pass-20260829`

Stage 3에서는 다음 변경을 중간 고정했다.

- `convert_verify_corpus_ratchet` 4분할을 16분할로 확대하고 size-greedy bucket으로 변경.
- `hwp5_roundtrip_baseline` 소형/대형 sweep을 여러 testcase로 분할.
- `overflow_cell_baseline`, `text_overlap_baseline`, `off_canvas_baseline`,
  `oracle_page_count_baseline`을 partition 기반으로 분할.
- `security_corpus_regression`, `injection_scan_contract`의 정상 corpus clean sweep에서
  반복 CLI 호출을 줄이는 in-process 경로를 도입.
- `samples/issue2063_huge_cellbreak_table.hwp` page_count 중복 측정을 #2063 sentinel로 통합.

## 현재 확인된 검증 이슈

다음 명령은 잘못된 검증 명령이었다.

```bash
cargo test --profile release-test --target-dir target/pr-review \
  --test convert_verify_corpus_ratchet \
  --test hwp5_roundtrip_baseline \
  --test security_corpus_regression \
  --test injection_scan_contract \
  --test issue_2063 \
  --test issue_1842 \
  --test issue_2070_rowbreak_density \
  --test overflow_cell_baseline \
  --no-run
```

현재 저장소는 다수 `tests/*.rs` 파일을 `tests/generated/regression_suite_*.rs`로 묶어
실행한다. 따라서 `convert_verify_corpus_ratchet` 같은 파일명이 Cargo test target으로
직접 노출되지 않는다. 검증은 `scripts/rust-test-suite-manifest.mjs --prepare` 이후
생성된 suite mapping을 기준으로 수행해야 한다.

## 다음 작업

1. generated suite mapping을 최신화하고 변경 테스트가 포함된 suite를 확인한다.
2. compile error와 clippy/fmt 가능성을 먼저 보정한다.
3. 관련 suite와 direct target을 `ci-duration-observation` 프로필로 실행해 testcase duration을
   측정한다.
4. 측정 결과와 남은 병목을 다음 stage 문서에 기록한다.

PR 생성은 사용자 승인 전까지 진행하지 않는다.
