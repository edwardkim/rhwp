# Task M100 #4969 W10-Q2-D — emitted-run owner 재감사

## 판정

Q2-D 구현 전 owner 재감사 결과, Q2-C 결과를 `TextRunNode`의 새 public field로 직접 넣는 초안은 기각한다.
대신 **`ComposedParagraph`가 line transaction `Arc`를 보존하고, 최종 emitted node는 `PageRenderTree`의
직렬화 제외 `NodeId → sidecar` 표에 연결하는 구조**가 현행 계보와 가장 작게 맞는다.

이 감사는 제품 코드를 바꾸지 않는다. Q2-D 수정 수행계획은
[`task_m100_4969_w10_q2_d.md`](../plans/archives/task_m100_4969_w10_q2_d.md)에 제시한다.

## 현행 계보

```text
Paragraph + ResolvedStyleSet
  -> composer/line_breaking.rs
       LineBreakResult { start_idx, end_idx, max_font_size, has_line_break }
  -> LineSeg / ComposedLine / ComposedTextRun
  -> paragraph_layout.rs
       emitted_run_layout_positions()가 W9 scalar positions를 재측정
  -> RenderNode::new(TextRunNode, bbox)
  -> LayerBuilder
       LayerNode.source_node_id = RenderNode.id
  -> lower_font_native_glyph_sidecars()
       nominal cmap GlyphRun 또는 font-native outline
```

Q2-C final-line `Arc`는 현재 이 계보 어디에도 저장되지 않는다. `LineBreakResult`는 scalar range만 보존하고,
`ComposedParagraph`와 `PageRenderTree`에도 shaping transaction 필드가 없다. 따라서 Q2-C boundary만 live로 열면
paragraph layout이 W9/K0로 다시 측정하고 nominal paint가 재생되는 기존 owner 불일치가 그대로 발생한다.

## 왜 TextRunNode field를 기각하는가

현재 source에는 `TextRunNode { ... }` literal이 24개 파일에 81개 있다. public struct에 internal field를 추가하면
모든 fixture·backend·diagnostic literal을 고쳐야 하고, 외부 Rust 소비자 struct literal도 깨뜨릴 수 있다.
`layout_positions`에 shaping을 끼워 넣는 방식은 W9의 공개 scalar N+1 계약을 훼손하므로 더 나쁘다.

반면 `PageRenderTree`는 다음 조건을 이미 갖는다.

- `frame`처럼 `#[serde(skip)]`인 page-local 파생 상태를 소유한다.
- 모든 emitted `RenderNode`에 unique `NodeId`를 발급한다.
- LayerBuilder가 `LayerNode.source_node_id`에 같은 ID를 전달한다.
- page tree clone은 `Arc`를 복제 없이 공유할 수 있다.
- page 단위 4,096 entry 상한과 lifecycle clear를 자연스럽게 적용할 수 있다.

`PageRenderTree` literal은 6개 파일 10개뿐이고 대부분 `PageRenderTree::new`로 생성된다. 따라서 적용 결과를
`TextRunNode` schema와 분리하면서도 최종 emitted node에 정확히 묶을 수 있다.

## ComposedParagraph가 필요한 이유

Page sidecar만으로는 line owner를 증명할 수 없다. Q2-C 결과가 composer에서 paragraph layout까지 도달하려면
중간 owner가 하나 필요하다. `compose_paragraph()` 호출은 151개지만 `ComposedParagraph` literal은 8개 파일 14개다.
기존 함수는 sidecar `None`을 만드는 호환 경로로 유지하고, exact source context를 받은 좁은
`compose_paragraph_with_horizontal_shaping()`만 Q2-C `Arc`를 보존하는 것이 안전하다.

`ComposedParagraph`는 직렬화되지 않는 파생 객체다. Q2-C final line range·target `Arc`·attempt trace를 여기에
보존하면 paragraph layout이 재측정하지 않고 최종 run 범위를 대사할 수 있다. final emitted run이 target range와
정확히 일치하지 않으면 sidecar를 게시하지 않는다.

## 기존 public GlyphRun 재사용 가능성

paint schema의 `LayerGlyphRunPaint`는 이미 다음을 표현한다.

- exact font face key와 font instance
- glyph IDs, positions, advances
- UTF-8·UTF-16 source range와 glyph cluster range
- LTR, bidi level 0, horizontal writing mode
- shaping engine·feature·diagnostics

따라서 applied Q2-D lane은 새 public schema field 없이 기존 GlyphRun variant를 사용할 수 있다. 현재 nominal
lowerer는 옛한글 Jamo와 combining mark를 명시적으로 거부하므로 common sidecar를 먼저 소비하면 nominal GlyphRun과
중복될 위험도 줄어든다. rejected attempt는 Q2-D에서 page-local sidecar와 lowering report까지만 연결하고, 실제
TextSource public annotation이 필요하다고 판정될 때만 schema minor를 올린다.

## 새로 확인된 원자성 경계

pagination/edit reflow가 `LineSeg`만 모델에 남긴 뒤 common `Arc`를 잃으면, 나중 paint 실패 시 shaped boundary와
fallback glyph가 섞인다. 그러므로 첫 Q2-D activation은 다음 두 lane으로 제한한다.

1. 저장 또는 기존 line range가 Q2-C 결정과 **동일한 경우**: range를 바꾸지 않고 common sidecar·bbox·paint만 연다.
2. `LineSeg`를 모델에 쓰지 않는 좁은 NO_LS render-only path: Q2-C line outcome과 `ComposedParagraph`를 같은
   transaction에서 만들 때만 boundary 변경과 sidecar를 함께 연다.

일반 edit reflow, stored-prefix 보존, split-cell recovery, multi-interval frame, source가 외부 등록뿐인 문단은
Q2-D 최초 activation에서 제외한다. 이 경계를 먼저 지키는 것이 transient `Arc` 없이 line range만 저장하는 것보다
중요하다.

## 보호 불변식

1. `TextRunNode.layout_positions`는 기존 W9 scalar N+1 계약 그대로 유지한다.
2. common sidecar가 있으면 `layout_positions`는 반드시 `None`이다.
3. line selection·bbox·next-run origin·GlyphRun은 같은 Q2-C target `Arc`를 소비한다.
4. final emitted run이 target scalar·UTF-8·UTF-16 범위와 정확히 일치하지 않으면 전체 run을 fallback한다.
5. PageRenderTree serialization과 legacy `to_json()`에는 sidecar가 나타나지 않는다.
6. sidecar와 rejected trace는 page당 4,096개로 제한하고 raw text·font bytes·host path를 기록하지 않는다.
7. portable GlyphRun은 sidecar source handle이 동일 embedded bytes·face index와 대사될 때만 게시한다.
8. common GlyphRun과 nominal W9 GlyphRun을 한 TextRun alternative에 동시에 만들지 않는다.
9. sidecar lookup·lowering 실패는 boundary 변경 없이 W9 K1 또는 K0 TextRun으로 되돌린다.
10. 일반 edit reflow의 boundary publication은 Arc 수명과 rollback이 한 transaction임을 증명하기 전까지 닫는다.
