---
kind: report
status: completed
canonical: mydocs/plans/task_m100_5955.md
last_verified: 2026-08-24
---

# Task M100 #5955 — Stage W7.5-1 executable contract red

## 1. 판정

Stage W7.5-1의 RED 계약을 완료했다. v2 lifecycle registry, append-only change set과 v1→v2 migration의
최소 schema vocabulary를 만들고, 실제 구현이 들어갈 API 경계를 고정했다. 새 테스트는 단순 import 실패가
아니라 `ERR_W75_NOT_IMPLEMENTED`에서만 실패한다. 기존 W7 registry와 다섯 projection 검증 29건은 계속
통과했다.

이 단계에서는 reducer·semantic validator·canonical v2 registry를 구현하지 않았다. projection generator도
여전히 봉인 v1 registry만 읽는다. 따라서 RED 12건은 결함 회귀가 아니라 다음 W7.5-2가 해결해야 할
실행 계약이다.

## 2. 확정한 계약

### 2.1 schema 2.0 registry

`font_rule_registry_v2.schema.json`은 다음을 표현한다.

- v1 registry digest와 적용한 change set의 계보
- rule의 `active|retired` lifecycle, predecessor·successor와 변경 command
- immutable selection tuple SHA-256과 backend별 projection sequence
- legacy W1/W6 evidence와 일반 evidence record의 분리
- 전체 rule 4,096개, rule당 evidence 16개, successor·predecessor 각각 8개, 문자열 2,048자,
  metric entry 600개의 상한
- nested object를 포함한 unknown field 거부

### 2.2 append-only change set

`font_rule_change_set.schema.json`은 한 command가 정확히 한 decision plane을 소유하도록 하고 다음 operation만
허용한다.

| operation | 의미 |
| --- | --- |
| `augment-evidence` | selection tuple을 바꾸지 않고 기존 rule의 evidence만 보강 |
| `add-rule` | 새로운 의미와 새로운 `ruleId` 도입 |
| `retire-rule` | 역사 row를 보존하며 runtime projection에서 제외 |
| `retire-and-replace` | 기존 rule을 retire하고 새 `ruleId`로 semantic correction |

한 change set은 operation 64개와 evidence record 128개가 상한이다. parent registry SHA-256, sequence와 대상
projection의 예상 active delta, 비대상 projection 네 개의 무변화 선언을 필수화했다.

### 2.3 초기 migration

v1→v2 migration schema는 830개 mapping과 다섯 projection delta를 전건으로 기록한다. 초기 canonical
migration의 허용 disposition은 `carry-forward`뿐이며 830 active, 0 retired와 projection semantic
`unchanged`를 요구한다.

## 3. fixture vocabulary

공개 synthetic 이름만 사용한 작은 fixture 다섯 개를 추가했다. font binary, private corpus, host path와 실제
제품 mapping은 포함하지 않는다.

| fixture | 기대 lifecycle |
| --- | --- |
| `carry-forward` | 같은 ID·selection tuple·projection 유지 |
| `evidence-only` | 같은 ID·tuple을 유지하고 evidence head만 이동 |
| `add-rule` | 새 ID와 active projection 한 행 추가 |
| `retire-rule` | 역사 row는 보존하고 active projection 한 행 제거 |
| `retire-and-replace` | old/new 계보 연결과 active slot 승계 |

## 4. RED test 결과

```text
tests 15
pass 3
fail 12
```

통과한 3건은 v1 봉인 해시, schema 보안 상한과 positive fixture vocabulary다. 실패한 12건은 모두 다음
stub 경계 중 하나에서 `ERR_W75_NOT_IMPLEMENTED`로 종료됐다.

- `reduceRegistryV2`
- `validateChangeSet`
- `validateRegistryV2`
- `buildMigrationV1ToV2`
- `projectActiveRules`

따라서 failure는 fixture parse, 누락 import나 기존 W7 drift 때문이 아니다. 다음 계약이 아직 구현되지 않았기
때문이다.

- carry-forward, evidence-only, add, retire, replace positive lifecycle 5건
- 초기 830-rule carry-forward migration 1건
- in-place semantic mutation, stale parent, cross-plane, evidence self-cycle, retired projection,
  unsafe path·operation 상한 negative 6건

## 5. v1 보호 불변식 검증

`assertSealedV1Artifacts`는 다음 네 artifact의 현재 byte SHA-256을 검사한다.

| artifact | SHA-256 | 결과 |
| --- | --- | --- |
| v1 registry | `f549ca3a8807be712cc197daf14d96abb1e5f075ac55f1d9142db67c1a56681a` | 동일 |
| v1 schema | `068327e9f49843c54d0f4da16d6f0081bca86b38fe85e362c8416462f15d3ab4` | 동일 |
| projection manifest | `77089c7dfbb3c6759161d839f5cb8b753c3271e07bb556d6eba87ef45cfaa20d` | 동일 |
| W7 migration | `11b93350a0702c75af07ffde7bae4aff2dab332c43ad9bb57d1e3cf1a1747e83` | 동일 |

기존 검증 결과는 다음과 같다.

| 검증 | 결과 |
| --- | --- |
| W7 registry·projection·baseline Node contract | 29/29 통과 |
| `font_rule_registry.mjs check` | 통과 |
| `font_rule_projection_gen.mjs check` | 통과 |
| `font_rule_projection_baseline.mjs check` | 통과 |
| 신규 schema·fixture JSON parse | 통과 |

## 6. 구현 경계 self-review

- v1 artifact 네 파일은 수정하지 않았다.
- canonical `font_rule_registry_v2.json`, change-set 정본과 migration JSON은 생성하지 않았다.
- 기존 projection source·manifest·sentinel과 runtime 코드를 수정하지 않았다.
- 실제 font mapping이나 backend 의미를 바꾸지 않았다.
- `scripts/font_rule_registry_v2.mjs check`는 봉인 guard를 통과한 뒤 의도적으로 RED에서 종료한다.
- schema의 수기 semantic validator와 path·graph 검사는 아직 없으며, 이를 통과한 것처럼 기록하지 않았다.

## 7. Stage W7.5-2 인계

다음 승인을 받으면 W7.5-2에서 side-effect-free reducer와 semantic validator를 구현한다. 우선순위는
change-set structural·security 검증, evidence DAG와 stale parent 판정, lifecycle reducer, 초기 830-rule
carry-forward migration 순이다. 종료 조건은 신규 focused test GREEN, v1 네 artifact byte 불변, v2 830
active/0 retired와 per-rule selection tuple delta 0이다.

projection generator의 v2 전환은 W7.5-3이므로 W7.5-2 승인에 포함되지 않는다.
