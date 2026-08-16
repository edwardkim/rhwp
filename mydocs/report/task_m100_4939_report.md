---
kind: report
status: completed
canonical: mydocs/report/task_m100_4939_report.md
last_verified: 2026-08-16
---

# Task M100 #4939 — 폰트 규칙 기준선과 Font Rule Ledger 결과 보고서

## 1. 결론

Issue #4939의 범위인 W0 기준선과 W1 Font Rule Ledger를 완료했다.

- 30개 source boundary와 12개 owner를 deterministic snapshot으로 고정했다.
- `FONT_METRICS` 600행, 401개 unique name, 1,856개 lookup projection을 byte-stable baseline으로
  고정했다.
- 실행 source에서 1,352개 rule candidate를 전수 수집했다.
- candidate 전부를 evidence가 있는 1,507개 ledger rule로 판정했다.
- Canvas2D CSS supply와 CanvasKit SFNT supply, source exact와 Hancom missing-font oracle을 분리했다.
- 미확정 44개 규칙은 삭제하거나 identity로 추정하지 않고 `unknown`과 후속 질문으로 보존했다.
- baseline·ledger 재생성, focused gate, native/WASM 167페이지 패리티가 모두 통과했다.
- 제품 font selection과 renderer source 변경은 0개다.

따라서 #4939의 기술 완료 조건은 충족했다. 다만 이 보고서는 로컬 branch의 완료 판정이며, remote push,
PR, issue comment·close와 후속 W2 착수는 각각 별도 승인 대상이다.

## 2. 산출물과 authority

| 산출물 | 역할 |
| --- | --- |
| [수행계획](../plans/task_m100_4939.md) | 범위·승인 게이트 정본 |
| [원인 계보 보고서](font_metrics_fallback_causal_lineage_20260816.md) | 역사·위험·FI-01~FI-14 정본 |
| [조사 README](../tech/investigations/issue-4939/README.md) | W0/W1 산출물 진입점 |
| `font_rule_baseline.json` | W0 source·metric·lookup·fixture snapshot |
| `font_rule_candidates.json` | W1 자동 수집 candidate 1,352개 |
| `font_rule_ledger.json` | 판정된 조사 원장 1,507개 |
| [Font Rule Ledger 요약](../tech/investigations/issue-4939/font_rule_ledger_summary.md) | relation·evidence·unknown·감사 결과 |
| [Stage 5 기록](../working/task_m100_4939_stage5.md) | 정확한 재생성·테스트·build 결과 |

JSON은 historical investigation snapshot이다. W7 승인 전에는 `src/`, `rhwp-studio/src/`, `web/`이
이를 import하거나 runtime registry로 소비하지 않는다.

## 3. W0 기준선 최종 감사

| 항목 | 값 |
| --- | --- |
| source commit | `795e7b5fac24cfef79017e9120516570851a03b2` |
| hashed input | 21개 repository-relative file |
| source owner / boundary | 12 / 30 |
| metric entry / unique name | 600 / 401 |
| duplicate `(name,bold,italic)` key | 0 |
| known input / style projection | 464 / 1,856 |
| 저장 baseline SHA-256 | `a0fac05c3138471eb3e7404fc701f0053caa6c01a923afae60fd4da64064b466` |
| Stage 5 재생성 SHA-256 | 동일 |
| byte delta | 0 |

lookup의 exact → bold-only → name-first 물리 순서는 유지됐고 Rust legacy equivalence test 9개가
통과했다. source digest가 바뀌면 재생성기는 실패하므로 historical snapshot을 현재 HEAD로 조용히
재기록하지 않는다.

## 4. W1 원장 최종 감사

| 항목 | 값 |
| --- | ---: |
| source candidate | 1,352 |
| ledger rule | 1,507 |
| 승인 profile split candidate | 154 |
| unknown | 44 |
| identity alias | 0 |
| 설명된 conflict group | 14 |
| 설명된 self-loop | 5 |
| multi-node cycle | 0 |
| orphan candidate/evidence/test | 0 |
| validator error | 0 |

승인 profile split은 Studio `FONT_LIST` 153개와 정부상징 oracle 1개뿐이다. 나머지 candidate는 정확히
한 ledger 행에 대응한다. 같은 decision key의 다중 target은 모두 unique order를 가지며 동일 target
중복은 없다.

relation 결과는 다음과 같다.

| relation | rule |
| --- | ---: |
| `metric-entry` | 600 |
| `metric-surrogate` | 24 |
| `measured-overlay` | 1 |
| `style-fallback` | 181 |
| `generic-fallback` | 2 |
| `paint-substitute` | 269 |
| `official-successor` | 11 |
| `document-substitution` | 1 |
| `supply-source` | 361 |
| `capability-detection` | 9 |
| `oracle-observation` | 4 |
| `unknown` | 44 |

## 5. FI-01~FI-14 disposition

`충족`은 이번 작업에서 직접 검사했음을 뜻한다. `보존(비대상)`은 제품 동작을 바꾸지 않아 불변식을
침범하지 않았지만, 이후 실제 동작 변경 단계에서 별도 실증이 필요하다는 뜻이다.

| FI | 판정 | 이번 근거 | 후속 경계 |
| --- | --- | --- | --- |
| FI-01 | **충족** | fresh native/WASM 7문서 167페이지 byte match, mismatch 0 | renderer 변경 때 동일 gate 반복 |
| FI-02 | **충족** | 제품 source 0-delta, local-font/renderer baseline test 통과 | opt-in local layout은 별도 profile 필요 |
| FI-03 | **충족** | `layout-name`, `layout-metric`, `paint` plane과 backend/profile을 별도 ledger field로 보존 | W2 trace에서 실제 선택 이유 노출 |
| FI-04 | **충족** | `identity-alias` 0개, ROKG는 `official-successor`, self-loop도 non-identity로 기록 | SFNT/byte evidence 전에는 identity 승격 금지 |
| FI-05 | **충족** | document substitution, successor, surrogate, overlay, paint, oracle, unknown relation 분리 | 44개 unknown은 W5/W8 조사 |
| FI-06 | **충족** | Canvas2D/CanvasKit 혼합 행 0, `FONT_LIST` 153개 profile 분할, local-font/CanvasKit test 통과 | raw probe와 SFNT bytes를 W2 trace에서도 분리 |
| FI-07 | **충족** | W0 baseline byte delta 0, 600 metric·1,856 lookup projection, legacy equivalence test 통과 | 명시적 migration 전 물리 순서 유지 |
| FI-08 | **충족(혼합 방지)** | 정부상징을 `source-exact`, `official-successor`, `hancom-missing-font`로 분리하고 font digest·한컴/PDF producer evidence 연결 | 신규 oracle의 완전 provenance tuple은 W5에서 강제 |
| FI-09 | **보존(비대상)** | page count만 metric 정확성으로 승인하지 않았고 private/fresh layout POC를 완료 증거로 사용하지 않음 | W3/W8에서 stored LineSeg와 fresh layout 별도 cohort |
| FI-10 | **충족** | private corpus 접근·재계측·식별 목록 기록 0, 공개 fixture만 사용 | publication 승인은 별도 유지 |
| FI-11 | **충족** | metric table·생성 파일·overlay 변경 0, baseline 완전 등가 | W6 전 대량 metric 재생성 금지 |
| FI-12 | **충족** | version 분기 추가 0, availability·profile·backend capability를 조건으로 기록 | W2도 feature detection 기반 유지 |
| FI-13 | **충족** | `supply`와 `layout-metric` plane 분리, webfont supply에 metric 호환 주장 없음 | W3 coverage에서 supply hit와 metric hit 분리 |
| FI-14 | **보존(비대상)** | fallback target을 바꾸지 않았고 100% 장평 단문 probe로 개선을 승인하지 않음 | W4/W8에서 장평·자간·고정 프레임 누적 advance gate |

FI-08의 `충족(혼합 방지)`는 #4739에서 확보한 기존 profile provenance를 W1 원장에 올바르게 분리했다는
뜻이다. 모든 미래 PDF oracle이 이미 완전한 schema로 수집된다는 뜻은 아니며 그 강제는 W5의 산출물이다.

## 6. 검증 결과

| gate | 결과 |
| --- | --- |
| validation HEAD | `487da51cafe9dc3d1abeec01608c9227c6bed4ea` |
| W0 baseline 재생성 | byte equal, SHA-256 `a0fac05c…b466` |
| W1 ledger 재생성 | byte equal, SHA-256 `284afd72…8c23` |
| ledger validator | error 0 |
| Stage 1~4 Node contract | 25/25 PASS |
| Rust font metrics | 9/9 PASS |
| Studio font contract | 33/33 PASS |
| frontend font asset | 6/6 PASS |
| native/WASM parity | 7문서, 167페이지, mismatch 0 |
| product source diff | 0 |
| private corpus use | 0 |
| 변경 문서 상대 링크 | 6개 문서, 이상 없음 |
| 변경된 metadata 필수 경로 | 1개 문서, error 0 |

build 산출물의 SHA-256도 Stage 2와 동일했다.

- native: `dedabb18064973f7483a71bb8e8a707011bd1b2df379979db19746caa6d88b30`
- `pkg/rhwp.js`: `20d445f1e9c424a7d72d94bfe17032608bfad4d9a1af2a56275f00b91162cb2c`
- `pkg/rhwp_bg.wasm`: `ebacf1dc16f13ab26b901fe10082ea958a739f3b31d209a9947d31797b75cbb8`

저장소 전체 metadata 검사는 이번 변경과 무관한 기존
`mydocs/tech/benchmark_vs_alternatives.md`의 front matter 누락 4건을 보고했다. 사용자 작업을 임의로
수정하지 않고, 이번 diff에서 metadata 검사가 적용되는 issue-4939 README만 분리해 error 0을 확인했다.

## 7. Issue #4939 완료 조건 disposition

| 완료 조건 | 판정 |
| --- | --- |
| W0가 같은 입력에서 byte-identical 재생성 | 충족 |
| 600 metric, 401 unique name, lookup 순서 일치 | 충족 |
| 모든 source owner의 candidate/disposition 존재 | 충족: 12 owner, 30 boundary |
| 모든 candidate가 ledger 행 또는 승인 split에 대응 | 충족: 누락 0 |
| 근거 미상은 `unknown`으로 보존 | 충족: 44개 |
| relation과 decision plane 분리 | 충족 |
| exact/missing, Canvas2D/CanvasKit profile 분리 | 충족 |
| product source·selection·output delta 0 | 충족 |
| private corpus와 font bytes 비포함 | 충족 |
| W2 입력 가능, runtime dependency 없음 | 충족 |

## 8. 관련 이슈 disposition

| 이슈 | 최종 처리 |
| --- | --- |
| #67, #69, #259, #1224 | layout metric·paint·alias 역사 evidence로 연결; 재개·대체 없음 |
| #536, #4709 | 향후 backend resource와 decision trace 상위 추적; 이번 범위 밖 |
| #1328, #2217, #4741, #4881 | enumeration·probe·SFNT·CanvasKit capability 근거; relation 분리 완료 |
| #2156, #4701 | LineSeg/fresh layout 및 잘못된 oracle 혼합 경고; 완료 근거로 사용하지 않음 |
| #2279, #2430 | identity 오판과 metric 측정 교정 근거; 무근거 identity 승격 금지 |
| #4046 | native/WASM parity gate 재실행 완료 |
| #4168 | metric lookup 물리 순서와 legacy equivalence gate 재실행 완료 |
| #4739 | exact/successor/missing profile과 두 font digest 연결 완료 |
| #4823 | webfont supply와 metric compatibility 분리 근거 |
| #4939 | W0·W1 기술 완료; integration과 issue close는 별도 승인 대기 |

## 9. W2 인계 계약

후속 W2는 `font_rule_ledger.schema.json`과 `font_rule_ledger.json`을 read-only 조사 입력으로 사용할 수
있다. 최소 trace는 다음 ID를 유지해야 한다.

1. candidate evidence anchor와 `ruleId`
2. document face·language slot·altType·document `substFont`
3. layout name/metric relation과 문자별 hit/miss
4. paint chain과 local/web source
5. Canvas2D·CanvasKit·native capability와 선택 실패 이유
6. oracle profile과 known limitation

W2는 원장을 runtime canonical registry로 import하지 않는다. 먼저 현행 resolver의 결과를 읽기 전용으로
설명하며, 44개 `unknown`을 추정으로 메우지 않는다. 제품 규칙 projection은 W7의 별도 승인 전까지
금지한다.

## 10. 남은 위험

- 43개 metric name mapping의 multilingual identity 대 surrogate 판정이 남아 있다.
- generic width estimator의 exact miss·heuristic·synthetic style provenance 분리가 남아 있다.
- W1 원장은 현재 규칙을 설명하지만 실제 문서 한 글자의 선택 경로를 아직 출력하지 않는다.
- 장평·자간·고정 프레임 위험 순위와 개별 face 개선은 W3~W8의 별도 작업이다.
- 신규 PDF oracle을 완전 provenance tuple로 강제하는 schema는 W5에서 마련해야 한다.

이 잔여 항목은 #4939의 누락이 아니라 계획에서 명시한 후속 W2~W8 입력이다.
