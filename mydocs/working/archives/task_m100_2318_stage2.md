# #2318 2단계 완료보고 — rust 정정 (master_page provenance plane cap)

- 계획서: `mydocs/plans/task_m100_2318.md`
- 브랜치: `local/task2318`

## 계획 대비 변경점 2건

1. **skia 도 결함 범위였음** — 계획서의 "SVG/skia 정상" 중 skia 는 오판.
   skia 는 plane 재생(renderer.rs:419 ORDERED 순회)을 하므로 동일 증상 존재:
   정정 전 skia PNG 실측에서 바탕쪽 쪽번호 "1" 이 본문 "Ctrl+Y" 를 덮음
   (한컴 2022 PDF 는 "Ctrl+Y" 가 "1" 위). 본 정정으로 skia 도 함께 해소
   (정정 후 PNG 재실측: 한컴과 일치). SVG 만 node_z_plane 의 MasterPage
   plane 1 직접 배치(#1167)로 원래 정상.
2. **구현 방식 단순화** — 워커 5곳(web_canvas/skia/canvaskit_policy/
   plane summary/replay_order 스캐너)에 그룹 컨텍스트 boolean 을 각각 꿰는
   대신, `RenderLayerInfo` 에 `master_page: bool` provenance 필드를 추가하고
   분류기(`paint_op_replay_plane_with_layer`/`render_layer_replay_plane`)
   **한 지점**에서 cap. 모든 워커는 이미 layer 상속(node.layer.or(inherited))
   으로 분류기를 호출하므로 자동 반영 — 단일 진실 원천 유지.

## 수정 내용

| 파일 | 내용 |
|------|------|
| `src/renderer/render_tree.rs` | `RenderLayerInfo.master_page: bool` 추가 (serde: false 시 생략) + `for_master_page()` 헬퍼 |
| `src/paint/replay_order.rs` | `cap_master_page_plane` — Background 제외 전 plane 을 BehindText 상한 (#1167 SVG 계약과 동일 의미) |
| `src/renderer/layout.rs` | ①`render_layer_from_master_control` 에 flag 부여 (전지 배경 강등 가드 유지) ②mp_node 그룹에 provenance layer 부여 — layer 없는 자식(바탕쪽 텍스트 라인)도 상속으로 cap 적용 |
| `src/paint/json.rs` | layer JSON 에 `"masterPage":true` 조건부 방출 (3단계 studio TS 가 소비) |

바탕쪽 **내부** 정렬(`sort_paper_render_nodes`)은 원본 wrap 을 그대로 사용
하므로 불변 — 1단계 보존 가드 테스트로 고정.

## 게이트

- `issue_2318_master_page_plane` 2/2 **red→green** (cap + wrap 보존)
- `cargo test --lib` 2280/2280
- plane/layer 인접 통합 테스트 green: issue_516(2), issue_2222(9), issue_1144(4),
  issue_1143(2), issue_1017(1), issue_938(7), issue_948(3), issue_1187(3), issue_850(1)
- SVG z-order 계약 불변: issue_1167(3), issue_1197(1), svg_snapshot(1),
  issue_1271(1), issue_898(3), issue_1113(1), issue_1100(8)
- skia PNG 시각 실측: 정정 전 "1"이 텍스트 덮음 → 정정 후 한컴과 일치
  (`output/png/issue2318/{,after/}shortcut.png`)
- `cargo fmt --check` 통과 (수정 파일 한정)

## 다음 단계

3단계: studio TS 정정 — `replay-plane.ts`/`page-renderer.ts`(plane 집계)/
`canvaskit-renderer.ts` 에 layer.masterPage cap 반영 + studio 단위 테스트 +
WASM 빌드 + CDP e2e 스크린샷 검증.
