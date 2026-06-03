# Stage 2 보고서 — Task M100-1167: svg.rs plane multi-pass 순회

## 목표

SVG `render_node` 를 z-order plane 순서로 순회하여 BehindText 워터마크를 본문 뒤로 정합 (GREEN).

## 변경

### `src/renderer/svg.rs`

1. **`node_z_plane(node) -> u8`** (신규): 노드의 z-order plane 키.
   - `PageBackground` → 0 (가장 먼저=아래)
   - `Image` with `text_wrap=BehindText` → 1
   - 그 외(텍스트·표·일반 그림) → 2 (Flow)
   - `Image` with `text_wrap=InFrontOfText` → 3 (가장 나중=위)
   - PaintOp `paint_op_replay_plane()` 과 동일 의미.

2. **`children_need_plane_reorder(node) -> bool`** (신규): 자식 중 plane≠2 가 있을 때만 정렬 (대부분 Flow 만 가지므로 정렬 비용 회피).

3. **`render_node` 자식 순회**: 재정렬 필요 시 `sort_by_key(node_z_plane)` (안정 정렬 — 같은 plane 내 트리 순서 보존) 후 순회.

## 1차 회귀 발견·정정 (작업지시자 시각 판정 게이트)

초기 구현은 `node_z_plane` 에 PageBackground 분류가 없어, BehindText 워터마크(plane 1)가 PageBackground(plane 2 로 오분류)보다 앞으로 정렬됨 → **흰 배경 rect 가 워터마크를 덮어 워터마크가 안 보임**.

- 작업지시자 시각 판정: "워터마크 이미지가 보이지 않습니다."
- 정정: `PageBackground → plane 0` 추가. 순서가 `흰배경(0) → 워터마크(1) → 본문(2) → 직인(3)` 으로 정상화.
- 재판정: "이제 워터마크가 보입니다." + "actual.svg 가 정답."

## 검증

| 항목 | 결과 |
|------|------|
| issue_1167 회귀 가드 | ✅ ok (BehindText `<image>` < 본문 첫 `<text>`) |
| 정량 (정정 후 SVG) | 흰배경 줄 40 < 워터마크 41·42 < 본문 48 |
| svg_snapshot 전체 | ✅ 8 passed (issue_677 골든 정답 상태로 갱신) |
| 작업지시자 시각 판정 | ✅ 워터마크 보임 + actual.svg 정답 확정 |

## 골든 갱신

`tests/golden_svg/issue-677/bokhakwonseo-page1.svg` 를 정정 상태(워터마크 본문 앞)로 `UPDATE_GOLDEN=1` 갱신. 종전 골든은 z-order 결함(워터마크 줄 810=본문 뒤)을 박제하고 있었음.

## 산출물

- `output/poc/issue_1167/복학원서.svg` (시각 판정 통과)

## 다음 (Stage 3)

통합 검증(--tests 전수 + native-skia + fmt + clippy) + InFrontOfText(인 서명) plane 정합 확인 + 최종 보고서.
