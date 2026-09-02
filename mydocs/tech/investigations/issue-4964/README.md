# #4964 W6 font metric 계보 분리

이 디렉터리는 `FONT_METRICS`의 generated 영역과 measured/manual overlay를 물리적으로 분리하기 전에
행동을 동결하는 W6 machine-readable evidence를 보관한다.

## Authority

| 산출물 | 역할 |
| --- | --- |
| `font_metric_pre_split_baseline.json` | 분리 전 600개 composition·metric data·문자 폭·lookup hash |
| `font_metric_lineage_manifest.schema.json` | 600행 provenance·evidence·상태 계약 |
| `font_metric_lineage_manifest.json` | W1·W5·#2430·추적 font 증거를 연결한 W6 lineage 정본 |
| `font_metric_generator_canary_plan.json` | 공개 추적 TTF와 합성 TTC face의 명시적 생성 순서·identity·license 계약 |
| `scripts/font_metric_lineage.mjs` | 현재 Rust source를 읽어 기준선을 생성·검사하는 도구 |
| `scripts/tests/font_metric_lineage.test.mjs` | 누락·순서 변경·폭 변경·overlay identity 변경의 negative contract |
| `scripts/tests/font_metric_gen.test.mjs` | generator 결정성·TTC face·core/overlay ownership·fail-closed 계약 |
| `mydocs/plans/task_m100_4964.md` | W6 범위·불변식·승인 게이트 정본 |
| `mydocs/working/task_m100_4964_w6_stage5.md` | 통합 불변식·native/WASM 검증과 환경 경계 |
| `mydocs/report/task_m100_4964_report.md` | W6 결과·잔여 unknown·후행 W7 인계 최종 보고 |

## Runtime source 경계

| 경로 | 소유권 |
| --- | --- |
| `src/renderer/font_metrics_data.rs` | type·alias·lookup/index와 600-entry 논리 view |
| `src/renderer/font_metrics_generated.rs` | historical generated region 0..594 |
| `src/renderer/font_metrics_overlays.rs` | #2430 measured/manual overlay 595..599 |

facade의 `FONT_METRICS.iter()`는 generated 배열 다음에 overlay 배열을 `chain`한다. 따라서 기존
first-match index는 유지되지만 두 data 영역은 서로 다른 파일 소유권을 갖는다.

pre-split 기준선은 provenance manifest가 아니다. 0..594가 역사적 generated 영역이고 595..599가
#2430 measured overlay라는 분리 전 경계를 고정한다. lineage manifest가 600행 각각의 provenance
상태와 W1/W5 evidence를 소유한다. generated 영역이라는 위치만으로 source-exact를 선언하지 않는다.

## 재현

저장된 기준선 검사:

```bash
node scripts/font_metric_lineage.mjs --check
node scripts/font_metric_lineage.mjs --check-manifest
node --test scripts/tests/font_metric_lineage.test.mjs
```

승인된 기준선 갱신 단계에서만 생성:

```bash
node scripts/font_metric_lineage.mjs --generate
node scripts/font_metric_lineage.mjs --check
node scripts/font_metric_lineage.mjs --generate-manifest
node scripts/font_metric_lineage.mjs --check-manifest
```

#2430 측정 TSV와 마지막 5개 ASCII 배열의 독립 검증:

```bash
python3 tools/task2430/gen_metrics.py \
  --verify \
  --ladder-dir tools/task2430/measured
```

generator 공개 canary와 ownership 검사:

```bash
node --test scripts/tests/font_metric_gen.test.mjs
```

수동 생성은 디렉터리 전체를 받지 않는다. plan에 `order`, `path`, `faceIndex`, 예상 family/style과
license·provenance evidence를 명시하고 generated fragment와 metadata를 서로 다른 출력으로 지정한다.

```bash
cargo run --bin font-metric-gen -- \
  --plan mydocs/tech/investigations/issue-4964/font_metric_generator_canary_plan.json \
  --generated-output <generated-output.rs> \
  --metadata-output <provenance-output.json>
```

`font_metrics_data.rs`와 `font_metrics_overlays.rs`는 직접 경로뿐 아니라 symlink로도 출력할 수 없다.
기존 595개 전체의 원본 plan이 복원되지 않았으므로 `targetRegion: canary` 출력도 canonical generated
DB에 쓸 수 없다. 그 경로는 `targetRegion: historical-generated-0-594`와 정확히 595개 입력을 선언한
plan에만 열린다.

## hash 의미

- `compositionSha256`: 600개 index·name·style·em·range/Hangul symbol 순서
- `metricDataSha256`: 각 항목이 참조하는 Latin range와 Hangul map/grid의 값
- `widthProjectionSha256`: 모든 항목의 저장 Latin codepoint와 U+AC00..U+D7A3 결과
- `lookupProjectionSha256`: 모든 metric/alias 이름과 미등록 sentinel의 네 style 선택 결과

machine baseline에는 실행 시각, 절대 경로와 사용자명을 넣지 않는다. 같은 source에서 생성한 canonical
JSON은 마지막 newline까지 같아야 한다.

## lineage 판독

- `entryId`는 name·bold·italic identity hash이며 현재 index를 포함하지 않는다.
- `currentIndex`는 기존 first-match 순서를 보호하는 0..599 composition 좌표다.
- `origin.status: unknown`은 실패나 누락이 아니라 source-exact 승격을 금지하는 명시적 상태다.
- `origin.declarationCommit`은 현재 entry 선언 줄의 Git anchor다. 참조 width 배열 전체의 생성 commit을
  뜻하지 않으며 origin 증명으로 단독 사용하지 않는다.
- `fontSource.verificationScope: printable-ascii-only`는 추적 Noto Sans KR Regular가 ASCII 범위에서만
  현재 metric과 대조됐다는 뜻이다.
- W5 `identity-exact` 연결은 한컴 readback face identity 증거다. metric 전체가 SFNT-exact라는 뜻이
  아니다.

## 범위 경계

- font metric 값, alias, fallback과 renderer 출력은 변경하지 않는다.
- private corpus와 한컴/Hyper-V Oracle을 다시 실행하지 않는다.
- local-only font bytes를 복사하거나 추적하지 않는다.
- `unknown` provenance는 Stage W6-2에서 명시적으로 유지한다.
