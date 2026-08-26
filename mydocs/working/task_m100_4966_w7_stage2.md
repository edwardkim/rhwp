---
kind: report
status: completed
canonical: mydocs/plans/task_m100_4966.md
last_verified: 2026-08-23
---

# Task M100 #4966 — Stage W7-2 canonical schema와 migration manifest

## 1. 판정

Stage W7-2는 통과했다. W7-1에서 동결한 830개 projection rule을
`assets/font-rules/font_rule_registry.json`으로 일회 이행하고, W1 rule/candidate와 W6 metric
`entryId`가 끊기지 않았음을 830행 migration manifest로 고정했다. Rust와 Studio runtime source는
아직 이 registry를 소비하지 않으므로 제품 동작은 바뀌지 않았다.

canonical registry의 한 행은 한 relation, 한 decision plane과 정확히 한 backend projection만 가진다.
서로 닮은 source/target 문자열을 이유로 layout name, metric, paint와 supply를 합치지 않는다.

## 2. authority와 산출물

| 산출물 | 역할 |
| --- | --- |
| `assets/font-rules/font_rule_registry.schema.json` | versioned 제품 registry 구조와 상한 |
| `assets/font-rules/font_rule_registry.json` | Stage W7-3 이후 정적 산출물의 canonical 입력 |
| `font_rule_registry_migration.schema.json` | 일회성 W1/W6 이행 증명의 구조 |
| `font_rule_registry_migration.json` | pre-migration rule과 registry rule의 830행 hash 대응 |
| `scripts/font_rule_registry.mjs` | 결정론적 generate/check와 semantic fail-closed validator |
| `scripts/tests/font_rule_registry.test.mjs` | 허용표·anchor·순서·unknown·경로·최소 영향 부정 계약 |

W1 원장과 W6 lineage는 provenance 입력으로 남지만 runtime 입력은 아니다. migration manifest는 이행
감사 증거이며 이후 규칙 편집의 정본도 아니다. Stage W7-3 generator는 canonical registry만 읽도록 한다.

## 3. registry 폐합

| projection | rule | 허용 decision plane / relation |
| --- | ---: | --- |
| `rust-layout-name` | 171 | `layout-name` / `style-fallback` |
| `rust-layout-metric` | 67 | `layout-metric` / `metric-surrogate`, active `unknown` |
| `canvas2d-paint` | 281 | `paint` / substitution·successor·paint·style·generic |
| `canvas2d-webfont` | 153 | `supply` / `supply-source` |
| `canvaskit-sfnt` | 158 | `supply|detection` / `supply-source|capability-detection` |
| **합계** | **830** | rule ID 중복 0, 복수 projection 0 |

관계별 합계는 capability detection 1, document substitution 1, generic fallback 1, metric surrogate
24, official successor 11, paint substitute 267, style fallback 172, supply source 310과 unknown 43이다.

Rust metric 67행은 W6 metric value를 복제하지 않고 대상 name에 해당하는 안정 `entryId` 97개를
참조한다. 각 참조의 name과 W6 current index 순서를 검증하므로 다른 유효 ID로 바꿔치기해도 실패한다.

## 4. active unknown 보호

W1 active unknown 44개 중 실행 projection인 metric alias 43개는 다음 조건을 동시에 만족해야 한다.

- `relationType=unknown`
- `decisionPlane=layout-metric`
- `projection=rust-layout-metric`
- `mode=legacy-preservation`
- `evidenceStatus=unknown`

삭제, `metric-surrogate`로의 추정 승격, paint/supply로의 이동은 모두 거부한다. 나머지 measurement
predicate 1개는 W7-1 판정대로 hand-written runtime reference이며 registry에 억지로 넣지 않았다.

## 5. CanvasKit capability와 계획의 분리

CanvasKit 글꼴별 공급 153행은 W1의 SFNT capability 판정과 현재
`resolveCanvasKitFontPlan`이 만든 URL 계획을 별도 필드로 보존한다.

| W1 declared capability | 현재 online runtime plan | 행 |
| --- | --- | ---: |
| unavailable | planned | 125 |
| SFNT source | planned | 28 |
| SFNT source | unavailable | 0 |

W7-1 분석 중 잠정적으로 기록한 반대 방향 3건은 글꼴별 공급 행과 plan/capability 계약 행을 섞어 센
비동일 모집단이었다. 같은 153개 글꼴 행으로 다시 join한 정본 수치는 **125/0**이다.

`runtimePlanStatus=planned`는 URL을 선택했다는 뜻일 뿐, bytes 다운로드·SFNT parsing·Typeface 생성
성공을 뜻하지 않는다. 따라서 125개 불일치를 `capabilityAgreement=false`로 기록하고 성공 필드를
두지 않았다. 이 불일치의 실행 동작 수정은 Stage W7-2 범위가 아니다.

## 6. fail-closed 계약

validator와 schema는 다음을 거부한다.

- 830행·projection별 분모, 고유 `ruleId` 또는 canonical hash 변화
- relation/decision plane/backend 허용표 밖의 교차 투영
- W1 rule/candidate/source boundary 또는 W6 metric name/entry 순서의 깨진 anchor
- active unknown 삭제·의미 승격과 legacy-preservation의 다른 relation 사용
- 동일 decision group의 null·중복·비연속 order
- schema 밖 필드와 rule-to-rule dependency 필드; 규칙 의존 그래프를 만들 수 없으므로 순환도 구조적으로 금지
- 2,048자 문자열, 830행 registry, 153-font plan과 evidence/metric reference 상한 초과
- `..`, 절대 host path, `file://`, 비 HTTPS 외부 URL과 `fonts/` 밖 local font URL
- Canvas2D supply를 metric으로 투영하거나 URL 계획을 Typeface 성공으로 표시하는 행

generator는 네트워크를 사용하지 않고 W7-1 snapshot의 실행 tuple만 정규화한다. private corpus,
로컬 font root, 사용자명, wall-clock과 font bytes는 registry·migration에 들어가지 않는다.

## 7. digest와 이행 회계

| 항목 | 결과 |
| --- | --- |
| registry rule | 830 |
| direct migration | 787 |
| active unknown legacy migration | 43 |
| W1 candidate link / unique candidate | 830 / 677 |
| W6 metric entry reference | 97 |
| registry rules SHA-256 | `34838af25531327b9e697b065ed5771a11f310c970a9923c83a0b6e1235a68bd` |
| migration mappings SHA-256 | `c73b9350f2116d7534446ea5686b0230df8f652e5686ebf3643ed04ede5f6143` |

schema digest를 포함해 canonical 입력이 바뀌면 `generate` 결과도 바뀌고, `check`는 stale registry와
migration pair를 함께 거부한다. 단일 Canvas2D paint rule mutation에서는 다섯 projection 중
`canvas2d-paint` hash만 바뀌는 것도 계약으로 확인했다.

## 8. 재현과 검증

```bash
node scripts/font_rule_registry.mjs check
node --test scripts/tests/font_rule_registry.test.mjs

node --test \
  scripts/tests/font_rule_projection_baseline.test.mjs \
  scripts/tests/font_rule_candidates.test.mjs \
  scripts/tests/font_rule_ledger.test.mjs \
  scripts/tests/font_rule_ledger_evidence.test.mjs \
  scripts/tests/font_metric_lineage.test.mjs

python3 scripts/check_markdown_links.py --changed-from upstream/devel
git diff --check
```

registry를 의도적으로 다시 이행할 때만 다음 명령을 쓴다.

```bash
node scripts/font_rule_registry.mjs generate
```

생성 뒤 `check`와 부정 계약을 통과하기 전에는 두 JSON 중 하나만 따로 커밋하지 않는다.

검증 결과는 registry 부정 계약 11/11, W1·W6·W7 결합 Node 계약 53/53 통과다. canonical
generate/check, Draft 2020-12 schema validation, JSON parse, 변경 문서 606개 링크 검사와
`git diff --check`도 통과했다. runtime source가 바뀌지 않았으므로 Rust·Studio build와 renderer
검증은 소비자 전환 단계인 W7-4·5 및 최종 W7-6에 남겼다.

## 9. Stage W7-3 인계

Stage W7-3는 이 registry를 입력으로 다섯 backend별 **정적 산출물**을 만드는 generator만 구현한다.
Rust와 Studio consumer 전환은 각각 W7-4·5이므로 아직 source table을 삭제하거나 읽기 경로를 바꾸지
않는다. 출력 경로 allowlist, paired write, stale/manual-edit 검출과 동일 입력 byte 결정론을 먼저
증명한 뒤 다음 단계 승인을 받는다.
