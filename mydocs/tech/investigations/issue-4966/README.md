---
kind: investigation
status: active
canonical: mydocs/plans/task_m100_4966.md
last_verified: 2026-08-23
---

# Issue #4966 canonical font registry와 backend projection

이 디렉터리는 W7 migration 전 font 규칙과 backend별 선택 결과를 재현 가능한 기준선으로 고정한다.

## 권위와 소비 경계

- [수행계획](../../../plans/task_m100_4966.md)이 범위, 보호 불변식과 단계별 승인 게이트의 정본이다.
- [W1 조사 원장](../issue-4939/README.md)은 rule relation·evidence의 선행 근거다.
- [W6 metric lineage](../issue-4964/README.md)는 600개 metric 값·순서·계보의 정본이다.
- `font_rule_projection_baseline.json`은 W7 migration 전 행동 snapshot이며 canonical runtime registry가
  아니다.
- 제품 runtime은 이 조사 JSON을 읽지 않는다. W7의 canonical registry와 정적 projection은 Stage
  W7-2·3의 schema·generator 승인 뒤 별도 파일로 만든다.
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
