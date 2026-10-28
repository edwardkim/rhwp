# 작업 보고 — Task M100 #4961 Stage 5

- **이슈**: [#4961](https://github.com/edwardkim/rhwp/issues/4961)
- **상위 추적**: [#4960](https://github.com/edwardkim/rhwp/issues/4960)
- **후속**: [#4962](https://github.com/edwardkim/rhwp/issues/4962)
- **브랜치**: `local/task4961-font-decision-trace`
- **Stage 5 시작 HEAD**: `43a726449c4349949704cdcad64ff0b1eb2d058c`
- **기준**: `upstream/devel@418e5b191d23cf0618ce99f0cfec332c19ac1bc2`
- **작성일**: 2026-08-17 KST

## 1. 결론

Stage 5 종료 게이트를 통과했다. Font Decision Trace는 opt-in query로만 동작하며 기존 font 선택,
renderer 출력과 serialization을 바꾸지 않는다. W1 원장 join, trace 계약, 공개 HWP/HWPX E2E,
전체 Rust·Studio·editor·fresh WASM, native Skia와 native/WASM SVG parity가 모두 녹색이다.

headless Chrome과 Windows 호스트 Chrome CDP에서 실제 `@rhwp/editor` RPC를 호출했다. trace 전후의
SVG 문자열과 HWP bytes는 각각 동일했고, trace 호출 구간의 `fetch`, `FontFace.load`, Local Font Access
호출은 모두 0건이었다. 따라서 trace가 font load·권한 요청·repaint·serialization mutation을 일으키지
않는다는 브라우저 실환경 근거도 확보했다.

PR self-review에서 native 단독 query가 준비된 renderer의 custom·bundled inventory 없이 새 observer를
만드는 결함을 확인했다. Stage 6은 native 완료 판정을 준비된 `SkiaLayerRenderer` snapshot 경로로 한정하고,
snapshot 없는 `DocumentCore` query를 `nativeRendererSnapshotRequired`로 닫았다. 이 보정은 Stage 5의
브라우저/WASM 결과와 renderer 출력 0-delta 판정을 바꾸지 않는다.

## 2. Stage 5 변경

### 2.1 브라우저 0-delta E2E 보강

`rhwp-studio/e2e/embed-transport.test.mjs`에 다음을 추가했다.

- 실제 `getFontDecisionTrace(0, {maxCharacters: 64})` 호출
- `schemaVersion: 1`, `status: truncated`, 64 records와 Studio backend snapshot 판정
- 호출 전후 `getPageSvg(0)` 문자열과 `exportHwp()` bytes의 완전 동일성
- 호출 구간의 `fetch`, `FontFace.load`, `queryLocalFonts` 호출 0건
- 기존 load/export, renderer diagnostics, forged peer 거부와 legacy transport 계약 유지

### 2.2 문서

- `npm/editor/README.md`: public SDK 사용법, 상한, hash와 Canvas2D 관찰 한계
- `mydocs/tech/investigations/issue-4961/README.md`: Stage 5 증거와 #4962 인계
- `mydocs/tech/wasm_agent_surface/browser_bridge.md`: 실제 브라우저 read-only 계약 검증
- `mydocs/report/task_m100_4961_report.md`: FI·완료 조건 전항 판정과 알려진 한계
- `mydocs/orders/20260817.md`: Stage 3~5 완료 상태
- private boundary 최종 감사에서 Stage 1 보고서와 negative test에 남은 메인테이너 로컬 절대 경로
  문자열을 제거했다. 차단 계약은 사용자·저장소를 식별하지 않는 합성 path로 그대로 검증한다.

제품 resolver, metric DB, fallback target, font asset와 workflow는 Stage 5에서 변경하지 않았다.

## 3. 검증 결과

### 3.1 W1·trace 집중 계약

```bash
node --test scripts/tests/font_rule_ledger.test.mjs
node --test scripts/tests/font_decision_trace_contract.test.mjs
node --test scripts/tests/font_decision_trace_e2e.test.mjs
cargo test --profile release-test --lib decision -- --nocapture
cargo test --profile release-test --lib font_metrics -- --nocapture
cargo test --profile release-test --lib text_measurement -- --nocapture
cargo test --profile release-test --features native-skia --lib stage4_public -- --nocapture
```

결과:

- W1 ledger **10 passed**
- trace contract **12 passed**
- 공개 WASM E2E **3 passed**
- Rust decision **9 passed**, font metric **10 passed**, text measurement **39 passed**
- native Skia 공개 Stage 4 **2 passed**

Studio trace·embed focused 22건과 production build, editor transport·diagnostics 13건도 통과했다.

### 3.2 전체 로컬 gate

로컬 검증 4.3.2의 대형 복합 변경 순서를 같은 checkout에서 직렬 실행했다.

| gate | 결과 |
| --- | --- |
| `cargo build --release` | 통과 |
| `cargo test --release --lib` | 4,076 passed, 13 ignored |
| `cargo nextest run ... --tests --test-threads 12 --no-fail-fast` | 6,523 passed, 38 skipped |
| native Skia lib | 59 passed |
| missing picture integration | 2 passed |
| direct PDF integration | 4 passed |
| `cargo fmt --check`, `git diff --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| `cargo test --doc` | 8 passed, 2 ignored |
| Studio `npx tsc --noEmit` | 통과 |
| Studio 전체 `npm test` | 940 passed, 1 skipped |
| editor `npm test` | 24 passed |
| generated WASM/editor embed contract | 3 passed |
| editor declaration compile, package dry-run | 통과 |
| 표준 Docker optimized WASM | 통과 |

nextest 설치본은 0.9.137이고 저장소 권장은 0.9.140이라는 비차단 경고가 있었다. 실제 6,523건은 모두
통과했다. editor declaration compile과 package dry-run을 처음 저장소 루트에서 실행해 manifest를 찾지
못한 절차 오류가 한 번 있었고, 문서가 지정한 `rhwp-studio`와 `npm/editor`에서 각각 재실행해 통과했다.

### 3.3 native/WASM 공개 SVG parity

fresh `target/release/rhwp`와 Docker `pkg`를 사용해 Stage 4 manifest의 공개 HWP/HWPX 6개 page 0을
`scripts/svg_native_wasm_diff.mjs`로 비교했다.

```text
문서 6개: match 6
비교 page: 6
byte mismatch: 0
```

nextest에 포함된 기존 SVG snapshot 9건도 전부 통과했다. 이는 portable target parity와 기존 golden
출력 0-delta를 서로 다른 축에서 고정한다.

### 3.4 브라우저 실환경

- Vite: 메인테이너가 실행한 `http://127.0.0.1:7700`, HTTP 200
- headless Chrome embed E2E: 통과
- Windows 호스트 CDP: Chrome 151.0.7922.138, protocol 1.3, `http://localhost:19222`
- 호스트 Chrome embed E2E: 통과

두 환경의 관측값은 같았다.

| 관측 | 결과 |
| --- | --- |
| trace | v1, `truncated`, 64 records |
| Canvas2D | `complete` |
| CanvasKit | `notObserved` |
| trace 전후 SVG | byte-identical |
| trace 전후 HWP serialization | byte-identical |
| trace 중 fetch | 0 |
| trace 중 `FontFace.load` | 0 |
| trace 중 Local Font Access | 0 |

`CanvasKit: notObserved`는 실패가 아니다. 현재 renderer가 Canvas2D인 세션에서 준비된 CanvasKit plan은
trace에 보존하되 실제 glyph 선택을 관찰했다고 승격하지 않는 fail-closed 결과다.

## 4. FI-01~FI-14 disposition

| ID | 판정 | Stage 5 근거 |
| --- | --- | --- |
| FI-01 | 통과 | fresh native/WASM 공개 SVG 6/6 byte parity, 기존 SVG snapshot 9건 |
| FI-02 | 통과 | trace 전후 SVG 동일, font load·권한 호출 0; local 상태는 paint evidence만 보강 |
| FI-03 | 통과 | schema가 `layoutMetric`과 backend별 `paint`를 분리하고 한 record에서 설명 |
| FI-04 | 통과 | identity alias를 새로 만들지 않았고 W1 relation/evidence를 읽기 전용으로 유지 |
| FI-05 | 통과 | document substitution, style/metric relation, paint supply, heuristic을 별도 field/reason으로 보존 |
| FI-06 | 통과 | CSS chain, local enumeration, web supply와 CanvasKit SFNT/typeface를 별도 capability로 판정 |
| FI-07 | 통과 | W1 baseline 전수 lookup, metric decision과 기존 font metric 회귀 통과 |
| FI-08 | 보존 | exact·missing fixture를 섞지 않으며 검증 oracle이 없으면 `notProvided`; W5 값을 추정하지 않음 |
| FI-09 | 보존 | 저장 LineSeg를 fresh layout 정답으로 승격하지 않으며 page count만으로 승인하지 않음 |
| FI-10 | 통과 | 공개 fixture만 사용; 실제 private 원문·식별 목록·host path 없음, 합성 차단 vector만 유지 |
| FI-11 | 통과 | metric 생성물·overlay를 재생성하지 않았고 lookup/출력 등가를 먼저 검증 |
| FI-12 | 통과 | 동일 객체 HWP/HWPX hash parity와 실제 `substFont` 상태 차이를 feature detection으로 판정 |
| FI-13 | 통과 | webfont supply evidence와 layout metric compatibility를 별도 층으로 유지 |
| FI-14 | 보존·인계 | ratio·letter spacing·추가 advance를 문자 decision에 기록; 실제 10k 위험 집계는 #4962 범위 |

FI-08·FI-09·FI-14는 W2가 oracle·layout 정책을 바꾸지 않는다는 의미의 보존 판정이다. 해당 후속 계측을
수행했다고 과장하지 않는다.

## 5. #4961 완료 조건 disposition

| 완료 조건 | 판정 |
| --- | --- |
| 공개 HWP/HWPX document face → 최종 reason 계보 | 통과 |
| language slot·`altType`·`substFont`·metric hit/miss 보존 | 통과 |
| Rust/native/Canvas2D/CanvasKit 차이를 한 schema로 설명 | 통과 |
| W1 candidate evidence와 `ruleId` 기계 join | 통과 |
| portable/normalized hash 결정론 | 통과 |
| 상한·unsupported·join 실패 fail-closed | 통과 |
| trace 미호출/호출 전후 font 계약·renderer·serialization·초기화 0-delta | 통과 |
| private corpus·허가되지 않은 font bytes 비포함 | 통과 |
| API 문서·최종 보고·W3/W4 인계 | 통과 |

## 6. 알려진 한계와 후속 인계

- Canvas2D는 실제 glyph face를 공개하지 않으므로 CSS chain·supply만 `notObserved`로 기록한다.
- CanvasKit이 현재 renderer가 아니면 SFNT/typeface plan은 `planned` 또는 `notObserved`이며 실제 선택으로
  승격하지 않는다.
- WASM 단독 query에서 native Skia와 Studio snapshot은 `unsupported`다.
- native build의 `DocumentCore` 단독 query도 준비된 renderer snapshot이 없으면
  `nativeRendererSnapshotRequired`로 `unsupported`다. 실제 native 후보 관측은
  `SkiaLayerRenderer::get_font_decision_trace`를 사용한다.
- W1 source digest drift는 Stage 2~3의 관찰 wrapper 추가를 명시하는 진단이며 selection 실패가 아니다.
- header/footer의 target-dependent 상대 marker는 source 좌표가 아니므로 `null/unavailable`이다.
- v1은 page 단위, 기본 1,024문자와 hard maximum 4,096문자다.
- 검증된 PDF oracle이 없으면 `oracle.status=notProvided`다. Oracle schema/ladder는 #4963 범위다.
- 실제 10k 문자 coverage와 장평·자간·고정 프레임 위험 순위는 #4962에서 이 trace를 소비한다. W2 산출물은
  runtime query, 분류 reason, portable hash, 공개 fixture와 private boundary 계약이다.

## 7. Stage 종료

Stage 5 구현·검증·문서화는 완료했다. remote push, PR 생성, GitHub comment와 #4961 상태 변경은 수행하지
않았다. Stage 5 변경을 별도 commit으로 고정한 뒤 다음 절차 승인을 요청한다.
