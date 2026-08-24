---
kind: investigation
status: active
canonical: mydocs/plans/task_m100_5955.md
last_verified: 2026-08-24
---

# Issue #5955 — font rule lifecycle registry와 trace audit

이 디렉터리는 봉인 schema 1.0에서 lifecycle registry 2.0으로의 이행과 W2 Font Decision Trace의 offline
lifecycle join 계약을 보존한다. 실제 font mapping 변경은 후속 #4967의 별도 change set과 승인 없이는 이
조사 범위에 들어오지 않는다.

## 권위와 산출물

- [수행계획](../../../plans/task_m100_5955.md)은 범위·보호 불변식과 승인 게이트의 정본이다.
- `font_rule_registry_v1_to_v2_migration.json`은 초기 830개 rule의 carry-forward와 semantic 0-delta를
  전건으로 증명한다.
- `font_rule_lifecycle_audit.schema.json`은 trace 원문과 분리된 audit query model의 machine-readable
  계약이다.
- current runtime projection authority는 `assets/font-rules/font_rule_registry_v2.json`의 active rule이다.
- schema 1.0 registry와 W1 ledger는 lifecycle 이전의 역사·reference-only 대사에만 사용한다.

## lifecycle 판정

| 판정 | 의미 |
| --- | --- |
| `carried-forward-active` | 초기 v1→v2 migration에서 같은 ID·selection으로 유지된 active rule |
| `introduced-active` | 승인된 후속 change set이 도입한 active rule |
| `retired` | 역사 row는 남지만 active projection에서 제외되고 successor가 없는 rule |
| `replaced` | retirement와 successor가 함께 기록된 과거 rule |
| `historical-reference-only` | 봉인 W1 ledger에는 있으나 W7 finite projection 소유가 아닌 rule |
| `trace-declared-source-drift` | W2 trace가 `ledgerSourceDrift`라고 명시한 현재 identity |
| `dangling` | v2 lifecycle·봉인 W1 ledger·W2 drift 선언 어디에도 근거가 없는 ID |

resolver는 v2 registry 전체, successor/predecessor graph와 evidence parent graph를 먼저 검증한다. cycle,
cross-plane replacement, dangling evidence가 있으면 한 ID도 분류하지 않고 fail-closed한다.

## offline CLI

W2 trace JSON 파일을 stdout의 canonical audit JSON으로 변환한다.

```bash
node scripts/font_rule_lifecycle_audit.mjs --trace <font-decision-trace.json>
```

CLI는 출력 경로를 받거나 파일을 수정하지 않는다. 입력 host path는 audit JSON에 복사하지 않으며 symlink,
regular file이 아닌 입력과 16 MiB 초과를 거부한다. API 경계는 4,096 records, record당 provenance 64개,
backend별 rule ID 4,096개와 총 reference 262,144개가 상한이다. record ID와 rule ID는 최대 2,048자의 stable
identifier만 허용한다.

audit는 다음 두 W2 경로를 순서대로 보존한다.

- `/records/<n>/provenance/<n>/ruleId`
- `/records/<n>/paint/<backend>/ruleIds/<n>`

trace 객체 자체, `layoutHash`, `normalizedHash`, renderer output과 document bytes는 변경하지 않는다. audit의
`referencesSha256`은 별도 join 결과만 고정한다.

## 재현 명령

```bash
node scripts/font_rule_registry_v2.mjs check
node scripts/font_rule_projection_gen.mjs check
node --test \
  scripts/tests/font_rule_lifecycle_audit.test.mjs \
  scripts/tests/font_rule_registry_v2.test.mjs \
  scripts/tests/font_decision_trace_contract.test.mjs
```

private corpus, font bytes, 사용자 경로와 식별 파일 목록을 audit artifact나 이 디렉터리에 기록하지 않는다.
