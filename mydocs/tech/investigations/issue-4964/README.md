# #4964 W6 font metric 계보 분리

이 디렉터리는 `FONT_METRICS`의 generated 영역과 measured/manual overlay를 물리적으로 분리하기 전에
행동을 동결하는 W6 machine-readable evidence를 보관한다.

## Authority

| 산출물 | 역할 |
| --- | --- |
| `font_metric_pre_split_baseline.json` | 분리 전 600개 composition·metric data·문자 폭·lookup hash |
| `scripts/font_metric_lineage.mjs` | 현재 Rust source를 읽어 기준선을 생성·검사하는 도구 |
| `scripts/tests/font_metric_lineage.test.mjs` | 누락·순서 변경·폭 변경·overlay identity 변경의 negative contract |
| `mydocs/plans/task_m100_4964.md` | W6 범위·불변식·승인 게이트 정본 |

이 기준선은 provenance manifest가 아니다. 0..594가 역사적 generated 영역이고 595..599가 #2430
measured overlay라는 분리 전 경계를 고정할 뿐, generated 영역을 source-exact로 승격하지 않는다.
600행 provenance와 W1/W5 evidence 연결은 Stage W6-2에서 별도 schema/manifest로 만든다.

## 재현

저장된 기준선 검사:

```bash
node scripts/font_metric_lineage.mjs --check
node --test scripts/tests/font_metric_lineage.test.mjs
```

승인된 기준선 갱신 단계에서만 생성:

```bash
node scripts/font_metric_lineage.mjs --generate
node scripts/font_metric_lineage.mjs --check
```

#2430 측정 TSV와 마지막 5개 ASCII 배열의 독립 검증:

```bash
python3 tools/task2430/gen_metrics.py \
  --verify \
  --ladder-dir tools/task2430/measured
```

## hash 의미

- `compositionSha256`: 600개 index·name·style·em·range/Hangul symbol 순서
- `metricDataSha256`: 각 항목이 참조하는 Latin range와 Hangul map/grid의 값
- `widthProjectionSha256`: 모든 항목의 저장 Latin codepoint와 U+AC00..U+D7A3 결과
- `lookupProjectionSha256`: 모든 metric/alias 이름과 미등록 sentinel의 네 style 선택 결과

machine baseline에는 실행 시각, 절대 경로와 사용자명을 넣지 않는다. 같은 source에서 생성한 canonical
JSON은 마지막 newline까지 같아야 한다.

## 범위 경계

- font metric 값, alias, fallback과 renderer 출력은 변경하지 않는다.
- private corpus와 한컴/Hyper-V Oracle을 다시 실행하지 않는다.
- local-only font bytes를 복사하거나 추적하지 않는다.
- `unknown` provenance는 Stage W6-2에서 명시적으로 유지한다.
