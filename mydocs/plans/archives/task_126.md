# 타스크 126 수행계획서 — Canvas DPR 스케일링

## 배경

### 현재 문제

rhwp-studio는 Canvas 렌더링 시 `devicePixelRatio(DPR)`를 적용하지 않는다. 고해상도 디스플레이(레티나, 4K)에서 텍스트와 선이 흐릿하게 보인다.

현재 흐름:
```
WASM renderPageToCanvas(page, canvas)  → scale=1.0 고정
canvas.width = pageWidthPx             → 1x 물리 픽셀
canvas.style.width = pageWidthPx × zoom → CSS 확대/축소
→ DPR=2 디스플레이에서 1x 해상도를 2x로 늘림 → 흐릿
```

### 일부 상용 제품 분석 결과

일부 상용 제품 캔버스 렌더링 분석(`mydocs/tech/investigations/issue-124/canvas_rendering_analysis.md`)에서 한컴 웹기안기, 구글 독스, 일부 상용 웹오피스 모두 동일한 DPR 패턴을 사용한다:

```
canvas.width  = pageWidth  × zoom × DPR    (물리 픽셀)
canvas.style.width  = pageWidth × zoom + "px"  (CSS 논리 픽셀)
ctx.scale(zoom × DPR, zoom × DPR)
```

### 기존 구현 상태

- 타스크 123에서 WASM `render_page_to_canvas`에 `scale: f64` 파라미터 추가 완료
- `web/editor.js`에는 `renderScale = zoom × dpr` 패턴이 이미 구현됨
- **rhwp-studio(TypeScript)에만 미적용** — 항상 scale=1.0으로 렌더링

### 해결 방향

rhwp-studio의 `page-renderer.ts`와 `canvas-view.ts`에서 `renderScale = zoom × DPR`을 계산하여 WASM에 전달하고, CSS 표시 크기를 `물리 픽셀 / DPR`로 설정한다.

## 핵심 수식

```
renderScale = zoom × DPR

canvas.width  = pageWidthPx  × renderScale    (WASM이 자동 설정)
canvas.height = pageHeightPx × renderScale

canvas.style.width  = canvas.width  / DPR + "px"  (CSS 논리 픽셀)
canvas.style.height = canvas.height / DPR + "px"

ctx.scale(renderScale, renderScale)  (WASM 내부에서 적용)
```

## 변경 영향 분석

| 구성 요소 | DPR 영향 | 변경 필요 | 근거 |
|-----------|---------|----------|------|
| page-renderer.ts | WASM에 scale 전달 | **예** | renderScale 계산 및 전달 |
| canvas-view.ts | CSS 크기 계산 | **예** | 물리 픽셀 / DPR |
| wasm_api.rs | scale 상한 확장 | **예** | zoom 3.0 × DPR 4.0 = 12.0 |
| margin guides | ctx 스케일 적용 | **예** | WASM 후 ctx transform 재설정 |
| virtual-scroll.ts | CSS 논리 픽셀 기준 | 아니오 | DPR은 캔버스 내부 해상도에만 관여 |
| selection-renderer.ts | HTML DIV 기반 | 아니오 | CSS 좌표 × zoom으로 이미 정확 |
| cell-selection-renderer.ts | HTML DIV 기반 | 아니오 | 위와 동일 |
| caret-renderer.ts | HTML DIV 기반 | 아니오 | CSS 논리 좌표 기준 |
| input-handler.ts | 마우스 좌표 ÷ zoom | 아니오 | DPR은 CSS가 흡수 |
| main.ts | zoom 계산 | 아니오 | CSS 논리 크기 기준 |
| coordinate-system.ts | CSS 논리 좌표 | 아니오 | DPR 무관 |

## 구현 단계 (3단계)

---

### 1단계: Rust WASM scale 범위 확장

**파일**: `src/wasm_api.rs` (라인 212)

**변경**: `scale.clamp(0.25, 8.0)` → `scale.clamp(0.25, 12.0)`

zoom 3.0 × DPR 4.0 = 12.0을 지원하기 위함 (4K 모니터 + 최대 줌).

---

### 2단계: TypeScript DPR 스케일링 적용

**파일 1**: `rhwp-studio/src/view/page-renderer.ts`

- `renderPage()` 시그니처에 `scale: number` 파라미터 추가
- `drawMarginGuides()`에 `ctx.setTransform(scale, 0, 0, scale, 0, 0)` 적용
  - WASM 렌더링 후 ctx transform 상태가 불확실하므로 명시적으로 설정
- `scheduleReRender()`에도 동일 scale 전달

**파일 2**: `rhwp-studio/src/view/canvas-view.ts`

- `renderPage()` 내부에서 `renderScale = zoom × devicePixelRatio` 계산
- WASM 호출: `pageRenderer.renderPage(pageIdx, canvas, renderScale)`
- CSS 크기: `canvas.style.width = canvas.width / dpr + "px"`
- 기존 `if (zoom !== 1.0)` 분기 제거 → 항상 CSS 크기 설정

---

### 3단계: 통합 테스트 및 검증

| 항목 | 방법 |
|------|------|
| 571개 회귀 테스트 | `docker compose run --rm test` |
| WASM 빌드 | `docker compose run --rm wasm` |
| TypeScript 타입 체크 | `npx tsc --noEmit` |
| 기본 렌더링 (줌 100%) | DPR=1 환경에서 기존과 동일 확인 |
| 고해상도 렌더링 | DPR=2 환경에서 텍스트 선명도 확인 |
| 줌 확대/축소 | 200%, 50%에서 선명도 확인 |
| 마우스 히트테스트 | 줌 + DPR 상태에서 클릭 → 캐럿 정확성 |
| 선택 영역 | 드래그 선택 → 하이라이트 위치 정확성 |
| 캐럿 | 줌 변경 후 캐럿 위치 정확성 |
| 폭/쪽 맞춤 | zoom-fit-width/page 정상 동작 |

---

## 변경 파일 요약

| 파일 | 변경 내용 | 규모 |
|------|-----------|------|
| `src/wasm_api.rs` | scale 상한 8.0→12.0, 주석 갱신 | 1줄 |
| `rhwp-studio/src/view/page-renderer.ts` | scale 파라미터 추가, margin guides에 setTransform 적용 | ~15줄 |
| `rhwp-studio/src/view/canvas-view.ts` | DPR 계산, renderScale 전달, CSS 크기 = 물리/DPR | ~10줄 |

## 기대 효과

| 항목 | 현재 | 적용 후 |
|------|------|---------|
| DPR=2 선명도 | 흐릿 (1x 해상도 CSS 확대) | 선명 (2x 물리 픽셀 렌더링) |
| 줌 200% 선명도 | 선명 (타스크 123 scale) | 선명 + DPR 대응 |
| WASM 크기 | 1.83MB | 1.83MB (변화 없음) |
| 변경 규모 | — | 3개 파일, ~25줄 |

## 한컴과의 아키텍처 비교

```
[한컴 웹기안기]
  HWP → 서버(HWP필터) → JSON 커맨드 → JS Canvas 2D fillText
  DPR: ctx.scale(DPR) + 캔버스 크기 × DPR

[rhwp (적용 후)]
  HWP → Rust WASM 파서 → 렌더트리 → Rust Canvas 2D fillText
  DPR: ctx.scale(zoom×DPR) + 캔버스 크기 × zoom × DPR

  * 공통점: 둘 다 브라우저 Canvas 2D fillText 사용, DPR 스케일링 동일
  * 차이점: 한컴은 JS에서 fillText 호출, rhwp는 Rust/WASM에서 호출
  * 장점: 서버 불필요, 단일 WASM 바이너리(1.83MB)
```
