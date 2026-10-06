# #6040 — 동일 physical surface 재사용의 표시 계약 조사

- 일자: 2026-09-10 KST
- 제품: `a54d97575`, 사용자 확인 기록: `9fea3506f`
- 단계: Stage 4.2 잔여 수용 조사와 Stage 4.3 제한 실험 계획. 제품 구현 없음.
- 승인: 고배율 동일 surface의 불필요한 재렌더 조건을 조사·재현하고 계획을 작성한다.
- GitHub #6040 본문·댓글 4건을 다시 확인했다. OPEN, assignee `postmelee`.
- [수행 계획](../plans/task_m100_6040.md), [구현 계획](../plans/task_m100_6040_impl.md)

## 결론

**물리 scale/key 동일만으로 렌더를 생략하는 수정은 안전하지 않다.** 현재 key는 문서/revision,
쪽 geometry, backend, profile, physical scale, layer 수를 포함하지만 실제 Canvas2D bitmap에는
표시 배율에 의존하는 여백 꺾쇠도 그려진다. DOM flow-image 내부 좌표와 지연 raster callback도
별도 displayScale을 사용한다. `rhwpRenderedZoom`만 현재값으로 고치면 계약을 숨기는 것이다.

제한 실험은 가치가 있지만, **354→500% 같은 완료 surface의 같은 physical scale 재확정**만
목표로 삼는다. 300→500%에서 필요한 물리 해상도 증가, 아직 고화질이 완성되지 않은 surface,
새 쪽 cold 준비·첫 렌더를 모두 해결하는 안은 아니다. 따라서 실험 효과가 작으면 제품에
넣지 않고 별도 구조 개선으로 이관할 수 있어야 한다.

## 현재 코드 재현

[재현 스크립트 — 로컬 보존 안내](assets/issue6040-post-stack/README.md)를 저장소 root에서 실행했다.
실제 CanvasView의 descriptor/dispatch와 margin guide 수식을 Vite SSR로 호출했다.
renderer는 횟수를 세는 stub이다. 아래 숫자는 브라우저 raster·GPU·성능 실측이 아니다.

```sh
node mydocs/working/assets/issue6040-clamp-reuse/reproduce.mjs
cd rhwp-studio
node --experimental-strip-types --test --test-name-pattern='physical key가 같아도 visible' tests/canvas-view-zoom-scheduler.test.ts
```

기존 테스트 1/1 pass는 **현재 구 zoom 확정에 재렌더 1회가 필요하도록 구현돼 있음**을 확인한
것이지 새 최적화 성공 테스트가 아니다. source와 기존 test는 변경하지 않았다.

1122.5×1587.4 논리 쪽, raw DPR 2, Canvas2D, 3-layer, 문서/revision 고정:

| 전환 | physical scale 전 → 후 | key 동일 | 현재 renderCanvas 요청 |
| --- | --- | --- | ---: |
| 300→500% | 6 → 6.136964072485382 | 아니오 | 1 |
| 354→500% | 6.136964072485382 → 동일 | 예 | 1 |
| 500→354% | 6.136964072485382 → 동일 | 예 | 1 |
| 500→200% | 6.136964072485382 → 4 | 아니오 | 1 |

현재 `renderScheduledPage()`는 같은 key여도 `isPageZoomPreview()`가 true이면 `renderCanvas()`를
호출한다. Stage 4.1에서 metadata/preview가 끝나지 않는 문제를 보정한 조건이다. 삭제만 하면
이전 미완료 문제가 다시 나타날 수 있다. `refreshRenderSurfacePlan(true)`도 staleZoom을 따로
처리하므로 예약 경로 한 곳의 횟수만 줄였다고 전체 경로가 해결됐다고 주장할 수 없다.

## 같지 않은 표시 요소

### 1. Canvas에 합성된 여백 꺾쇠

`PageRenderer.renderPage()`와 지연 `reRenderPageCanvases()`는 `drawMarginGuides()`를 호출한다.
`resolvePageMarginGuideLineWidth()`는 표시 선 두께를 0.8–1.5 CSS px로 제한한다.
354%의 bitmap을 그대로 500%로 늘리면 정상 1.5px가 아니라 **2.118644px**가 되고,
500% 것을 354%로 줄이면 **1.062px**가 된다. physical scale은 같아도 원래 그릴 선 폭은 다르다.

꺾쇠 주변만 clear하고 다시 그리는 것은 겹친 문서 내용을 지울 수 있다. 문서 patch replay로
복원하더라도 큰 Canvas를 수정하면 native 업로드 비용이 작아진다고 보장할 수 없다.
화면 전체 크기의 별도 guide Canvas도 피한다. 작은 UI 안내선을 위해 큰 surface를 추가하면
최적화 목적과 어긋난다.

### 2. DOM flow-image와 비동기 보정

`createOrReuseFlowImageLayer()`의 frame/clip/crop 좌표는 displayScale을 곱한 CSS px다.
바깥 쪽 크기만 바꾸고 preview transform을 지우면 자식 그림 좌표가 옛 배율에 남는다.
load callback도 생성 시 displayScale을 캡처한다.

`scheduleReRender()`의 job은 renderScale, canvas, policy.displayScale을 캡처한다.
`isPageSurfaceComplete()`는 reRenderJobs와 DOM 이미지 complete/naturalWidth를 검사하지만
이는 GPU 게시 완료까지 보장하는 API는 아니다. 미완료 job을 취소하거나 완료 플래그를 강제로
올려 재사용을 허용하지 않는다. CanvasKit의 준비/fallback 계약도 Canvas2D와 섞지 않는다.

### 3. CSS와 소유권

main/각 overlay의 width·height뿐 아니라 top/left/transform/origin, renderedZoom,
requested/effective DPR, physical scale, surface shape/key·actual pixels를 함께 일치시켜야 한다.
detached LRU 복원과 active surface 정착은 다른 진입점이다. 첫 제한 실험에서는 active만
다루고 LRU 복원·strict 편집·HF 편집·CanvasKit은 기존 렌더 경로를 유지한다.

## 다음 계획과 중단선

[Stage 4.3 계획](../plans/task_m100_6040_impl.md#stage-43--고배율-동일-surface-확정의-제한-실험)은
Canvas2D의 배율 의존 꺾쇠를 작은 DOM/SVG 표시로 분리하는 한정 후보와 안전 gate를 검증한다.
먼저 geometry/paint 순서·이미지 완료·오래된 callback 계약을 고정한다. 이미 꺾쇠가 그려진
기존 bitmap은 그대로 재사용하지 않고, 분리 계약으로 새로 완성한 surface만 후보가 될 수 있다.

이는 승인 전 계획이며 DOM/SVG layer를 이번 조사에서 실제 추가하지 않았다. 좋은 결과가
나올 것을 미리 가정하지 않는다. 꺾쇠 분리·제한 gate의 복잡도에 비해 eligible 전환이 드물거나
실제 Firefox 입력 지연이 줄지 않으면 무래스터 재사용을 채택하지 않는다.
저화질·예산 상향·원본 bitmap 복제·Worker/타일·cold metadata 준비 변경은 포함하지 않는다.
새로 수집한 사용자 trace는 없으며 현재 멈춤의 원인을 이 재현 하나로 확정하지 않는다.

로컬 검증: 위 prototype 단언과 기존 회귀 테스트 1건, `git diff --check`.
제품 무변경이므로 전체 build/WASM 재빌드·시각 성능 계측은 재실행하지 않았다.
4200 서버의 제품은 `a54d97575` 그대로이며 원격 게시·push·PR은 수행하지 않았다.
