# Task M100 #2217 Stage 6 - 초기 페이지 렌더 트리 중복 제거

## 목표

`samples/issue2217/20200830.hwp`를 실제 Chrome에서 열 때 상태 표시가
`82% - 페이지 렌더 준비 중...`에 장시간 머무르는 문제를 줄인다.

## 재현 및 분석

- 실제 Chrome 새 탭에서 같은 문서를 다시 열면, 82% 이후 첫 페이지 렌더가
  브라우저 메인 스레드를 장시간 점유한다.
- Canvas2D 초기 렌더는 overlay 요약, `flow-dynamic` 캔버스, flow 이미지 추출을
  위해 동일 페이지의 `PageRenderTree`를 여러 번 복제하고 `PageLayerTree`를 다시
  만든다.
- `build_page_layer_tree()`는 캐시된 `PageRenderTree`를 값으로 반환받아 매 호출마다
  깊은 복제를 수행한다. 이미지가 없는 경우에도 flow 이미지 JSON을 다시 요청한다.
- 이후 화면 단계 계측에서 Canvas2D 첫 페이지 자체는 약 0.5초였고, 실제 장기 대기는
  `Toolbar.initFontDropdown()`이 저장된 로컬 글꼴 약 2,700개를 native select option으로
  동기 삽입하면서 다음 paint를 막는 데서 발생했다. 마지막으로 그려진 82% 상태가 계속
  남아 Canvas 렌더 지연처럼 보였다.

## 변경 범위

1. `PageLayerTree` 생성 시 캐시된 `PageRenderTree`를 참조로 사용해 불필요한 깊은
   복제를 없앤다.
2. flow 이미지가 없는 페이지에서는 flow 이미지 JSON을 요청하지 않는다.
3. 로컬 글꼴 option은 문서 초기화가 아니라 글꼴 목록을 여는 시점에 fragment로 한 번만
   생성한다.
4. 기존 페이지 트리 캐시 무효화 경계와 초기/편집 렌더 동작을 회귀 검증한다.

## 검증 계획

- 관련 Rust 단위 테스트와 Studio focused 테스트를 수행한다.
- WASM build 후 실제 Chrome에서 `20200830.hwp` 재로딩 시간을 다시 확인한다.

## 검증 결과

- `cargo fmt --check`: passed.
- `cargo test --profile release-test --test issue_938 issue_938_overlay_watermark_is_hancom_baked_png`: passed.
- `cargo test --profile release-test --test issue_850_answer_sheet_name_hit_test issue_850_exam_social_overlay_images_api_stays_compact_for_input_loop`: passed.
- `node --test tests/render-backend.test.ts tests/document-initialization-order.test.ts tests/toolbar-local-font-options.test.ts`: 40 passed.
- `npm run build`: passed.
- `wasm-pack build --target web --out-dir pkg`: passed.
- 실제 Chrome 단일 새 탭, 저장된 로컬 글꼴이 있는 상태에서 `20200830.hwp`를 연 뒤
  5.3초에 4페이지 완료와 활성 textarea를 확인했다. 초기 로컬 글꼴 option은 0개이며,
  글꼴 목록을 열면 670개를 지연 생성하는 것을 확인했다.
