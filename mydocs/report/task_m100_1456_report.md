# Task M100 #1456 최종 결과보고서 — rhwp-studio 캔버스 차트/OLE(rawSvg) 비동기 디코드 재렌더 안전망 보정

- 이슈: #1456 "rhwp-studio 캔버스: 차트/OLE(rawSvg) 비동기 디코드 재렌더 안전망 미작동 — #1181 불완전 수정(재렌더 게이트가 rawSvg 미카운트)"
- 마일스톤: M100 (v1.0.0)
- 브랜치: `local/task1456`
- 작성일: 2026-06-24
- 관련: #1181(불완전 수정한 원 안전망), #1154(prefetch 도입), #275(OLE rawSvg 캔버스), #1431/#1453(차트 트래킹·C1a 검증 중 발견)

## 1. 문제와 배경

rhwp-studio 캔버스에서 **HWP 차트/OLE가 첫 로드 시 공백**으로 뜨고 맨 처음 열린 차트가 이후 파일에도 고착되는 결함이 C1a(#1453) 육안검증 중 발견됐다. native `export-svg`(SVG 경로)는 정상이며 **studio 캔버스 경로 전용** 결함이었다. #1181이 고치려 했으나 불완전 수정으로 남아 있었다.

## 2. 근본 원인

1. studio 캔버스는 차트/OLE를 `rawSvg` PaintOp → `web_canvas.rs render_raw_svg` → `draw_image`(비동기 `HtmlImageElement`)로 래스터화. 캐시 미스 + 디코드 전이면 공백이고 onload 재렌더 트리거가 없음.
2. #1181 재렌더 안전망(`scheduleReRender`)은 `imageCount > 0`일 때만 작동.
3. `imageCount`를 산정하는 TS측 `collectLayerPlaneSummary`가 `op.type==='image'`만 세고 **`rawSvg` 누락** → 차트/OLE 단독 페이지는 `imageCount=0` → 안전망 즉시 bail → 공백 고착.
4. 풀 트리 JSON은 `"type":"rawSvg"`를 정상 emit(`paint/json.rs:938`) — 데이터는 있는데 TS가 안 셈.

### #1181이 불완전했던 이유
#1181(commit `b54ad826`)은 RawSvg를 **호출처 없는 죽은 엔드포인트** `get_page_overlay_images`의 `image_count`(`rendering.rs:844`)에 더했다. 실제 트리거가 쓰는 `imageCount`는 TS `collectLayerPlaneSummary`가 독립 계산하므로 #1181 수정은 트리거 경로와 미연결 → 무효였다.

## 3. 수정 내용

**핵심(#1, TS 전용)** — [rhwp-studio/src/view/page-renderer.ts](rhwp-studio/src/view/page-renderer.ts):
- `LayerPlaneSummary`에 `rawSvgCount` 필드 추가(`imageCount`=래스터 이미지 의미 유지)
- `collectLayerPlaneSummary`가 `op.type==='rawSvg'` 카운트
- `renderPage`가 `scheduleReRender(..., imageCount + rawSvgCount)` 합산 전달

→ 차트/OLE 단독 페이지에서 트리거 카운트 > 0 → 200/600/1500ms 재렌더 + `prefetchLayerImages`(이미 rawSvg data URL 추출 지원) 발화 → 비동기 디코드 후 IMAGE_CACHE 히트로 렌더. Rust·렌더러 무변경.

**E2E 회귀 테스트** — [rhwp-studio/e2e/issue-1456-chart-rerender.test.mjs](rhwp-studio/e2e/issue-1456-chart-rerender.test.mjs): 새 세션 차트 HWP 비공백(케이스 A) + 서로 다른 차트 비재사용(케이스 B).

**범위 결정**: #1(핵심) 채택. #3(CanvasPool)은 런타임 검증에서 불요 확정. #2(Rust→JS onload 콜백)는 범위 밖(후속 이슈 후보).

## 4. 검증 결과

| 항목 | 결과 |
|---|---|
| `npx tsc --noEmit` | EXIT=0 |
| `render-backend.test.ts` | 23/23 PASS |
| `issue-1456-chart-rerender.test.mjs` (최신 pkg) | 5/5 PASS |

**전후(before/after) 대조** (동일 테스트, 수정 전 코드 = negative control):

| 단언 | 수정 후 | 수정 전 |
|---|---|---|
| 차트 A 유채색 픽셀 | 3.75% PASS | 0.011% FAIL(공백) |
| 차트 B 유채색 픽셀 | 2.83% PASS | 0.011% FAIL(공백) |
| 차트 B↔A 픽셀 차이 | 5.40% PASS | 0.00% FAIL(고착) |

→ 수정 전엔 첫 로드 공백+고착이 그대로 재현되고, 수정 후 둘 다 해소됨을 end-to-end로 확인.

## 5. 부수 발견 — 검증 환경 stale pkg (별건)

육안검증 중 "문서 로드 후 파일 메뉴 무반응" 보고 → 진단 결과 **로컬 `pkg/`(WASM)가 `getShowParagraphMarks` 추가 커밋 `448853ae`보다 이전 빌드**라 `this.doc.getShowParagraphMarks is not a function` 예외로 `getContext()`가 붕괴한 것. **#1456 변경과 무관**한 빌드 staleness(모든 문서·차트 종류 발생)였고, WASM `pkg/` 재빌드로 해소. 자세한 진단은 `mydocs/working/task_m100_1456_stage3.md` §4.

## 6. 인도물 / 커밋

| 산출물 | |
|---|---|
| 수행계획서 | `mydocs/plans/task_m100_1456.md` |
| 구현계획서 | `mydocs/plans/task_m100_1456_impl.md` |
| 단계 보고서 | `mydocs/working/task_m100_1456_stage{1,2,3}.md` |
| 최종 보고서 | 본 문서 |

커밋(`local/task1456`): `37ff4cad`(Stage 1 핵심) · `54bbebbc`(Stage 2 E2E) · `1e5b21de`(Stage 3 판정).

## 7. 남은 사항 / 후속

- **#2(견고화)**: `web_canvas.rs draw_image` 캐시 미스 시 onload 이벤트 구동 재렌더(타이머 폴링 대체). 별도 이슈 후보.
- **#1181 죽은 코드**(`get_page_overlay_images` rawSvg image_count): 기능 변경 비혼합 원칙으로 미정리. 후속 정리 후보.
- 차트 렌더 정합(라인 누적 C1d, ofPie 보조플롯·3D C2 등)은 #1431 Track C 별건.
