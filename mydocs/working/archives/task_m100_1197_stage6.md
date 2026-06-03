# Stage 6 완료 보고서 - Task M100-1197

- 이슈: #1197
- 제목: HWPX 용지 기준 BehindText 그림/표 z-order 보존
- 브랜치: `local/task1197`
- 작성일: 2026-06-02
- 상태: 완료

## 1. 배경

작업지시자가 `rhwp-studio` 서버에서 원본 샘플을 직접 로드해 확인한 결과,
중앙 배경 이미지는 보였지만 `01` 표기와 마지막 하단 문구가 누락되는 문제가 남아 있었다.

원인은 core layer replay 계약은 보강됐지만, `rhwp-studio` Canvas2D 페이지 합성기가
BehindText/InFrontOfText 이미지만 DOM `<img>` overlay 로 다시 붙이고 있었기 때문이다.
Stage 4에서 flow 렌더에서 BehindText/InFrontOfText plane 을 제외했으므로,
이미지가 아닌 BehindText 표와 InFrontOfText 도형/글상자는 flow 에서 빠진 뒤
별도 plane 에서도 그려지지 않았다.

## 2. 변경 내용

- `rhwp-studio` Canvas2D 렌더러가 BehindText/InFrontOfText 를 이미지 overlay 가 아니라
  filtered canvas layer 로 합성하도록 변경했다.
- `PageLayerTree`의 `LayerNode.layer` metadata 를 TypeScript 타입에 추가했다.
- CanvasKit renderer도 group/clip/leaf 노드의 inherited layer 를 따라
  non-image PaintOp 의 replay plane 을 판단하도록 보강했다.
- WebCanvas `WrapOnly` 렌더가 별도 plane canvas 에 그려질 때 흰 페이지 배경을 다시 칠하지 않도록
  transparent background 처리를 추가했다.
- 이미지 비동기 디코드 재시도는 flow canvas 와 behind/front canvas 를 함께 다시 그리도록 변경했다.
- `rhwp-studio` 단위 테스트에 layer metadata 우선순위와 filtered canvas layer 사용 검사를 추가했다.

## 3. 검증

통과한 명령:

```sh
npm test
npm run build
cargo fmt --all --check
cargo test --test issue_1197_svg_object_zorder -- --nocapture
cargo test --test issue_1167_svg_behindtext_zorder -- --nocapture
cargo test --lib replay_order
git diff --check
wasm-pack build --target web
```

비고:

- #1167 테스트는 기존 `LAYOUT_OVERFLOW` 진단 1건을 출력하지만 assertion 은 통과했다.
- Docker WASM 빌드는 Docker daemon 이 실행 중이 아니어서 수행하지 못했고,
  로컬 `wasm-pack build --target web`로 `pkg/`를 갱신했다.
- `rhwp-studio` 개발 서버는 `http://127.0.0.1:7700/`에서 재시작해 두었다.

## 4. 재검증 요청

작업지시자는 브라우저에서 hard reload 후 원본 샘플 파일을 다시 로드해 확인한다.

확인 기준:

- 중앙 배경 이미지는 기존처럼 표시된다.
- `01` 표기와 마지막 하단 문구가 누락되지 않는다.
- BehindText 표/도형이 본문 흐름을 덮거나 사라지지 않는다.
