# Task M100 #4962 W3 Stage 3-F — 공개 분류 fixture

- **Issue**: [#4962](https://github.com/edwardkim/rhwp/issues/4962)
- **상위 추적**: [#4960](https://github.com/edwardkim/rhwp/issues/4960)
- **계획**: [`task_m100_4962.md`](../plans/task_m100_4962.md)
- **선행 hard gate**: [`task_m100_4962_stage3_isolation.md`](task_m100_4962_stage3_isolation.md)
- **구현 commit**: `c2e9cb15f`, `8203d7832`
- **날짜**: 2026-08-21 KST
- **단계 상태**: Stage 3의 두 번째 hard gate 완료, private pilot 미착수

## 1. 승인 범위와 결론

이번 절편은 저장소에 이미 있는 공개 HWP/HWPX와 공개 blank template에서 파생한 메모리상 문서만
사용해 W3 coverage 분류와 non-applicable 경계를 고정했다. private 10k corpus는 열거나 supervisor에
전달하지 않았고, pilot·전수 계측·원격 push도 수행하지 않았다.

현재 실제 결정 경로에서 도달 가능한 6개 양성 category와 5개 non-applicable width source를 실제 W2/W3
결정으로 재현했다. 일곱 번째 `identity-alias-hit`은 W1의 verified identity alias가 0이므로 실제 문서에서
양성으로 만들 수 없다. 이를 위해 존재하지 않는 alias나 fallback을 추가하면 측정기가 대상을 바꾸는
순환 오류가 된다. 따라서 해당 category는 0 golden과 decision-table contract로 유지한다.

## 2. 공개 golden

정본은
[`font_metric_coverage_public_fixtures.json`](../tech/investigations/issue-4962/font_metric_coverage_public_fixtures.json)이다.
문서 경로는 저장소의 공개 sample만 가리키며 aggregate에 경로·파일명·문자·raw trace를 넣지 않는다.

| 공개 문서 | 형식 | layout | coverage | not applicable | 핵심 분류 |
| --- | --- | ---: | ---: | ---: | --- |
| `3-10월_교육_통합_2022.hwp` | HWP | 11,094 | 10,868 | 226 | exact 10,521, char miss 307, face miss 40 |
| 같은 공개 문서 | HWPX | 11,094 | 10,868 | 226 | HWP와 동일 |
| `156636617_240617 … 현황(확정치).hwp` | HWP | 16,872 | 16,685 | 187 | exact 14,833, overlay 1,188, char miss 642, heuristic 22 |

HWP/HWPX 쌍은 counts, categories, joins, legacy usage와 width-source 문자 합계가 같다. 다만 두 container의
문단·run 경계 표현이 달라 `decisionUsage`의 `paragraphCount`·`runCount`까지 byte-equal하다고 주장하지
않는다. format을 포함하는 aggregate와 legacy hash도 의도적으로 서로 다르다.

## 3. 분류 도달성

공개 blank template에 새 font face와 한 문단을 메모리상으로 추가해 다음 실제 결정을 고정했다. 생성
문서는 파일로 저장하거나 fixture binary로 커밋하지 않는다.

| category | 공개 입력 | 실제 width source | 판정 |
| --- | --- | --- | --- |
| `measured-overlay` | KoPub돋움체 Light + `가` | `kopubTable` | 양성 도달 |
| `metric-surrogate` | 본한글 + `가` | `embeddedMetric` | 양성 도달 |
| `exact-hit` | 함초롬바탕 + `가` | `embeddedMetric` | 양성 도달 |
| `char-miss` | 함초롬바탕 + emoji | `heuristicHalfwidth` | 양성 도달 |
| `face-miss` | 존재하지 않는 face + `A` | `heuristicHalfwidth` | 양성 도달 |
| `heuristic` | 존재하지 않는 face + 아래아 | `areaDotFallback` | 양성 도달 |
| `identity-alias-hit` | 해당 없음 | 해당 없음 | W1 verified relation 0, contract-only |

각 양성 문서는 W3 category count 1을 확인하고, 같은 `DocumentCore`의 W2 trace와 W3
`decisionUsage`에서 width-source count가 같은지 대조한다. 모든 경우 `identity-alias-hit=0`도 함께
고정한다.

## 4. non-applicable 경계

한 공개 blank 파생 문서에 분해 자모, inline object placeholder, HWP PUA filler, figure space와 tab을 넣어
다음 폭 출처를 실제로 통과시켰다.

| width source | 문자 수 | coverage 포함 여부 |
| --- | ---: | --- |
| `clusterContinuation` | 2 | 제외 |
| `inlineObjectPlaceholder` | 1 | 제외 |
| `hwpPuaFiller` | 1 | 제외 |
| `figureSpace` | 1 | 제외 |
| `tabAdvance` | 1 | 제외 |

분해 자모의 첫 글자는 cluster continuation이 아니라 실제 `char-miss + heuristicFullwidth`이므로 coverage
1건이다. 결과는 layout 7 = coverage 1 + not applicable 6이며, W2와 W3의 width-source count가 같다.

## 5. 검증 결과

신규 integration source는 일반 checkout의 `tests/cases/`에만 두었다. detached review worktree에서
generated suite를 준비했고 W3는 `regression_suite_024`, 기존 W2는 `regression_suite_011`에 배정됐다.

```bash
cargo test --test regression_suite_024 issue_4962_font_metric_coverage -- --nocapture
```

결과: **8 passed, 0 failed**. 공개 실문서 golden, HWP/HWPX portable fields, 6개 양성 category,
5개 non-applicable source, W2/W3 동등성, 결정성·privacy·분모·자원 보호가 통과했다.

```bash
cargo test --test regression_suite_011 issue_4961_font_decision_trace -- --nocapture
```

결과: **4 passed, 0 failed**. 기존 W2 공개 HWP/HWPX·fallback·상한 계약이 유지됐다.

추가 결과:

- W3 contract·supervisor: **17 passed, 0 failed**
- generated manifest: 825 sources, 4,012 static test attrs, 32 suites + 9 exceptions 대사 통과
- Rust unit tier: 4,225 tests / 299 modules, 정책 통과
- `cargo fmt --all -- --check`: 통과
- `git diff --check`: 통과

검증 뒤 detached review worktree와 generated suite를 제거했다. source checkout에는 integration 원본과
공개 JSON 정본만 남겼다.

## 6. 다음 승인 게이트

다음 절편은 private 문서를 실행하는 단계가 아니다. 기존 local risk ranking과 format·usage aggregate만
읽어 deterministic pilot cohort의 선택 규칙, 규모, 예상 범위와 중단 기준을 먼저 문서화한다. 기존
자료로 선정할 수 없는 차원만 명시하며 전수 재수집을 선택 절차에 섞지 않는다.

선정 보고를 메인테이너가 승인한 뒤에만 Stage 3 private 소규모 pilot을 문서별 격리 supervisor로 두 번
실행한다. 현재 승인으로는 private corpus 접근, pilot 실행, 10k 전수 delta pass, 원격 push와 PR을
진행하지 않는다.
