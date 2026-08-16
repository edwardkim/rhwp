# Task M100 #4961 Stage 1 — schema·identity·상한 RED

- **Issue**: [#4961](https://github.com/edwardkim/rhwp/issues/4961)
- **상위 추적**: [#4960](https://github.com/edwardkim/rhwp/issues/4960)
- **계획**: [`task_m100_4961.md`](../plans/task_m100_4961.md)
- **브랜치**: `local/task4961-font-decision-trace`
- **기준**: `upstream/devel` `418e5b191d23cf0618ce99f0cfec332c19ac1bc2`
- **날짜**: 2026-08-17 KST
- **단계 상태**: Stage 1 기술 완료, Stage 2 메인테이너 승인 대기

## 1. 수행 범위

Stage 1은 Font Decision Trace의 machine-readable 계약만 고정했다. `src/`, `rhwp-studio/src/`,
`Cargo.toml`, package manifest와 기존 fixture bytes는 변경하지 않았다.

산출물은 다음과 같다.

| 경로 | 결과 |
| --- | --- |
| `mydocs/tech/investigations/issue-4961/README.md` | W1 authority, fail-closed, hash와 공개 fixture 경계 |
| `font_decision_trace.schema.json` | 별도 trace v1 봉투·문자 record·backend certainty·상한 |
| `font_decision_identity_vectors.json` | W1과 byte-identical한 candidate/rule ID golden 4건 |
| `public_fixtures.json` | repository-tracked HWP/HWPX 후보 path·size·SHA-256 |
| `scripts/font_decision_trace_contract.mjs` | identity, ledger join, limit, hash, drift, 민감정보·fixture 검사 |
| `scripts/tests/font_decision_trace_contract.test.mjs` | 정상·변이·실패 계약 11건 |

## 2. RED에서 확인한 결함

계약 test를 먼저 추가하고 실행했을 때 다음과 같이 실패했다.

```text
Error [ERR_MODULE_NOT_FOUND]: Cannot find module
'scripts/font_decision_trace_contract.mjs'
tests 1, pass 0, fail 1
```

이는 제품 결함 재현이 아니라 Stage 1 계약 구현이 아직 없음을 확인하는 RED다. 이후 최소 contract
module을 추가해 같은 test target을 GREEN으로 전환했다.

초기 GREEN 전환 중 Studio substitution vector에서 ledger의 `order: 0`을 candidate identity에도 넣은
오류를 test가 잡았다. W1 collector 원문을 다시 확인한 결과 `order`는 ledger의 판정 정보이고 해당
candidate identity에서는 `null`이었다. golden을 W1 candidate anchor
`candidate.674bc70d924bf5707bcf`와 일치하도록 정정했다. ledger 표현을 candidate 입력으로 역투영하지
않아야 한다는 경계를 실제 test로 확인했다.

## 3. 고정한 계약

### 3.1 별도 v1 봉투

기존 `getRendererDiagnostics` v1은 바꾸지 않는다. 새 trace는 `schemaVersion: 1`과 다음 상태를 가진다.

- trace: `complete`, `truncated`, `unsupported`, `failed`
- backend: `complete`, `unsupported`, `notObserved`, `failed`
- certainty: `observed`, `resolved`, `planned`, `notObserved`, `unsupported`

문자 record는 source, document, layout name, layout metric, backend별 paint, provenance와 oracle 상태를
분리한다. Canvas2D처럼 실제 glyph face를 관찰하지 못한 backend는 `notObserved`를 사용할 수 있다.

### 3.2 상한과 fail-closed

- 기본 1,024문자, hard maximum 4,096문자
- page index는 0 이상의 safe integer
- `0`, 음수, 실수, 문자열, 4,096 초과는 거부
- requested limit과 applied limit이 다르면 silent clamp로 실패
- `truncated`에는 `characterLimitExceeded`가 필수
- 누락 수를 모르면 `null`과 `recordsOmittedUnknown`을 함께 사용
- candidate가 ledger와 join되지 않으면 `ruleId: null`, `ledgerRuleMissing`

### 3.3 W1 identity

W1 canonical JSON과 SHA-256 규약을 그대로 재사용한다. 대표 golden은 다음 네 종류다.

| vector | ruleId |
| --- | --- |
| metric alias/surrogate | `rule.rust-metric.0b74264b87ed459ef5db` |
| metric entry 0 | `rule.rust-metric.b461c7b55f3d0a77a68d` |
| measurement predicate | `rule.rust-measurement.28970f1984ed0e4ce06d` |
| Studio substitution | `rule.studio-substitution.674bc70d924bf5707bcf` |

원장 전체를 runtime registry로 import하지 않는다. 이미 내려진 결정에서 identity를 만들고 exact
`ruleId`·owner·candidate evidence anchor가 존재할 때만 relation과 evidence status를 붙인다.

### 3.4 hash

- `layoutHash`: source·document·layout name·layout metric·provenance만 포함
- `normalizedHash`: backend capability와 paint 결과까지 포함
- object key는 W1과 같은 canonical JSON으로 정렬
- capability·failure·known limitation은 code-unit 순서로 stable dedupe/sort
- fallback `candidates` 배열은 정책 순서이므로 보존
- hash 자신, timestamp, elapsed time, stack은 projection에서 제외

`localeCompare`는 ICU와 locale에 따라 결과가 달라질 수 있으므로 사용하지 않고 W1 object key와 같은
결정론적 code-unit 비교를 사용했다.

### 3.5 공개 fixture

Stage 4 후보는 이미 repository에 추적된 같은 문서의 HWP/HWPX 쌍이다.

| format | path | size | SHA-256 |
| --- | --- | ---: | --- |
| HWP | `samples/3-10월_교육_통합_2022.hwp` | 2,950,656 | `5197fc670bb1783edd859a0fdfcddddd2a421aeb819cabae5887f47827942d55` |
| HWPX | `samples/3-10월_교육_통합_2022.hwpx` | 2,677,940 | `f37246219339c8900b19f3da65afc105fbf6823afe5f2bb357a7e4c2b14b0314` |

검사는 repository-relative `samples/` path, Git 추적 상태, size, digest, format과
`privateCorpus: false`를 전부 확인한다. `/home/edward/mygithub/hwpsamples`와 private 10k corpus는 읽거나
manifest에 넣지 않았다.

## 4. 검증 결과

### 4.1 신규 계약

```bash
node --test scripts/tests/font_decision_trace_contract.test.mjs
```

결과: **11 passed, 0 failed**.

검증 항목:

- candidate/rule ID golden 4건과 exact ledger anchor
- ledger miss에서 guessed ID 금지
- 기본·최대 상한과 invalid limit 거부
- v1 봉투, count, truncation, invalid page와 silent clamp
- W1 source digest drift와 repository-relative 경로만 반환
- unordered diagnostic 정규화와 fallback chain 순서 보존
- backend 상태와 무관한 portable layout hash
- 절대 home path, token과 error stack 거부
- HWP/HWPX fixture 추적·digest·private boundary
- schema enum과 4,096 hard limit

```bash
node scripts/font_decision_trace_contract.mjs check
```

결과: `font decision trace Stage 1 contracts: ok`.

### 4.2 W1 회귀

```bash
node --test scripts/tests/font_rule_ledger.test.mjs
```

결과: **10 passed, 0 failed**. 30개 source boundary, 600 metric entry, 401 unique metric name과 lookup
projection이 유지됐다.

승인된 계획 초안에는 존재하지 않는 `font_rule_ledger.mjs check` subcommand가 적혀 있었다. 실제 CLI는
`boundary|collect|baseline`만 지원하므로 이를 성공으로 가장하지 않았다. 원장의 schema·boundary·baseline
회귀를 수행하는 위 test target으로 계획의 명령을 정정했다.

### 4.3 schema·문서·diff

```bash
node --input-type=module # Ajv 2020 strict compile
python3 scripts/check_markdown_links.py \
  mydocs/orders/20260817.md \
  mydocs/plans/task_m100_4961.md \
  mydocs/tech/investigations/issue-4961/README.md
node --check scripts/font_decision_trace_contract.mjs
node --check scripts/tests/font_decision_trace_contract.test.mjs
git diff --check
```

- JSON Schema Draft 2020-12 strict compile: 통과
- Markdown 3개, 내부 상대 링크: 이상 없음
- 두 `.mjs` syntax: 통과
- whitespace diff: 통과

## 5. 보호 불변식 판정

| 항목 | Stage 1 판정 |
| --- | --- |
| product font selection·metric·paint | 변경 없음 |
| native/WASM·Studio output | production source 변경 없음 |
| W1 ledger runtime authority화 | 하지 않음 |
| unknown 추정 | 하지 않음 |
| backend capability 혼합 | schema에서 분리 |
| version branching | 없음 |
| private corpus·font bytes | 포함하지 않음 |
| host path·token·stack | failure test로 차단 |

## 6. 잔여 위험과 Stage 2 인계

- 현재 schema와 JavaScript contract는 구현 전 규약이다. Rust DTO와 WASM JSON이 아직 이 schema를
  생성하지 않는다.
- `ResolvedCharStyle`이 잃는 document face·`altType`·`substFont`를 source 좌표로 복구하는 구현이
  필요하다.
- exact·bold-only·name-first와 문자 glyph miss를 설명하는 Rust metric decision이 아직 없다.
- 중복 폭 사다리를 하나의 `CharWidthDecision`으로 합칠 때 0-delta 전수·대표 비교가 필요하다.
- Stage 2에서 해당 source가 바뀌면 W1 historical digest drift가 발생한다. 이를 원장 수정으로 숨기지
  않고 trace에 `ledgerSourceDrift`로 명시한 뒤 현재 candidate identity join을 별도로 검증해야 한다.

Stage 2는 Rust layout decision trace만 수행한다. Studio backend 보강은 Stage 3 승인 전에는 시작하지
않는다.

## 7. 다음 승인 지점

Stage 1 변경은 이 보고서를 포함한 별도 commit으로 고정한다. 메인테이너가 Stage 2 진행을 승인하면
style·metric·문자 폭의 읽기 전용 decision과 native/WASM query 구현을 시작한다. remote push와 PR 생성은
여전히 별도 승인 대상이다.
