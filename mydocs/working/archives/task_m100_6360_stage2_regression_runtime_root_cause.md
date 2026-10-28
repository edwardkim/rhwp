---
kind: working
status: done
issue: 6360
---

# #6360 Stage 2: 장시간 회귀 테스트 자체의 병목 분석

## 기준 시점

- 작업 branch: `fix/pdf-reference-fast-pass-20260829`
- 기준 HEAD: `3dc1fd382e0bac6c336fbed6ffaf626208c05e20`
- duration metrics ref: `ci-metrics/nextest-target-durations`
- metrics commit: `cf480430570a652406506c5bdf04bdead435c75f`
- 측정 source run: `33249621253` (`refs/heads/devel`, SHA `3dc1fd382e0bac6c336fbed6ffaf626208c05e20`)

Stage 1은 `regression_suite_NNN` 이름이 mutable이라 과거 target duration이 현재 source
구성에 잘못 귀속되는 문제를 다뤘다. Stage 2는 그 다음 층이다. duration 귀속이 올바르더라도,
개별 testcase가 너무 큰 단위로 묶여 있으면 nextest와 archive 배분이 더 이상 줄일 수 없는
벽시계 하한이 생긴다.

## 현재 병목

최근 v2 duration policy의 상위 testcase는 다음과 같다.

| 순위 | testcase | 측정 시간 |
| ---: | --- | ---: |
| 1 | `convert_verify_corpus_ratchet::ratchet_partition_2` | 816.636s |
| 2 | `security_corpus_regression::negative_corpus_sweep_is_clean_across_all_three_detectors` | 676.796s |
| 3 | `text_overlap_baseline::text_overlaps_do_not_grow` | 575.871s |
| 4 | `oracle_page_count_baseline::page_counts_do_not_drift_from_hancom_oracle` | 562.099s |
| 5 | `issue_2063::huge_cellbreak_table_paginates_without_quadratic_blowup` | 553.539s |
| 6 | `hwp5_roundtrip_baseline::baseline_all_samples_roundtrip` | 504.140s |
| 7 | `overflow_cell_baseline::overflow_cell_lines_do_not_grow` | 318.447s |
| 8 | `injection_scan_contract::every_normal_sample_is_clean` | 290.286s |
| 9 | `issue_1842::issue_1842_cellbreak_synthetic_lineheight_em_not_inflated` | 283.225s |
| 10 | `issue_2070_rowbreak_density::huge_cellbreak_table_pin` | 280.843s |
| 11 | `off_canvas_baseline::off_canvas_does_not_grow` | 261.268s |
| 12 | `ir_field_sweep_baseline::ir_field_sweep_does_not_regress` | 164.759s |

현재 selector 출력은 다음과 같다.

| archive group | targets | estimated_seconds | estimated_wall_seconds | max_testcase_seconds |
| --- | ---: | ---: | ---: | ---: |
| `integration-b` | 20 | 2349.028 | 816.636 | 816.636 |
| `integration-c` | 12 | 2352.001 | 676.796 | 676.796 |
| `integration-d` | 16 | 2349.006 | 587.252 | 575.871 |

`estimated_seconds`는 3개 group이 거의 같지만, `estimated_wall_seconds`는 `max_testcase_seconds`에
막힌다. 즉 archive 배분기는 정상적으로 일을 나눠도, 816초짜리 단일 testcase가 들어간 group은
816초보다 빨라질 수 없다.

## 원인 분류

### 1. 전수형 corpus sweep이 testcase 하나 또는 소수 partition에 묶임

대상:

- `tests/convert_verify_corpus_ratchet.rs`
- `tests/hwp5_roundtrip_baseline.rs`
- `tests/cases/text_overlap_baseline.rs`
- `tests/cases/oracle_page_count_baseline.rs`
- `tests/overflow_cell_baseline.rs`
- `tests/cases/off_canvas_baseline.rs`
- `tests/ir_field_sweep_baseline.rs`

이 계열은 많은 HWP/HWPX 샘플을 순회한다. 일부 파일은 내부 thread pool을 사용하지만 nextest
입장에서는 여전히 testcase 1개다. 특히 `convert_verify_corpus_ratchet`는 4 partition으로
나누고도 `ratchet_partition_2` 하나가 816초다. 현재 분배 기준은 size 내림차순과 index modulo라
문서별 실제 비용과 맞지 않을 수 있다.

관련 baseline 크기:

| fixture | 행 수 |
| --- | ---: |
| `tests/fixtures/oracle_page_count_baseline.tsv` | 559 |
| `tests/fixtures/ir_field_sweep_baseline.tsv` | 557 |
| `tests/fixtures/text_overlap_baseline.tsv` | 174 |
| `tests/fixtures/off_canvas_baseline.tsv` | 78 |
| `tests/fixtures/overflow_cell_baseline.tsv` | 13 |

### 2. 보안 corpus sweep이 같은 문서를 여러 subprocess로 반복 검사

대상:

- `tests/security_corpus_regression.rs`
- `tests/injection_scan_contract.rs`

`security_corpus_regression::negative_corpus_sweep_is_clean_across_all_three_detectors`는
정상 corpus 문서마다 CLI를 세 번 실행한다.

- `rhwp inspect hidden-text`
- `rhwp inspect injection --include-fields`
- `rhwp inspect unicode`

따라서 문서 읽기, 파싱, 일부 조판 인덱스 생성, subprocess 생성 비용이 반복된다.
`injection_scan_contract::every_normal_sample_is_clean`도 정상 샘플마다 CLI를 별도로 실행한다.
이 축은 테스트 의미를 바꾸지 않고 in-process query API와 병렬 문서 queue로 줄일 수 있다.

### 3. 초대형 CellBreak 문서 page_count가 여러 테스트에서 중복 측정됨

대상 문서:

- `samples/issue2063_huge_cellbreak_table.hwp`

중복 측정 위치:

- `tests/issue_2063.rs::huge_cellbreak_table_paginates_without_quadratic_blowup`
- `tests/issue_1842.rs::issue_1842_cellbreak_synthetic_lineheight_em_not_inflated`
- `tests/issue_2070_rowbreak_density.rs::huge_cellbreak_table_pin`

세 테스트는 서로 다른 이슈의 문맥을 설명하지만, 실행상으로는 같은 대형 문서의 `page_count()`를
반복 호출한다. 각 측정이 280초 이상이고 #2063 단독 측정은 553초다. 이 중복은 archive 배분으로
없앨 수 없다. 하나의 통합 sentinel이 page_count 범위와 pin을 동시에 검증하도록 합치고, 나머지
테스트는 같은 문서를 다시 열지 않도록 줄이는 편이 맞다.

## 이번 stage의 개선 범위

사용자 지시에 따라 다음 1, 2, 3만 개선한다.

1. 전수형 테스트의 coarse testcase를 더 작은 deterministic partition으로 나눈다.
2. 보안 sweep의 반복 CLI/subprocess 구조를 in-process 및 병렬 queue 기반으로 줄인다.
3. `issue2063_huge_cellbreak_table.hwp`의 중복 `page_count()` 측정을 통합한다.

이번 stage에서는 PR impact gating이나 nightly 분리는 하지 않는다. 그것은 정책 변경이 크고,
사용자 요청 범위의 1, 2, 3 밖이다.

## 측정 기준

코드 수정 뒤 최소 측정은 다음을 사용한다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
cargo nextest run --cargo-profile release-test --target-dir target/pr-review \
  --test convert_verify_corpus_ratchet \
  --test security_corpus_regression \
  --test injection_scan_contract \
  --test issue_2063 \
  --test issue_1842 \
  --test issue_2070_rowbreak_density \
  --test overflow_cell_baseline \
  --test regression_suite_004 \
  --test regression_suite_012 \
  --test regression_suite_005 \
  --test regression_suite_014 \
  --profile ci-duration-observation --no-fail-fast
```

측정 결과는 기존 v2 policy 상위 testcase와 비교한다. 특히 확인할 값은 다음이다.

- 최장 testcase가 816초에서 얼마나 줄었는가.
- B/C/D selector의 `max_testcase_seconds`가 낮아졌는가.
- 전체 실행시간 감소가 단순 분산 효과인지, 실제 중복 제거 효과인지 구분된다.
