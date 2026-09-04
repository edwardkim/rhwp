# Task M100 #1456 Stage 3 완료보고서 — 고착(#3) 불요 확정 + 무회귀 + 최신 pkg 재검증

- 이슈: #1456
- 브랜치: `local/task1456`
- 작성일: 2026-06-24
- 구현계획서: `mydocs/plans/task_m100_1456_impl.md` (3단계)

## 1. 최신 pkg 재검증

Stage 2 검증 도중 로컬 `pkg/`(WASM)가 stale함이 드러나(아래 §3) 재빌드한 뒤, **최신 pkg 기준으로 #1456 E2E와 단위 테스트를 재실행**해 정식 검증으로 삼았다.

| 항목 | 결과 |
|---|---|
| `issue-1456-chart-rerender.test.mjs`(headless) | 차트 A 3.754% / B 2.834% 유채색, B↔A 차이 5.40% → **5/5 PASS** |
| `render-backend.test.ts` | **23/23 PASS** |

> E2E 수치가 stale pkg 실행과 동일(3.754 / 2.834 / 5.40) — OLE 차트 rawSvg 렌더 경로가 pkg 버전 간 안정적임을 확인.

## 2. 고착(#3 CanvasPool) 불요 확정

구현계획서 3단계 분기: 케이스 B(서로 다른 내용 차트 간 고착) 재현 여부로 #3 필요성 판정.

- **케이스 B 결과**: 같은 세션에서 `세로막대형` → `원형`(서로 다른 내용) 교체 로드 시, 두 캔버스가 **5.40% 차이**로 각각 **비공백** 렌더됨. B가 A의 캐시/잔존 픽셀을 재사용하지 않음.
- **근거**: 재렌더(`reRenderPageCanvases`)가 `renderPageToCanvasFiltered`로 flow 캔버스를 **통째로 다시 그려** CanvasPool 잔존 픽셀을 덮어쓴다. 따라서 재렌더 트리거 보정(#1)만으로 *공백*뿐 아니라 *서로 다른 차트 간 고착*까지 결정적으로 해소된다.
- **결론**: **#3(CanvasPool acquire 픽셀 클리어)는 불필요**. 본 이슈 범위에서 제외 확정.

> 보조 참고: stale pkg(수정 전 코드)에서는 A·B 모두 0.011%(공백)·차이 0.00%로, 이슈가 보고한 "맨 처음 차트 고착"이 *공백끼리의 0% 차이*로 재현됐다. 즉 고착의 실체는 "재렌더 부재로 둘 다 공백"이었고, #1로 양쪽이 각자 렌더되며 해소됨.

## 3. 무회귀

- `render-backend.test.ts` 23/23 PASS.
- 변경 면적이 `rawSvg` 카운트 추가에 한정되어 `imageCount`(래스터 이미지) 경로·비-rawSvg 페이지 렌더에 영향 없음. `LayerPlaneSummary` 필드 추가는 기존 소비처(트리거 호출부 1곳)만 합산 변경.

## 4. 검증 과정 메모 — 메뉴 안먹힘은 #1456 무관(stale pkg)

작업지시자 육안검증 중 "문서 로드 후 파일 메뉴(새로 만들기/열기 등) 무반응" 보고 → systematic-debugging으로 진단:

- 원인: 로컬 `pkg/`(2026-06-21 20:47 빌드)가 `getShowParagraphMarks` 추가 커밋 `448853ae`("task 1452", 21:52)보다 **이전 빌드** → stale WASM에 해당 export 없음.
- 메커니즘: `getContext()`(`main.ts:106`)가 `wasm.getShowParagraphMarks()` 호출 → 문서 로드 시 `this.doc.getShowParagraphMarks is not a function` 예외 → 커맨드 디스패치 붕괴. (문서 미로드 시 `wasm-bridge.ts:1657` doc-null 가드로 `false` 반환 → 메뉴 정상 = 로드 후에만 깨짐.)
- probe 증거: 로드 전 `file:new-doc` 클릭 → create-new-document 발화(1), 로드 후 → 발화(0) + 동일 PAGEERROR 2건.
- **#1456(page-renderer.ts) 변경과 전혀 무관** — 모든 문서·모든 차트 종류에 발생하는 빌드 staleness.
- 조치: WASM `pkg/` 재빌드(`docker compose ... run --rm wasm`, 22:25 산출) → `getShowParagraphMarks` 포함 확인 → 메뉴 정상화. (`pkg/`는 gitignore 산출물이라 커밋 무관.)

## 5. 결론

- #1456 핵심 수정(#1)으로 차트/OLE rawSvg 단독 페이지의 첫 로드 공백·고착이 결정적으로 해소됨(전후 대조 + 최신 pkg 재검증).
- #3(CanvasPool) 불요 확정. #2(이벤트 구동 onload 콜백)는 범위 밖(후속 이슈 후보).
- 다음: Stage 4 최종 보고 + 이슈 클로즈.
