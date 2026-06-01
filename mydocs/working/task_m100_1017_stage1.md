# Task M100 #1017 Stage 1 완료 보고서 — PageLayerTree replay z-order 현황 진단

## 1. 요약

#1017의 문제는 현재 `upstream/devel` `74137424` 기준으로 여전히 재현 가능한 구조다.

`samples/복학원서.hwp` 1페이지 PageLayerTree에서 중앙 baked watermark는 `wrap=behindText`, `mime=image/png`, `bakedWatermark=true`로 올바르게 resolved 되어 있다. 그러나 replay 순서상 본문 flow text 대부분이 먼저 나오고, 중앙 watermark image op가 거의 마지막에 나온다.

따라서 native Skia / browser CanvasKit direct renderer처럼 PageLayerTree를 tree/leaf 순서대로 replay하는 backend에서는 `BehindText` 이미지가 flow content 위에 그려질 수 있다. 이는 #1016의 payload 문제가 아니라 #1017의 replay z-order 문제다.

## 2. 진단 범위

- 대상 브랜치: `local/task1017`
- base: `upstream/devel` `74137424`
- 대상 문서: `samples/복학원서.hwp`
- 대상 페이지: page 0 (한컴 1페이지)
- 코드 수정 여부: 없음
- 진단용 임시 프로그램: `/private/tmp/rhwp1017diag` (저장소 밖)

## 3. native Skia replay 경로

관련 코드:

- `src/renderer/skia/renderer.rs:440`
- `src/renderer/skia/renderer.rs:487`
- `src/renderer/skia/renderer.rs:776`

현재 구조:

```text
LayerNodeKind::Leaf { ops }
  1차: glyph variant 후보 수집
  2차: for op in ops 순서대로 match/replay
    PaintOp::Image => resolved bytes 우선 draw_image(...)
```

확인 결과:

- native Skia는 `PaintOp::Image.resolved` bytes와 `suppress_effects`는 존중한다.
- 하지만 `image.text_wrap == BehindText/InFrontOfText`를 replay plane으로 사용하지 않는다.
- `ops` 순서와 tree child 순서를 그대로 따른다.

## 4. browser CanvasKit direct replay 경로

관련 코드:

- `rhwp-studio/src/view/page-renderer.ts:48`
- `rhwp-studio/src/view/page-renderer.ts:84`
- `rhwp-studio/src/view/canvaskit-renderer.ts:222`
- `rhwp-studio/src/view/canvaskit-renderer.ts:248`

현재 구조:

```text
PageRenderer.renderPage(...)
  if backend == canvaskit:
    getPageLayerTreeObject(...)
    canvaskitRenderer.renderPage(...)
    return

CanvasKitLayerRenderer.renderNode(...)
  group: children 순서대로 renderNode
  clipRect: save/clip/render child/restore
  leaf: for op of node.ops 순서대로 renderOp
```

확인 결과:

- CanvasKit direct renderer는 Canvas2D overlay fallback을 사용하지 않는다.
- `renderLeaf()`는 JSON leaf `ops` 순서를 그대로 replay한다.
- `op.wrap === "behindText"` / `"inFrontOfText"`를 z-order plane으로 partition하지 않는다.

CanvasKit replay plan 진단:

```text
CANVASKIT_SUMMARY
  totalItems=166
  directItems=62
  directRequiredItems=104
  hiddenOverlayViolations=104

CANVASKIT_ITEM
  path=root/group/2/clip/child/group/0/group/6/leaf/0
  status=direct
  detail=resolved=formatConverted;crop;wrap=behindText

CANVASKIT_ITEM
  path=root/group/3/leaf/0
  status=direct
  detail=resolved=bakedWatermark;crop;wrap=behindText
```

의미:

- CanvasKit 정책 진단은 `wrap=behindText`를 detail로만 기록한다.
- 두 image op 모두 `direct`로 분류된다.
- 현재 policy에는 "direct replay는 가능하지만 z-order plane 조정이 필요하다"는 상태가 없다.

## 5. Studio Canvas2D 기준 경로

관련 코드:

- `rhwp-studio/src/view/page-renderer.ts:53`
- `rhwp-studio/src/view/page-renderer.ts:136`
- `rhwp-studio/src/view/page-renderer.ts:155`
- `rhwp-studio/src/view/page-renderer.ts:166`
- `src/renderer/web_canvas.rs:304`
- `src/renderer/web_canvas.rs:1097`

현재 기준 동작:

```text
Canvas2D 기본 경로:
  renderPageToCanvasFiltered(..., "flow")
    -> FlowOnly: BehindText/InFrontOfText image 제외
  applyOverlays(...)
    -> page background layer
    -> behind overlay layer
    -> flow canvas
    -> front overlay layer
```

`WebCanvasRenderer::should_render_image()` 정책:

```text
All: 모든 image 렌더
FlowOnly: BehindText/InFrontOfText image 제외
WrapOnly(BehindText): BehindText image만 렌더
WrapOnly(InFrontOfText): InFrontOfText image만 렌더
```

이 경로가 #1017의 기준 z-order contract다.

## 6. 복학원서 PageLayerTree op 순서

진단 명령:

```text
cargo run --quiet --manifest-path /private/tmp/rhwp1017diag/Cargo.toml
```

요약:

```text
schema=1.14
page=793.707x1122.507
total_ops=128
images=2
behind=2
front=0
textRuns=102
first_text_seq=7
behind_after_first_text=2
```

핵심 op 순서:

```text
0001 root/g0[0] pageBackground
0007 root/g2/clip/g0/g5/g0[0] textRun ""
0008 root/g2/clip/g0/g6[0] image wrap=behindText mime=image/png baked=false watermark=false
0010..0126 root/g2/... textRun/line/rectangle flow content
0127 root/g3[0] image wrap=behindText mime=image/png baked=true watermark=true
0128 root/g4[0] ellipse
```

중앙 watermark:

```text
seq=127
path=root/g3[0]
type=image
wrap=behindText
mime=image/png
baked=true
watermark=true
bbox=x=137.7 y=270.2 w=495.0 h=495.7
```

결론:

- 중앙 watermark가 flow text/line/table content 뒤에 위치한다.
- native Skia / CanvasKit direct replay가 이 순서를 그대로 따르면 BehindText 이미지가 본문 위에 그려진다.
- #1017은 실제로 필요한 후속 작업이다.

## 7. 구현 계획에서 다뤄야 할 경계 조건

### 7.1 단순 leaf 내부 정렬만으로는 부족

중앙 watermark는 `root/g3` leaf에 있고, 본문 flow content는 `root/g2/clip/...` 아래에 있다. 즉 같은 leaf 내부 순서만 바꿔서는 해결되지 않는다. tree child 순서 전체를 plane 기준으로 재구성해야 한다.

### 7.2 단순 flatten은 위험

flow content는 body clip, table cell clip, group context 아래에 있다. 단순히 모든 op를 `Vec`으로 flatten해 bucket별 replay하면 다음 context를 잃을 수 있다.

- clipRect
- table cell clip
- group/cache hint
- shape transform
- source node / diagnostics path

따라서 Stage 2 구현 계획에서는 다음 중 하나를 결정해야 한다.

- tree를 매 pass 재순회하며 특정 plane만 replay한다.
- clip/group context를 포함한 replay item을 만든다.
- PageLayerTree에 additive paint plane metadata를 추가하고 backend별 traversal을 공유한다.

### 7.3 CanvasKit replay plan도 갱신 필요

현재 CanvasKit replay plan은 `wrap=behindText` image를 `direct`로만 분류한다. #1017 구현 후에는 direct replay가 가능하더라도 z-order plane policy 적용 여부를 진단할 수 있어야 한다.

## 8. Stage 2 권장 방향

Stage 2 구현 계획서에서는 후보를 다음 순서로 비교한다.

1. **multi-pass tree traversal**
   - pass: background / behind image / flow / front image
   - 장점: clip/group context를 보존하기 쉽다.
   - 주의: text source id / glyph variant selection 같은 state가 pass 분리와 충돌하지 않아야 한다.

2. **context-preserving replay item**
   - tree traversal 중 clip/group context를 stack으로 포함한 replay item 생성
   - 장점: native Skia / CanvasKit 정책 공유 가능성이 높다.
   - 주의: 구현량이 커질 수 있다.

3. **JSON additive paint plane**
   - `PaintOp::Image` 또는 wrapper에 `paintPlane` 추가
   - 장점: JSON consumer가 명시적으로 알 수 있다.
   - 주의: schema minor bump와 기존 consumer 영향 검토 필요.

현재 진단 기준으로는 **multi-pass tree traversal**이 가장 작은 변경으로 보인다. 다만 Stage 2에서 text source id와 glyph fallback state를 먼저 검토해야 한다.

## 9. 실행한 검증

```text
sed/rg 기반 코드 경로 확인
cargo run --quiet --manifest-path /private/tmp/rhwp1017diag/Cargo.toml
```

비고:

- 최초 진단 프로젝트 실행은 sandbox network 제한 때문에 crates.io index 확인에 실패했다.
- 승인된 네트워크 실행 후 의존성 확인과 진단 실행이 성공했다.
- 저장소 소스 코드는 수정하지 않았다.

## 10. 다음 단계

작업지시자 승인 후 Stage 2에서 구현 계획서를 작성한다. 구현 계획서는 최소 3단계, 최대 6단계로 분리하고, native Skia와 CanvasKit direct replay를 같은 z-order contract로 묶는 방식을 확정한다.
