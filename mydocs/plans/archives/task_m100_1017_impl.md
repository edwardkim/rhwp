# Task M100 #1017 구현 계획서 — PageLayerTree replay z-order plane 적용

## 1. 목적

#1017은 #1016에서 확정한 resolved image payload를 HWP의 `BehindText` / `InFrontOfText` 의미에 맞는 순서로 replay하게 만드는 작업이다.

핵심은 `PaintOp::Image` payload를 다시 손대는 것이 아니라, PageLayerTree direct replay backend가 다음 logical plane 순서를 따르게 하는 것이다.

```text
page background
-> behindText image ops
-> flow ops
-> inFrontOfText image ops
```

Stage 1 진단 결과, `samples/복학원서.hwp` 1페이지 중앙 watermark는 이미 `wrap=behindText`, `mime=image/png`, `bakedWatermark=true`로 resolved 되어 있지만, raw replay sequence가 `127`이라 본문 flow text 뒤에 있다. 따라서 native Skia / CanvasKit direct replay의 tree/leaf 순서 재생만으로는 한컴 z-order contract를 만족할 수 없다.

## 2. Stage 1 결론

Stage 1 보고서:

```text
mydocs/working/task_m100_1017_stage1.md
```

확인된 상태:

| 경로 | 현재 상태 |
|------|-----------|
| PageLayerTree JSON | image op에 `wrap:"behindText"`와 `bakedWatermark:true` 존재 |
| native Skia | tree child 순서 + leaf `ops` 순서대로 replay |
| CanvasKit direct | JSON tree child 순서 + leaf `ops` 순서대로 replay |
| Studio Canvas2D | `flow` / `behind` / `front` 분리 후 DOM layer 합성 |
| CanvasKit replay plan | `wrap=behindText`를 detail로만 기록, z-order plane 미표현 |

`복학원서.hwp` 1페이지 진단:

```text
0001 pageBackground
0007 first textRun
0008 image wrap=behindText (logo)
0010..0126 flow text/line/rectangle
0127 image wrap=behindText bakedWatermark=true (central watermark)
0128 ellipse
```

## 3. 설계 결정

### 3.1 선택안 — multi-pass tree traversal

native Skia와 CanvasKit direct renderer가 PageLayerTree를 다음 pass 순서로 재순회한다.

```text
Background
BehindText
Flow
InFrontOfText
```

각 pass는 tree/group/clip context를 그대로 통과하되, leaf에서 현재 pass에 해당하는 op만 replay한다.

분류:

| op | replay plane |
|----|--------------|
| `PaintOp::PageBackground` | Background |
| `PaintOp::Image` + `text_wrap=BehindText` | BehindText |
| `PaintOp::Image` + `text_wrap=InFrontOfText` | InFrontOfText |
| 그 외 모든 op | Flow |

### 3.2 선택 이유

- Stage 1에서 확인한 `root/g3` 중앙 watermark처럼 다른 tree branch에 있는 behind image도 처리할 수 있다.
- tree를 flatten하지 않으므로 clip/group context를 보존한다.
- PageLayerTree JSON에는 이미 `wrap` 필드가 있어 schema 변경 없이 CanvasKit이 같은 정책을 적용할 수 있다.
- Studio Canvas2D overlay 경로와 같은 z-order contract를 direct renderer 내부에서 구현할 수 있다.

### 3.3 비선택안

**leaf 내부 정렬**은 제외한다. 중앙 watermark가 본문 flow와 다른 root child에 있으므로 leaf 내부 정렬만으로 해결되지 않는다.

**단순 Vec flatten**은 제외한다. body clip, table cell clip, group/cache hint, transform context를 잃을 위험이 높다.

**PageLayerTree JSON `paintPlane` 필드 추가**는 이번 첫 구현에서는 제외한다. 기존 `wrap`으로 CanvasKit direct replay가 충분히 분류 가능하다. 다만 CanvasKit replay plan 진단에는 additive `replayPlane` 정보를 추가해 정책 적용 여부를 확인 가능하게 한다.

## 4. 구현 단계

### Stage 2 — replay plane contract와 RED 테스트

목표:

- z-order plane 분류 규칙을 코드와 테스트로 고정한다.
- native Skia와 CanvasKit이 같은 plane 용어를 쓰게 한다.

작업:

1. Rust 쪽에 작은 plane 분류 helper를 추가한다.
   - 후보 파일: `src/paint/replay_order.rs` 또는 `src/renderer/layer_renderer.rs`
   - 후보 enum:
     ```rust
     pub enum PaintReplayPlane {
         Background,
         BehindText,
         Flow,
         InFrontOfText,
     }
     ```
2. `PaintOp` 기준 분류 함수 추가:
   ```rust
   pub fn paint_op_replay_plane(op: &PaintOp) -> PaintReplayPlane
   ```
3. 단위 테스트 추가:
   - `PageBackground` → `Background`
   - `Image(text_wrap=BehindText)` → `BehindText`
   - `Image(text_wrap=InFrontOfText)` → `InFrontOfText`
   - 일반 image / text / vector op → `Flow`
4. `tests/issue_1017.rs` 추가:
   - `samples/복학원서.hwp` PageLayerTree raw order에서 중앙 baked watermark가 first textRun 뒤에 있음을 확인
   - 동시에 `wrap=behindText`, `bakedWatermark=true`임을 확인

산출물:

```text
mydocs/working/task_m100_1017_stage2.md
```

검증 후보:

```text
cargo test --test issue_1017
cargo test --lib paint::replay_order
```

### Stage 3 — native Skia multi-pass replay 구현

목표:

- native Skia PNG export가 `Background -> BehindText -> Flow -> InFrontOfText` 순서로 PageLayerTree를 replay한다.

작업:

1. `src/renderer/skia/renderer.rs`의 `render_node()`에 replay plane 인자를 추가하거나 wrapper를 둔다.
2. page render entry에서 `PaintReplayPlane::ORDERED` 순서로 root를 여러 번 순회한다.
3. `LayerNodeKind::Leaf` 처리에서 현재 plane과 맞지 않는 op는 skip한다.
4. glyph variant selection은 Flow pass에서만 의미 있게 사용한다.
   - Background / BehindText / InFrontOfText pass에서 TextRun/GlyphRun은 skip되므로 `next_text_source_id`가 증가하지 않아야 한다.
   - Flow pass에서 기존 text source id 증가 순서를 유지한다.
5. native Skia unit test 추가:
   - raw op 순서가 `Flow rectangle -> BehindText image`여도 최종 픽셀은 flow rectangle이 위에 있어야 한다.
   - raw op 순서가 `InFrontOfText image -> Flow rectangle`이어도 최종 픽셀은 front image가 위에 있어야 한다.

산출물:

```text
mydocs/working/task_m100_1017_stage3.md
```

검증 후보:

```text
cargo test --features native-skia skia --lib
cargo run --features native-skia --bin rhwp -- export-png samples/복학원서.hwp -p 0 -o output/task1017-stage3
```

### Stage 4 — CanvasKit direct replay plane 적용

목표:

- browser CanvasKit direct renderer가 Canvas2D overlay fallback 없이 native Skia와 같은 plane 순서를 따른다.

작업:

1. TypeScript 쪽 replay plane helper 추가.
   - 후보 파일: `rhwp-studio/src/view/canvaskit/replay-plane.ts`
   - 후보 API:
     ```ts
     export type CanvasKitReplayPlane = 'background' | 'behindText' | 'flow' | 'inFrontOfText';
     export const CANVASKIT_REPLAY_PLANES = ['background', 'behindText', 'flow', 'inFrontOfText'] as const;
     export function layerPaintOpReplayPlane(op: LayerPaintOp): CanvasKitReplayPlane;
     ```
2. `CanvasKitLayerRenderer.renderPage()`가 plane 순서대로 `renderNode(canvas, tree.root, profile, plane)`을 호출하게 한다.
3. `renderLeaf()`에서 현재 plane과 맞는 op만 `renderOp()`로 보낸다.
4. 기존 source regression 유지:
   - CanvasKit renderer가 `getContext('2d')`
   - `renderPageToCanvas`
   - `rhwpOverlay`
   를 도입하지 않아야 한다.
5. `rhwp-studio/tests/render-backend.test.ts`에 plane helper 테스트 추가:
   - `image.wrap === 'behindText'` → `behindText`
   - `image.wrap === 'inFrontOfText'` → `inFrontOfText`
   - `pageBackground` → `background`
   - text/vector/default image → `flow`

산출물:

```text
mydocs/working/task_m100_1017_stage4.md
```

검증 후보:

```text
npm --prefix rhwp-studio test
npm --prefix rhwp-studio run build
```

### Stage 5 — CanvasKit replay plan 진단 갱신

목표:

- CanvasKit direct replay plan이 z-order plane policy를 드러내게 한다.

작업:

1. `src/renderer/canvaskit_policy.rs`의 `CanvasKitReplayItem`에 additive field 추가:
   ```rust
   pub replay_plane: CanvasKitReplayPlane
   ```
2. `item_for_op()`에서 Rust plane helper를 사용해 각 item의 plane을 기록한다.
3. `CanvasKitReplaySummary`에 plane별 count를 추가할지 검토한다.
   - 최소 구현은 item-level `replayPlane`만 둔다.
4. unit test 갱신:
   - baked watermark image item이 `replayPlane=behindText`
   - 일반 textRun item이 `replayPlane=flow`
   - page background item이 `replayPlane=background`

산출물:

```text
mydocs/working/task_m100_1017_stage5.md
```

검증 후보:

```text
cargo test --lib canvaskit_policy
cargo test --test issue_1017
```

### Stage 6 — 통합 검증과 최종 보고

목표:

- #1017 완료 조건을 검증하고 최종 결과를 정리한다.

작업:

1. `samples/복학원서.hwp` 1페이지 native Skia PNG export 생성.
2. 중앙 baked watermark가 글 내용 위에 그려지지 않는지 확인한다.
3. opaque baked PNG의 흰 사각 배경이 본문 위에 노출되지 않는지 확인한다.
4. 기존 회귀 테스트 유지:
   - #938 resolved watermark
   - #516 overlay JSON / Canvas2D layer split
   - CanvasKit direct replay no hidden overlay
5. 최종 보고서 작성.
6. 오늘할일 상태 갱신.

산출물:

```text
mydocs/report/task_m100_1017_report.md
```

검증 후보:

```text
cargo test --test issue_1017
cargo test --test issue_938
cargo test --test issue_516
cargo test --features native-skia skia --lib
cargo run --features native-skia --bin rhwp -- export-png samples/복학원서.hwp -p 0 -o output/task1017-final
npm --prefix rhwp-studio test
npm --prefix rhwp-studio run build
```

## 5. 파일별 예상 변경

| 파일 | 변경 |
|------|------|
| `src/paint/replay_order.rs` 또는 `src/renderer/layer_renderer.rs` | Rust replay plane enum/helper |
| `src/paint/mod.rs` | helper module export 필요 시 |
| `src/renderer/skia/renderer.rs` | multi-pass replay + native unit tests |
| `src/renderer/canvaskit_policy.rs` | replay plan에 plane 진단 추가 |
| `tests/issue_1017.rs` | `복학원서.hwp` raw order/plane 회귀 테스트 |
| `rhwp-studio/src/view/canvaskit/replay-plane.ts` | TS replay plane helper |
| `rhwp-studio/src/view/canvaskit-renderer.ts` | plane별 renderNode 순회 |
| `rhwp-studio/tests/render-backend.test.ts` | CanvasKit plane/source regression |
| `mydocs/working/task_m100_1017_stage*.md` | 단계별 완료 보고 |
| `mydocs/report/task_m100_1017_report.md` | 최종 보고 |
| `mydocs/orders/20260529.md` | 상태 갱신 |

## 6. 제외 범위

- #1016의 `ResolvedImagePayload` 구조 변경.
- baked watermark PNG를 alpha-transparent PNG로 변경.
- PageLayerTree JSON schema minor bump.
- Studio Canvas2D HTML Hybrid overlay 제거.
- CanvasKit glyph/text parity 전체 해결.
- 이미지가 아닌 shape/table의 `BehindText` / `InFrontOfText` z-order 일반화.
- HWP layout / pagination / anchor 위치 보정.
- HWP3 전용 분기 추가.

## 7. 위험 및 대응

| 위험 | 대응 |
|------|------|
| multi-pass로 TextRun source id가 어긋남 | text op는 Flow pass에서만 source id 증가. Stage 3 unit test와 기존 text replay 테스트 확인 |
| clip/group context 손실 | flatten 금지, tree를 pass별 재순회 |
| CanvasKit과 native Skia plane 분류 불일치 | Rust/TS helper 각각 테스트하고 같은 분류표를 문서화 |
| front image가 flow 뒤에 오지 않는 회귀 | native Skia 픽셀 테스트 + TS plane test |
| PageLayerTree consumer 호환성 | JSON 필드 추가 없이 기존 `wrap` 사용 |
| native-skia 빌드 시간/환경 | 우선 일반 Rust 테스트로 contract 검증, feature test는 Stage 3/6에서 실행 |
| LFS 예산 초과 | `pdf-large/` 의존 검증 제외, `samples/복학원서.hwp` 중심 |

## 8. 승인 요청

본 구현 계획은 Stage 2~6의 5단계로 진행한다.

승인되면 Stage 2에서 replay plane helper와 RED/contract 테스트를 먼저 추가하고, 아직 renderer 동작 변경은 Stage 3부터 진행한다.
