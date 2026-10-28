# Stage 4 완료보고서 - Task #1918

**단계**: Stage 4 - flow 정적 이미지 재렌더 정책 분리
**이슈**: #1918 표 입력/삭제 시 워터마크·정적 이미지 레이어 포함 페이지가 매 입력마다 고비용 재렌더링됨
**브랜치**: `local/task1918`
**작성일**: 2026-07-05

---

## 1. 작업 요약

Stage 3에서 background/behind/front overlay canvas 재사용을 적용했지만,
`253E164F57A1BC6934-empty.hwp`, `143E433F503322BD33.hwp`처럼 flow 내부에
정적 이미지 또는 OLE/RawSvg가 있는 문서는 text-edit 때도 `flow` 전체가 다시 그려졌다.

이번 단계에서는 WASM Canvas2D 필터를 `flow-dynamic`과 `flow-static`으로 나누고,
text-edit fast path에서만 정적 flow canvas를 별도로 유지하도록 구현했다.
일반 렌더, 확대/축소, resize, full refresh는 기존 `flow` 렌더 경로를 유지한다.

## 2. 변경 내용

### Rust WASM Canvas 필터

- `LayerFilter::FlowDynamic`을 추가했다.
  - flow plane 중 `Image`, `RawSvg`를 제외하고 렌더링한다.
  - text/shape/table 등 입력에 따라 바뀌는 본문 요소만 다시 그리는 용도다.
- `LayerFilter::FlowStatic`을 추가했다.
  - page background와 flow plane의 `Image`, `RawSvg`만 렌더링한다.
  - watermark, OLE preview, chart preview처럼 입력 중 변하지 않는 정적 리소스를 별도 canvas에 유지한다.
- `renderPageToCanvasFiltered`의 허용 layer kind에 `flow-dynamic`, `flow-static`을 추가했다.

### Overlay summary 확장

- `get_page_overlay_images_native`가 다음 값을 추가로 반환한다.
  - `flowImageCount`
  - `flowRawSvgCount`
- PageRenderer의 summary fallback도 `PageLayerTree`를 순회할 때 flow plane의 `image/rawSvg` 수를 따로 계산한다.
- `flowStaticCount = flowImageCount + flowRawSvgCount`로 text-edit fast path 적용 여부를 판단한다.

### PageRenderer 재렌더 정책

- text-edit + static overlay reuse 허용 + behind overlay 없음 + flow 정적 리소스 있음 조건에서만
  `flow-dynamic` + `flow-static` 분리 경로를 사용한다.
- `flow-static` canvas는 기존 overlay canvas와 같은 key 방식으로 재사용한다.
- `flow-dynamic` 또는 `flow-static`이 현재 WASM 산출물에서 지원되지 않으면 `flowSplitSupported = false`로 기록하고
  즉시 기존 `flow` 렌더로 fallback한다.
- image/rawSvg async decode 안전망의 deferred rerender가 `ReRenderPolicy`를 받도록 바꿨다.
  - static flow 재사용 중이면 지연 재렌더에서 `flow-static`만 다시 그린다.
  - text-edit fast path에서 background/behind/front overlay는 다시 그리지 않는다.
  - retry key에 summary signature를 포함해 페이지의 정적 리소스 구성이 바뀌면 재시도 상태가 갱신된다.

## 3. 검증 결과

```bash
node --test tests/render-backend.test.ts
```

- 결과: 통과
- 세부: 29 passed, 0 failed

```bash
npm test
```

- 결과: 통과
- 세부: 159 passed, 0 failed

```bash
npm run build
```

- 결과: 통과
- 비고: 기존과 같은 Vite chunk size 경고가 출력됨

```bash
cargo test get_page_overlay_images --lib
```

- 결과: 통과
- 세부: lib test compile 통과, 필터 조건상 실행 대상 0개

```bash
cargo test issue_938_overlay_watermark_is_hancom_baked_png --test issue_938
```

- 결과: 통과

```bash
npm run e2e:renderer-contract
```

- 결과: 통과
- 세부: renderer backend contract guard passed

```bash
CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
VITE_URL="http://127.0.0.1:7700" \
node e2e/issue-1456-chart-rerender.test.mjs --mode=headless
```

- 결과: 통과
- chart A colored ratio: 3.754% > 0.3%
- chart B colored ratio: 2.834% > 0.3%
- chart diff ratio: 5.40% > 2%

```bash
git diff --check
```

- 결과: 통과

## 4. 환경 제한 및 남은 확인

Docker 기반 WASM 재빌드는 이번 환경에서 완료하지 못했다.

```bash
docker compose --env-file .env.docker run --rm wasm
```

- 실패: Docker CLI에 compose plugin이 없음

```bash
docker-compose --env-file .env.docker run --rm wasm
```

- 실패: Docker daemon에 연결할 수 없음

`/Users/melee/.cargo/bin/wasm-pack`은 존재하지만, 저장소 규칙상 WASM 빌드는 Docker 경로를 사용해야 하므로
로컬 `wasm-pack`으로 우회하지 않았다.

따라서 이번 단계의 브라우저 E2E는 현재 로컬 `pkg/` 산출물을 기준으로 한 호환성 검증이며,
새 `flow-dynamic`/`flow-static` 필터가 포함된 WASM runtime의 실제 성능 probe는 Stage 5에서
Docker WASM 빌드가 가능한 환경으로 진행해야 한다.

Stage 5에서 확인할 대상:

- `samples/복학원서.hwp`
- `samples/253E164F57A1BC6934-empty.hwp`
- `samples/143E433F503322BD33.hwp`
- `samples/table-001.hwp`
