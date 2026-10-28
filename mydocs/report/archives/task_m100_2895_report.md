# Task #M100-2895 처리 결과

## 이슈
edwardkim/rhwp#2895 — 동일 이미지 겹침-clip 함수가 트리 순서를 z-순서로 오인해 plane 이 다른 이미지 쌍에서 역방향으로 잘림

## 원인
`PageRenderTree::clip_overlapping_same_bin_images` (`src/renderer/render_tree.rs`, Task #1154 도입)가
동일 `bin_data_id` 이미지 페어의 clip 방향을 "RenderTree 순회(children 삽입) 순서 = 실제 페인트 순서"라는
암묵적 가정으로 판단했다. 그러나 `src/paint/replay_order.rs` 가 정의하는 실제 재생 순서는
`Background → BehindText → Flow → InFrontOfText` 4단계 plane 으로 분리되어 있어, `text_wrap` 이 다른
두 이미지는 트리 순서와 무관하게 plane 순서로 재생된다. 트리 순서상 먼저 오는 Flow 이미지와 나중에
오는 BehindText 이미지가 겹치면, 실제로는 BehindText 가 더 아래에 그려짐에도 기존 함수는 Flow 쪽을
"아래에 깔린 쪽"으로 오판해 역방향으로 clip 했다.

## 수정
- `src/renderer/render_tree.rs`
  - `ClipReplayPlane` (BehindText/Flow/InFrontOfText) 타입 추가 — `ImageNode.text_wrap` 으로부터
    `paint_op_replay_plane_with_layer` 와 동일한 3-way 분류를 적용.
  - `collect_image_nodes` 가 각 이미지의 plane 을 함께 수집하도록 튜플 확장.
  - `clip_overlapping_same_bin_images` 의 페어 검출 루프에 `plane_a != plane_b` 가드를 추가해,
    plane 이 다른 페어는 clip 대상에서 제외(보수적 최소 수정 — 정확한 역방향 clip 계산 대신 미적용).

## 테스트 (red → green)
`test_clip_skips_cross_plane_pairs` (`src/renderer/render_tree.rs` tests 모듈) 추가.

- Red (가드 임시 비활성화 후 `cargo test`): `assertion left == right failed / left: 50.0 / right: 200.0`
  — Flow 이미지가 역방향으로 50 까지 clip 됨을 실측.
- Green (가드 복원 후 `cargo test`): 두 이미지 모두 원래 height(200.0) 유지, 11개 clip 관련 테스트 전부 통과.

## 검증
- `cargo build --lib`: 성공
- `cargo test --lib render_tree::tests::test_clip`: 11 passed
- `cargo clippy --all-targets --profile release-test -- -D warnings`: 경고 없음
- `rustfmt --edition 2021 src/renderer/render_tree.rs`: 적용 (개행 스타일 외 diff 없음)

## 영향 범위
`src/renderer/render_tree.rs`, `src/renderer/render_tree.rs` 내 테스트만 변경. 다른 렌더러/직렬화 코드는
건드리지 않았으며, 기존 clip 로직(같은 plane 내 동일-bin 이미지 겹침)은 그대로 유지된다.
