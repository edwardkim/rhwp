# Task M100 #1456 구현계획서 — rhwp-studio 캔버스 rawSvg 재렌더 트리거 보정

- 이슈: #1456
- 브랜치: `local/task1456`
- 작성일: 2026-06-24
- 수행계획서: `mydocs/plans/task_m100_1456.md`

## 구현 개요

재렌더 트리거 게이트(`scheduleReRender`의 `imageCount > 0`)가 차트/OLE 단독 페이지를 즉시 bail하는 원인은, 트리거 카운트를 산정하는 TS측 `collectLayerPlaneSummary`가 `op.type==='rawSvg'`를 안 세기 때문이다. `LayerPlaneSummary`에 `rawSvgCount`를 추가하고, 트리거 호출부에서 `imageCount + rawSvgCount`를 전달하여 rawSvg 단독 페이지도 `scheduleReRender`(200/600/1500ms 재렌더 + `prefetchLayerImages`)가 발화되게 한다. **Rust·렌더러는 손대지 않는다.**

대상 파일은 [rhwp-studio/src/view/page-renderer.ts](rhwp-studio/src/view/page-renderer.ts) 1개(핵심) + E2E 테스트 신규. CanvasPool(#3)은 3단계 런타임 검증에서 고착 재현 시에만 조건부.

---

## 1단계 — 핵심 수정 (#1): rawSvg 재렌더 트리거 카운트

**대상**: `rhwp-studio/src/view/page-renderer.ts`

(a) `LayerPlaneSummary`(L6-10)에 필드 추가:

```ts
interface LayerPlaneSummary {
  hasBehind: boolean;
  hasFront: boolean;
  imageCount: number;
  rawSvgCount: number;  // OLE/차트 rawSvg op 수 — 비동기 디코드 재렌더 트리거용(의미 분리)
}
```

(b) 초기화 2곳에 `rawSvgCount: 0` 추가: `applyOverlays` early-return(L96), `getLayerPlaneSummary`(L225).

(c) `collectLayerPlaneSummary`(L449) — rawSvg 카운트 추가:

```ts
if (op.type === 'image') {
  summary.imageCount += 1;
} else if (op.type === 'rawSvg') {
  // 차트/OLE 미리보기. web_canvas draw_image 비동기 디코드 경로를 타므로
  // image 와 동일하게 재렌더 트리거 대상에 포함한다(#1456).
  summary.rawSvgCount += 1;
}
```

(d) 트리거 호출부 `renderPage`(L42) — 합산 전달:

```ts
this.scheduleReRender(pageIdx, canvas, renderScale, overlays.imageCount + overlays.rawSvgCount);
```

> `renderPageFlow`(L221, `scheduleReRender(..., 0)`)는 현재 라이브 경로(`canvas-view.ts:184`는 `renderPage` 사용)가 아니므로 본 단계 범위에서 제외. 필요 시 후속.

**완료 기준**: `cd rhwp-studio && npm run build`(타입체크) 통과. 기존 단위 테스트 무회귀. 변경은 page-renderer.ts 한정.
단계별 보고서 `mydocs/working/task_m100_1456_stage1.md` 작성 + 소스 커밋.

## 2단계 — E2E 회귀 테스트

**대상**: 신규 `rhwp-studio/e2e/issue-1456-chart-rerender.test.mjs` (`helpers.mjs` 재사용)

- `runTest` 골격 + `loadHwpFile`(샘플 fetch) + `captureCanvasScreenshot`(첫 페이지 캔버스 PNG) + `PNG.sync.read` 픽셀 분석.
- **케이스 A (비공백, 핵심 #1 회귀)**: 새 세션(`loadApp`로 페이지 로드 = IMAGE_CACHE 초기화)에서 차트 HWP 1개 로드(예: `chart/세로막대형/묶은세로막대형.hwp`). 재렌더 타이머·prefetch 발화 위해 충분 대기(추가 ~2s). 캔버스 PNG에서 **유채색(비백·비회색) 픽셀 비율**이 임계 초과 단언 — 공백/백지면 미달. (수정 전 fail → 수정 후 pass 가 회귀 의미.)
- **케이스 B (비재사용 / 고착, 3단계 판정 입력)**: **같은 세션**에서 차트 A 로드→캡처, 이어서 다른 종류 차트 B(예: `chart/원형/원형.hwp`) 로드→캡처. `comparePngBuffers`로 A↔B **유의미 차이** 단언(B가 A 픽셀 재사용 안 함).
- 실행 전제: WASM `pkg/` 최신 필요. 미빌드/구버전이면 Docker(`docker compose --env-file .env.docker run --rm wasm`). Vite(`npx vite --host 0.0.0.0 --port 7700`) + headless(`--mode=headless`, macOS는 `CHROME_PATH` 필요).

**완료 기준**: 케이스 A pass(차트 HWP 비공백). 케이스 B는 실행 결과를 3단계 판정 입력으로 기록.
단계별 보고서 `mydocs/working/task_m100_1456_stage2.md` 작성 + 커밋.

## 3단계 — 런타임 고착 검증 → 조건부 CanvasPool(#3) + 최종 검증

- 2단계 **케이스 B** 결과로 *서로 다른 내용 차트 간 고착* 재현 여부 판정:
  - **재현 안 됨**(B가 A와 충분히 다름) → #3 불필요. 근거를 보고서에 기록.
  - **재현됨**(B가 A 픽셀 잔존) → `rhwp-studio/src/view/canvas-pool.ts` `acquire`(L6) 재사용 캔버스 픽셀 클리어(예: `canvas.width = canvas.width` 또는 `ctx.clearRect`) + 필요 시 오버레이 레이어 정리 추가. 케이스 B를 회귀 가드로 고정.
- 무회귀: 기존 핵심 e2e(`canvas-render-diff.test.mjs` 등) 통과 확인.

**완료 기준**: 고착 판정 확정 + (수정 시) 케이스 B pass + 기존 e2e 무회귀.
단계별 보고서 `mydocs/working/task_m100_1456_stage3.md` 작성 + (수정 시)커밋.

---

## 변경 파일 예상

| 파일 | 변경 |
|---|---|
| `rhwp-studio/src/view/page-renderer.ts` | `LayerPlaneSummary.rawSvgCount` + `collectLayerPlaneSummary` rawSvg 카운트 + 트리거 합산 (핵심) |
| `rhwp-studio/e2e/issue-1456-chart-rerender.test.mjs` | 신규 회귀 테스트(케이스 A 비공백 / 케이스 B 비재사용) |
| `rhwp-studio/src/view/canvas-pool.ts` | **조건부**(3단계 고착 재현 시): `acquire` 픽셀 클리어 |
| `mydocs/working/task_m100_1456_stage{1..3}.md` | 단계별 보고서 |
| `mydocs/report/task_m100_1456_report.md` | 최종 보고서 |

## 위험 / 주의

- **비공백 픽셀 임계**: 유채색 픽셀 비율 임계는 차트 면적에 의존 — chart-dominant 샘플 선택으로 마진 확보. 텍스트(흑백)와 구분 위해 회색 제외(채도 기준).
- **헤드리스 환경(macOS)**: 호스트 CDP 기본값은 WSL용. macOS는 `--mode=headless` + `CHROME_PATH`. Docker WASM/Chrome 미가용 시 케이스 A는 코드리뷰+수동 재현으로 대체하고 테스트는 CI/메인테이너 환경용으로 커밋.
- **#1181 죽은 코드**(`get_page_overlay_images` rawSvg image_count): 본 이슈에서 제거하지 않음(기능 변경 비혼합). 보고서 관찰 기록.
- 포맷: 수정 파일 범위만(`cargo fmt --all` 전체 금지 — 단 본 작업은 Rust 무변경).
