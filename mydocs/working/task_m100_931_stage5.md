# Task #931 Stage 5 완료 보고서

## 1. 배경

Stage 4 완료 후 작업지시자 시각 검증에서 워터마크가 아예 사라진 것처럼 보이는 문제가 확인됐다.

초기 Stage 2~4 정정은 BehindText overlay의 배율과 clipping을 맞췄지만, 실제 화면에서는 flow canvas가 흰 페이지 배경을 그린 뒤 그 위에 본문을 그리기 때문에 canvas 뒤에 있는 BehindText DOM overlay가 가려졌다.

## 2. 원인 재확인

100% 화면에서 DOM 상태:

- BehindText overlay layer: 존재
- 워터마크 `<img>`: 존재, 로드 완료
- 워터마크 rect: `495.03 × 495.72`
- canvas rect: `793 × 1122`

하지만 overlay on/off 스크린샷 픽셀 비교 결과:

```text
on_vs_off: nonzero=0
```

즉 DOM overlay는 존재하지만 실제 화면 픽셀에는 영향을 주지 않았다. 원인은 다음 2가지였다.

1. `WebCanvasRenderer::begin_page()`가 flow canvas 전체를 흰색으로 채운다.
2. CSS에서도 canvas 배경이 `var(--color-surface)`로 지정되어 있다.

따라서 BehindText overlay를 canvas 뒤에 두면 배율은 맞아도 흰 canvas 배경에 가려진다.

## 3. 수정 파일

- `src/renderer/web_canvas.rs`
- `rhwp-studio/src/view/page-renderer.ts`
- `rhwp-studio/src/view/canvas-view.ts`

## 4. 수정 내용

### 4.1 flow canvas 배경 투명화

`src/renderer/web_canvas.rs`에서 `LayerFilter::FlowOnly` 렌더 중 해당 페이지에 BehindText 이미지가 있으면 page background를 canvas에 그리지 않도록 했다.

핵심:

- `transparent_page_background` 상태 추가
- `layer_tree_contains_image_wrap(..., TextWrap::BehindText)`로 BehindText 존재 여부 확인
- flow canvas의 `begin_page()`에서 흰색 fill 생략
- flow 렌더 중 `PaintOp::PageBackground` skip

이 변경으로 flow canvas는 본문/표/선/텍스트만 그리고 배경은 투명하게 유지된다.

### 4.2 DOM layer 순서 정정

`rhwp-studio/src/view/page-renderer.ts`에서 BehindText가 있는 페이지를 다음 순서로 구성했다.

```text
z=0  page background layer (white)
z=1  BehindText overlay layer
z=2  transparent flow canvas
z=3  InFrontOfText overlay layer
```

canvas 자체는 BehindText 페이지에서 `background: transparent`로 설정한다.

### 4.3 워터마크 alpha 동기화

기존 canvas renderer는 워터마크 이미지에 `globalAlpha = 0.17`을 적용했다. DOM overlay도 같은 시각 정책을 따르도록 `img.watermark`에 다음 style을 추가했다.

```ts
el.style.opacity = '0.17';
```

### 4.4 orphan layer 정리

overlay가 canvas sibling으로 존재하므로 canvas pool에서 canvas만 제거하면 background/overlay layer가 남을 수 있다. 이를 방지하기 위해 `PageRenderer.removePageLayers()`와 `removeAllPageLayers()`를 추가하고, `CanvasView`의 page release/re-render 경로에서 함께 정리하도록 했다.

## 5. 검증

### 5.1 빌드

```bash
cargo build
cd rhwp-studio && npm run build
docker-compose --env-file .env.docker run --rm wasm
```

결과: 통과

참고: 현재 환경은 `docker compose` 플러그인이 아니라 `docker-compose` 바이너리를 사용한다.

### 5.2 회귀 테스트

```bash
cargo test --test issue_516
```

결과: `8 passed`

### 5.3 headless Chrome 시각/DOM 검증

스크립트:

- 임시 실행: headless Chrome + Puppeteer
- 결과 JSON: `/private/tmp/rhwp-watermark-analysis/stage5-result.json`
- 스크린샷:
  - `/private/tmp/rhwp-watermark-analysis/stage5_zoom_100.png`
  - `/private/tmp/rhwp-watermark-analysis/stage5_zoom_25.png`

#### 100% 줌

| 대상 | 측정값 |
|------|--------|
| background layer | `793 × 1122`, `z=0` |
| behind layer | `793 × 1122`, `z=1` |
| canvas | `793 × 1122`, `z=2`, `background=transparent` |
| watermark | `495.03 × 495.72`, `opacity=0.17`, `mix-blend-mode=multiply` |

#### 25% 줌

| 대상 | 측정값 |
|------|--------|
| background layer | `198 × 280`, `z=0` |
| behind layer | `198 × 280`, `z=1` |
| canvas | `198 × 280`, `z=2`, `background=transparent` |
| watermark | `123.75 × 123.92`, `opacity=0.17`, `mix-blend-mode=multiply` |

결론:

- 워터마크가 다시 보인다.
- 25%에서도 canvas와 BehindText layer가 같은 크기로 축소된다.
- 워터마크 자체도 원본 bbox의 25%로 축소된다.
- 텍스트/표는 워터마크 위의 transparent flow canvas에 정상 출력된다.

### 5.4 in-app Browser 확인

Codex in-app Browser에서 직접 검증 URL을 다시 열어 워터마크가 보이는 것을 확인했다.

현재 in-app Browser viewport는 1024px 미만이라 앱의 모바일 폭맞춤 로직이 자동으로 91% 줌을 적용했다. 이 상태에서 다음 layer 구성이 확인됐다.

| 대상 | 측정값 |
|------|--------|
| background layer | `723 × 1022.5`, `z=0` |
| behind layer | `723 × 1022.5`, `z=1` |
| canvas | `723 × 1022.5`, `z=2`, `background=transparent` |
| watermark | `450.94 × 451.57`, `opacity=0.17` |
| console error/warn | 없음 |

## 6. 직접 검증 URL

dev server는 계속 실행 중이다.

```text
http://127.0.0.1:7700/?url=/samples/%EB%B3%B5%ED%95%99%EC%9B%90%EC%84%9C.hwp&filename=%EB%B3%B5%ED%95%99%EC%9B%90%EC%84%9C.hwp
```

## 7. 결론

Stage 5 보정으로 Stage 4에서 남은 시각 가림 문제가 해결됐다.

- BehindText overlay는 제거하지 않았다.
- BehindText overlay는 canvas와 같은 zoom 배율을 따른다.
- flow canvas는 투명 배경으로 본문만 렌더링한다.
- 페이지 배경, BehindText, 본문 canvas가 서로 다른 layer로 분리되어 HWP z-order 의미가 보존된다.

## 8. 승인 요청

Task #931 Stage 5 보정 결과 검토 및 최종 완료 승인 요청.
