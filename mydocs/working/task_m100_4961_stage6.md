# 작업 보고 — Task M100 #4961 Stage 6

- **이슈**: [#4961](https://github.com/edwardkim/rhwp/issues/4961)
- **PR**: [#5122](https://github.com/edwardkim/rhwp/pull/5122)
- **브랜치**: `local/task4961-font-decision-trace`
- **Stage 6 시작 HEAD**: `e732ab114bd64b033cd5fa190d6d9cd922d39511`
- **합성 기준**: `upstream/devel@8d4fb781c2f253f4a9993938f51e6bf415d8488e`
- **작성일**: 2026-08-17 KST

## 1. 결론

PR self-review에서 native `DocumentCore` 단독 query가 실제 renderer와 분리된 새
`SkiaLayerRenderer`를 만들고도 native 관측을 `complete`로 기록하는 결함을 확인했다. 새 renderer에는
호출자가 `with_font_paths`로 준비한 custom font와 기존 renderer의 bundled inventory가 없으므로,
trace가 실제 paint 후보 계보를 설명한다는 계약을 만족하지 못했다.

Stage 6은 native 관측을 이미 준비된 `SkiaLayerRenderer` snapshot에 결합했다. standalone
`DocumentCore` query는 새 font를 읽거나 빈 inventory를 관측하지 않고
`nativeRendererSnapshotRequired`로 fail-closed한다. 제품 font 선택, layout metric, renderer 출력,
WASM 공개 API와 serialization은 변경하지 않았다.

## 2. 원인과 보정

### 2.1 원인

- `DocumentCore::get_font_decision_trace_native`가 query 내부에서 `SkiaLayerRenderer::new()`를 만들었다.
- `new()`에는 실제 호출자가 `with_font_paths`로 적재한 custom font inventory가 없다.
- 이 임시 observer의 결과를 native backend `complete`로 요약해 snapshot 출처 차이를 숨겼다.
- font load 0 불변식 자체는 지켰지만, 준비된 renderer 상태를 관측한다는 더 중요한 불변식을 위반했다.

### 2.2 구현

- `DocumentCore` 내부 query는 선택적으로 native observer를 주입받는다.
- `SkiaLayerRenderer::get_font_decision_trace`가 현재 renderer의 `font_decision`을 observer로 전달한다.
- standalone native query는 snapshot 부재를 record와 backend summary 모두에서
  `nativeRendererSnapshotRequired`로 반환한다.
- native Skia feature 자체가 없으면 기존 `nativeSkiaFeatureUnavailable` reason을 유지한다.
- query 경로는 font path를 읽거나 typeface를 새로 적재하지 않는다.

## 3. 회귀 테스트

`tests/cases/issue_4961_font_decision_trace.rs`에 다음 경계를 고정했다.

1. native feature가 있어도 standalone `DocumentCore` trace는 native를 `unsupported`로 반환한다.
2. `ttfs/opensource`를 준비한 renderer snapshot trace는 native summary가 `complete`다.
3. 준비된 inventory에서 선택된 record는 `source=custom`과
   `nativeGlyphCoverageObserved` capability를 함께 가진다.
4. 기존 공개 HWP/HWPX, 결정론, 상한과 unsupported 회귀는 동일 계약으로 유지된다.

## 4. 검증 결과

| gate | 결과 |
| --- | --- |
| default focused trace | 4 passed |
| native Skia focused trace | 6 passed |
| W1 ledger / trace contract | 10 / 12 passed |
| fresh `wasm-pack build --target web --out-dir pkg` | 통과 |
| fresh WASM 공개 E2E | 3 passed |
| release-test nextest | 6,526 passed, 38 skipped |
| native Skia lib / missing picture / direct PDF | 58 / 2 / 4 passed |
| default / native Skia / wasm32 clippy | 통과 |
| manifest / unit tier 정합성 | 2,484 static attrs / 4,225 source tests, 통과 |
| `cargo fmt --check`, `git diff --check` | 통과 |
| 최신 `upstream/devel` 합성 | 충돌 없음, CI policy 31 / workflow 27 / focused Rust 4 passed |

native integration의 과거 direct target 이름은 test auto-sharding 뒤 더 이상 존재하지 않아 한 번
`no test target named`으로 종료됐다. 이는 제품 실패가 아니라 문서화된 명령의 절차 drift다. 저장소
router인 `scripts/run-rust-test.mjs --cargo-test ...`로 동일 두 suite를 다시 실행해 각각 2/2와 4/4
통과를 확인했다.

현재 nextest 설치본 0.9.137이 저장소 권장 0.9.140보다 낮다는 비차단 경고가 있었고, 실제 실행된
6,526건은 모두 통과했다.

## 5. 보호 불변식 판정

| 불변식 | 판정 |
| --- | --- |
| 실제 native 관측은 준비된 renderer inventory와 동일하다 | 통과 |
| snapshot이 없으면 관측 성공으로 승격하지 않는다 | 통과 |
| trace가 font path read 또는 font load를 시작하지 않는다 | 통과 |
| 기존 layout metric과 fallback 선택을 바꾸지 않는다 | 통과 |
| WASM·브라우저 계약과 공개 fixture 결과를 바꾸지 않는다 | 통과 |
| private corpus 또는 재배포 불가 font bytes를 추가하지 않는다 | 통과 |

변경은 trace-only native observer 결합과 진단 reason에 한정된다. 화면·SVG·PDF·serialization 출력은
변경하지 않으므로 Stage 5에서 통과한 실제 브라우저 0-delta 및 시각 판정은 유효하며 새 pixel 판정
대상이 아니다.

## 6. Stage 종료와 PR 후속

Stage 6 구현·로컬 검증·문서 정산은 완료했다. 최종 code candidate를 최신 `upstream/devel`과 합성해
충돌과 whitespace가 없음을 확인했고, 합성 tree에서도 CI policy·workflow·manifest·tier와 focused Rust
gate가 통과했다. code candidate는 로컬 commit으로 고정했으며 remote push는 별도 승인 대상이다.

새 code candidate를 push한 뒤 PR #5122의 Full CI가 녹색이 되어야 self-review 문서를 trailing
review-only commit으로 추가할 수 있다. 현재 녹색인 기존 원격 HEAD의 CI는 Stage 6 변경을 검증한 결과로
간주하지 않는다.
