---
kind: working
status: active
issue: 6360
---

# #6360 Stage 7: 잔여 장기 baseline 병목 후속 처리 계획

## 현재 상태

#6387 merge 후 `devel` post-merge CI와 duration refresh는 정상 동작했다.

- merge commit: `f5440811042f9c5ab7580d3a64204cf1d1e39dd8`
- PR CI run: `33261604465`
- post-merge `devel` CI run: `33262385098`
- `ci-metrics/nextest-target-durations`: `37521962e`
- `measurement_sources`: B/C/D 모두 `refs/pull/6387/merge`, run `33261604465`

Stage 6에서 `hwp5_roundtrip_baseline`, `convert_verify_corpus_ratchet`,
`text_overlap_baseline`의 `issue2063_huge_cellbreak_table.hwp` 중복 조판은 제거했다.
따라서 #6360은 닫지 않고, 남은 장기 baseline 축을 별도 단계로 줄인다.

## 최신 병목

`ci-metrics/nextest-target-durations` 최신 측정값 기준 상위 testcase는 다음과 같다.

| testcase | 측정 시간 |
| --- | ---: |
| `overflow_cell_baseline::overflow_cell_lines_do_not_grow_partition_2` | 385.481초 |
| `oracle_page_count_baseline::page_counts_do_not_drift_from_hancom_oracle_partition_2` | 327.829초 |
| `off_canvas_baseline::off_canvas_does_not_grow_partition_2` | 191.485초 |
| `issue_2063::huge_cellbreak_table_paginates_without_quadratic_blowup` | 184.596초 |
| `ir_field_sweep_baseline::ir_field_sweep_does_not_regress` | 143.078초 |
| `hwp5_roundtrip_baseline::baseline_all_samples_roundtrip_partition_14` | 84.669초 |

현재 selector projection은 B/C/D 총량은 거의 맞추지만, 단일 testcase critical path가 archive
wall time 하한을 만든다.

| archive group | selected targets | estimated wall | max testcase | 지배 항목 |
| --- | ---: | ---: | ---: | --- |
| `integration-b` | 17 | 385.481초 | 385.481초 | `overflow_cell_baseline` partition 2 |
| `integration-c` | 20 | 327.829초 | 327.829초 | `oracle_page_count_baseline` partition 2 |
| `integration-d` | 11 | 315.130초 | 191.485초 | `off_canvas_baseline` + bulk 합산 |

## 원인 가설

- file-size greedy partition은 대형 문서가 실제 layout 비용과 비례한다고 가정한다. 현재는 이
  가정이 깨졌다. 같은 partition 안의 일부 문서가 전체 시간을 지배한다.
- `hwp5_roundtrip_baseline`, `convert_verify_corpus_ratchet`, `text_overlap_baseline`에는
  slow-sample stderr 로그가 있지만, `overflow_cell_baseline`, `oracle_page_count_baseline`,
  `off_canvas_baseline`에는 같은 수준의 샘플별 시간 증거가 없다.
- partition 수만 늘리는 것은 충분하지 않다. 단일 문서 하나가 3분 이상이면 그 testcase 자체가
  archive wall time 하한이 된다.
- `text_overlap_baseline`과 `off_canvas_baseline`은 모두 `layout_anomaly` 계열 신호를 전수 스캔한다.
  같은 문서를 두 번 조판·스캔하는 구조라 중복 비용을 제거할 여지가 크다.
- `oracle_page_count_baseline`은 문서 전체 `page_count()`가 본질이라 페이지 일부만 검사하면 쪽수
  oracle 의미가 깨진다. 이 축은 먼저 느린 샘플을 특정한 뒤 전용 sentinel 여부를 판단해야 한다.

## Stage 7-A: 관측 보강

목표는 코드 최적화 전에 어떤 샘플이 느린지 증거를 남기는 것이다.

1. `overflow_cell_baseline`, `oracle_page_count_baseline`, `off_canvas_baseline`에
   `SLOW_SAMPLE_LOG_THRESHOLD=30초` 기준 stderr 로그를 추가한다.
2. 기존 dump 환경변수와 충돌하지 않게, 기본 동작은 pass/fail만 유지한다.
3. targeted nextest를 실행해 `mydocs/pr/assets/task_m100_6360_stage7_*` 아래에 stderr와 요약 TSV를
   보관한다.

예상 검증 명령:

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
cargo nextest run --cargo-profile release-test --target-dir target/pr-review \
  --test overflow_cell_baseline \
  --test regression_suite_004 \
  --test regression_suite_005 \
  -E 'test(overflow_cell_baseline) | test(off_canvas_baseline) | test(oracle_page_count_baseline)' \
  --no-fail-fast --test-threads 12
```

판단 기준:

- slow sample이 1~2개로 집중되면 전용 sentinel 또는 알고리즘 개선 후보로 분리한다.
- 여러 대형 문서가 고르게 느리면 partition 확대와 layout scan 중복 제거를 우선한다.

## Stage 7-A 구현 기록

이번 stage에서는 먼저 300초 이상 critical path를 만든 세 baseline을 직접 줄인다.

- `overflow_cell_baseline`: `render_page_svg_native()` 대신 `build_page_render_tree()`까지만 수행한다.
  overflow-cell 카운터는 레이아웃 중 증가하므로 SVG 문자열 생성 비용은 원장 판정에 필요하지 않다.
- `overflow_cell_baseline`, `off_canvas_baseline`, `oracle_page_count_baseline`: partition을 8개에서
  16개로 늘려 단일 testcase wall time 하한을 낮춘다.
- 세 baseline 모두 30초 이상 걸린 샘플을 stderr에 남겨 다음 CI duration refresh에서 병목 문서를
  바로 특정할 수 있게 한다.
- `oracle_page_count_baseline`: `samples/issue2063_huge_cellbreak_table.hwp`는
  `tests/issue_2063.rs::huge_cellbreak_table_paginates_without_quadratic_blowup`가 page-count pin과
  완주 성능 sentinel을 전담하므로 전수 oracle 원장에서는 중복 실행만 제외한다.

검증 후 이 문서에는 다음 값을 추가한다.

- targeted nextest 결과
- 변경 후 상위 partition wall time
- `overflow_cell_baseline`의 render-tree 경로 전환이 baseline 증가를 만들지 않았는지 여부

## Stage 7-A 로컬 검증 결과

### 중간 확인

16분할과 slow-sample 로그만 적용한 상태에서 `off_canvas_baseline` partition 8을 단독 실행했다.

```bash
cargo nextest run --cargo-profile release-test --target-dir target/pr-review \
  --test regression_suite_029 \
  -E 'test(off_canvas_baseline::off_canvas_does_not_grow_partition_8)' \
  --success-output immediate-final --no-fail-fast --test-threads 1
```

결과:

- `off_canvas_baseline::off_canvas_does_not_grow_partition_8`: 221.664초, PASS
- slow sample: `issue2063_huge_cellbreak_table.hwp` 221.524초

따라서 Stage 6에서 이미 `text_overlap_baseline`, `hwp5_roundtrip_baseline`,
`convert_verify_corpus_ratchet`에서 제외했던 동일 초대형 CellBreak fixture가 남은 병목도 만들고
있음을 확인했다.

### 최종 targeted nextest

`issue2063_huge_cellbreak_table.hwp`를 전용 sentinel 담당 fixture로 `overflow_cell_baseline`,
`off_canvas_baseline`, `oracle_page_count_baseline`의 중복 스캔 대상에서 제외한 뒤 다시 실행했다.

```bash
cargo nextest run --cargo-profile release-test --target-dir target/pr-review \
  --test overflow_cell_baseline \
  --test regression_suite_024 \
  --test regression_suite_029 \
  -E 'test(overflow_cell_lines_do_not_grow_partition) | \
      test(off_canvas_baseline::off_canvas_does_not_grow_partition) | \
      test(oracle_page_count_baseline::page_counts_do_not_drift_from_hancom_oracle_partition)' \
  --no-fail-fast --test-threads 12
```

결과:

- Summary: 48 tests run, 48 passed, 274 skipped
- nextest summary wall: 52.255초
- shell wall: 58.80초

최장 partition:

| baseline | 최장 testcase |
| --- | ---: |
| `overflow_cell_baseline` | 41.705초 |
| `off_canvas_baseline` | 32.426초 |
| `oracle_page_count_baseline` | 6.569초 |

### overflow 원장 동등성

`RHWP_OVERFLOW_CELL_DUMP=/tmp/task_m100_6360_stage7_overflow_20260830.tsv`로 16개 partition dump를
생성한 뒤, `issue2063_huge_cellbreak_table.hwp`를 제외한 기존
`tests/fixtures/overflow_cell_baseline.tsv`와 정렬 비교했다.

```bash
diff -u \
  <(grep -v '^#' tests/fixtures/overflow_cell_baseline.tsv |
    grep -v '^issue2063_huge_cellbreak_table\.hwp\t' |
    sed '/^[[:space:]]*$/d' | LC_ALL=C sort) \
  <(cat /tmp/task_m100_6360_stage7_overflow_20260830.tsv.part*-of16 |
    sed '/^[[:space:]]*$/d' | LC_ALL=C sort)
```

결과:

- diff 없음
- `render_page_svg_native()`에서 `build_page_render_tree()`로 바꿔도 overflow-cell 원장 값은 유지된다.

## Stage 7-A 판단

현재 stage의 수정으로 최신 CI duration policy의 300초 이상 critical path 후보 세 개는 로컬 기준
모두 60초 미만으로 내려갔다.

| 기존 병목 | 최신 policy 측정 | 로컬 개선 후 |
| --- | ---: | ---: |
| `overflow_cell_baseline` partition 2 | 385.481초 | 최장 41.705초 |
| `oracle_page_count_baseline` partition 2 | 327.829초 | 최장 6.569초 |
| `off_canvas_baseline` partition 2 | 191.485초 | 최장 32.426초 |

남은 확인은 PR CI에서 `ci-metrics/nextest-target-durations`가 같은 경향으로 갱신되는지 보는 것이다.

## Stage 7-B: layout-anomaly 중복 스캔 축소

`text_overlap_baseline`과 `off_canvas_baseline`은 같은 `scan_document`/`scan_page` 계열 정보를
필요로 한다. 후속 PR에서는 두 baseline을 한 번의 문서 스캔에서 동시에 계산하는 방식을 검토한다.

권장 구조:

- 새 공통 helper: 문서별 layout anomaly scan 1회 수행
- text-overlap count와 off-canvas count를 동시에 집계
- 기존 `tests/fixtures/text_overlap_baseline.tsv`, `tests/fixtures/off_canvas_baseline.tsv`는 유지
- 기존 테스트명은 compatibility를 보려면 얇은 wrapper로 남기거나, 하나의 통합 baseline target으로
  옮기되 PR 본문에 duration policy 변화 근거를 남긴다.

주의:

- 두 신호의 래칫 규약은 다르지 않지만, 실패 메시지는 각 신호가 명확히 분리되어야 한다.
- baseline 감소는 계속 pass로 유지하고, 신규/증가만 fail 한다.

## Stage 7-C: overflow-cell 비용 축소

`overflow_cell_baseline`은 현재 문서의 모든 페이지에 대해 `render_page_svg_native()`를 호출한다.
이 경로가 SVG 문자열 생성까지 수행한다면, overflow count만 얻기에는 과하다.

검토 순서:

1. `LAYOUT_OVERFLOW_CELL` 카운터가 어느 단계에서 증가하는지 추적한다.
2. 동일 count를 더 싼 layout/render-tree 경로에서 얻을 수 있으면 `count_doc()`를 교체한다.
3. 교체 전후로 `RHWP_OVERFLOW_CELL_DUMP`를 비교해 TSV가 동일한지 확인한다.
4. 동일하지 않으면 최적화를 중단하고, 느린 샘플 전용 sentinel/partition 쪽으로만 진행한다.

수용 조건:

- 전체 baseline TSV가 기존과 동일하거나 감소만 발생한다.
- `overflow_cell_lines_do_not_grow_partition_2`가 180초 아래로 내려가는지 측정한다.

## Stage 7-D: oracle page-count 장기 샘플 처리

`oracle_page_count_baseline`은 쪽수 oracle 자체가 전체 문서 layout에 의존한다. 따라서 임의 페이지
부분 검사로 대체하지 않는다.

검토 순서:

1. partition 2의 slow sample을 특정한다.
2. 느린 문서가 이미 별도 issue test에서 쪽수 pin 또는 동일 조판 invariant를 갖는지 확인한다.
3. 전용 sentinel이 있으면 baseline 전수 gate에서 중복 여부를 제거할 수 있는지 판단한다.
4. 전용 sentinel이 없으면 먼저 sentinel을 만든 뒤에만 전수 gate 제외를 검토한다.

수용 조건:

- `oracle_page_count_baseline`의 coverage 축소가 문서화된 sentinel로 보상된다.
- 특정 문서를 제외할 경우 제외 목록에는 이유와 담당 test target을 반드시 남긴다.

## Stage 7-E: 남은 단일 장기 test 정리

위 세 축 이후에도 다음 항목은 남을 수 있다.

- `issue_2063` sentinel 약 185초
- `ir_field_sweep_baseline` 약 143초

이 둘은 별도 stage로 둔다. Stage 7의 완료 조건은 B/C/D 전체를 완벽히 균등하게 만드는 것이 아니라,
현재 300초 이상 critical path를 먼저 제거하는 것이다.

## PR 단위

1. 관측 PR: slow-sample 로그와 stage 문서, targeted 측정 증적만 포함한다.
2. layout-anomaly PR: text-overlap/off-canvas 중복 스캔 제거와 검증 증적을 포함한다.
3. overflow-cell PR: SVG 생성 우회가 가능하면 코드 개선, 불가능하면 sentinel/분할 계획만 남긴다.
4. oracle PR: slow sample별 판단과 sentinel 보강을 포함한다.

각 PR은 `upstream/devel` 동기화 후 rebase하고, CI 완료 뒤 `ci-metrics/nextest-target-durations`의
B/C/D 실제 측정값을 #6360에 코멘트로 남긴다.

## #6360 close 기준

다음 조건을 모두 만족할 때 close한다.

- B/C/D PR CI가 모두 성공한다.
- post-merge trusted PR evidence reuse와 duration refresh가 성공한다.
- 최신 duration policy에서 300초 이상 단일 baseline testcase가 남지 않는다.
- 남은 120초 이상 테스트는 전용 sentinel 또는 별도 follow-up issue로 근거가 명확하다.
- issue comment에 전후 수치, PR, merge SHA, duration policy commit이 기록되어 있다.
