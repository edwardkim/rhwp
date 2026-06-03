# Stage 7 완료 보고서 - Task M100-1197

- 이슈: #1197
- 제목: HWPX 용지 기준 BehindText 그림/표 z-order 보존
- 브랜치: `local/task1197`
- 작성일: 2026-06-02
- 상태: 완료

## 1. 배경

Stage 6 이후 작업지시자가 원본 샘플을 다시 확인한 결과,
`01` 전면 표기는 표시됐지만 중앙 배경 그림이 사라졌다.

원인은 `rhwp-studio`가 BehindText page 를 합성할 때 flow canvas 를 투명하게 만들면서,
실제 `pageBackground` PaintOp 를 별도 layer 로 렌더하지 않고 단색 `div`만 배경으로 깔았기 때문이다.
샘플의 중앙 그림은 BehindText 객체가 아니라 page background plane 에 속하므로,
단색 div 로는 복구되지 않았다.

## 2. 변경 내용

- WASM `renderPageToCanvasFiltered()`에 `background` layer kind 를 추가했다.
- WebCanvas `LayerFilter::BackgroundOnly`를 추가해 page background plane 만 렌더할 수 있게 했다.
- `rhwp-studio` PageRenderer가 BehindText page 에서 단색 background div 대신
  `renderPageToCanvasFiltered(page, canvas, scale, 'background')` canvas 를 z-index 0에 배치하도록 변경했다.
- 지연 재렌더링도 `background`/`behind`/`front` filtered canvas 를 함께 갱신하도록 확장했다.
- `WasmBridge.renderPageToCanvasFiltered` TypeScript 계약에 `background`를 추가했다.
- `rhwp-studio` source-level 테스트가 background/behind/front filtered canvas 생성을 함께 확인하도록 보강했다.

## 3. 검증

통과한 명령:

```sh
npm test
npm run build
cargo fmt --all --check
cargo test --lib replay_order
cargo test --test issue_1197_svg_object_zorder -- --nocapture
cargo test --test issue_1167_svg_behindtext_zorder -- --nocapture
git diff --check
wasm-pack build --target web
```

비고:

- #1167 테스트는 기존 `LAYOUT_OVERFLOW` 진단 1건을 출력하지만 assertion 은 통과했다.
- sandbox 안의 `wasm-pack build --target web`는 `wasm-bindgen` 설치/실행 단계에서 권한 오류가 발생해,
  승인 경로에서 동일 명령을 재실행해 `pkg/`를 갱신했다.
- `rhwp-studio` 개발 서버는 새 WASM 산출물을 반영하도록 재시작했고,
  `http://127.0.0.1:7700/`에서 브라우저 로드, Vite overlay 없음, console error/warn 없음까지 확인했다.

## 4. 재검증 요청

작업지시자는 브라우저에서 hard reload 후 원본 샘플 파일을 다시 로드해 확인한다.

확인 기준:

- 중앙 배경 그림이 다시 표시된다.
- `01` 표기가 중앙 배경 위 전면 plane 에 표시된다.
- 마지막 하단 문구가 누락되지 않는다.
