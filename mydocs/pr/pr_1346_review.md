# PR #1346 검토 — WebCanvas layer adapter 축소 및 option metadata 분리

- PR: https://github.com/edwardkim/rhwp/pull/1346
- 제목: render: reduce WebCanvas layer adapter and split option metadata
- 작성일: 2026-06-10
- 작성자: `seo-rii`
- 관련 이슈: #536 "멀티 렌더러 지원 트래킹 이슈"
- base: `devel`
- head: `seo-rii:render-p22` (`58168282`)
- 로컬 검토 브랜치: `local/pr1346-upstream`

## 1. 요약 판단

수용 가능으로 판단한다. Blocking 이슈는 찾지 못했다.

이 PR은 WebCanvas layer replay에서 `PaintOp` leaf를 임시 `RenderNode`로 다시 감싸는
adapter를 제거하고, 기존 RenderNode 경로와 같은 leaf drawing helper를 공유하게
정리한다. 동시에 PageLayerTree JSON schema minor를 `1.16`으로 올리고
`buildOptions`/`debugOptions`를 canonical metadata로 추가한다.

다만 PR 브랜치에는 중간 merge commit(`68683a26`)이 포함되어 있으므로, 수용 시에는
현재 `local/devel` 기준으로 non-merge commit 3개만 순서대로 cherry-pick하는 것이
좋다.

## 2. PR 정보

| 항목 | 값 |
|---|---|
| 상태 | open |
| draft | false |
| mergeable | MERGEABLE |
| 변경량 | 10 files, +722 / -631 |
| milestone | v1.0.0 |
| label | enhancement |

커밋:

- `04bc8ed0` — refactor(render): replay WebCanvas PaintOps directly
- `f52df364` — feat(render): split layer option metadata
- `68683a26` — Merge branch 'devel' into render-p22
- `58168282` — fix(render): backfill layer option compatibility mirrors

수용 후보:

- `04bc8ed0`
- `f52df364`
- `58168282`

제외 후보:

- `68683a26` — PR branch 내부 동기화 merge commit

## 3. 변경 검토

### 3.1 WebCanvas PaintOp 직접 replay

`src/renderer/web_canvas.rs`:

- 기존 `render_node()` 안의 leaf별 렌더링 로직을 helper로 분리했다.
- `render_layer_node()`의 `LayerNodeKind::Leaf` 분기에서 `PaintOp`를 `RenderNode::new`
  로 재조립하지 않고 `render_paint_op()`로 직접 dispatch한다.
- 도형 transform은 RenderNode 경로와 PaintOp 직접 경로에서 복원 시점이 다르다.
  - RenderNode 경로: 기존처럼 자식 렌더 후 `close_shape_transform_for_node()`
  - PaintOp 경로: leaf payload 렌더 직후 `close_shape_transform_if_needed()`
- 기존 no-op PaintOp(`GlyphRun`, `GlyphOutline`, `CharOverlap`, `TextControlMark`,
  `TabLeader`, `TextDecoration`) 정책은 유지된다.

검토 결과:

- `ImageNode` resolved payload 경로가 유지된다.
- watermark/filter/global alpha reset이 helper 내부에 남아 있다.
- external image placeholder 처리도 유지된다.
- layer filter와 body/table/textbox clip 구조는 기존 `render_layer_node()` 흐름을 유지한다.

### 3.2 PageLayerTree option metadata 분리

`src/paint/json.rs`, `src/paint/schema.rs`, `rhwp-studio/src/core/types.ts`,
`rhwp-studio/src/core/wasm-bridge.ts`:

- schema minor: `1.15` → `1.16`
- 신규 canonical metadata:
  - `buildOptions.showTransparentBorders`
  - `buildOptions.clipEnabled`
  - `debugOptions.debugOverlay`
- compatibility mirror:
  - 기존 `outputOptions.showParagraphMarks`
  - 기존 `outputOptions.showControlCodes`
  - 기존 `outputOptions.showTransparentBorders`
  - 기존 `outputOptions.clipEnabled`
  - 기존 `outputOptions.debugOverlay`
- Studio bridge는 old/new JSON을 normalize한다.

검토 결과:

- 구 WASM JSON에서 `buildOptions`/`debugOptions`가 없으면 `outputOptions`에서 보강한다.
- 신 WASM JSON에서도 `outputOptions` mirror가 유지되어 기존 소비자가 깨지지 않는다.
- fallback JSON에도 mirror 필드가 추가되어 호환성이 맞춰졌다.

### 3.3 기술적 평가

종합 평가는 긍정적이다. 이 PR은 기능을 크게 늘리는 PR이라기보다, P22 단계의
렌더러 구조를 더 올바른 방향으로 정리하는 내부 품질 개선 PR에 가깝다.

#### 아키텍처 적합성

현재 PageLayerTree의 leaf는 이미 `PaintOp`라는 backend replay payload를 가지고 있다.
그런데 기존 WebCanvas layer 경로는 이 `PaintOp`를 다시 임시 `RenderNode`로 감싸
기존 `render_node()`에 태우고 있었다. 이 방식은 전환기 adapter로는 실용적이지만,
장기적으로는 다음 문제가 있다.

- semantic container(`RenderNode`)와 paint payload(`PaintOp`)의 책임이 다시 섞인다.
- PaintOp replay backend가 늘어날수록 WebCanvas만 임시 wrapper 의존을 유지하게 된다.
- transform/clip/filter 같은 canvas state 복원 지점이 adapter 구조에 숨어 추론이 어렵다.

이번 PR은 leaf drawing helper를 공통화한 뒤, RenderNode 경로와 PaintOp 직접 경로가
같은 draw primitive를 호출하게 만든다. 따라서 아키텍처 방향은 타당하다. 특히
`RenderNode::new` 재조립을 제거한 것은 PageLayerTree를 "렌더 트리의 JSON 복제본"이
아니라 "paint replay 계약"으로 독립시키는 데 맞다.

#### 구현 안정성

핵심 안정성 포인트는 canvas state 복원이다. 기존 RenderNode 경로는 자식 렌더링 후
transform을 복원했고, 새 PaintOp 직접 replay 경로는 leaf payload 렌더 직후 transform을
복원한다. 이 차이는 의도된 차이로 보인다.

- RenderNode 경로: 구조 노드와 자식 렌더를 포함하므로 기존 후처리 복원 유지
- PaintOp 경로: leaf op 자체가 최종 paint 단위이므로 helper 내부에서 즉시 복원

검토한 범위에서는 `filter`, `globalAlpha`, `textAlign`, `textBaseline`, line dash,
shape transform 복원 경로가 기존 동작을 보존한다. 특히 image watermark/effect 처리와
external image placeholder가 helper 이동 후에도 남아 있는 점이 중요하다.

남는 리스크는 `PaintOp::GlyphRun`, `GlyphOutline`, `CharOverlap`, `TextControlMark`,
`TabLeader`, `TextDecoration`이 WebCanvas 직접 replay에서 여전히 no-op이라는 점이다.
다만 이는 이번 PR이 새로 만든 결함이라기보다 기존 전환기 정책을 유지한 것이다.
CanvasKit/native Skia 쪽 direct replay coverage와의 차이는 후속 P22/P23 범위로 보는
것이 맞다.

#### Schema / 호환성 평가

`buildOptions`와 `debugOptions`를 `outputOptions`에서 분리하는 방향은 좋다.
`outputOptions`는 원래 "표시 출력 옵션" 성격인데, `clipEnabled`나 `debugOverlay`는
렌더 트리 생성/디버그 정책에 가깝다. 따라서 다음처럼 의미를 분리하는 것은 API
계약을 명확히 한다.

- `buildOptions`: layer tree 생성/clip 관련 정책
- `debugOptions`: debug overlay 관련 정책
- `outputOptions`: 문단부호/조판부호 표시 및 기존 mirror

중요한 점은 compatibility mirror를 유지했다는 것이다. PR #1346은 `schemaMinorVersion`
을 `1.16`으로 올리되 기존 `outputOptions.showTransparentBorders`,
`outputOptions.clipEnabled`, `outputOptions.debugOverlay`를 제거하지 않는다. Studio
bridge도 old/new JSON을 양방향 normalize한다. 이 때문에 구 WASM과 신 Studio, 신 WASM과
구 소비자 사이의 단기 호환성 리스크는 낮다.

Rust 내부 `PageLayerTree` 모델은 여전히 `LayerOutputOptions` 하나를 들고 있고, 이번
분리는 JSON/export 계약과 TypeScript bridge 레벨에서 이뤄진다. 이 선택은 보수적이다.
내부 모델까지 `BuildOptions`/`DebugOptions`로 나누면 더 깔끔할 수 있지만, 변경 폭이
커지고 다른 backend(Skia/CanvasKit/SVG layer)에 연쇄 영향을 줄 수 있다. 이번 PR의
범위에서는 외부 계약만 먼저 분리하는 것이 적절하다.

#### 테스트 품질 평가

검증은 PR 성격에 대체로 맞다.

- Rust JSON 테스트가 문자열 contains에서 `serde_json::Value` 기반 assertion으로 보강됐다.
- Studio bridge는 old/new option metadata normalize를 테스트한다.
- WebCanvas contract 테스트는 `RenderNode::new` 재도입을 막는 회귀 가드 역할을 한다.
- GitHub Canvas visual diff가 pass라서 leaf helper 분리의 시각 회귀 가능성을 낮춘다.

단, `tests/render_p22_web_canvas_contract.rs`는 source string assertion에 의존한다.
이 방식은 의도 회귀 방지에는 가볍고 효과적이지만, 장기적으로는 실제 LayerTree fixture를
WebCanvas replay에 태워 op별 동작을 검증하는 테스트가 더 낫다. 현재 PR을 막을 정도의
문제는 아니며, 후속 개선 권고로 두면 충분하다.

#### 수용 가치

수용 가치는 다음 세 가지다.

- WebCanvas layer replay가 PageLayerTree/PaintOp 계약에 더 직접적으로 정렬된다.
- RenderNode 임시 재조립이 제거되어 future backend parity 작업의 복잡도가 줄어든다.
- option metadata의 의미가 명확해져 Studio bridge와 외부 소비자 계약이 안정된다.

따라서 기술적으로는 "중간 adapter를 걷어내는 정리 PR"로서 수용 가치가 있다. 위험은
렌더러 state reset 계열에 집중되지만, 현재 CI와 로컬 검증 결과를 보면 수용 가능한
수준이다.

## 4. CI 상태

GitHub checks:

- Build & Test — pass
- Canvas visual diff — pass
- CodeQL — pass
- Analyze rust — pass
- Analyze javascript-typescript — pass
- Analyze python — pass
- WASM Build — skipped

## 5. 로컬 검증

검토 브랜치: `local/pr1346-upstream`

| 명령 | 결과 |
|---|---|
| `cargo fmt --check` | 통과 |
| `CARGO_INCREMENTAL=0 cargo test --lib serializes_layer_option_metadata -- --nocapture` | 통과 |
| `CARGO_INCREMENTAL=0 cargo test --lib test_page_layer_tree_export_preserves_output_options -- --nocapture` | 통과 |
| `CARGO_INCREMENTAL=0 cargo test --lib layer_tree_schema_constants_match_schema -- --nocapture` | 통과 |
| `CARGO_INCREMENTAL=0 cargo test --test render_p22_web_canvas_contract -- --nocapture` | 통과 |
| `cd rhwp-studio && node --test tests/render-backend.test.ts` | 23 passed |
| `CARGO_INCREMENTAL=0 cargo check --lib --target wasm32-unknown-unknown -j 2` | 통과 |
| `git diff --check local/devel...local/pr1346-upstream` | 통과 |

## 6. 리스크

| 리스크 | 평가 | 완화 |
|---|---|---|
| WebCanvas helper 분리 중 state reset 누락 | 낮음~중간 | filter/globalAlpha/transform reset 경로 확인, Canvas visual diff pass |
| PaintOp 직접 replay와 RenderNode replay 간 미세 시각 차이 | 중간 | PR 자체 Canvas visual diff pass, 수용 후 wasm build 및 필요 시 시각 판정 |
| PageLayerTree JSON 소비자 호환성 | 낮음 | `outputOptions` mirror 유지, Studio bridge old/new normalize |
| 테스트가 source string assertion에 일부 의존 | 낮음 | contract 목적은 명확하나 장기적으로 AST/행동 기반 테스트가 더 좋음 |
| PR branch에 merge commit 포함 | 낮음 | 수용 시 non-merge commit만 cherry-pick |

## 7. 수용 절차 제안

작업지시자 승인 후:

1. `local/devel` 기준으로 non-merge commit 3개 cherry-pick
   - `04bc8ed0`
   - `f52df364`
   - `58168282`
2. 충돌 여부 확인
3. 동일 검증 재실행
4. WASM 빌드
5. 필요 시 작업지시자 시각 판정
6. 승인 시 no-ff merge/push 및 PR 처리

## 8. 승인 요청

위 검토 결과 기준으로 PR #1346 수용 절차를 진행해도 되는지 승인 요청한다.
