# Task M100 #4961 Stage 2 — Rust layout decision trace

- **Issue**: [#4961](https://github.com/edwardkim/rhwp/issues/4961)
- **상위 추적**: [#4960](https://github.com/edwardkim/rhwp/issues/4960)
- **계획**: [`task_m100_4961.md`](../../plans/archives/task_m100_4961.md)
- **브랜치**: `local/task4961-font-decision-trace`
- **기준**: Stage 1 commit `86be469b7375168b30f3f0c7c26989e2cb4a2f8e`
- **날짜**: 2026-08-17 KST
- **단계 상태**: Stage 2 기술 완료, Stage 3 메인테이너 승인 대기

## 1. 결과

Stage 2는 기존 font 선택과 조판 폭을 바꾸지 않고, 이미 내려진 layout 결정을 문자별로 설명하는
읽기 전용 query를 구현했다.

| 경계 | 결과 |
| --- | --- |
| style | 원문 face, 7개 언어 slot, `altType`, embedded, document `substFont`, substitution boundary |
| metric | requested/alias face, exact·bold-only·name-first rung, entry index, bold fallback |
| width | KoPub, metric, space·punctuation overlay, metric miss, 세 heuristic, 장평·자간·추가 advance |
| query | page source 좌표와 DocInfo를 결합한 기본 1,024자·최대 4,096자 bounded JSON |
| WASM | `getFontDecisionTrace(pageNum, optionsJson)` — native query를 그대로 위임 |
| hash | W1 canonical JSON 규약의 `layoutHash`와 backend 포함 `normalizedHash` |

Studio와 paint backend 관측은 구현하지 않았다. Stage 2 record의 native·Canvas2D·CanvasKit은 모두
명시적 `unsupported`이며 Stage 3 보강 사유를 가진다.

## 2. 보호 불변식

### 2.1 기존 projection 유지

- `lookup_font_name`은 새 decision의 CSS chain을 join한 값만 반환한다.
- `find_metric`은 새 decision에서 기존 `MetricMatch`만 반환한다.
- 메트릭 색인은 entry index와 rung을 추가로 보존하지만 실제 metric pointer와 `bold_fallback`은 종전
  선형 스캔과 전체 metric name·alias·미등록명 × bold/italic 4조합에서 동일하다.
- `estimate_text_width`, `estimate_text_width_unrounded`, `compute_char_positions`는 하나의
  `CharWidthDecision`을 사용하며 기존 탭 분기는 유지한다.
- trace 호출은 font load, 권한 요청, repaint, document mutation과 backend 선택을 수행하지 않는다.

### 2.2 W1 historical drift

W1 `font_rule_candidates.json`, baseline과 ledger는 수정하지 않았다. Stage 2 구현으로 다음 네 source의
digest가 달라졌으며 contract test가 이 목록 외 drift를 거부한다.

- `src/renderer/font_metrics_data.rs`
- `src/renderer/layout/text_measurement.rs`
- `src/renderer/mod.rs`
- `src/renderer/style_resolver.rs`

candidate identity와 `ruleId` join은 유지하되 trace의 Rust provenance와 최상위 reason에
`ledgerSourceDrift`를 기록한다. document `substFont`는 W1 candidate가 아니므로 guessed rule을 만들지
않고 `ruleId: null`, `ledgerRuleMissing`으로 끝낸다.

## 3. 검증 결과

### 3.1 release-test

```bash
cargo test --profile release-test --lib decision -- --nocapture
cargo test --profile release-test --lib font_metrics -- --nocapture
cargo test --profile release-test --lib text_measurement -- --nocapture
```

- decision: **7 passed**
- font metrics: **10 passed**
- 기존 text measurement: **39 passed**

검증 범위에는 W1 identity golden, exact alias/rung/entry, 7개 language slot, document `substFont`,
KoPub·metric overlay·metric/CJK/narrow/generic heuristic, 세 기존 폭 projection의 동일성, invalid
page/options와 deterministic bounded public-fixture trace가 포함된다.
공개 trace의 모든 non-null `ruleId`는 W1 ledger의 owner·relation type·evidence status·candidate anchor와
exact join되는지 별도로 확인했다.

### 3.2 W1·trace 계약

```bash
node --test scripts/tests/font_decision_trace_contract.test.mjs
node --test scripts/tests/font_rule_ledger.test.mjs
node scripts/font_decision_trace_contract.mjs check
```

- trace contract: **11 passed**
- W1 ledger: **10 passed**
- repository contract check: `font decision trace Stage 1 contracts: ok`

### 3.3 native/WASM parity

```bash
cargo check --target wasm32-unknown-unknown --lib
docker compose --env-file .env.docker run --rm wasm
```

표준 Docker optimized WASM build를 통과했다. repository-tracked `samples/task-001.hwp`의 page 0,
`maxCharacters=8`을 native `DocumentCore`와 WASM `HwpDocument.getFontDecisionTrace`에서 각각 호출해
전체 JSON을 byte 비교했다.

```text
records=8, status=truncated, byteLength=25548
layoutHash=aa11ab8f9186471e2b4b10e3e135135b6f0344adc19450a53019c85c4b327676
normalizedHash=4f852f753b8899499cb23d78967ac4b7ac966762fd05f269ce7ed2bb918550f2
native JSON == WASM JSON
```

WASM 결과를 Stage 1 `validateTraceEnvelope`에 다시 넣어 envelope·count·hash 재계산도 통과했다.
`pkg/`는 build 산출물이며 Git 추적 변경에 포함하지 않는다.

### 3.4 기본 검사

```bash
cargo check --lib
cargo clippy --lib -- -D warnings
cargo fmt --all -- --check
git diff --check
```

모두 통과했다.

## 4. 잔여 범위

- native Skia 실제 face/glyph, Canvas2D local/web/generic, CanvasKit SFNT 후보는 Stage 3 범위다.
- Stage 2는 source IR join 실패를 추정으로 메우지 않고 record `source.status=unavailable`과
  `sourceMappingMismatch`로 남긴다.
- Stage 4의 공개 HWP/HWPX 전체 fixture와 backend normalized trace 비교는 아직 수행하지 않았다.
- Stage 5의 full Rust·Studio·native-skia·시각/출력 감사도 아직 수행하지 않았다.

## 5. 다음 승인 지점

Stage 2 변경은 이 보고서를 포함한 별도 commit으로 고정한다. 메인테이너가 Stage 3 진행을 승인하면
backend capability와 paint trace를 구현한다. remote push와 PR 생성은 별도 승인 대상이다.
