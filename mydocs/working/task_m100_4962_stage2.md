# Task M100 #4962 W3 Stage 2 — decision delta collector

- **Issue**: [#4962](https://github.com/edwardkim/rhwp/issues/4962)
- **상위 추적**: [#4960](https://github.com/edwardkim/rhwp/issues/4960)
- **계획**: [`task_m100_4962.md`](../plans/archives/task_m100_4962.md)
- **브랜치**: `task_m100_4962`
- **착수 기준**: `upstream/devel` `fb434269eea237cc12053914560a2dbaf16270bf`
- **Stage 1 기준**: `63b7f4ec2`
- **구현 commit**: `9d7a0ef76`, `126274195`
- **날짜**: 2026-08-21 KST
- **단계 상태**: Stage 2 기능 완료, [Stage 2-S 보안 보강](task_m100_4962_stage2_security.md)으로 보호 계약 갱신

## 1. 결론

commit `631287d47`의 POC 재귀 walker와 v2 usage key를 보존하면서 실제 renderer의
`CharWidthDecision`을 결합하는 읽기 전용 delta collector를 구현했다. 이 collector는 page나 문자 수
상한 없이 source paragraph를 streaming 집계하고, 문자·code point·파일명·경로·문서별 hash·raw trace를
출력하지 않는다.

공개 fixture에서 같은 입력의 JSON과 두 SHA-256이 반복 일치했고, 5,000자를 한 문단에 추가한 경우에도
`truncatedCharacters=0`과 모든 분모 대사가 유지됐다. collector 호출 전후 W2 trace가 byte-equal이며,
기존 W2 공개 HWP/HWPX·fallback profile 4건도 그대로 통과했다. 제품의 metric 값, lookup 순서,
fallback target, paint face, 기본 render 호출 경로와 공개 CLI·WASM·npm surface는 변경하지 않았다.

이번 단계에서는 private 10k corpus를 열거나 새 통계를 만들지 않았다. 기존 POC 전체 aggregate는 Stage 1
기준선으로 계속 재사용하며, Stage 3의 공개 경계 fixture와 비공개 소규모 pilot은 별도 승인 대상이다.

## 2. 구현 경계

| 경로 | 책임 |
| --- | --- |
| `src/document_core/queries/font_metric_coverage.rs` | 재귀 source walker, 실제 문자 decision 결합, streaming aggregate, 분류·대사·hash |
| `src/document_core/queries/font_decision.rs` | W2와 공유하는 언어 slot·metric alias relation helper |
| `src/renderer/font_decision.rs` | 기존 canonical hash 구현의 crate 내부 재사용과 aggregate volatile field 제외 |
| `tests/cases/issue_4962_font_metric_coverage.rs` | 공개 fixture 결정성·분모·privacy·long-page·W2 불변 계약 |

진입점은 숨김 처리한 native read-only query
`DocumentCore::get_font_metric_coverage_analysis_native(options_json)`다. Stage 2-S에서 supervisor용 취소
companion을 추가했지만 별도 adapter나 CLI command는 없으므로 기본 문서 열기·편집·조판·paint에는
collector 비용이 발생하지 않는다.

> **보안 정정:** 이 문서의 최초 완료 뒤 long-page 무상한을 자원 무제한으로 오해할 수 있는 결함을
> 발견했다. 현재 정본은 Stage 2-S의 명시적 자원 예산·전체 실패 계약이며, 아래 최초 검증 결과는
> 기능 기준선으로 보존한다.

## 3. POC 재사용과 delta

### 3.1 그대로 유지한 projection

- 본문, 표 셀, 글상자, 캡션, 머리말·꼬리말, 각주·미주, 마스터 페이지, 메모, 숨은 설명 재귀 순회
- 원본 font face, language slot, bold·italic, 장평, 자간, kerning
- context bit, alignment, stored LineSeg 존재 여부
- document·paragraph·run·character count
- usage row의 `charCount` 내림차순, font 오름차순 정렬

`legacyUsage`는 위 필드만 가진 POC v2 호환 projection이다. `legacyProjectionHash`는 format, paragraph,
joined character와 이 row를 canonical SHA-256으로 고정한다. invalid CharShape·font 참조는 성공 usage로
꾸미지 않고 `excluded`에만 들어간다.

### 3.2 새 decision delta

`decisionUsage`는 같은 legacy key에 다음 실제 결정을 붙인다.

- normalized layout face, document `substFont`, `altType`, 실제 layout family
- metric requested/resolved face, entry index, `matchKind`, `characterMatch`
- 최종 `widthSource`, W1 relation type·evidence status
- 상호배타 coverage category와 `sourceJoinStatus=joined`

W2와 W3는 둘 다 `trace_char_width_decisions`를 호출한다. metric alias가 surrogate인지 판단하는 조건도
W2 helper 한 곳으로 추출해 별도 lookup 사다리를 만들지 않았다.

## 4. 분류와 fail-closed

Stage 1의 우선순위를 그대로 적용한다.

1. measured overlay
2. verified identity alias hit
3. metric surrogate
4. exact hit
5. character miss
6. face miss
7. explicit heuristic

cluster continuation, inline-object placeholder, HWP PUA filler, figure space와 tab advance는 coverage 성공이
아닌 `notApplicableCharacters`다. 내부 중간 상태 `metricMiss`·`metricCharacterMiss`, 알 수 없는 새
`widthSource`, metric entry와 character match가 모순되는 조합은 heuristic에 흡수하지 않고 query를
실패시킨다. W1에 verified identity alias가 없으므로 `identity-alias-hit`은 공개 fixture에서도 0이다.

## 5. 독립 분모와 hash

collector가 직렬화 전에 다음을 다시 계산해 하나라도 다르면 실패한다.

```text
layoutCharacters
  = coverageCharacters + notApplicableCharacters + excludedCharacters

coverageCharacters
  = sum(seven categories)

layoutCharacters
  = joined + layoutOnly + excluded

sum(legacyUsage.charCount)
  = sum(decisionUsage.charCount) = joined
```

source walker 방식이므로 이번 구현의 `layoutOnly`는 0이다. parser는 성공한 `DocumentCore`만 입력받으므로
문서 결과는 attempted 1, success 1이며 모든 failure key를 0으로 명시한다. backend snapshot을 요청하지
않으므로 backend 네 상태도 생략하지 않고 0으로 기록한다.

`aggregateHash`는 hash 자신과 timestamp·elapsed/duration류를 제외한다. 두 usage map은 정렬된 key로
집계되고 POC projection도 기존 정렬을 복원했으므로 실행 순서에 영향을 받지 않는다.

## 6. 검증 결과

### 6.1 신규 integration source

일반 checkout에는 generated suite를 만들지 않았다. 구현 commit의 detached review worktree에서만
`node scripts/rust-test-suite-manifest.mjs --prepare`를 실행해
`regression_suite_019`에 배정한 뒤 다음을 실행했다.

```bash
cargo test --test regression_suite_019 issue_4962_font_metric_coverage -- --nocapture
```

최종 정렬 보정 commit에서 재실행한 결과: **3 passed, 0 failed**, test 실행 0.20초.

- 같은 공개 HWP의 JSON·legacy hash·aggregate hash 반복 일치
- 7개 category, layout·coverage·join·document·backend 분모 대사
- forbidden key와 Linux·macOS·Windows home path 비노출
- 한 문단에 5,000자 추가 후에도 전수 집계, truncation 0
- collector 호출 전후 기존 W2 trace byte equality

검증 뒤 review worktree와 그 안의 generated suite를 제거했으며 source checkout에는
`tests/cases/issue_4962_font_metric_coverage.rs`만 남겼다.

### 6.2 W2 회귀

```bash
cargo test --test regression_suite_003 issue_4961_font_decision_trace -- --nocapture
```

결과: **4 passed, 0 failed**. 공개 HWP/HWPX parity, substFont feature detection, 상한 fail-closed와 W2 hash
계약이 모두 유지됐다.

### 6.3 W1·W2·W3 focused 계약

```bash
node --test scripts/tests/font_metric_coverage_contract.test.mjs
```

결과: **10 passed, 0 failed**.

```bash
node --test \
  scripts/tests/font_rule_ledger.test.mjs \
  scripts/tests/font_rule_ledger_evidence.test.mjs \
  scripts/tests/font_decision_trace_contract.test.mjs \
  scripts/tests/font_decision_trace_e2e.test.mjs
```

결과: **33 passed, 0 failed**.

### 6.4 빌드·정책

- `cargo check --lib`: 통과
- `cargo check --tests`: 통과
- `node scripts/rust-unit-test-tiers.mjs --check`: 4,225 tests, 정책 통과
- `cargo fmt --all -- --check`: 통과
- `git diff --check`: 통과

처음에는 분류기 격리 단위 테스트 2개도 통과했지만 source-side test 총량 가드가
`4227 > 4225`로 차단했다. 해당 테스트를 source에서 제거하고 공개 integration source로 책임을 옮긴 뒤
4,225 기준을 회복했다. 가드를 우회하거나 파생 inventory를 커밋하지 않았다.

## 7. 보호 불변식 판정

| 항목 | Stage 2 판정 |
| --- | --- |
| 기존 10k POC | 재실행·덮어쓰기 없음 |
| POC walker·usage key·정렬 | 유지 |
| metric DB·fallback·paint | 변경 없음 |
| default renderer 비용 | opt-in query 밖에서는 0-delta |
| W2 trace | 기존 4건과 호출 전후 byte equality 통과 |
| page 문자 상한 | 없음, 공개 5,000자 통과 |
| raw character·path·document identity | aggregate 비포함 |
| generated integration suite | review worktree에서만 생성 후 제거 |
| CLI·WASM·npm 공개 surface | 추가 없음 |

## 8. 다음 승인 게이트

Stage 3는 공개 HWP/HWPX로 일곱 category와 non-applicable 경계를 더 세분화하고, 기존 POC의 local risk
ranking으로 비공개 소규모 cohort만 선정해 처리량·peak memory·실패율·전수 예상 시간을 측정하는
단계다. 이 단계 전에는 private corpus pilot도 실행하지 않는다.

따라서 현재 완료 범위는 **collector와 공개 long-page 보호 계약**까지다. Stage 3 진행, private corpus
접근, 전수 10k delta pass, 원격 push와 PR은 이번 승인에 포함되지 않는다.
