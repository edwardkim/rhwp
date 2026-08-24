---
kind: report
status: completed
canonical: mydocs/plans/task_m100_5955.md
last_verified: 2026-08-24
---

# Task M100 #5955 — Stage W7.5-3 projection cutover와 minimal-impact 증명

## 1. 판정

Stage W7.5-3 구현과 검증을 완료했다. 다섯 backend projection과 metric lineage의 현재 규칙 소비자는
`font_rule_registry_v2.json`의 active rule만 읽는다. 봉인 v1 registry는 migration·역사 대사의 입력으로만
남고 제품 projection authority로는 거부된다.

초기 v2의 population은 830 active, 0 retired이며 projection별 수량은 `171/67/281/153/158`이다. v1과 v2의
830개 semantic row와 projection 순서를 전건 대사했고, 다섯 `projectionSha256`은 모두 동일했다. 실제 font
mapping, metric 값, paint·supply 정책과 runtime trace schema는 바꾸지 않았다.

## 2. authority와 lifecycle 전환

- projection generator는 v2 semantic validator를 먼저 통과한 registry만 입력으로 받는다.
- active rule은 `projectionSequence` 순으로 투영하고 retired rule은 registry에 남기되 모든 runtime output에서
  제외한다.
- manifest schema 2.0은 registry의 active/retired 수량과 projection별 동적 수량을 기록한다. production
  validator에서 초기 830 고정값을 제거했으며, 830 기준은 초기 이행 회귀 테스트에만 남겼다.
- source boundary는 legacy evidence 배열이 아니라 v2의 immutable `sourceBoundaryId`를 사용한다.
- `font_metric_lineage.mjs`도 v2 active layout-metric 규칙을 sequence 순으로 합성하며, 예상 수량은 v2 summary와
  대사한다.
- Rust public integration contract는 현재 v2 active registry를 사용한다. v1은 별도 역사 입력으로 읽어
  830개 의미와 다섯 projection 순서가 같은지 검사한다.

synthetic retirement contract에서 canvas2d-paint tail rule 하나를 retired로 바꿨을 때 registry에는 830개가
남고 manifest는 829 active/1 retired, paint output은 280개가 되며 retired ruleId가 output에 없음을 확인했다.

## 3. v1 manifest 봉인 경계 정정

Stage W7.5-2의 봉인 목록은 갱신되어야 하는 current manifest 경로 자체를 v1 artifact로 지정하고 있었다.
그 상태에서는 Stage 3 재생성 뒤 v2 validator가 current manifest 변경을 봉인 위반으로 올바르게 거부한다.

schema 1.0 manifest의 byte를 다음 역사 경로에 그대로 보존하고 봉인 guard가 이 불변 파일을 검증하도록
정정했다.

```text
mydocs/tech/investigations/issue-4966/font_rule_projection_manifest_v1.json
SHA-256 77089c7dfbb3c6759161d839f5cb8b753c3271e07bb556d6eba87ef45cfaa20d
```

current `assets/font-rules/font_rule_projection_manifest.json`은 schema 2.0, issue #5955와 v2 registry provenance를
가진다. 봉인 해시를 새 current 값으로 덮어써서 v1처럼 주장하는 우회는 하지 않았다.

## 4. semantic 0-delta

| projection | active | v1/v2 `projectionSha256` | 판정 |
| --- | ---: | --- | --- |
| `rust-layout-name` | 171 | `595cdcc1c8d81441c9e4585acb393e734f52e6da3e822babf0f722df2c791cee` | 동일 |
| `rust-layout-metric` | 67 | `c4659fc40246c5d4ad903578a61807c646681638cb4c8f9b7c802fb3f0c37cc2` | 동일 |
| `canvas2d-paint` | 281 | `c959e68087f6928edcafc74a1d3f9cd3885dd7540faf22b7663a49b6ad8835e4` | 동일 |
| `canvas2d-webfont` | 153 | `730cab042d68ffb019d5867102ee8b2b8e5be41c48170ca5fc75422005e3fbee` | 동일 |
| `canvaskit-sfnt` | 158 | `d9019fc756d4fd9334252704309bb2020c251d6a7d04dc0f5a6b2efb0f017668` | 동일 |

`projectionSha256`은 runtime이 소비하는 row만 해시한다. v2 lifecycle·evidence·sequence가 포함된 input object,
v2 sentinel과 manifest provenance는 달라졌으므로 `inputSha256`, generated source `contentSha256`과 bundle
digest는 달라지는 것이 정상이다. 이 provenance 변화와 selection semantic 변화 0건을 분리해 판정했다.

## 5. 검증 결과

| 검증 | 결과 |
| --- | --- |
| v1 registry, v2 registry, projection, pre-migration baseline check | 통과 |
| registry·projection focused Node contract | 53/53 통과 |
| metric lineage baseline과 semantic mutation contract | 9/9 통과 |
| projection·v2 registry·migration JSON Schema Draft 2020-12 | 통과 |
| Studio font decision trace focused test | 5/5 통과 |
| Rust public projection contract | 3/3 통과 |
| `cargo fmt --all`과 `cargo fmt --all -- --check` | 통과 |
| `git diff --check` | 통과 |

Rust helper가 현재 source 배정을 `regression_suite_032`로 계산했으나 committed manifest·harness의 실제 배정은
`regression_suite_015`라서 첫 선택은 0건으로 종료했다. 파생 파일을 재생성하거나 stage하지 않고 실제
allowlisted harness를 직접 선택해 3건을 통과시켰다. 이는 projection 계약 실패가 아니다.

## 6. 사전 존재 metric lineage evidence drift

전체 `font_metric_lineage.test.mjs` 중 deterministic manifest 1건은 이 Stage 이전 upstream 변경으로 계속
실패한다. committed lineage manifest가 causal lineage 보고서의 과거 SHA-256
`f45cf231...`을 봉인하고 있으나 현재 보고서 byte는 `559d4267...`이다. actual/expected 구조 비교에서 차이는
이 evidence digest 한 필드뿐이고 600개 metric identity·값·lookup projection·요약에는 차이가 없다.

manifest digest를 현재 값으로 갱신하면 봉인 v1 registry의 W6 input digest와 registry migration까지 연쇄
변경되어 #5955의 보호 불변식을 깨뜨린다. 따라서 이 Stage에서 수정하지 않았고, v1 registry check와 W6
baseline을 원 상태로 통과시켰다. 별도 lineage maintenance 범위에서 원인 commit과 봉인 정책을 다뤄야 한다.

## 7. 보호 불변식 self-review

- v1 registry·schema·W7 migration과 pre-migration baseline byte를 수정하지 않았다.
- v1 projection manifest는 원 byte를 역사 경로에 보존했고 current runtime authority로 재사용하지 않는다.
- 830개 active rule의 ID·selection tuple·projection 순서와 semantic hash를 바꾸지 않았다.
- actual mapping·metric value·paint·supply rule을 추가·수정·retire하지 않았다.
- generated Rust·TypeScript는 generator로만 갱신했으며 hand edit하지 않았다.
- 새 integration source·generated suite·manifest·Cargo target을 추가하지 않았다.
- lifecycle resolver, W2 offline trace join과 W8 correction은 구현하지 않았다.
- private corpus, font bytes와 host 식별 경로를 artifact에 넣지 않았다.

## 8. 다음 경계

결과 승인을 받으면 이 Stage 변경과 보고서를 한 경계 커밋으로 고정한다. 그 다음 Stage W7.5-4에서 active W2
trace `ruleId`를 lifecycle registry에 join하는 offline audit resolver를 구현한다. runtime trace envelope과
renderer output은 계속 변경하지 않으며 remote push는 별도 승인 대상이다.
