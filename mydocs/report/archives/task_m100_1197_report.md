# 최종 보고서 - Task M100-1197

- 이슈: #1197
- 제목: HWPX 용지 기준 BehindText 그림/표 z-order 보존
- 브랜치: `local/task1197`
- 작성일: 2026-06-02
- 상태: 작업지시자 원본 시각검증 완료, 작업 완료

## 1. 문제

HWPX 문서의 용지/페이지 기준 anchored 객체가 Picture/Table/Shape 타입별 경로로 렌더되면서
같은 `textWrap`/`zOrder` 축에서 합성되지 않았다. 그 결과 낮은 z-order BehindText 표 텍스트가
전체 페이지 이미지 위에 다시 표시되고, InFrontOfText 도형도 의도한 순서로 보존되지 않을 수 있었다.

## 2. 해결

`RenderNode` 공통 레이어 메타데이터를 추가하고, layout/SVG/PaintOp 경로가 동일한 레이어 계약을 따르도록 연결했다.

주요 변경:

- `RenderLayerInfo { text_wrap, z_order, stable_index }` 추가
- 용지/페이지 기준 Picture/Table/Shape top-level node에 layer metadata stamp
- paper/page anchored render node 정렬 키를 `(plane, z_order, stable_index)`로 통일
- SVG renderer가 `RenderNode.layer`를 우선 사용해 plane/z-order 정렬
- `LayerNode.layer` 추가 및 `LayerBuilder` lowering 시 metadata 보존
- PaintOp replay plane 계산을 `paint_op_replay_plane_with_layer()`로 확장
- CanvasKit/native Skia/WebCanvas가 inherited layer 기준으로 replay plane을 판단
- PageLayerTree JSON에 optional `layer` metadata 직렬화
- `rhwp-studio` Canvas2D 합성기가 BehindText/InFrontOfText 를 이미지 overlay 가 아니라 filtered canvas layer 로 표시
- TypeScript CanvasKit renderer가 `LayerNode.layer` metadata 를 상속해 non-image PaintOp plane 을 판단
- `rhwp-studio` Canvas2D 합성 순서를 `pageBackground canvas → BehindText canvas → flow canvas → InFrontOfText canvas`로 보정
- WASM `renderPageToCanvasFiltered('background')`와 WebCanvas `LayerFilter::BackgroundOnly` 추가
- `rhwp-studio` filtered overlay canvas 가 공통 CSS의 opaque canvas background 를 상속하지 않도록 투명 배경을 명시

## 3. 검증

통과한 주요 명령:

```sh
npm test
npm run build
cargo fmt --all --check
cargo test --test issue_1167_svg_behindtext_zorder -- --nocapture
cargo test --test issue_1197_svg_object_zorder -- --nocapture
cargo test --tests
cargo test --features native-skia --lib behind_text_layered_vector_replays_below_flow_across_tree_branches -- --nocapture
cargo test --lib replay_order
git diff --check
wasm-pack build --target web
```

PR 생성 직전 최신 `upstream/devel` rebase 후 추가로 통과한 명령:

```sh
cargo fmt --all -- --check
cargo test
cargo clippy -- -D warnings
npm test
npm run build
git diff --check
```

비고:

- #1167 테스트는 기존 `LAYOUT_OVERFLOW` 진단 1건을 출력하지만 assertion은 통과했다.
- Stage 9에서 작업지시자 확인에 따라 재현 샘플 HWPX/HWP와 참조 PDF를 PR #1252에 포함했다.
- 작업지시자 실서버 검증에서 드러난 `rhwp-studio` Canvas2D 소비자 누락은 Stage 6에서 보정했다.
- Stage 6 후 `01`은 표시됐지만 중앙 배경 그림이 사라지는 문제가 남아, Stage 7에서 page background filtered canvas 를 추가했다.
- Stage 7 후에도 `01`만 보이는 문제는 `front` overlay canvas 가 `#scroll-content canvas`의 흰 배경을 상속해 하위 layer 를 덮는 문제였고, Stage 8에서 overlay canvas 배경을 투명하게 고정했다.
- 제공 PDF는 46쪽, HWPX rhwp pagination 은 47쪽, HWP rhwp pagination 은 50쪽으로 확인됐다. PDF 2쪽 `MEMO`는 HWPX rhwp 3쪽, PDF 3쪽 `01 / 1주차`는 HWPX rhwp 4쪽에 대응한다. 이 page count 차이는 별도 pagination/partial table 문제로 판단한다.
- 작업지시자 원본 시각검증에서 중앙 배경 이미지, `01` 전면 표기, 하단 `1주차` 및 설명 텍스트 표시를 확인했다.
- Docker daemon 이 실행 중이 아니어서 Docker WASM 빌드는 수행하지 못했고, 로컬 `wasm-pack build --target web`로 `pkg/`를 갱신했다.

## 4. 시각검증 산출물

작업지시자 확인용 산출물:

- `samples/hwpx/[2027] 온새미로 1 본교재.hwpx`
- `samples/hwpx/hancom-hwp/[2027] 온새미로 1 본교재.hwp`
- `pdf-large/hwpx/[2027] 온새미로 1 본교재.pdf`
- `output/poc/issue1197/visual_check.html`
- `output/poc/issue1197/synthetic/issue1197_synthetic_zorder.svg`
- `output/poc/issue1197/issue1167/복학원서.svg`

확인 기준:

- #1197 synthetic: 낮은 `Z01_LOW_TABLE`은 파란 `Z11 IMAGE` 아래에 가려지고, `Z12_FINAL_TABLE`과 `01`은 위에 보인다.
- #1167 실제 샘플: BehindText 워터마크가 본문 텍스트를 덮지 않는다.

## 5. 커밋

- `44e77247` Task #1197: add z-order red test
- `a4f33ff1` Task #1197: add render layer metadata
- `114d6e30` Task #1197: stamp paper object layers
- `55e406d7` Task #1197: replay layered paint order
- `fbd482a3` Task #1197: document final verification
- `b2917fe0` Task #1197: fix studio layer replay
- `3e18f720` Task #1197: add studio background layer
- `6ca5a42a` Task #1197: keep studio overlays transparent
- `6252178f` Task #1197: finalize completion report

Stage 8 커밋은 `rhwp-studio` overlay canvas 투명 배경 보정, Stage 8 완료보고서, orders/최종 보고서 갱신을 포함한다.
마무리 커밋은 작업지시자 원본 시각검증 완료 상태를 최종 보고서와 orders 에 반영한다.
PR 생성 기록 커밋은 PR #1252 생성 상태를 최종 보고서와 orders 에 반영한다.
샘플 포함 커밋은 Stage 9 완료보고서와 HWPX/HWP/PDF 샘플 3종을 PR #1252에 반영한다.

## 6. 남은 결정

현재 작업 범위는 완료했다. 다음 항목은 별도 승인 후 진행한다.

- PR: [#1252](https://github.com/edwardkim/rhwp/pull/1252) (`postmelee:local/task1197` → `edwardkim/rhwp:devel`, Draft)
- 작업지시자 승인 시 issue close
- PDF 46쪽 vs rhwp 47쪽 page count 차이 별도 pagination 후속 이슈 처리
