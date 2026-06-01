# 구현계획서 — Task M100-1187: BookReview.hwp 글상자 clip 회귀 (5단계)

## 설계 요약

- 기존 `RenderNodeType::TextBox` 를 실제 글상자 콘텐츠 컨테이너로 사용한다.
- `layout_textbox_content` 는 계산된 `inner_area` 를 `TextBox` 노드의 `bbox` 로 두고, 글상자 내부 문단/인라인 개체/중첩 표를 그 노드 자식으로 배치한다.
- 도형 자체(`Rectangle`, `Path` 등)의 배경/테두리는 기존 shape node 에 남긴다. clip 은 shape 전체가 아니라 `TextBox` 콘텐츠에만 적용한다.
- SVG exporter 는 `RenderNodeType::TextBox` 에 대해 `textbox-clip-{node.id}` clipPath 를 생성하고 자식 그룹에 적용한다.
- paint layer builder 는 `TextBox` 노드를 `ClipRect` 로 감싸며 `ClipKind::TextBox` 를 추가해 Body/TableCell 과 구분한다.
- 원본 HWP 의 `line_seg.vertical_pos` 기반 배치는 유지한다. 이번 작업은 overflow 문단 삭제/재배치가 아니라 렌더 clip 보강이다.

## Stage 1 — RED 회귀 테스트와 현재 출력 구조 고정

**목표**: `BookReview.hwp` 회귀를 자동 테스트로 고정한다.

- 신규 테스트: `tests/issue_1187_textbox_clip.rs`
  - `samples/basic/BookReview.hwp` 1쪽 SVG 렌더.
  - SVG 에 `textbox-clip-` clipPath 가 존재해야 함을 단언.
  - 첫 글상자 clip rect 가 큰 점선 사각형 내부 텍스트 영역에 가까운지 확인:
    - 예상 범위: `x≈47.9`, `y≈516.6`, `w≈687.5`, `h≈487.9`.
    - 부동소수 출력 포맷 차이를 고려해 느슨한 범위 파싱.
  - author box 텍스트(`강우신지음`, `원앤원북스...`)가 SVG text sequence 에 남아 있는지 확인.
- 현재 `upstream/devel` 기준 RED 확인:
  - `textbox-clip-` 가 없어 실패해야 정상.
- 검증:
  - `cargo test --test issue_1187_textbox_clip`
- 보고서:
  - `mydocs/working/task_m100_1187_stage1.md`

## Stage 2 — 레이아웃: 글상자 콘텐츠를 TextBox 노드 아래로 이동

**목표**: 글상자 내부 콘텐츠만 clip 가능한 RenderNode 하위로 모은다.

- 파일: `src/renderer/layout/shape_layout.rs`
- `layout_textbox_content` 초반에서 `inner_area` 계산 직후:
  - `RenderNode::new(tree.next_id(), RenderNodeType::TextBox, BoundingBox::new(inner_area.x, inner_area.y, inner_area.width, inner_area.height))` 생성.
  - 이후 글상자 내부 콘텐츠 parent 로 `shape_node` 대신 `textbox_node` 사용.
- 적용 대상:
  - 일반 가로쓰기 문단의 `layout_composed_paragraph`.
  - overflow 수신 텍스트박스 분기.
  - 세로쓰기 분기 `layout_vertical_textbox_text_with_paras`.
  - 인라인/절대 picture, equation fallback, nested shape, embedded table.
- 주의:
  - 도형의 이미지 채우기, 배경, 테두리 노드는 shape node 에 유지.
  - 글상자 콘텐츠 배치 완료 후 `shape_node.children.push(textbox_node)`.
  - `drawing.text_box == None` 인 경우에는 기존처럼 아무 노드도 만들지 않음.
  - 빈 글상자라도 clip 노드가 과도하게 생기는 것이 문제되면, children 이 있을 때만 push 한다.
- 검증:
  - `cargo test --test issue_1187_textbox_clip` 는 아직 SVG clip 미적용 전이면 부분 실패 가능.
  - 렌더 트리 덤프 또는 단위 테스트로 `TextBox` 노드 생성 여부 확인.
- 보고서:
  - `mydocs/working/task_m100_1187_stage2.md`

## Stage 3 — SVG exporter: TextBox clipPath 적용

**목표**: SVG 출력에서 글상자 콘텐츠가 `inner_area` 밖으로 보이지 않게 한다.

- 파일: `src/renderer/svg.rs`
- `render_node` 진입부 match 에 `RenderNodeType::TextBox` 분기 추가:
  - `clip_id = format!("textbox-clip-{}", node.id)`
  - `<clipPath id="textbox-clip-{id}"><rect x=... y=... width=... height=.../></clipPath>`
  - 자식 렌더 전 `<g clip-path="url(#textbox-clip-{id})">` 시작.
- `render_node` 종료부에서 `RenderNodeType::TextBox` 일 때 `</g>` 닫기.
- 기존 debug overlay skip depth 에서 `TextBox` 는 계속 제외 유지.
- 조판부호 `[글상자]` 마커가 clip 안/밖 어느 쪽에 있어야 하는지 확인:
  - 기본 방침은 콘텐츠 컨테이너 성격에 맞춰 clip 안에 두되, 테스트에서 marker 존재 여부에 의존하지 않는다.
- 검증:
  - `cargo test --test issue_1187_textbox_clip`
  - `cargo build --bin rhwp`
  - `target/debug/rhwp export-svg samples/basic/BookReview.hwp -p 0 --debug-overlay --show-control-codes --show-grid -o output/debug/task1187`
- 보고서:
  - `mydocs/working/task_m100_1187_stage3.md`

## Stage 4 — paint layer: TextBox ClipRect 반영

**목표**: SVG 외 paint/replay 경로도 동일하게 글상자 clip 을 적용한다.

- 파일: `src/paint/layer_tree.rs`
  - `ClipKind::TextBox` 추가.
- 파일: `src/paint/builder.rs`
  - `RenderNodeType::TextBox` 분기 추가.
  - child 는 `GroupKind::TextBox` group 으로 만들고, 바깥은 `LayerNode::clip_rect(node.bbox, Some(node.id), node.bbox, child, ClipKind::TextBox)` 로 감싼다.
  - 기존 Body/TableCell clip 동작은 유지.
- 단위 테스트:
  - `builder.rs` test module 에 TextBox 노드가 `LayerNodeKind::ClipRect { clip_kind: ClipKind::TextBox }` 로 변환되는 테스트 추가.
- 검증:
  - `cargo test paint::builder`
  - `cargo test --test issue_1187_textbox_clip`
- 보고서:
  - `mydocs/working/task_m100_1187_stage4.md`

## Stage 5 — 통합 검증과 최종 정리

**목표**: BookReview 회귀 해소와 관련 경로 무회귀를 확인한다.

- 자동 검증:
  - `cargo fmt --check`
  - `cargo build --bin rhwp`
  - `cargo test --test issue_1187_textbox_clip`
  - `cargo test paint::builder`
  - 필요 시 글상자 관련 기존 테스트:
    - `cargo test --test issue_1052_footnote_in_textbox`
    - `cargo test --test issue_919_textbox_hit_test`
    - `cargo test --test issue_1028_hwpx_textbox_vertical`
- 시각/산출물 확인:
  - `BookReview.hwp` 1쪽 SVG 재생성.
  - 큰 점선 글상자 하단 밖의 5장/6장/에필로그 텍스트가 시각적으로 보이지 않는지 확인.
  - 우측 하단 author box 는 계속 보이는지 확인.
  - SVG 에 `textbox-clip-` 이 존재하는지 확인.
- 최종 문서:
  - `mydocs/report/task_m100_1187_report.md`
  - `mydocs/orders/20260531.md` 상태/비고 갱신

## 단계별 커밋 규칙

- 각 stage 완료 후 해당 소스 변경과 `mydocs/working/task_m100_1187_stage{N}.md` 를 함께 커밋한다.
- 최종 보고서와 오늘할일 갱신도 `local/task1187` 브랜치에서 커밋한다.
- 이슈 #1187 close 는 작업지시자 승인 후에만 수행한다.

## 리스크 점검

- `TextBox` clip 을 shape node 전체에 적용하면 배경/테두리/author box 와 같은 별도 도형이 잘릴 수 있으므로 콘텐츠 노드에만 적용한다.
- 세로쓰기와 overflow 수신 분기는 early return 이 있어 `textbox_node` push 누락 위험이 있다. 각 분기별로 push 경로를 명시적으로 확인한다.
- nested shape/table 이 shape node 아래에 남으면 clip 이 적용되지 않는다. 글상자 내부에서 생성되는 모든 자식 parent 를 `textbox_node` 로 통일한다.
- SVG 와 paint layer 의 clip 의미가 달라지면 멀티 렌더러 회귀가 난다. 같은 bbox(`inner_area`)를 사용한다.

## Stage 6 — PR 시각 피드백 반영: 글상자 vpos 중복 보정

**목표**: #1190 1차 수정 후 정상 목차 줄까지 잘리는 문제를 보정한다.

- 문제:
  - 큰 글상자 하단의 `5장`, `6장`, `에필로그` 줄이 한컴 기준에서는 보여야 하지만, PR 1차 수정본에서는 clip 밖으로 밀려 보이지 않았다.
  - `layout_textbox_content` 가 `line_seg.vertical_pos` 로 explicit y 를 이미 계산한 뒤 `layout_composed_paragraph` 의 column-top fallback 이 글상자 첫 문단에도 vpos 를 다시 더했다.
- 수정:
  - `src/renderer/layout/paragraph_layout.rs`
    - column-top `spacing_before`/`line_seg.vertical_pos` fallback 을 `cell_ctx.is_none()` 인 본문 흐름에만 적용한다.
  - `tests/issue_1187_textbox_clip.rs`
    - `5장`, `6장`, `에필로그` 목차 줄이 큰 글상자 clip bottom 안에 있는지 검사한다.
- 검증:
  - `cargo fmt --all -- --check`
  - `cargo build --bin rhwp`
  - `cargo test --test issue_1187_textbox_clip`
  - 관련 글상자/paint/svg layer 테스트
  - `wasm-pack build --target web --dev`
  - `npm run build`
  - rhwp-studio Browser 시각 확인

## Stage 7 — PR #1190 CI snapshot 실패 보정

**목표**: Stage 6 보정이 표 셀 레이아웃까지 건드린 부작용을 제거하고,
의도한 텍스트박스 clip 출력 변화만 snapshot golden 에 반영한다.

- 문제:
  - GitHub Actions `Build & Test` 에서 `cargo test --test svg_snapshot` 실패.
  - 실패 케이스: `issue_267_ktx_toc_page`, `issue_617_exam_kor_page5`.
  - Stage 6 의 `cell_ctx.is_none()` 조건은 글상자뿐 아니라 표 셀 문단에도
    영향을 줘 표 셀 column-top vpos fallback 을 꺼버렸다.
- 수정:
  - `layout_composed_paragraph` 에 `suppress_column_top_vpos_fallback` 플래그를 추가한다.
  - 글상자 내부 가로쓰기 문단 호출만 `true` 로 넘긴다.
  - 일반 본문/표 셀/캡션/각주는 `false` 로 유지한다.
  - 텍스트박스 clip 도입으로 의도적으로 바뀐 SVG snapshot golden 2개를 갱신한다.
- 검증:
  - `cargo test --test svg_snapshot`
  - `cargo test --test issue_1187_textbox_clip`
  - 관련 글상자/paint/svg layer 테스트
  - `cargo build --bin rhwp`
  - `wasm-pack build --target web --dev`
  - `npm run build`
