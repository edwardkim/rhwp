# 구현계획서 - Task M100-1197: HWPX 용지 기준 BehindText 그림/표 z-order 보존

- 이슈: #1197
- 브랜치: `local/task1197`
- 작성일: 2026-06-02
- 수행계획서: `mydocs/plans/task_m100_1197.md` (승인 완료)
- 기준 커밋: `upstream/devel` `f6ffe9d6`

## 설계 요약

#1197의 본질은 `BehindText` 객체를 단순히 본문 뒤 plane으로 보내는 문제가 아니라,
용지/페이지 기준 anchored 객체를 Picture/Table/Shape 타입과 무관하게 하나의 z-order 축에서
합성하는 문제다.

현재 코드에서 확인한 제약은 다음과 같다.

- `ImageNode`는 `text_wrap`을 보유하지만 `z_order`는 직접 보유하지 않는다.
- `TableNode`는 `section_index/para_index/control_index`만 보유하고 `text_wrap/z_order`가 없다.
- `RectangleNode`, `PathNode`, `EllipseNode` 등 도형 leaf도 공통 z-order 메타데이터를 갖지 않는다.
- SVG의 `node_z_plane()`과 PaintOp의 `paint_op_replay_plane()`은 사실상 `ImageNode.text_wrap` 중심이다.
- `layout.rs`의 `paper_images` 버킷은 이름과 달리 용지 기준 그림뿐 아니라 표/도형도 담는 경로가 있다.

따라서 구현은 "표를 이미지처럼 취급"하는 임시 분기가 아니라, 용지 기준 객체가 렌더 트리에 들어올 때
`text_wrap`, `z_order`, 안정 정렬 순서를 보존하는 방식으로 진행한다.

## 단계 구성 (5단계)

### Stage 1 - 재현 계측 및 RED 회귀 고정

목표: 결함을 코드 변경 전/초기 테스트로 고정하고, 재현 자료 부재 리스크를 제거한다.

- 저장소 내 #1197 원본 문서 존재 여부 재확인:
  - `[2027] 온새미로 1 본교재.hwpx`
  - 관련 PDF 정답지
- 원본 문서가 제공되면 이슈 본문 명령으로 현재 SVG를 산출한다.
  - `target/debug/rhwp export-svg "[2027] 온새미로 1 본교재.hwpx" -o /tmp/rhwp-onsaimiro-page4 -p 3`
  - SVG 요소 순서에서 낮은 zOrder 표 텍스트, 전체 페이지 이미지, 최종 표, `01` 도형 순서를 계측한다.
- 원본 문서가 없으면 최소 RED 테스트를 먼저 만든다.
  - `tests/issue_1197_svg_object_zorder.rs`
  - synthetic `PageRenderTree` 또는 최소 fixture로 root 자식에 `BehindText` 표/그림/표/`InFrontOfText` 도형을 배치한다.
  - 기대 순서: z=1 표 < z=11 그림 < z=12 표 < z=69 도형.
  - 현재 구조에서는 표가 Flow plane으로 분류되어 그림 위에 남는 것을 RED로 확인한다.
- Stage 1 보고서:
  - `mydocs/working/task_m100_1197_stage1.md`

검증:

- RED 테스트가 실제 결함을 설명하는 방식으로 실패한다.
- #1167 테스트는 현재 기준으로 통과 상태임을 기록한다.

### Stage 2 - 렌더 레이어 메타데이터 계약 확정

목표: Picture/Table/Shape 공통 z-order 계약을 RenderNode에서 표현한다.

구현 후보는 다음 순서로 검토한다.

1. `RenderNode` 공통 optional 메타데이터 추가
   - 예: `layer: Option<RenderLayerInfo>`
   - 필드: `text_wrap`, `z_order`, `stable_index`
   - `#[serde(default, skip_serializing_if = "Option::is_none")]`로 기존 JSON 소비자 영향 축소
2. 노드별 필드 추가
   - `TableNode.text_wrap/z_order`
   - `ImageNode.z_order`
   - `RectangleNode/PathNode/EllipseNode/LineNode` 등 도형 leaf별 `text_wrap/z_order`

기본안은 1번이다. 도형은 하나의 컨트롤이 여러 leaf를 만들 수 있으므로, 노드별 필드보다
top-level RenderNode에 같은 layer 정보를 stamp하는 방식이 덜 취약하다.

수정 후보:

| 파일 | 변경 |
|------|------|
| `src/renderer/render_tree.rs` | `RenderLayerInfo` 및 RenderNode optional layer 필드 추가 |
| `src/renderer/layout.rs` | 용지/페이지 기준 Picture/Table/Shape emit 시 layer 정보 부여 |
| `src/renderer/svg.rs` | `node_z_plane()`이 `RenderNode.layer`를 우선 사용하도록 변경 |

검증:

- `cargo test --test issue_1167_svg_behindtext_zorder`
- RenderNode JSON 직렬화 테스트 또는 기존 paint/json 테스트 통과
- Stage 2 보고서:
  - `mydocs/working/task_m100_1197_stage2.md`

### Stage 3 - 용지 기준 객체 수집/정렬 보정

목표: `paper_images` 경로에서 Picture/Table/Shape가 같은 zOrder 축으로 정렬되도록 한다.

- `paper_images`의 실제 의미를 "paper/page anchored render nodes"로 정리한다.
  - 필요 시 이름 변경은 최소화하고, 먼저 정렬 계약만 고정한다.
- 그림 경로:
  - `Control::Picture`에서 `pic.common.text_wrap`, `pic.common.z_order`를 layer로 전달한다.
- 표 경로:
  - `Control::Table`에서 `table.common.text_wrap`, `table.common.z_order`를 layer로 전달한다.
  - `layout_table()`이 하나의 top-level TableNode를 반환하는지 확인하고, 다중 top-level이면 같은 layer를 모두 stamp한다.
- 도형 경로:
  - `ShapeObject`의 `common().text_wrap`, `z_order()`를 shape top-level node들에 stamp한다.
  - 글상자처럼 내부 텍스트를 자식으로 갖는 경우 top-level 그룹 단위가 함께 이동하는지 확인한다.
- root에 추가하기 전 정렬:
  - 기본 키: `(plane, z_order, stable_index)`
  - plane 순서: PageBackground < BehindText < Flow < InFrontOfText
  - 같은 zOrder에서는 기존 생성 순서 유지.

주의:

- Body 자체는 Flow plane으로 유지한다.
- BehindText 객체는 Body보다 먼저, InFrontOfText 객체는 Body보다 나중에 렌더되어야 한다.
- `TopAndBottom`, `Square` 등 본문 흐름/어울림 객체는 기존 Flow 처리와 커서 전진 계약을 유지한다.

검증:

- Stage 1 RED 테스트 GREEN.
- #1167 BehindText 워터마크 테스트 GREEN.
- 관련 SVG 문자열에서 z=1 표 < z=11 그림 < z=12 표 < z=69 도형 순서 단언.
- Stage 3 보고서:
  - `mydocs/working/task_m100_1197_stage3.md`

### Stage 4 - PaintOp/CanvasKit/native Skia 경로 영향 점검

목표: SVG만 고쳐서 backend별 레이어 계약이 갈라지는 일을 막는다.

현재 `PaintOp`에는 공통 layer 메타데이터가 없고, `paint_op_replay_plane()`은 `PaintOp::Image`만
`BehindText/InFrontOfText`로 분류한다. 따라서 Stage 3 후 다음을 판단한다.

- RenderTree root 정렬만으로 native Skia/CanvasKit 출력도 정합하는지 확인한다.
- PaintOp replay plane이 여전히 표/도형을 Flow로 밀어 올려 #1197과 같은 순서 역전을 만들면,
  PaintOp 또는 LayerNode 쪽에 layer 메타데이터를 추가한다.

확장 후보:

| 파일 | 변경 |
|------|------|
| `src/paint/paint_op.rs` | PaintOp 공통 layer accessor 또는 variant별 layer 전달 |
| `src/paint/builder.rs` | RenderNode.layer를 PaintOp/LayerNode로 전파 |
| `src/paint/replay_order.rs` | 이미지 외 표/도형 op도 layer 기준 plane 판정 |
| `src/renderer/skia/renderer.rs` | replay plane 순서가 layer 계약을 보존하는지 확인 |
| `src/renderer/canvaskit_policy.rs` | CanvasKit replay metadata 확인 |

확장 여부 기준:

- #1197 재현이 SVG 전용이면 PaintOp 구조 변경은 보류한다.
- native Skia/CanvasKit에서도 같은 레이어 역전이 재현되면 PaintOp까지 확장한다.

검증:

- `cargo test --lib replay_order`
- native Skia 관련 테스트 또는 기존 layer tree 테스트 통과
- Stage 4 보고서:
  - `mydocs/working/task_m100_1197_stage4.md`

### Stage 5 - 통합 검증, 시각 판정, 최종 보고

목표: 회귀와 시각 정합을 확정하고 작업 종료 문서를 작성한다.

자동 검증:

- `cargo fmt --all --check`
- `cargo test --test issue_1167_svg_behindtext_zorder`
- `cargo test --test issue_1197_svg_object_zorder -- --nocapture`
- `cargo test --tests`

재현 문서가 확보된 경우:

- `target/debug/rhwp export-svg "[2027] 온새미로 1 본교재.hwpx" -o output/poc/issue1197/page4 -p 3`
- PDF/한컴 기준 대조 산출물 보관
- 낮은 zOrder 표 텍스트(`PROLOGUE`, 출처 목록)가 전체 페이지 이미지 위에 남지 않는지 확인
- 중앙 `01` 장식과 하단 `1주차`, `- 누적과 연결`, `- 세계와 자아의 관계` 정합 확인

문서:

- Stage 5 보고서:
  - `mydocs/working/task_m100_1197_stage5.md`
- 최종 보고서:
  - `mydocs/report/task_m100_1197_report.md`
- 오늘 할일:
  - `mydocs/orders/20260602.md` 상태 갱신

## 단계별 커밋 규칙

- 각 stage는 소스 변경과 해당 `_stage{N}.md` 보고서를 함께 커밋한다.
- unrelated 미추적 파일(`task_m100_1142/1143/1144` 문서, `tmp/`)은 포함하지 않는다.
- 최종 보고서와 orders 갱신은 완료 단계 커밋에 포함한다.

## 리스크와 가드

| 리스크 | 가드 |
|------|------|
| BehindText 표를 Flow로 남겨 이미지 위에 텍스트가 보임 | `RenderLayerInfo` 기반 plane 판정과 zOrder 테스트 |
| `paper_images` root append 순서 변경으로 워터마크/직인 회귀 | #1167 테스트와 BehindText/InFrontOfText fixture 동시 검증 |
| PaintOp 경로와 SVG 경로 계약 분리 | Stage 4에서 native Skia/CanvasKit 영향 확인 후 확장 여부 결정 |
| JSON/wasm 소비자 필드 영향 | optional + skip_serializing_if 기본, 기존 JSON 테스트 확인 |
| 특정 문서 전용 보정 | 샘플명/ci 하드코딩 금지, `text_wrap/z_order` 계약 기반 처리 |

> 본 문서는 구현계획서이다. 승인 후 Stage 1부터 진행한다.
