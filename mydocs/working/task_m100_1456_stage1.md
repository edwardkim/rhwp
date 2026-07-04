# Task M100 #1456 Stage 1 완료보고서 — rawSvg 재렌더 트리거 카운트 핵심 수정

- 이슈: #1456
- 브랜치: `local/task1456`
- 작성일: 2026-06-24
- 구현계획서: `mydocs/plans/task_m100_1456_impl.md` (1단계)

## 1. 수행 내용

근본 원인(재렌더 트리거 게이트 `imageCount > 0`가 rawSvg 단독 페이지를 즉시 bail)을 해소하기 위해, TS측 재렌더 트리거 카운트가 `rawSvg`(차트/OLE) op를 반영하도록 [rhwp-studio/src/view/page-renderer.ts](rhwp-studio/src/view/page-renderer.ts) 한 파일을 수정했다.

| 위치 | 변경 |
|---|---|
| `LayerPlaneSummary`(L6-11) | `rawSvgCount: number` 필드 추가 (image 와 의미 분리) |
| `applyOverlays` early-return(L99) | 초기값 `rawSvgCount: 0` 추가 |
| `getLayerPlaneSummary`(L228) | 초기값 `rawSvgCount: 0` 추가 |
| `collectLayerPlaneSummary`(L454) | `else if (op.type === 'rawSvg') summary.rawSvgCount += 1` 추가 |
| `renderPage`(L43) | `scheduleReRender(..., overlays.imageCount + overlays.rawSvgCount)` 합산 전달 |

diff: `1 file changed, 10 insertions(+), 3 deletions(-)`.

설계 선택: `imageCount`(래스터 이미지 op)의 의미를 보존하고 `rawSvgCount`를 **별도 필드**로 분리한 뒤 트리거 호출부에서만 합산. 향후 유지보수자가 두 경로(image / rawSvg)를 구분 가능.

범위 준수: Rust·렌더러 무변경. `renderPageFlow`(라이브 경로 아님, `canvas-view.ts:184`는 `renderPage` 사용)는 의도적으로 제외.

## 2. 동작 원리 (수정 효과)

차트/OLE 단독 페이지: `rawSvgCount > 0` → `scheduleReRender`가 더 이상 bail하지 않음 → 200/600/1500ms 지연 재렌더 + `prefetchLayerImages`(이미 rawSvg 임베디드 data URL 추출 지원, L405-412) 발화 → 비동기 디코드 완료 후 IMAGE_CACHE 히트로 차트가 그려짐.

## 3. 검증 결과

| 항목 | 명령 | 결과 |
|---|---|---|
| 타입 체크 | `npx tsc --noEmit` | **EXIT=0** (오류 없음) |
| 단위 테스트(관련) | `node --experimental-strip-types --test tests/render-backend.test.ts` | **23/23 PASS** |

- `render-backend.test.ts`는 page-renderer.ts 소스를 읽어 `collectLayerPlaneSummary(root, summary, null)` 호출 형태를 단언(L140) — 본 변경이 해당 시그니처를 보존하므로 통과.
- Node 22.12.0 환경 제약상 `.ts` 직접 실행에 `--experimental-strip-types` 플래그 필요(프로젝트 `npm test`는 strip-types 기본 지원 Node 가정). 환경 차이일 뿐 기능 결함 아님.

## 4. 비고

- 픽셀 단위 비공백 검증(E2E)은 Stage 2에서 수행.
- #1181의 죽은 코드(`get_page_overlay_images`의 rawSvg `image_count`)는 본 단계에서 손대지 않음(기능 변경 비혼합).

## 5. 다음 단계

Stage 2 — E2E 회귀 테스트(`issue-1456-chart-rerender.test.mjs`) 작성: 새 세션 차트 HWP 비공백(케이스 A) + 서로 다른 차트 비재사용(케이스 B).
