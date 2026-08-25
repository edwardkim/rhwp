---
kind: report
status: active
canonical: mydocs/plans/task_m100_5955.md
last_verified: 2026-08-25
---

# Task M100 #5955 — Stage W7.5-8 PR self-review semantic guard 정정

## 1. 진입 사유와 판정

PR #6049의 code head `c40d0f7245186b4407f518f1b085ed0b2b4b8c50`은 Full GitHub Actions
23 success/2 policy skip으로 끝났지만, 메인테이너 self-review에서 v2 change-set validator가 #5955의
보호 불변식을 위반하는 입력을 허용함을 확인했다. 이 head의 녹색 CI는 누락된 negative contract를 실행하지
않았으므로 merge 근거로 재사용할 수 없다.

PR을 Draft로 전환하고 [blocker comment](https://github.com/edwardkim/rhwp/pull/6049#issuecomment-5406226866)를
게시한 뒤 최소 정정을 수행했다. 이 단계의 판정은 **로컬 정정 완료 / 새 code head push·Full CI 전**이다.

## 2. 재현과 원인

정정 전 `validateChangeSet()`은 다음 입력을 모두 error 0으로 수용했다.

| 변조 | 정정 전 | 보호 불변식 위반 |
| --- | --- | --- |
| `canvas2d-paint` + `supply-source` relation | 허용 | paint와 supply relation 혼합 |
| paint rule + `metricEntryIds` | 허용 | metric anchor의 backend 소유권 위반 |
| paint rule + `file:///home/private/font.ttf` webfont payload | 허용 | non-supply payload·host path 유출 |

또한 `evidenceRecords={}` 또는 active rule의 `projections=null` 같은 malformed nested collection은 오류 목록을
반환하지 않고 `TypeError`를 발생시켰다.

근본 원인은 W7의 v1 validator가 갖고 있던 projection별 relation allowlist, metric anchor, Canvas2D
family·URL·external, CanvasKit capability agreement, non-supply payload 금지와 host path guard를 v2로
이행하지 않은 것이다. v2는 enum shape와 decision plane/projection 일치만 검사해 각 backend가 소유한
semantic payload의 결합을 검증하지 않았다. 기존 CI는 이 누락을 표현하는 negative fixture가 없어 통과했다.

## 3. 정정 내용

`scripts/font_rule_registry_v2.mjs`의 registry와 change-set payload가 같은 semantic validator를 사용한다.

- 다섯 projection마다 허용 decision plane과 relation 집합을 v1 계약과 동일하게 고정했다.
- `unknown` relation은 봉인 v1에서 이행한 `legacy-preservation` rule에만 허용하고 신규 payload에서는 거부한다.
- `rust-layout-metric`만 metric entry를 소유하며, #4964 lineage manifest의 대상 metric identity와 entry 순서가
  정확히 일치해야 한다. 다른 projection의 metric reference는 거부한다.
- Canvas2D webfont는 projection 전용 payload, `fontFamily == sourceFace`, HTTPS/external 또는 안전한
  `fonts/` 상대경로/local 조합만 허용한다.
- CanvasKit finite supply는 projection·profile 전용 payload, family 일치와 declared capability/runtime plan/
  `capabilityAgreement`의 논리 일치를 요구한다.
- 모든 non-supply projection의 supply payload와 POSIX·Windows host absolute path, `file://`, local font
  traversal을 거부한다.
- registry/change set의 잘못된 collection type과 malformed evidence/projection은 validator가 throw하지 않고
  오류 목록으로 fail-closed하도록 순회 입력을 정규화했다. 초기 세 재현 뒤 evidence parent, rule evidence,
  successor/predecessor collection까지 추가로 변조해 같은 totality 경계를 고정했다.

JSON Schema 파일은 바꾸지 않았다. schema는 shape·자료형·상한을, `font_rule_registry_v2.mjs`는 relation-
projection 조합, metric identity, supply family·URL·capability와 graph/path 같은 교차 필드 의미를 소유한다.
따라서 reducer는 schema 적합성만으로 change set을 수용하지 않는다.

## 4. negative contract

`scripts/tests/font_rule_registry_v2.test.mjs`에 다음 세 계약을 추가하고 malformed collection 검사를 확장했다.

1. v1 projection relation·metric·supply 소유 경계 유지
2. Canvas2D family/URL/external 및 CanvasKit capability agreement
3. 신규 unknown legacy rule 금지

최초 재현 세 건은 정정 후 각각 `mixes supply-source with canvas2d-paint`,
`only rust-layout-metric may reference metric entries`, host path와 non-supply payload 오류로 모두 거부됐다.

## 5. 완료한 로컬 검증

| 검증 | 결과 |
| --- | --- |
| v1 registry check | 통과 |
| v2 registry focused | 26/26 통과 |
| v2 registry deterministic check | 통과 |
| projection generator check | 통과 |
| pre-migration projection baseline check | 통과 |
| 전체 `scripts/tests/font_rule_*.test.mjs` | 96/96 통과 |
| Rust unit-tier | 4,221 tests / 299 modules / drift 0 |
| `node --check`, `git diff --check` | 통과 |

추가 진단으로 실행한 W6 lineage 전역 `--check-manifest`는 #5955가 수정하지 않은
`mydocs/report/font_metrics_fallback_causal_lineage_20260816.md`의 기존 evidence digest drift를 보고했다.
v1 registry check와 이번 semantic validator가 사용하는 600개 metric entry identity·순서는 통과했으며, 기존
문서 digest 부채를 이번 correction에 섞어 고치지 않았다.

## 6. 제출 경계와 남은 게이트

- canonical v1/v2 registry, migration, generated Rust·TypeScript와 runtime renderer는 변경하지 않았다.
- 새 integration source·generated suite·manifest, sample, PDF, font bytes와 visual asset은 없다.
- 제품 mapping과 native/WASM output은 직전 candidate와 동일하다. 이 correction은 future change set의 입력
  검증 경계만 강화한다.
- correction candidate에서 `cargo fmt --all`과 `cargo fmt --all -- --check`를 다시 실행해 통과했다.
- 새 code head를 push한 뒤 Full CI·CodeQL·Render Diff를 다시 통과시켜야 한다. 그 전에는 Ready 전환,
  self-review 수용 기록과 merge를 진행하지 않는다.
