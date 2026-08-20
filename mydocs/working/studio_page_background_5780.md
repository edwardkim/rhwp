---
kind: working
status: active
issue: 5780
---

# studio 에서 flow 그림이 있는 쪽의 쪽 배경색이 사라지는 결함 (#5780)

작업 브랜치: `fix/5780-studio-page-background`
대상: `rhwp-studio/src/view/page-renderer.ts` · `rhwp-studio/tests/render-backend.test.ts`

## 한 줄

DOM 그림 갈래에도 Background plane 을 그린 canvas 를 그림 layer 아래에 깐다 — 그 갈래에는
쪽 배경을 실을 평면이 하나도 없었다.

## 이슈가 요구한 것

`prism_downloads/서울특별시 성북구/3070000-202200004_…자원순환집행계획.hwp` 는 구역 배경이
`#1c3d62` 다. `export-svg` 는 남색을 칠하는데 studio 는 흰 종이만 낸다. 1쪽은 표지 글자가
흰색이라 **완전히 빈 흰 쪽**으로 보인다.

원인은 flow-static 분리의 두 갈래 중 하나에 배경 자리가 없는 것이다.

| 갈래 | 아래 평면 | Background plane |
|---|---|---|
| `flowImages.length > 0` | **DIV** (`background: var(--doc-paper)` 하드코딩) | **없음** |
| else | `flow-static` **canvas** | `FlowStatic` 필터가 그린다 |

위 본문 canvas 는 `flow-dynamic` 으로 그리는데
`LayerFilter::FlowDynamic => replay_plane == Flow && !is_flow_static` 라 Background plane 이
통째로 빠지고 `transparent_page_background = true` 다.

수정 전 실측(headless studio, `origin/devel`):

```
layer tree   1·2·3쪽 모두 "type":"pageBackground" "backgroundColor":"#1c3d62"   ← WASM 정상
요약         1쪽 flowImageCount=1  2쪽 =2  3쪽 =0
canvas 픽셀  1쪽 본문 canvas(z=1) [0,0,0,0]        ← 투명, 아래에 배경 평면 없음
             2쪽 본문 canvas(z=1) [0,0,0,0]
             3쪽 canvas(z=0)      [28,61,98,255]  ← 분리를 안 해서 정상
```

`flowImageCount > 0` 인 쪽만 배경을 잃는다.

## 고친 방법

1. DOM 그림 갈래에서 `createOrReuseFilteredCanvasLayer(…, 'background', …)` 로 배경 canvas 를
   만들어 그림 DIV **아래**에 깐다. 둘 다 `z-index: 0` 이고 DOM 순서로 DIV 가 위에 온다 —
   뒤따르는 `behind`(1)/`front`(2·3) layer 의 z 계약을 건드리지 않는다.
2. 그림 DIV 의 `background` 하드코딩(`var(--doc-paper)`)을 없앤다. 그게 남아 있으면 아래
   배경 canvas 를 가린다. 배경이 없는 쪽의 흰 종이도 `BackgroundOnly` 렌더가 그린다
   (`should_render_page_background()` → `begin_page` 가 종이를 칠한다).
3. `!layers.hasBehind` 가지의 `removeOverlayLayer(…, 'background')` 를 `usesDomFlowImages` 일 때
   건너뛴다 — `hasFront` 인 쪽이 이 가지를 타는데, 지우면 그 쪽만 다시 배경을 잃는다.

## 만지지 않은 경로

- `LayerFilter` 술어(`web_canvas.rs`) — 평면 정의는 그대로 두고, 없던 평면을 studio 가 깔게
  했다.
- `flow-static` canvas 갈래 — 이미 Background plane 을 싣고 있어 무변경.
- 어느 쪽에 배경을 칠할지(구역 첫 쪽 한정)는 #5717 · PR #5745 의 몫이다. 이 변경은 **배경을
  잃지 않게** 할 뿐이다.

## 시험 명령

```bash
cd rhwp-studio && npx tsc --noEmit                # exit 0
cd rhwp-studio && node --test tests/*.test.ts     # 1002 passed / 1 skipped / 0 failed
cargo fmt --all -- --check                        # exit 0 (Rust 무변경)
cargo clippy --all-targets -- -D warnings         # exit 0 (Rust 무변경)
```

신규 계약: `rhwp-studio/tests/render-backend.test.ts` —
`[#5780] DOM flow-image pages still get a Background plane layer under the images`
(배경 layer 생성 · 제거 가드 · DIV 종이색 하드코딩 부재 4개 앵커).

## 수정 후 실측 (headless studio, 같은 문서)

```
page 1  background-0 canvas z=0  px=[28,61,98,255]   flow-images-0 DIV 투명   본문 canvas z=1 투명
page 2  background-1 canvas z=0  px=[28,61,98,255]   flow-images-1 DIV 투명   front-1 canvas z=2
page 3  (main) canvas z=0        px=[28,61,98,255]   ← 종전 경로 그대로
```

1쪽이 남색 배경 + 흰 글자로 `export-svg` 와 같아진다.

## PR 메모

`gh pr create --base devel --body-file` · `closes #5780`
