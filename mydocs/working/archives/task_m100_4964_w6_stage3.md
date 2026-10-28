# Task M100 #4964 — Stage W6-3 data-only 물리 분리

- **수행계획**: [`../plans/task_m100_4964.md`](../../plans/archives/task_m100_4964.md)
- **선행 manifest**: [`task_m100_4964_w6_stage2.md`](task_m100_4964_w6_stage2.md)
- **기준 source**: `upstream/devel@d1ad0eb8784dbc55f0796e2ba8775f7363247b91`
- **단계 목적**: lookup 행동과 600-entry 순서를 바꾸지 않고 core·generated·overlay를 물리 분리한다.
- **판정**: 통과

## 1. 결과 구조

| 경로 | 줄 수 | 역할 | 항목 |
| --- | ---: | --- | ---: |
| `src/renderer/font_metrics_data.rs` | 660 | type·alias·lookup/index·논리 composition facade | view 600 |
| `src/renderer/font_metrics_generated.rs` | 45,656 | historical generated region | 595 |
| `src/renderer/font_metrics_overlays.rs` | 278 | #2430 measured/manual overlay | 5 |

기존 46,564줄 단일 파일에서 data만 옮겼다. facade는 다음 논리 계약을 유지한다.

```text
FONT_METRICS.iter()
  = GENERATED_FONT_METRICS[0..595].iter()
    chain MEASURED_FONT_METRIC_OVERLAYS[0..5].iter()
  = 기존 index 0..599
```

`FONT_METRICS` 이름과 `.iter()` 호출 형태를 유지해 lookup/index/test 호출자가 별도 배열을 직접
선택하지 못하게 했다. runtime은 정렬·복사·dedupe 없이 두 static slice를 순서대로 순회한다.

## 2. 이동 경계

### 2.1 generated

기존 index 0..594의 모든 Latin width array, LatinRange, Hangul group/map/grid와 FontMetric entry를
`font_metrics_generated.rs`로 이동했다. 파일 머리에는 이 영역의 위치가 source-exact를 뜻하지 않으며
lineage manifest 확인 없이 exact claim을 해서는 안 된다는 경고를 둔다.

### 2.2 overlay

다음 index 595..599의 측정 ASCII array, 합성 LatinRange와 entry를
`font_metrics_overlays.rs`로 이동했다.

1. `HanyangSinMyeongJo`
2. `HanyangJungGothic`
3. `HanyangKyunMyeongJo`
4. `HanyangKyunGothic`
5. `HumanMyeongJo`

overlay가 참조하던 `FONT_266/267/271/276`의 기타 Latin range와 Hangul table은 같은 Rust module
scope에서 include되므로 숫자 복제나 가시성 확대 없이 그대로 공유한다.

## 3. source consumer 전환

단일 `static FONT_METRICS: [FontMetric; 600]`을 파싱하던 소비자를 함께 수정했다.

| 소비자 | 변경 |
| --- | --- |
| `scripts/font_metric_lineage.mjs` | monolith와 split source를 모두 읽고 generated → overlay composition |
| `scripts/font_rule_ledger.mjs` | W1 baseline을 shared metric analyzer에서 생성 |
| `scripts/font_rule_candidates.mjs` | `rust-composed-metric-view`에서 600행 수집 |
| `font_rule_sources.json` | metric-table selector를 logical `FontMetrics` view로 변경 |
| `font_rule_candidates.test.mjs` | 역사 snapshot의 낡은 source digest 대신 현재 source boundary를 매 실행 수집 |
| `tools/task2430/gen_metrics.py` | facade의 `include!` fragment를 재귀적으로 읽어 measured overlay 검증 |

W1의 `candidateId`는 owner·selector ID와 rule identity로 계산되므로 `rust-metric.metric-table` 경계와
600개 rule identity는 유지된다. #4939의 committed candidate/ledger는 당시 source의 역사 snapshot으로
보존하고, 현재 source 검사는 fresh boundary에서 수행한다.

## 4. 분리 전후 불변 hash

| projection | Stage W6-1 | 분리 후 | 판정 |
| --- | --- | --- | --- |
| composition | `d4cdac86b3c6ee55d8b1aa921d662f1fc1241c2809cb9c8ffe991d56a045e69a` | 동일 | 통과 |
| metric data | `025812eac4bad179c5b87e23b15fdf08a4e4fb3f19a6e453738e03110a140bcf` | 동일 | 통과 |
| exhaustive width | `2cd1389a14401f6488041af3c54ff0ba5e982d944acd0b5bb56147056e3a7d1b` | 동일 | 통과 |
| lookup | `bb3008f9dc379bd580119a6a658388732e94358db2039dbb02d78c28ec992fdf` | 동일 | 통과 |

lineage manifest의 600행 semantic identity는 그대로이고 `storageRegion.sourcePath`만 generated 또는
overlay 실제 경로로 갱신됐다. 갱신된 `entriesSha256`은
`054f4725162ddc95c4b00e00186955b1d7f10599d401f66f75b6dd52a1147032`다.

## 5. 검증

| 명령 | 결과 |
| --- | --- |
| `node scripts/font_metric_lineage.mjs --check` | W6-1 네 hash 동일 |
| `node scripts/font_metric_lineage.mjs --check-manifest` | 600행·storage path·evidence 통과 |
| `node --test scripts/tests/font_metric_lineage.test.mjs scripts/tests/font_rule_ledger.test.mjs scripts/tests/font_rule_candidates.test.mjs` | 28/28 통과 |
| `python3 tools/task2430/gen_metrics.py --verify --ladder-dir tools/task2430/measured` | 5 face 모두 95/95 일치 |
| `cargo test --profile release-test --lib font_metrics` | 9/9 통과, 0 실패 |
| `node scripts/rust-unit-test-tiers.mjs --check` | 4,225 tests / 299 modules, 통과 |
| `cargo fmt --all` | 통과 |
| `cargo fmt --all -- --check` | 통과 |

직접 Rust unit test 한 개를 추가하는 방안은 source-side test 총량 정책 4,225개를 초과하므로 채택하지
않았다. 같은 보호는 W6의 600행·영역 수·순서·semantic hash negative contract로 수행한다.

Rust test 실행이 기존 `Cargo.lock`의 workspace package 두 행을 자동 재정렬했으나 W6와 무관한
파생 diff이므로 기준 checkout bytes로 복원해 변경에 포함하지 않았다.

## 6. Stage 판정과 다음 게이트

generated 595개와 overlay 5개가 물리적으로 분리됐고, Rust runtime 및 모든 source consumer가 기존
600개 순서와 결과를 유지한다. 따라서 Stage W6-4에서 `font-metric-gen`의 출력 소유권을 generated
영역으로 제한하고 digest·face index·naming record·compression error metadata를 추가할 수 있다.

Stage W6-4 전까지 생성기를 현재 data file에 실행하지 않는다. generator 보강은 메인테이너 승인 후
시작한다.
