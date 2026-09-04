---
kind: investigation
status: active
canonical: mydocs/plans/archives/task_m100_4966.md
last_verified: 2026-08-24
---

# Issue #4966 canonical font registry와 backend projection

이 디렉터리는 W7 migration 전 font 규칙과 backend별 선택 결과, canonical registry로의 일회 이행
대응을 재현 가능하게 고정한다.

## 권위와 소비 경계

- [수행계획](../../../plans/archives/task_m100_4966.md)이 범위, 보호 불변식과 단계별 승인 게이트의 정본이다.
- [W1 조사 원장](../issue-4939/README.md)은 rule relation·evidence의 선행 근거다.
- [W6 metric lineage](../issue-4964/README.md)는 600개 metric 값·순서·계보의 정본이다.
- `font_rule_projection_baseline.json`은 W7 migration 전 행동 snapshot이며 canonical runtime registry가
  아니다.
- `font_rule_registry_migration.json`은 W1/W6에서 canonical registry로의 일회 이행 감사 증거이며,
  이후 제품 규칙의 정본은 `assets/font-rules/font_rule_registry.json`이다.
- 제품 runtime은 이 조사 JSON이나 canonical JSON을 직접 읽지 않는다. 정적 projection은 Stage
  W7-3에서 만들고 소비자 전환은 W7-4·5에서 별도 승인한다.
- private corpus, host 절대 경로와 font bytes를 이 디렉터리에 기록하지 않는다.

## Stage W7-1 산출물

`font_rule_projection_baseline.json`은 다음을 고정한다.

- 현재 30개 W1 source boundary와 1,352개 candidate의 순서·identity
- W1 1,507개 rule과 candidate의 전건 연결
- W6 metric entry 600개의 안정 `entryId`, current index와 semantic hash
- Rust layout-name 171행과 layout-metric alias 67행
- Canvas2D paint 281행, webfont supply 153행과 CanvasKit SFNT/capability 158행
- Studio runtime의 265개 substitution 결과, 정부상징 successor 65개 availability 조합,
  webfont 153개 등록·load tuple과 CanvasKit 153개 online/offline plan
- backend별 projection hash와 전체 bundle hash

W1의 active `unknown` 44개는 폐기하지 않는다. Rust metric alias 43개는 원래 layout-metric
결정면에서만 legacy-preservation으로 동결하고, measurement predicate 1개는 hand-written runtime
reference로 남긴다. 어느 항목도 identity·paint·supply 관계로 승격하지 않는다.

## 재현 명령

```bash
node scripts/font_rule_projection_baseline.mjs check
node --test scripts/tests/font_rule_projection_baseline.test.mjs

node --test \
  scripts/tests/font_rule_ledger.test.mjs \
  scripts/tests/font_rule_candidates.test.mjs \
  scripts/tests/font_rule_ledger_evidence.test.mjs \
  scripts/tests/font_metric_lineage.test.mjs
```

의도한 Stage W7-1 기준선 갱신만 다음 명령을 사용한다.

```bash
node scripts/font_rule_projection_baseline.mjs generate
```

`generate` 뒤에는 `check`와 mutation negative contract를 다시 통과해야 한다. Stage W7-2 이후에는
이 파일을 registry 입력으로 직접 import하지 않고, 승인된 migration manifest의 전환 전 비교
기준으로만 사용한다.

## Stage W7-2 산출물

- `font_rule_registry_migration.schema.json`
- `font_rule_registry_migration.json`
- [`assets/font-rules/font_rule_registry.schema.json`](../../../../assets/font-rules/font_rule_registry.schema.json)
- [`assets/font-rules/font_rule_registry.json`](../../../../assets/font-rules/font_rule_registry.json)
- [Stage W7-2 보고서](../../../working/task_m100_4966_w7_stage2.md)

```bash
node scripts/font_rule_registry.mjs check
node --test scripts/tests/font_rule_registry.test.mjs
```

830개 registry rule은 다섯 projection에 정확히 한 번씩 배치된다. active unknown metric alias 43개는
legacy-preservation으로 유지되며, Rust metric은 W6 안정 ID 97개만 참조한다. CanvasKit의 W1 SFNT
판정과 현재 URL plan은 합치지 않고 별도 필드로 보존한다.

## Stage W7-3 산출물

- `assets/font-rules/font_rule_projection_manifest.schema.json`
- `assets/font-rules/font_rule_projection_manifest.json`
- `scripts/font_rule_projection_gen.mjs`
- Rust generated source 2개와 Studio TypeScript generated source 3개
- [Stage W7-3 보고서](../../../working/task_m100_4966_w7_stage3.md)

```bash
node scripts/font_rule_projection_gen.mjs check
node --test scripts/tests/font_rule_projection_gen.test.mjs
```

다섯 generated source는 전용 디렉터리의 whole-file ownership을 가지며 아직 runtime이 import하지 않는다.
전체 registry digest는 manifest에만 있고 source에는 backend별 input/projection digest를 넣어 한 규칙의
변경이 무관한 backend source를 갱신하지 않도록 했다.

`font_rule_projection_manifest_v1.json`은 #5955 Stage W7.5-3에서 current manifest가 schema 2.0과 v2
registry provenance로 전환되기 직전의 schema 1.0 manifest를 byte 그대로 보존한 역사 snapshot이다.
현재 projection authority로 읽지 않으며 v1 cutover 감사와 봉인 SHA-256 검증에만 사용한다.

## Stage W7-4·5 runtime 전환

Stage W7-4에서 Rust layout-name 171행과 layout-metric 67행이 generated projection을 소비하게 됐다.
전환 전 표는 test oracle로만 보존하며 production build에는 중복 표가 없다. language·alt-type 분기,
metric exact → bold-only → name-first 사다리와 600개 metric 값·순서는 기존 hand-written owner가 유지한다.

Stage W7-5에서 Studio의 substitution 265행, 정부상징 successor 10행과 webfont catalog 153행의 literal
payload를 제거했다. Canvas2D paint 281행, webfont supply 153행과 CanvasKit SFNT 158행을 각 generated
projection에서 읽는다. document `substFont`, local enumeration/probe, offline filter, glyph coverage와
실제 SFNT byte 판정은 동적 상태이므로 registry로 옮기지 않았다.

관련 보고서는 [W7-4](../../../working/task_m100_4966_w7_stage4.md)와
[W7-5](../../../working/task_m100_4966_w7_stage5.md)에 있다.

## Registry schema 1.0 운영 절차

### 봉인 범위

현재 `font_rule_registry.schema.json` 1.0은 W1/W6에서 일회 이행한 **830개 active rule의 봉인된
정본**이다. 다음 계약은 의도적이다.

- `status`는 `active`만 허용한다.
- 830개 rule과 projection별 171/67/281/153/158 수량을 고정한다.
- 모든 rule은 W1 `ruleId`·candidate와 연결되고 metric rule은 W6 `entryId`를 참조한다.
- `font_rule_registry.mjs check`는 현재 파일을 동결 baseline·W1·W6에서 다시 계산해 byte 대사한다.

따라서 schema 1.0에서 JSON을 직접 고쳐 규칙을 추가·수정하거나 행을 삭제해 폐기하는 것은 지원 절차가
아니다. hash만 다시 계산해 통과시키는 우회도 금지한다. 이는 조용한 정책 변경을 막기 위한 W7 보호
불변식이며, 현재 제품 동작의 **read-only canonical authority**라는 뜻이다.

### 현행 정정과 재생성

동일 의미의 생성기 결함이나 projection 직렬화 결함처럼 registry rule 자체를 바꾸지 않는 정정은 다음
순서를 따른다.

```bash
node scripts/font_rule_registry.mjs check
node scripts/font_rule_projection_gen.mjs generate
node scripts/font_rule_projection_gen.mjs check
node scripts/font_rule_projection_baseline.mjs check
node --test \
  scripts/tests/font_rule_registry.test.mjs \
  scripts/tests/font_rule_projection_gen.test.mjs \
  scripts/tests/font_rule_projection_baseline.test.mjs
```

generator는 다섯 고정 출력만 소유한다. semantic input이 바뀌지 않은 backend의 `projectionSha256`은
그대로여야 한다. generator source 변경 때문에 file content hash나 bundle content hash가 바뀌는 것은
별도 manifest diff로 설명한다. generated Rust·TypeScript 파일은 직접 편집하지 않는다.

### 규칙 추가·수정·폐기

제품 규칙을 실제로 바꿀 때는 먼저 별도 이슈와 승인된 계획으로 registry의 다음 schema 판을 설계한다.
schema 1.0 파일을 임시로 느슨하게 만들지 않는다.

1. **추가**: 새 `ruleId`, decision plane·relation, backend projection, evidence와 상한을 정의한다. 기존
   W1 snapshot을 거짓으로 갱신하지 말고 새 evidence 계보를 schema에 명시한다.
2. **수정**: 기존 `ruleId`를 유지할 수 있는 의미 보정인지, identity가 달라 새 rule과 retirement가
   필요한지 판정한다. 변경 전 rule과 선택 결과를 감사 가능한 형태로 남긴다.
3. **폐기**: 행을 삭제하지 않는다. 새 schema에서 `retired` 상태와 후속 rule·사유·근거를 보존하고,
   generator만 inactive rule을 runtime projection에서 제외하게 한다.
4. schema·validator·mutation test를 먼저 red → green으로 만든 뒤 registry를 이행하고 projection을
   재생성한다.
5. `git diff --name-only`와 manifest를 대사해 의도한 backend generated output만 semantic hash가
   바뀌었는지 확인한다.
6. 해당 backend focused test, W2 `ruleId` join, native/WASM parity와 전체 검증을 통과한 뒤 통합한다.

즉 W8의 첫 실제 mapping 보정은 schema 1.0 JSON 직접 수정이 아니라 **변경 가능한 다음 registry 판의
승인과 이행**부터 시작해야 한다.

## Stage W7-6 최종 검증

- W1·W2·W6·W7 Node contract: 77/77
- release-test nextest: 최신 devel 통합 뒤 8,201/8,201, 정책 skip 41
- native-skia library와 지정 회귀: 통과
- Studio: TypeScript, 1,070 pass·1 skip, production build 통과
- Docker optimized WASM: 통과
- 공개 W1 7문서 167쪽과 HWP/HWPX trace 대표 6문서 page 0의 native/WASM SVG mismatch 0
- fresh WASM trace 3/3, Studio backend 집중 검사 38/38

최종 명령·초기 스키마 계약 실패와 정정 근거는
[W7-6 보고서](../../../working/task_m100_4966_w7_stage6.md), 전체 완료 판정과 W8 인계는
[최종 보고서](../../../report/archives/task_m100_4966_report.md)에 기록한다.
