---
kind: working
status: done
issue: 6360
---

# #6360 Stage 6: 장기 baseline testcase partition 축소

## 배경

Stage 5 이후 #6384 merge와 post-merge CI를 확인했다. v2 duration artifact 수집과 trusted
post-merge duration refresh는 정상화됐지만, 실제 B/C/D wall-clock 균형은 단일 장기 testcase의
critical path에 막혔다.

최신 `ci-metrics/nextest-target-durations` 기준 상위 병목은 다음과 같다.

| testcase | 측정 시간 |
| --- | ---: |
| `hwp5_roundtrip_baseline::baseline_all_samples_roundtrip_partition_7` | 574.496초 |
| `convert_verify_corpus_ratchet::ratchet_partition_4` | 390.869초 |
| `text_overlap_baseline::text_overlaps_do_not_grow_partition_2` | 304.212초 |

## 개선 방침

- archive selector 휴리스틱보다 먼저 testcase 자체를 더 잘게 쪼갠다.
- source 파일 수와 Cargo integration target 수는 유지하고, `#[test]` partition 수만 늘려 nextest가
  같은 target 내부 작업을 더 작은 testcase로 스케줄하게 한다.
- 너무 과한 분할은 빈 partition 위험이 있으므로 1차에서는 검증 비용 대비 효과가 큰 수준으로 제한한다.
- 전수 baseline 안에서 pathological fixture를 반복 조판하지 않는다. `issue2063_huge_cellbreak_table.hwp`
  는 `tests/issue_2063.rs` sentinel 이 성능·페이지 pin 을 전담하므로, roundtrip/verify/text-overlap
  전수 래칫에서는 중복 실행하지 않는다.

## 수정 내용

- `tests/hwp5_roundtrip_baseline.rs`
  - small sample partition: 8 -> 16
  - partition 내부 문서 처리를 최대 8 worker 로 병렬화
  - `issue2063_huge_cellbreak_table.hwp` 는 `issue_2063` sentinel 전담 fixture 로 제외
  - 30초 이상 걸린 문서는 stderr 로 샘플명을 출력
- `tests/convert_verify_corpus_ratchet.rs`
  - ratchet partition: 16 -> 24
  - skip/size-cap 을 partition 전에 먼저 적용해 빈 partition 을 만들지 않도록 보정
  - partition 내부 문서 처리를 최대 8 worker 로 병렬화
  - `issue2063_huge_cellbreak_table.hwp` 는 `issue_2063` sentinel 전담 fixture 로 제외
  - 30초 이상 걸린 문서는 stderr 로 샘플명을 출력
- `tests/cases/text_overlap_baseline.rs`
  - text-overlap partition: 8 -> 16
  - `issue2063_huge_cellbreak_table.hwp` 는 `issue_2063` sentinel 전담 fixture 로 제외
  - 30초 이상 걸린 문서는 stderr 로 샘플명을 출력

## 기대 효과

- 전체 CPU 작업량은 크게 변하지 않는다.
- PR CI의 B/C/D archive wall-clock 하한을 만들던 전수 baseline 단일 testcase 시간이 낮아진다.
- 첫 PR run은 기존 duration policy의 오래된 test_case 이름별 값 때문에 배분 예측이 보수적으로 남을 수
  있지만, merge 후 duration refresh가 새 partition별 값을 수집하면 다음 run부터 반영된다.

## 검증

- `cargo fmt --all`: 통과
- `cargo fmt --all -- --check`: 통과
- `node scripts/rust-test-suite-manifest.mjs --prepare && node scripts/rust-test-suite-manifest.mjs --check`: 통과
  - 1032 sources / 4531 static test attrs / 32 suites + 16 exceptions = 48/48 integration targets
  - nextest 최소 6559 cases

### 1차 targeted nextest

명령:

```bash
cargo nextest run --cargo-profile release-test --target-dir target/pr-review \
  --test regression_suite_005 --test regression_suite_008 --test regression_suite_009 \
  -E 'test(hwp5_roundtrip_baseline) | test(convert_verify_corpus_ratchet) | test(text_overlap_baseline)' \
  --no-fail-fast --test-threads 12
```

결과:

- 60 passed / 1 failed / 434 skipped
- wall-clock: 14분 03초
- 실패: `convert_verify_corpus_ratchet::ratchet_partition_0`
- 원인: partition 을 먼저 나눈 뒤 size-cap 을 적용해, partition 0이
  `2025 행정업무운영 편람(최종).hwpx` 1건만 포함했다가 크기 상한으로 빠져 실제 검사 대상이 0건이 됐다.

보정:

- `convert_verify_corpus_ratchet` 의 명시 skip/size-cap 을 partition 전에 적용.
- 같은 단계에서 partition 내부 worker 병렬 처리를 추가.

### 2차 targeted nextest

명령:

```bash
cargo nextest run --cargo-profile release-test --target-dir target/pr-review \
  --test regression_suite_005 --test regression_suite_008 --test regression_suite_009 \
  -E 'test(hwp5_roundtrip_baseline) | test(convert_verify_corpus_ratchet) | test(text_overlap_baseline)' \
  --no-fail-fast --test-threads 12
```

결과:

- 61 passed / 434 skipped
- wall-clock: 8분 51초
- 남은 병목:
  - `hwp5_roundtrip_baseline::baseline_all_samples_roundtrip_partition_4`: 521.188초
  - `convert_verify_corpus_ratchet::ratchet_partition_9`: 498.920초
  - `text_overlap_baseline::text_overlaps_do_not_grow_partition_8`: 312.784초

slow-sample 로그 재실행 결과 세 항목의 공통 원인이
`samples/issue2063_huge_cellbreak_table.hwp` 로 확인됐다.

| 테스트 축 | 느린 샘플 실측 |
| --- | ---: |
| `hwp5_roundtrip_baseline` partition 4 | 504.907초 |
| `convert_verify_corpus_ratchet` partition 9 | 504.162초 |
| `text_overlap_baseline` partition 8 | 227.330초 |

해당 fixture는 `tests/issue_2063.rs` 가 `huge_cellbreak_table_paginates_without_quadratic_blowup`
단일 sentinel 로 완주성과 161쪽 pin 을 검증한다. 따라서 전수 baseline 세 축에서는 중복 조판을 제거했다.

### 최종 targeted nextest

명령:

```bash
cargo nextest run --cargo-profile release-test --target-dir target/pr-review \
  --test regression_suite_003 --test regression_suite_024 --test regression_suite_026 --test regression_suite_030 \
  -E 'test(hwp5_roundtrip_baseline) | test(convert_verify_corpus_ratchet) | test(text_overlap_baseline) | test(issue_2063)' \
  --no-fail-fast --test-threads 12
```

결과:

- 62 passed / 559 skipped
- wall-clock: 4분 12초
- `issue_2063::huge_cellbreak_table_paginates_without_quadratic_blowup`: 233.315초

baseline 세 축만 따로 재측정:

```bash
cargo nextest run --cargo-profile release-test --target-dir target/pr-review \
  --test regression_suite_003 --test regression_suite_026 --test regression_suite_030 \
  -E 'test(hwp5_roundtrip_baseline) | test(convert_verify_corpus_ratchet) | test(text_overlap_baseline)' \
  --no-fail-fast --test-threads 12
```

- 61 passed / 416 skipped
- wall-clock: 1분 21초
- 남은 최장 전수 baseline: `hwp5_roundtrip_baseline::baseline_all_samples_roundtrip_partition_14`
  80.443초
- slow-sample 확인: `samples/task2070/1130000-201900011_D0150004-1-002_2017년기준 시장구조조사.hwp`
  60.383초

`task2070` 문서는 `tests/issue_2070_rowbreak_density.rs` 가 페이지 pin 을 갖고 있지만,
roundtrip 무손실 coverage 자체는 `hwp5_roundtrip_baseline` 이 담당한다. #2063처럼 동일 축의 전용
sentinel 이 있는 상태가 아니므로 이번 단계에서는 제외하지 않았다.

## 결론

- B/C/D archive wall-clock 를 잡아먹던 중복 baseline 병목은 `issue2063_huge_cellbreak_table.hwp`
  의 반복 조판이었다.
- 해당 문서를 전수 baseline 세 축에서 제외하고 `issue_2063` sentinel 로 단일화해, affected baseline
  묶음은 8분51초에서 1분21초로 감소했다.
- #2063 sentinel 자체는 233초이며, 이는 별도 조판 성능 개선 이슈로 다룰 수 있다.
