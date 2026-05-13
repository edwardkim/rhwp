# Task #850 Stage 6 완료 보고서

## 단계 목표

`samples/exam_social.hwp` 성명 칸 입력은 Stage 5에서 정상화되었지만, 사용자 검증에서 입력 지연이 확인되었다. 이번 단계의 목표는 #850 수정이 성능 문제를 새로 만든 것인지 분리하고, 입력 루프에서 불필요한 page tree 재구성, 대형 JSON 생성, 지연 재렌더 예약을 제거하는 것이다.

## 원인 분석

`v0.7.10`에서는 성명 칸 hit-test가 중첩 표 path로 진입하지 않았기 때문에 느린 path 기반 입력 경로가 노출되지 않았다. `v0.7.11`의 #717 이후 빈 셀 hit-test가 `cellPath`를 반환하면서 성명 칸 입력도 `insertTextInCellByPath`와 `getCursorRectByPath` 경로를 사용하게 되었고, 이때 다음 비용이 드러났다.

- `getCursorRectByPath`: 매 입력마다 `build_page_tree()`를 새로 호출했다.
- `PageRenderer.applyOverlays`: 매 렌더마다 `getPageLayerTree()`를 호출해 page 0 기준 약 1.4MB JSON을 생성하고 파싱했다.
- `scheduleReRender`: 이미지가 없는 flow 렌더까지 지연 재렌더 후보로 취급할 수 있는 구조였고, 같은 이미지 수인 경우에도 기존 타이머를 먼저 취소했다.

따라서 #850의 정확성 수정이 직접 지연을 만든 것은 아니다. 다만 #850 수정으로 입력이 실제로 성공하게 되면서, #717 이후 노출된 nested path 입력 경로의 기존 비용이 사용자에게 체감된 것이다.

## 수정

수정 파일:

- `src/document_core/queries/cursor_rect.rs`
- `src/document_core/queries/rendering.rs`
- `src/wasm_api.rs`
- `rhwp-studio/src/core/wasm-bridge.ts`
- `rhwp-studio/src/view/page-renderer.ts`
- `rhwp-studio/src/view/canvas-view.ts`

수정 내용:

1. `get_cursor_rect_by_path_native()`가 `build_page_tree_cached()`를 사용하도록 변경했다.
2. `build_page_layer_tree()`도 캐시된 page tree를 사용하도록 변경했다.
3. 입력 렌더 루프용 `getPageOverlayImages(pageNum)` WASM API를 추가했다.
   - behind/front overlay image만 JSON으로 반환한다.
   - flow image는 base64 payload를 포함하지 않고 `imageCount`만 계산한다.
4. Studio `PageRenderer`가 우선 `getPageOverlayImages()`를 사용하고, 구버전 WASM일 때만 `getPageLayerTree()`로 fallback하도록 했다.
5. flow 렌더에서는 image retry를 예약하지 않도록 `imageCount=0`을 전달했다.
6. 같은 `imageCount`면 지연 재렌더 타이머를 건드리지 않고 반환하도록 `scheduleReRender()` 조건 순서를 정리했다.
7. 문서 reset 시 overlay image retry 상태도 같이 초기화하도록 했다.

## 성능 측정

WASM 직접 측정 (`samples/exam_social.hwp`, 성명 칸 path):

```text
getPageOverlayImages(0) first   0.91ms, len=39
getPageOverlayImages(0) second  0.54ms, len=39
getPageLayerTree(0)             16.81ms, len=1,402,745
getCursorRectByPath first       0.60ms
getCursorRectByPath second      0.15ms
insertTextInCellByPath          0.12~0.35ms
입력 후 overlay 조회            3.33~3.59ms, len=39
입력 후 cursor 조회             0.15~0.18ms
```

브라우저 입력 검증 (`홍길동` 입력):

```json
{
  "afterText": "홍길동",
  "overlayLength": 39,
  "layerLength": 1403612,
  "renderPageCalls": 6,
  "scheduleReRenderImageCounts": [5, 1, 5, 1, 5, 1]
}
```

입력 3자 동안 보이는 2개 페이지 렌더만 발생했다. 위 `scheduleReRender` 호출은 각 visible page 렌더의 image count 확인이며, 같은 image count에서는 새 지연 재렌더 타이머를 만들지 않는다.

브라우저 검증에서는 폰트 요청을 차단해 초기화 지연을 제거했다. console의 `Failed to load resource: net::ERR_FAILED`는 이 폰트 차단으로 발생한 검증 환경 로그이며, #850 오류인 `컨트롤 인덱스 0 범위 초과`는 새로 발생하지 않았다.

## 검증

```bash
cargo test --test issue_850_answer_sheet_name_hit_test -- --nocapture
```

```text
test result: ok. 3 passed; 0 failed
```

```bash
cargo test --test issue_717_table_cell_hit_test -- --nocapture
```

```text
test result: ok. 3 passed; 0 failed
```

```bash
cargo test --lib test_task105_nested_table_path_api -- --nocapture
```

```text
test result: ok. 1 passed; 0 failed
```

```bash
cd rhwp-studio && npm run build
```

```text
tsc && vite build 통과
```

```bash
docker-compose run --rm wasm
```

```text
Your wasm pkg is ready to publish at /app/pkg.
```

```bash
cargo test
```

```text
test result: ok. 1232 passed; 0 failed; 2 ignored
integration/doc tests 통과
```

## 결론

성능 문제의 본질은 #850 정확성 수정 자체가 아니라, `v0.7.11` 이후 성명 칸이 nested path 입력 경로를 타면서 캐시 미사용 cursor 조회와 대형 page layer JSON 생성이 입력 루프에 들어온 것이다.

이번 단계에서 cursor 조회를 캐시 경로로 바꾸고, Studio 렌더가 입력마다 1.4MB page layer JSON을 만들지 않도록 compact overlay image API로 분리했다. 브라우저 검증에서 성명 칸에 `홍길동`이 정상 입력되었고, 기존 `컨트롤 인덱스 0 범위 초과` 오류는 재현되지 않았다.
