# Stage 3.2B — 고배율 재핀치의 Firefox 업로드 병목 조사

- Issue: #6040
- 조사일: 2026-09-09 KST
- 제품 head: `0345107c586c2088802d6629ba13ac6a66e3694c`
- 브랜치: `codex/issue-6040-post-stack-zoom`
- 사용자 승인 범위: 프로파일 근거로 불필요한 재렌더·복사 유발 경로를 조사하고 근거가 있는 최소 보정을 검토한다.
- 결과: 조사 완료, 제품 보정 없음. B 전체 수용·회귀 해소·성능 개선 판정은 하지 않는다.

## 1. 입력과 해석 범위

사용자가 `exam_kor.hwp`, Canvas2D, 100→500% 부근의 재핀치에서 중간 배율이 생략되는 느낌을
보고했다. 읽기 화질 복원은 수용했지만 이 고배율 응답성은 별도 차단 조건이다.

| 증거 | 관찰 | 한계 |
| --- | --- | --- |
| 인앱 수동 구간 | DPR 2, 1507×863, 7.958초, rAF 중앙값 8.3ms/최대 174.2ms, 핀치 dispatch 지연 최대 49.1ms | 물리 입력 횟수·시점이 Firefox와 다름 |
| Firefox 패널 JSON | DPR 2, 1512×845, 4.839초, rAF 중앙값 8ms/최대 410ms, dispatch 지연 최대 153ms | Long Tasks API 미지원; 빈 배열은 긴 작업 0건이 아님 |
| Firefox native profile | Firefox 155, macOS 26.5.2, Apple M4 Pro, 7.269초, symbolicated | 위 JSON과 별도 실행; 시점을 직접 대응시키지 않음 |

native 파일 이름은 `Firefox 2026-09-09 03.00 profile.json.gz`이며 SHA-256은
`c16e3594efaa4dbca9348d17bf67350bedd5a2c4ac36477671f2e218fa8e9592`다.
사용자가 개인정보 제외 옵션으로 내보냈다. 원문은 로컬 사용자 파일로만 보존하며 저장소·원격에
복제하지 않는다. 숨겨진 thread 제외 후에도 문서 main, GPU main, Renderer, CanvasRenderer,
parent Compositor가 남아 있어 추가 개인정보 포함 export를 요청하지 않았다.

## 2. native profile의 핵심 근거

시간은 `meta.profilingStartTime` 기준 상대값이다. `phase=1`인 완료 interval의
`endTime-startTime`을 사용한다. 서로 겹친 main/renderer interval을 합산하지 않는다.

| 스레드/마커 | 시작(ms) | 지속(ms) | 내용 |
| --- | ---: | ---: | --- |
| Renderer / Texture uploads | 2104.11 | 984.60 | 1,245,159,424 bytes uploaded, 164 items |
| Renderer / Texture uploads | 3112.70 | 680.94 | 1,245,159,424 bytes uploaded, 164 items |
| 문서 main / ViewManagerFlush | 2104.04 | 309.39 | Transaction ID 8094 |
| 문서 main / LongTask | 2102.82 | 310.72 | 위 화면 갱신을 포함하는 task |
| 문서 main / setTimeout callback | 3880.00 | 241.58 | `ensureDeferredTask` callback |

업로드 bytes는 렌더러가 보고한 작업량이다. 파일 크기·상주 GPU 메모리·실제 장치 버스 전송량과
동일시하지 않으며 164항목을 문서 164쪽으로 해석하지 않는다. 다른 탭이 함께 기록된 전체 window의
Renderer이므로 전량을 특정 페이지 하나에 귀속시키거나 두 업로드가 같은 bitmap의 중복이라고
단정하지 않는다.

2104~3089ms 구간에서 확인한 주요 stack:

- 문서 main: `GetTextureClient → BorrowDrawTarget → CopyToTextureClient → CopySurface →
  SkCanvas::writePixels → _platform_memmove`.
- Renderer: `upload_to_texture_cache → TextureUploader → glTexSubImage2D` 및 메모리 복사.
- 같은 구간 main에는 event-loop 대기 sample도 많다. 전체 구간의 idle 비율로 짧은 응답 지연을
  부정하지 않으며, leaf의 category만 읽으면 native frame의 category 상속을 놓치므로 category
  집계는 결론에 사용하지 않는다.

`RefreshObserver`의 Accessibility/Synthetic mouse 이름이 긴 interval에 붙어 있지만 이름만으로
접근성 기능을 원인으로 지목하지 않는다. 샘플 stack과 직접적인 Texture uploads 마커를 우선한다.

**판정:** 페이지 raster CPU 작업만이 아니라 화면 게시에 따른 native 복사·업로드가 큰 병목이다.
WASM draw 반환 뒤에도 작업이 남으므로 JS scheduler의 페이지별 분할·취소만으로 전체 응답성을
보장할 수 없다. Firefox 자체 결함이나 회귀 도입 commit은 아직 확정하지 않았다.

## 3. 제품 코드 대조와 실행 확인

`pageSurfaceDescriptor()`는 document/revision, geometry, backend, profile, clamped renderScale,
layer 수로 bitmap key를 만든다. `buildVisibleRenderWork()`는 exact key를 재예약하지 않는다.
`updateRenderedPageZoomPreview()`는 기존 bitmap의 CSS transform을 갱신하며 bitmap 크기 자체를
바꾸지 않는다. 따라서 이 두 경로에 추정 기반의 raster 생략·캐시를 추가하지 않았다.

실제 Vite SSR CanvasView prototype에서 `width=1122.5`, `height=1587.4`, raw DPR 2,
4-layer로 descriptor/visible queue를 실행했다. renderer와 browser를 실행한 성능 테스트는 아니다.

| 조건 | 결과 |
| --- | --- |
| 354→500%, 동일 content/backend/profile | clamp 후 key 동일, visible raster 예약 0건 |
| 500→200%, 물리 renderScale 변경 | key 변경, visible raster 예약 1건 |
| 동일 고배율에서 document revision 변경 | key 변경, visible raster 예약 1건 |

4-layer의 해당 target 예약은 268,517,080px다(각 축 안전 예약 +1px 포함). Canvas 하나의 67M
clamp가 여러 layer의 전체 비용까지 67M으로 제한하는 것은 아니다. 보이는 일부 영역과 관계없이
페이지 전체를 고해상도로 보유하는 구조가 큰 비용 후보다.

위 검사는 **줌 정착 queue 경로**에 한정된다. strict/편집/이미지 완료/폭 resize의 모든 경로가
중복이 없다는 증명은 아니다. 특히 `refreshRenderSurfacePlan(true)`는 stale renderedZoom에
재렌더할 수 있고, `ensureDeferredTask`의 retained-transition도 별도 비용으로 남는다.
이 부수 경로를 바꾸는 것으로 984ms 업로드가 해결된다고 주장하지 않는다.

추가 실행:

```sh
cd rhwp-studio
node --experimental-strip-types --test \
  tests/canvas-view-zoom-scheduler.test.ts \
  tests/canvas-view-zoom-preview.test.ts
```

샌드박스 밖에서 **43/43 통과**. 현재 source/WASM 변경이 없으므로 전체 build·시각 검증은
재실행하지 않았다. 기존 4200 서버의 제품 후보도 바꾸지 않았다.

## 4. 채택하지 않은 즉석 수정과 다음 승인 경계

- quiet timer 연장 또는 zoom 보간 감도 변경: 업로드 비용을 없애지 못하며 입력 UX를 바꾼다.
- visible DPR 재하향: 읽기·편집 화질 보호 요구와 충돌한다.
- 무조건 `will-change` 또는 snapshot/staging 추가: 추가 surface·복사·업로드를 만들 가능성이
  있어 계측 없이 제품에 넣지 않는다.
- 범용 재렌더 생략: 편집·decode 완료를 놓칠 수 있으며 현재 exact-key 경로와 중복된다.

다음은 **승인 전 제안**이다. 기존 B의 전 페이지 bitmap 수명 보정과 구분해서 고배율 가시 영역
렌더링을 별도 설계·제한 실험한다. 화면에 필요한 물리 픽셀 밀도는 유지하고 작업 면적을 줄이는
것이 목표다. 가시 영역/타일의 clip, 좌표, 원본 paint 순서, 1~4 layer, 스크롤 시 새 영역,
그림·표·selection·캐럿, 취소/오래된 결과, temporary+resident 비용의 계약을 먼저 작성한다.

성공 기준은 raster 시간만이 아니라 Firefox Texture uploads/복사량·frame gap·입력 지연 및
스크롤 시 빈 영역·읽기 화질·peak memory다. 기존 bitmap을 먼저 잘라 복제하는 것과 원래부터
부분 영역만 렌더하는 것은 비용이 다르므로 같은 최적화로 섞지 않는다. 저배율 overview LOD,
전 문서 선렌더, 무제한 cache, Worker 도입을 자동 포함하지 않는다.

이는 기존 계획의 타일링 제외 경계를 넓히므로 작업지시자 승인 후 상세 설계부터 시작한다.
이번 조사만으로 타일 방식 채택 또는 B 완료를 선언하지 않는다. 원격 게시·push 없음.

## 5. 첫 진입 다중 페이지 스크롤 — 추가 조사 경계

2026-09-09 사용자는 첫 로딩 후 빠르게 4페이지 배치로 이동하고 즉시 스크롤하면 빈 칸과 끊김이
보인다고 추가 보고했다. 현재 후보에서 이 물리 입력을 새로 계측한 결과는 아직 없다. 위 고배율
프로파일과 같은 원인으로 단정하지 않는다. 4페이지 배치의 실제 모드·열 수·배율도 기록해야 한다.

참고한 [브라우저의 메인 스레드는 비싸다](https://kciter.so/posts/the-expensive-main-thread/)는
작업 분할·병합·우선순위·지연 및 Worker 이동을 구분한다. 여기서 예약 개선과 작업량 절감을
혼동하지 않는 원칙을 적용한다. API의 실제 제어권 양보와 Worker Canvas 사용은
[Scheduler.yield 문서](https://developer.mozilla.org/en-US/docs/Web/API/Scheduler/yield),
[OffscreenCanvas 문서](https://developer.mozilla.org/en-US/docs/Web/API/OffscreenCanvas)를 함께 확인했다.
새 API 채택이나 지원 브라우저 확대·축소는 이번 조사에 포함하지 않는다.

현재 제품 코드에서 확인한 사실:

- `CanvasView.updateVisiblePages()`는 scroll/scroll-settled/zoom-settled의 visible을 scheduler에
  전달한다. initial/resize/strict/편집 경로는 별도로 동기 visible 처리가 남아 있다.
- scroll에서는 `allowVisibleFastPath=true`다. 전체 문서/visible 수가 아니라 **실제 예약할
  visible 작업이 2개 이하**일 때 scheduler가 호출 스택에서 즉시 slice를 실행한다.
- 그 외 visible은 rAF, 선택 prefetch는 idle 또는 timeout을 사용한다. visible 대기가 있으면
  prefetch를 진행하지 않으며 새 generation으로 오래된 대기 작업을 교체한다.
- 4ms는 page 경계에서만 확인하는 soft budget이다. `work.run()` 내부의 동기 raster를
  선점하지 못한다. rAF로 이동하더라도 이 호출이 길면 paint 전 작업이 길어진다.
- B는 이미 그린 bitmap 재사용이며 처음부터 bitmap이 없는 페이지의 즉시 표시를 보장하지 않는다.
  빈 화면은 문서 파일 로딩 미완료뿐 아니라 raster/cache miss/queue 대기에서도 생길 수 있다.

실제 `PageRenderScheduler`를 import하고 한 page가 가짜 시계 40ms를 소비하게 하는 계약 확인을
실행했다. 브라우저·WASM을 실행하지 않았으며 40ms는 실측값이 아니다.

| 설정 | setDesiredWork 반환 전 실행 | 첫 예약 frame 이후 |
| --- | --- | --- |
| fast path 허용, visible 작업 2개 | 1개, 가짜 시간 40ms | 나머지 1개 실행 |
| fast path 허용, visible 작업 4개 | 0개 | 1개 실행, 3개 대기 |
| fast path 금지, visible 작업 2개 | 0개 | 1개 실행, 1개 대기 |

따라서 fast path 제거는 입력 호출 스택의 부하는 옮길 수 있지만 한 page의 실행 시간을 줄이지
못한다. 반대로 모든 새 페이지를 스크롤 정착까지 미루면 blank 노출 시간이 길어질 수 있다.
사용자 제보의 인과관계나 fast path 제거 채택을 이 계약 확인만으로 확정하지 않는다.

다음 비교는 문서 cold 첫 진입과 fully-settled warm 왕복을 구분하고 같은 source 기준선/후보,
WASM·폰트·브라우저·viewport·DPR·배치·입력을 사용한다. visible 최초 내용 표시와 최종 화질 도달,
queue 대기/실행·cache hit/miss·rAF·입력 지연을 함께 측정한다. main JS/WASM이 짧아도 지연이
남으면 native 복사/업로드를 별도로 조사한다. 화질 하향으로 성공 기준을 우회하지 않는다.

이 회귀 조사와 기존 B 범위의 보정은 #6040에 남긴다. Worker나 렌더 단위 변경이 필요하면
별도 이슈의 Stage 1 설계로 분리한다. 이번에는 제품 코드·4200 서버·원격 게시를 변경하지 않았다.
