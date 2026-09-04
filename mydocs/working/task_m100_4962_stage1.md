# Task M100 #4962 W3 Stage 1 — 기존 자산·분류·분모 계약

- **Issue**: [#4962](https://github.com/edwardkim/rhwp/issues/4962)
- **상위 추적**: [#4960](https://github.com/edwardkim/rhwp/issues/4960)
- **계획**: [`task_m100_4962.md`](../plans/archives/task_m100_4962.md)
- **브랜치**: `task_m100_4962`
- **착수 기준**: `upstream/devel` `fb434269eea237cc12053914560a2dbaf16270bf`
- **계획 기준 commit**: `b82eaeefa8b9fd80916a4fd2d2d4623a42463ead`
- **날짜**: 2026-08-21 KST
- **단계 상태**: Stage 1 기술 완료, Stage 2 메인테이너 승인 대기

## 1. 결론

기존 10k 계측은 유효하며 폐기하거나 같은 편집 습관 축으로 다시 실행할 이유가 없다. 보존된 v2
aggregate는 문서 10,000건 중 9,948건, 실제 조판 문자 54,938,759자와 57,395개 usage row를 재현
가능하게 유지한다. 전체·HWP·HWPX 결과의 corpus·totals와 주요 distribution도 가산 일치한다.

W3가 새로 읽어야 하는 값은 기존 aggregate에서 소실된 문자별 renderer decision뿐이다. Stage 1은
기존 POC projection과 W1 의미를 동결하고, 7개 상호배타 분류·독립 분모·privacy·hash 계약을
machine-readable JSON과 10개 focused test로 고정했다. 제품 source, metric·fallback 정책, renderer
output과 private corpus 원문은 변경하지 않았다.

## 2. RED와 GREEN

계약 test를 먼저 추가해 실행했을 때 다음처럼 실패했다.

```text
Error [ERR_MODULE_NOT_FOUND]: Cannot find module
'scripts/font_metric_coverage_contract.mjs'
tests 1, pass 0, fail 1
```

이는 기존 계측 자료가 없다는 뜻이 아니라, 기존 자료와 W3 delta 사이의 계약 구현이 아직 없다는
Stage 1 RED다. 이후 contract JSON·schema와 최소 validator를 추가해 같은 target을 10/10 GREEN으로
전환했다.

## 3. 기존 POC 자산 동결

### 3.1 재사용 자산

| 로컬 gitignored 산출물 | 판정 |
| --- | --- |
| `summary-10k-v2.json` | 전체 10k 기준선으로 재사용 |
| `summary-hwp-v2.json` | HWP format projection으로 재사용 |
| `summary-hwpx-v2.json` | HWPX format projection으로 재사용 |
| `smoke-20.json`, `smoke-hwpx.json` | collector 소표본 비교에 재사용 |
| `briefing-20260816.md` | 방법·한계 기록으로 재사용 |
| commit `631287d4708f144011162179d61f8272cf072ff6` | 기존 재귀 POC walker source로 재사용 |

전체 v2 artifact의 비식별 projection은 다음 값으로 동결했다.

| 항목 | 값 |
| --- | ---: |
| discovered / attempted | 10,000 / 10,000 |
| HWP / HWPX | 6,582 / 3,418 |
| success / failure | 9,948 / 52 |
| paragraphs | 3,776,306 |
| actual layout characters | 54,938,759 |
| declared fonts / usage rows | 1,414 / 57,395 |
| face-level mapped / unmapped characters | 50,085,082 / 4,853,677 |
| de-identified projection SHA-256 | `423d0116143a74e336dd4e17fb5bb2aed48f0f5876b898e6afed1bd3e982e691` |

projection은 corpus·totals, row schema, font·usage·주요 distribution 문자 합과 mapped/unmapped 합만
포함한다. `inputRoot`, `riskDocuments`, 문서별 `source`·BLAKE3와 원문은 포함하지 않는다. HWP와 HWPX
format 결과의 모든 numeric corpus·failure category·totals와 alignment·base size·context·ratio·spacing
distribution이 전체 결과에 가산되는 것도 자동 검사했다.

### 3.2 재사용과 delta 경계

다음 값은 기존 POC projection을 그대로 사용한다.

- format, font, language slot, bold·italic
- 장평·자간, kerning, context, alignment, stored LineSeg 존재 여부
- 문서·문단·run·문자 수

다음 값만 Stage 2 collector의 delta 대상이다.

- `altType`, `substFont`, layout name resolution
- `metricEntry`, `matchKind`, `characterMatch`, `widthSource`
- W1 relation과 최종 coverage category
- source join과 별도 backend 관찰 상태

재사용 field와 delta field가 겹치면 contract 자체가 실패한다.

## 4. W1 의미 drift 감사

W1 historical snapshot `795e7b5fac24cfef79017e9120516570851a03b2`와 현재 source를 같은
collector·candidate generator로 메모리상 재생성해 비교했다. 기존 JSON을 덮어쓰거나 새 snapshot을
커밋하지 않았다.

| 항목 | 결과 |
| --- | ---: |
| source boundaries | 30 |
| current rule candidates | 1,352 |
| candidate identity 추가 / 삭제 / 의미 변경 | 0 / 0 / 0 |
| source digest drift boundary | 20 |
| metric entries | 600, 변화 없음 |
| metric table projection | 변화 없음 |
| lookup projection | 변화 없음 |
| ledger rule anchor가 없는 candidate | 0 |

digest가 바뀐 20개 boundary는 다음과 같다.

- Rust style resolution 4개: legacy Latin, HFT, TTF, heavy display
- Rust metric 3개: alias, table, lookup
- Rust measurement 2개: estimate width, Hancom space
- Rust paint chain 3개: installed aliases, weight suffix, generic fallback
- native Skia 2개: system family/style, text replay
- Studio substitution·supply·Canvas patch 6개

candidate identity가 모두 동일하고 metric/lookup projection도 같으므로 **의미 drift가 아니라
digest-only drift**로 판정했다. W1 원장을 현재 source digest로 조용히 덮어쓰지 않고 W2의
`ledgerSourceDrift` 상태를 유지한다.

현재 원장의 W3 관련 relation population도 동결했다.

| relation | rule 수 | W3 처리 |
| --- | ---: | --- |
| `identity-alias` | 0 | 실제 count는 0이어야 하며 추정 승격 금지 |
| `metric-surrogate` | 24 | character hit일 때만 해당 분류 |
| `measured-overlay` | 1 | width source 우선순위와 함께 판정 |
| `metric-entry` | 600 | exact·char miss의 entry 근거 |

## 5. W2 decision inventory와 분류

W2 v1의 `matchKind`는 `exact`, `boldOnly`, `nameFirst`, `none`, `characterMatch`는 `hit`, `miss`,
`notApplicable`이다. 최종 `CharWidthDecision`의 현재 width source는 다음처럼 분리했다.

| 역할 | width source |
| --- | --- |
| measured overlay | `kopubTable`, `metricSpaceOverlay`, `metricNarrowPunctuationOverlay`, `metricHalfwidthPunctuationOverlay` |
| metric character hit | `embeddedMetric`, `metricHalfSpace` |
| metric miss 후 fallback | `heuristicFullwidth`, `heuristicNarrow`, `heuristicHalfwidth` |
| 명시적 비-metric 정책 | `areaDotFallback` |
| non-applicable | `clusterContinuation`, `inlineObjectPlaceholder`, `hwpPuaFiller`, `figureSpace`, `tabAdvance` |
| 최종 trace에 나오면 실패 | 내부 중간값 `metricMiss`, `metricCharacterMiss` |

같은 `heuristic*` width source라도 `metricEntry`가 있고 `characterMatch=miss`이면 `char-miss`, entry가
없고 `characterMatch=notApplicable`이면 `face-miss`다. width source 문자열 하나만으로 두 miss를
합치지 않는다.

분류 우선순위는 다음과 같다.

1. measured overlay
2. verified identity alias hit
3. metric surrogate hit
4. exact metric hit
5. character miss
6. face miss
7. explicit heuristic policy

identity alias는 `verified-by-bytes` relation만 인정한다. 현재 W1 원장에는 해당 규칙이 0개이므로 새
증거 없이 이름이 비슷하다는 이유로 분류를 만들 수 없다. 새 width source, 새 match kind, unverified
identity alias 또는 모순된 metric 상태는 `heuristic`에 흡수하지 않고 fail closed한다.

## 6. 분모와 long-page 계약

다음 합계를 서로 대체하지 않는다.

```text
layoutCharacters
  = coverageCharacters + notApplicableCharacters + excludedCharacters

coverageCharacters
  = measured-overlay + identity-alias-hit + metric-surrogate + exact-hit
  + char-miss + face-miss + heuristic

layoutCharacters
  = joined + layoutOnly + excluded

attemptedDocuments
  = successDocuments + drm + empty + encrypted + parser + unsupported

backendRequested
  = complete + unsupported + notObserved + failed
```

collector는 streaming이고 page 문자 상한이 없어야 하며 `truncatedCharacters`는 반드시 0이어야 한다.
source join, parser와 backend 상태가 발생하지 않았더라도 key와 0 count를 생략할 수 없다. 이를 통해
긴 page 뒤쪽 누락이나 unsupported backend를 metric miss 또는 성공으로 가장하는 일을 막는다.

## 7. privacy와 hash

정식 aggregate에는 다음 key를 허용하지 않는다.

- `inputRoot`, `source`, `path`, `fileName`·`filename`
- 문서별 `blake3`·`documentHash`
- `character`, `codePoint`, `records`, `rawTrace`, `riskDocuments`

문자 decision은 streaming 집계 후 폐기한다. 문자열에서도 host home path, access token과 stack trace를
검출한다. `sourceCommit`처럼 재현에 필요한 비식별 필드는 허용한다.

canonical hash는 object key를 code-unit 순서로 정렬하고 hash 자신, timestamp, generated time과
elapsed/duration을 제외한다. 같은 수치와 source commit이면 key 입력 순서나 실행 시간에 관계없이 같은
SHA-256이 나온다.

## 8. 산출물

| 경로 | 역할 |
| --- | --- |
| [`font_metric_coverage_contract.schema.json`](../tech/investigations/issue-4962/font_metric_coverage_contract.schema.json) | Draft 2020-12 schema |
| [`font_metric_coverage_contract.json`](../tech/investigations/issue-4962/font_metric_coverage_contract.json) | 기존 자산·분류·분모 계약 |
| `scripts/font_metric_coverage_contract.mjs` | 검사·분류·대사·privacy·hash 구현 |
| `scripts/tests/font_metric_coverage_contract.test.mjs` | Stage 1 focused test 10건 |
| [`issue-4962/README.md`](../tech/investigations/issue-4962/README.md) | 조사 사용 경계와 명령 |

## 9. 검증 결과

### 9.1 focused contract

```bash
node --test scripts/tests/font_metric_coverage_contract.test.mjs
```

결과: **10 passed, 0 failed**.

검증 범위:

- 7개 상호배타 분류와 모든 분류 대표 vector
- non-applicable 분리와 새 source·match kind fail closed
- long-page truncation 금지
- join·parser·backend 상태 key와 독립 분모 대사
- canonical hash와 volatile field 제외
- raw identity·path·token·stack privacy 실패
- current W1 candidate 의미와 ledger relation population
- 기존 POC 비식별 projection과 HWP+HWPX 가산성

### 9.2 독립 contract·기존 POC 검사

```bash
node scripts/font_metric_coverage_contract.mjs check \
  --poc output/poc/font-layout-habits/summary-10k-v2.json \
  --poc-hwp output/poc/font-layout-habits/summary-hwp-v2.json \
  --poc-hwpx output/poc/font-layout-habits/summary-hwpx-v2.json
```

결과:

```text
font metric coverage Stage 1 contracts: ok; W1 1352 candidates,
20 digest-only boundaries; POC v2 baseline ok
```

### 9.3 schema·syntax

- JSON Schema Draft 2020-12 Ajv strict compile: 통과
- contract instance validation: 통과
- 두 `.mjs`의 `node --check`: 통과
- `git diff --check`: 통과

### 9.4 historical W1 snapshot guard

```bash
node --test scripts/tests/font_rule_candidates.test.mjs
```

결과: **1 passed, 7 failed**. 일곱 실패는 모두 다음 fail-closed 메시지다.

```text
source digest changed since W0: src/renderer/style_resolver.rs
```

이 target은 historical `font_rule_candidates.json`의 source digest를 현재 checkout과 직접 비교하므로,
20개 boundary digest drift가 있는 현재 `devel`에서는 의도대로 닫힌다. Stage 1은 이 실패를 성공으로
기록하지 않았다. historical snapshot을 덮어쓰는 대신 현재 boundary를 메모리상 재생성한 뒤 같은
candidate generator를 적용하는 새 검사에서 identity 추가·삭제·의미 변경 0건과 1,352개 ledger anchor
폐합을 확인했다. 기존 W1·W2의 나머지 focused 계약은 **22 passed, 0 failed**였다.

## 10. 보호 불변식 판정

| 항목 | Stage 1 판정 |
| --- | --- |
| 기존 10k 편집 습관 통계 | 재생성·덮어쓰기 없음 |
| metric 값·lookup·fallback | 변경 없음 |
| renderer source/output | 변경 없음 |
| W1 원장 | runtime registry화·digest 덮어쓰기 없음 |
| identity alias | 0개를 그대로 보존, 추정 없음 |
| raw trace·private 원문 | 저장·출력·커밋 없음 |
| full 10k delta pass | 실행하지 않음 |
| backend/fresh layout | 실행하지 않음 |

## 11. 잔여 위험과 Stage 2 인계

- contract와 validator는 collector 구현 전 규약이다. 아직 actual document walker가 W3 aggregate를
  생성하지 않는다.
- W2 trace는 page 4,096자 상한이 있으므로 그대로 corpus runner로 반복 사용할 수 없다. Stage 2는 기존
  POC 재귀 walker에 같은 `CharWidthDecision`을 streaming 결합해야 한다.
- W2의 모든 record에는 generic measurement provenance가 있지만 Hancom space overlay의 구체 W1
  candidate를 항상 연결하지는 않는다. Stage 2에서 width source와 해당 source boundary를 정확히 join
  하되 lookup 사다리를 복제해서는 안 된다.
- 기존 briefing의 두 번 실행 hash 일치 주장은 현재 v2 전체 artifact 한 벌만 남아 독립 재검증할 수
  없다. 새 delta의 반복 결정성은 Stage 3 pilot 이후 별도 승인된 실행으로 검증한다.
- source join과 LineSeg 유효성은 아직 구현되지 않았다. format/version 분기가 아니라 현재 객체 상태의
  feature detection으로 Stage 2에 추가한다.

## 12. 다음 승인 지점

Stage 1 변경은 별도 commit으로 고정한다. 메인테이너가 Stage 2를 승인하면 기존 POC walker와 현재
`CharWidthDecision`을 공유하는 streaming decision delta collector 구현을 시작한다. Stage 2 승인에는
private 10k 전수 실행, backend/fresh-layout 전수 실행, remote push 또는 PR 생성 권한이 포함되지 않는다.
