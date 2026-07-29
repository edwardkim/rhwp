# Task M100 #2424 Stage A 완료보고서 — 최신 56-input pagination subphase 기준선

## 1. 목적

#2430 폰트 메트릭 교정 뒤 flow boundary가 44번째에서 56번째 숫자 입력으로 이동했다. 최신
`upstream/devel@12f8a820`에서 #2424 병목의 소유 subphase와 correctness 제약이 그대로인지 다시 확정한다.

## 2. 환경과 방법

- macOS 26.5.2, arm64
- Rust/Cargo 1.93.1
- profile: `release-test`
- fixture: `issue1949_giant_cell_nested_tables_perf.hwp/.hwpx`
- 각 형식 5회 새 document load → 56회 deferred insert → explicit full flush
- Native 전용 `RHWP_2424_PROFILE=1` 게이트로 pagination subphase와 block-table 호출 시간을 기록
- timing assertion은 두지 않고 56번째 `cellFlowChanged=true`, initial/flushed 115쪽을 단언

명령:

```bash
cargo test --profile release-test --test issue_2424_pagination_subphase_probe --no-run
RHWP_2424_PROFILE=1 RHWP_2424_REPEATS=5 cargo test --profile release-test \
  --test issue_2424_pagination_subphase_probe -- --ignored --nocapture
cargo test --profile release-test --lib \
  issue2214_scoped_cache_coherence_preserves_transient_pagination -- --nocapture
```

## 3. incremental flush 결과

아래 값은 5회 중앙값이며 괄호는 최소–최대다.

| 형식 | 전체 | measurement | typeset | 단일 block-table | postprocess |
|---|---:|---:|---:|---:|---:|
| HWP | 1058.381ms (991.596–3336.130) | 13.079ms (12.762–23.822) | 1044.644ms (978.363–3311.536) | 1044.595ms (978.313–3311.436) | 0.033ms (0.030–0.052) |
| HWPX | 1112.625ms (1070.689–1273.221) | 13.195ms (12.486–15.146) | 1099.749ms (1057.034–1258.723) | 1099.712ms (1056.985–1258.674) | 0.030ms (0.021–0.032) |

두 형식 모두 typeset이 전체의 약 98.7~98.8%다. 그 시간은
`section=0/para=0/control=2/rows=3` 단일 block-table에서 114개 후속 페이지를 만드는 호출이 사실상
전부 차지했다. normalization은 0.4~0.7ms, postprocess는 0.06ms 미만이었다.

HWP run 3의 3336.130ms outlier에서도 typeset 3311.536ms, block-table 3311.436ms로 같은 병목을
가리켰다. outlier를 제외해도 약 1초의 동기 main-thread block이 유지된다.

raw profile은 `mydocs/report/assets/task_m100_2424_stage1_baseline.json`에 보존한다.

## 4. correctness 재확인

최신 #2214 focused test는 통과했다.

- HWP/HWPX 각각 `PartialTable fragments=115`
- full flush 뒤 `changed_after_flush_count=113`
- page 0 cursor/cell bounds와 cut은 transient/full oracle 정합
- 전체 115쪽 유지

따라서 visible page 뒤 결과를 그대로 재사용하거나 incomplete fingerprint로 조기 수렴할 수 없다.

## 5. 결정

Stage B 이후 방향은 유지한다.

1. deferred target revision과 structure fingerprint를 먼저 고정한다.
2. 최신 table-flow 보정을 보존하면서 continuation loop를 owned cursor/context와 caller-controlled step으로
   분리한다.
3. 113개 downstream cut이 모두 바뀌므로 synchronous resumable만으로 완료하지 않고 browser
   chunk/yield까지 연결한다.
4. unsupported/stale/invalid state는 기존 full pagination으로 fallback한다.

Stage A timer와 ignored probe는 before/after 비교가 끝날 때까지 진단 자산으로 유지한다.

## 6. 검증

| 명령 | 결과 |
|---|---|
| `cargo fmt --all -- --check` | 통과 |
| Stage A probe `--no-run` | 통과 |
| Stage A probe HWP/HWPX × 5 | 1 passed |
| #2214 exact-cut focused test | 1 passed, 115 fragments·113 changed cuts |
| `git diff --check` | 통과 예정(커밋 전 최종 확인) |
