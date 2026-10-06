---
kind: report
status: completed
canonical: mydocs/plans/archives/task_m100_5955.md
last_verified: 2026-08-24
---

# Task M100 #5955 — Stage W7.5-2 v2 reducer와 initial migration

## 1. 판정

Stage W7.5-2를 완료했다. 봉인 v1 registry에서 schema 2.0 lifecycle registry를 만드는 side-effect-free
reducer와 수기 semantic validator를 구현하고, 초기 v1→v2 registry·migration을 canonical artifact로
생성했다.

초기 결과는 830 active, 0 retired, 830 carry-forward다. ruleId와 selection tuple이 달라진 mapping은 0개고,
다섯 projection의 before/after semantic SHA-256은 각각 동일하다. 기존 projection generator와 runtime
consumer는 아직 v1을 읽으며, v2 authority 전환은 수행하지 않았다.

## 2. reducer 계약

`scripts/font_rule_registry_v2.mjs`는 다음 책임을 수행한다.

- canonical v1 네 artifact의 SHA-256 봉인 확인
- v1 rule 830개에 projection별 연속 sequence와 immutable selection tuple SHA-256 부여
- append-only change set의 parent digest·sequence·단일 decision plane 검증
- evidence-only, add, retire, retire-and-replace를 clone에 원자적으로 적용
- 오류 발생 시 입력 registry와 canonical artifact를 수정하지 않는 fail-closed 처리
- active rule만 projection하는 query helper
- v1→v2 per-rule migration과 다섯 projection semantic delta 생성·검증

`generate`와 `check`는 checkout에 고정된 path만 사용한다. 초기 migration은 caller가 전달한 v1 객체의
canonical SHA-256도 확인하므로, 파일은 봉인 상태여도 메모리에서 바꾼 v1 입력은 거부한다.

## 3. lifecycle semantic validator

수기 validator는 JSON schema의 shape 검사에 더해 다음 의미를 확인한다.

- 전체 lifecycle에서 `ruleId` 전역 유일
- selection tuple hash 재계산 일치
- active projection sequence의 backend별 유일·연속성
- status와 retirement metadata의 일치
- predecessor·successor 양방향 대사, dangling·cycle·cross-plane 거부
- evidence ID 유일성, parent DAG, dangling·self-parent·cycle 거부
- HTTPS evidence의 digest·provenance, repository evidence의 path·digest·symlink 경계
- stale parent, sequence gap, duplicate change set·operation과 동일 rule 중복 mutation 거부
- 한 command의 현재 rule·신규 rule·replacement가 선언한 decision plane과 projection을 벗어나지 않음
- retired rule은 registry에 보존하되 active projection query에서 제외
- rule 4,096, operation 64, evidence 128, 참조·문자열·metric entry 상한

plain retirement는 immutable projection sequence의 중간 구멍을 만들 수 없으므로 해당 projection의 tail에서만
허용한다. 중간 slot의 semantic correction은 `retire-and-replace`로 새 ID가 기존 slot을 승계해야 한다.

## 4. 초기 v2 registry

| 항목 | 결과 |
| --- | --- |
| source commit | `a1f9872e28aea6755b656161ed1802f73308da58` |
| 전체/active/retired | 830 / 830 / 0 |
| applied change set | 0 |
| sealed-v1 evidence record | 1 |
| ruleId mismatch | 0 |
| source boundary mismatch | 0 |
| projection sequence gap·duplicate | 0 |
| rules SHA-256 | `bd9469aa16156a16ea262f608015cb0b78e925700ae7df69c38602ba6670c029` |

projection population은 v1과 같은 `171/67/281/153/158`이다.

## 5. v1→v2 migration 대사

| projection | rule | before/after semantic SHA-256 | 결과 |
| --- | ---: | --- | --- |
| `rust-layout-name` | 171 | `37f999676888cc8e19cc1f8f6c8976dba5f59f1959199323db4be4208b670a47` | unchanged |
| `rust-layout-metric` | 67 | `bdcd6902c11eacce7088bfaccfe29d76b9df0f895952c2eb8788aa439c6b7aed` | unchanged |
| `canvas2d-paint` | 281 | `5b795e8bcdcb9b223997f0b317f836cf60075501fd4fc3ee40b69a4e3305ac01` | unchanged |
| `canvas2d-webfont` | 153 | `d124099214c82d08f139cffffba79722db9fb6b3fbd30dc165df378caef5000c` | unchanged |
| `canvaskit-sfnt` | 158 | `ea1a07ed27d53ad326bd9cfdfb8ad9f5a4ca50db6b8d611b6673086272cbc4bf` | unchanged |

830개 mapping 모두 v1/v2 ruleId가 같고 before/after selection tuple SHA-256이 같다. lifecycle metadata가
추가되므로 전체 rule object SHA-256은 달라지지만 이를 semantic delta로 해석하지 않는다.

## 6. canonical artifact

| artifact | SHA-256 |
| --- | --- |
| change-set schema | `52049c2fe0c68b538b7e377d7b73108040842bb2702770f65ed54f973c9fe48c` |
| v2 registry schema | `e3412357b2943c00c8a5d2fbb5b0371a8c8c9721157c67e5e38d68ae40a7d7b3` |
| v2 registry | `fbab4413007a29600e5d667503e80b861ec4096827a8936943bdf74e58a5ae16` |
| migration schema | `f955a0918d1d09de28807a77b8d245c48f1d634721bd8cd35de458a697f07af9` |
| migration | `54b17603e3ae52eac7b37f32b1f36b778ad01343501fdcf05bb3ae145f82fb5a` |

Stage W7.5-1의 fixture path·digest를 실제 repository file과 canonical base registry SHA-256에 맞게
현행화했다. 이 evidence는 synthetic text이며 font binary나 private corpus 정보가 아니다.

## 7. 검증 결과

| 검증 | 결과 |
| --- | --- |
| v2 focused lifecycle·migration contract | 22/22 통과 |
| v2 generator `check` | 통과 |
| JSON Schema Draft 2020-12 meta-schema 검사 | 통과 |
| positive fixture의 registry/change-set schema validation | 통과 |
| canonical v2 registry schema validation | 통과 |
| canonical migration schema validation | 통과 |
| v1 봉인 artifact SHA-256 4건 | 동일 |
| mapping ruleId·tuple mismatch | 0 |
| v1 evidence/v2 semantic source boundary mismatch | 0 |
| active projection sequence gap·duplicate | 0 |

mutation contract에는 caller-mutated v1, in-place mutation, stale parent, cross-plane, decision-plane relabel,
evidence cycle, retired projection, unsafe path, operation 상한, malformed nested value와 undeclared evidence가
포함된다.

## 8. 보호 불변식 self-review

- v1 registry·schema·projection manifest·W7 migration byte를 수정하지 않았다.
- 초기 v2의 830개 semantic tuple과 backend별 순서를 바꾸지 않았다.
- actual font mapping·metric·paint·supply rule을 추가·수정·retire하지 않았다.
- fixture change set은 canonical v2 registry에 적용하지 않았다.
- `assets/font-rules/font_rule_change_sets/`에는 아직 제품 command를 만들지 않았다.
- existing projection manifest·generated Rust/TypeScript·runtime consumer를 수정하지 않았다.
- `resolveRuleLifecycle`는 W7.5-4 범위를 선점하지 않고 명시적으로 거부한다.

## 9. Stage W7.5-3 인계

다음 승인을 받으면 projection generator와 current consumer를 v2 active-only authority로 전환한다. 다섯
projection population과 semantic hash는 이 migration 기준선과 같아야 하며, provenance sentinel 때문에
source file hash가 바뀌는 경우 semantic 0-delta와 분리해 기록한다.

W7.5-3 승인에는 lifecycle trace resolver, W8 실제 mapping correction, remote push가 포함되지 않는다.

## 10. Stage W7.5-2C 정정

W7.5-3 착수 검사에서 신규 rule payload가 projection lookup의 `sourceBoundaryId`를 표현하지 못하는 계약
누락을 발견했다. 메인테이너가 계획 변경을 승인한 뒤 v2 rule·change-set payload와 selection tuple에 단일
`sourceBoundaryId`를 추가했다.

초기 830개는 v1 `evidence.sourceBoundaryIds[0]`에서 전건 이행했고 boundary mismatch는 0이다. legacy
evidence 배열은 감사 근거로 그대로 남는다. 신규 rule은 legacy evidence가 `null`이어도 독립적인 semantic
boundary를 가지며, evidence-only operation은 같은 ID의 boundary를 바꿀 수 없다. 이 정정으로 파생
selection tuple·registry·migration hash는 다시 계산했지만 before/after tuple과 실제 projection row의 의미는
변하지 않았다.
