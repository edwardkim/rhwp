# Stage 3 완료보고서 - Task #1918

**단계**: Stage 3 - text edit overlay canvas 재사용
**이슈**: #1918 표 입력/삭제 시 워터마크·정적 이미지 레이어 포함 페이지가 매 입력마다 고비용 재렌더링됨
**브랜치**: `local/task1918`
**작성일**: 2026-07-05

---

## 1. 작업 요약

`document-page-invalidated`의 `reason: 'text-edit'`를 `CanvasView`에서 `PageRenderer`까지 전달하고,
해당 fast path에서만 기존 background/behind/front overlay canvas를 재사용하도록 구현했다.

일반 렌더, zoom/resize, full document refresh, CanvasKit 렌더 경로는 기존처럼 overlay를 제거하고 다시 만든다.
text-edit fast path에서도 cache key가 맞지 않으면 즉시 기존 overlay를 제거하고 새로 렌더링한다.

## 2. 변경 내용

### CanvasView

- `refreshInvalidatedPage`가 payload의 `reason`을 읽는다.
- `reason === 'text-edit'`일 때 `PageRenderContext`로 `allowStaticOverlayReuse: true`를 전달한다.
- payload가 없거나 text-edit이 아니면 `allowStaticOverlayReuse: false`로 둔다.

### PageRenderer

- `PageRenderContext`를 추가했다.
- `applyOverlays`가 text-edit fast path에서만 기존 overlay canvas를 찾고 key가 맞으면 재사용한다.
- overlay key는 다음 값을 포함한다.
  - page index
  - render scale
  - canvas physical width/height
  - layer kind
  - render profile
  - backend
  - layer summary signature
- 재사용된 overlay도 매 refresh마다 `applyPageLayerBox`를 다시 호출해 위치, CSS 크기, transform을 최신화한다.
- fallback `PageLayerTree` summary signature는 text 내용 변화에 흔들리지 않도록 overlay 관련 count/plane 값만 사용한다.

### 테스트

- `CanvasView`가 text-edit invalidation을 static overlay reuse context로 전달하는 정적 계약 검사를 추가했다.
- `PageRenderer`가 key 일치 시에만 overlay canvas를 재사용하고, non-text refresh에서는 기존 제거 경로를 유지하는
  정적 계약 검사를 추가했다.

## 3. 검증 결과

```bash
npm test
```

- 결과: 통과
- 세부: 156 passed, 0 failed

```bash
npm run build
```

- 결과: 통과
- 비고: 기존과 같은 Vite chunk size 경고가 출력됨

```bash
CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
VITE_URL="http://127.0.0.1:7700" \
node /private/tmp/rhwp_stage1_probe.mjs --mode=headless
```

- 결과: 통과
- `samples/복학원서.hwp` text-edit refresh:
  - Stage 1 기준: `renderPageToCanvasFiltered` 4회 (`flow/background/behind/front`)
  - Stage 3 후: `renderPageToCanvasFiltered` 1회 (`flow`)
  - `background`, `behind`, `front` filtered render 호출: 0회
- `samples/253E164F57A1BC6934-empty.hwp`, `samples/143E433F503322BD33.hwp`, `samples/table-001.hwp`:
  - 모두 `flow` 1회만 호출됨

## 4. 남은 범위

- Stage 4에서 image decode 재렌더 안전망과 flow 내부 정적 이미지/OLE 비용을 분리해야 한다.
- Stage 3는 overlay 재사용 단계이므로 `253E164F57A1BC6934-empty.hwp`, `143E433F503322BD33.hwp`의
  flow 내부 이미지 비용은 아직 본질적으로 줄이지 않았다.
- 현재 dev server는 아직 Stage 2 Rust WASM 산출물을 갱신하지 않은 상태라, probe에서 `getPageLayerTree` fallback이
  1회 호출됐다. WASM 갱신 후에는 Stage 2의 경량 overlay summary 경로가 우선된다.
