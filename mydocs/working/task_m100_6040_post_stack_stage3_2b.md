# #6040 Stage 3.2B — 전역 해제 없는 페이지별 갱신 비교 후보

- 일자: 2026-09-08 KST
- 상태: **B1 구현·로컬 게이트·버튼 smoke 및 첫 실제 핀치 분석 완료. 입력 재개 경합을 발견해 B 전체 수용은 미완료**
- 제품 head: `14a306d6c` (기본 후보 `4fbaa9fa5` + 높이 변경 우회 보정)
- A 비교 기준: `712e892f5`; B 착수 계획: `521ddeec1`
- [수행 계획](../plans/task_m100_6040.md), [구현 계획](../plans/task_m100_6040_impl.md),
  [A 실제 핀치 결과](task_m100_6040_post_stack_stage3_2a.md)
- [최소 계측 근거 — 로컬 보존 안내](assets/issue6040-post-stack/README.md)

## 구현한 범위

### 2026-09-09 수용 보정 — 줌 중 폭 resize

계획 커밋 `f59a2b569`에 따라 기존 높이 전용 resize 지연을 폭·높이 모두로 확장했다.
animation/quiet 중에는 공통 geometry와 중심 앵커를 즉시 갱신하고 stale prefetch 큐만
취소한다. 줌 세대는 유지하며 최종 줌 정착이 최신 visible을 예약한다. 스크롤바 폭을
상수로 추정하지 않고, 줌 중 실제 창 resize도 같은 계약으로 처리한다. 평상시 idle
resize, DPR·surface 예산, strict 편집, 문서·renderer 전환은 이번 보정에서 바꾸지 않았다.

- 보정 전 새 계약 테스트: 46개 중 25 pass / **21 fail**. 폭 변경 시 `resize` 동기
  갱신 또는 전체 release로 우회하는 기존 동작을 확인했다.
- 보정 후 zoom preview·scheduler·anchor 집중 검사: **69 pass / 0 fail**.
  양축 스크롤 × 5종 배치 × animation/quiet와 폭 1185→1200→700→1700 전환을 확인했다.
  main/overlay 좌표 일치, 세대 유지, 최종 예약 1회, idle grid resize 계약도 검사했다.
- Studio 전체: **1,583 total / 1,582 pass / 1 skip / 0 fail**.
  `npx tsc --noEmit`, `npm run build`, `git diff --check` 통과. 기존 bundle 크기 경고 유지.
- Chrome 152 인앱, Canvas2D, DPR 2, viewport 1280×720에서 4쪽 보건소 실문서를
  자동 배치로 100→34→50→100% 전환했다. 최종 열 수 4/3/1, visible 쪽 수 4/4/1,
  모든 visible DPR 2, 각 전환 `page.releaseAll` 0, pending image 0, 관찰 오류 0,
  trace complete를 확인했다. 마지막 100% 문서 본문·표의 화면도 확인했다.
- 단일 버튼 smoke의 visibleStable은 각각 188.3/174.3/152.0ms였다. 이는 진단용
  no-opt WASM의 known-work 표본이며 전후 성능 개선율이나 실제 핀치 paint 지표가 아니다.

**폭 resize 보정의 사용자 확인 통과, B 전체 수용은 미완료.** 제품 커밋 `3992866fa`의
4200 서버에서 Firefox 자동 배치의 축소→빠른 확대·줌 중 창 resize와 페이지 점프·빈 화면·
정착 후 흐림 확인을 요청했고, 사용자는 "문제 없어."라고 응답했다. 이를 해당 보정의
정성적 사용자 수용으로 기록한다. 새 계측 JSON이나 실제 ResizeObserver 발생 자료를
받은 것은 아니므로 모든 폭 변경 타이밍의 자동 검증을 대체하지 않는다.
위 인앱 표본의 편집 영역 폭도 모두 1260px로 일정했다. animation/quiet가
끝난 뒤 늦게 도착한 resize는 기존 idle 전역 갱신 경로에 남아 있다. 기존 Stage 3.3의
strict 편집·문서/renderer 경합, CanvasKit, 동일 조건 A/B와 메모리 게이트도 미완료다.
Cold layer metadata 병목은 별도 이슈 대상이며 이 보정이나 #6040 완료 필수 조건에
추가하지 않았다. 이 절은 아래 과거 기록의 폭 resize 항목을 부분 보정한 최신 상태다.

A는 입력 중 반복 정착을 줄였지만 마지막 정착에서 visible 전체를 동기로 그렸다. B1은
`zoom-settled`를 기존 PageRenderScheduler의 visible 큐로 보낸다. 전역 Canvas/LRU 해제를
없애고, 빈 visible → 중심에 가까운 기존 visible → retained 순으로 갱신한다. 줌에는 1·2쪽
동기 fast path도 사용하지 않는다. 기존 4ms soft budget / slice당 최대 2쪽은 그대로이며
한 쪽 안의 WASM 실행을 선점하는 구현은 아니다.

- 구 bitmap은 개별 갱신까지 최신 VirtualScroll 위치와 renderedZoom 기반 preview를 유지한다.
  동일 key는 재렌더하지 않고 exact LRU만 최종 결과로 재사용한다.
- 새 입력은 기존 generation/queue 취소로 미실행 작업을 버린다. 편집·strict는 quiet를 flush한
  다음 예약에 머물지 않고 최신 visible을 동기 완료한다. 같은 DPR인 구 zoom도 이때 갱신한다.
- 아직 대기 중인 같은 그림의 이미지 job은 새 canvas/scale 대상으로 교체한다. 이미 완료된
  동일 그림의 재시도 생략은 유지한다. job 교체 뒤 옛 decode 완료는 새 bitmap을 덮지 못한다.
  아직 갱신하지 않은 구 bitmap의 이미지 작업은 그 bitmap에 유효하며 일괄 취소하지 않는다.
- 구 zoom의 actual pixels를 작은 target estimate로 축소 계상하지 않는다. in-place 전환은
  `max(actual, target)` 하나를 예약하고, exact 완료 뒤 actual로 돌아간다. 별도 staging 복제,
  DPR 변경, 예산 상향, Worker, thumbnail 생성은 없다.
- DEV의 `zoom.raster`는 이제 예약 진입이다. main span·큐·visibleStable을 함께 해석한다.
  clamp로 physical scale이 같아도 구 zoom preview를 최신 완료로 세지 않는다.

**완성된 별도 Canvas를 swap하는 경로는 아직 아니다.** 해당 한 쪽은 기존 in-place 방식으로
갱신하므로 비동기 이미지 공백·렌더 실패·큰 단일 raster까지 무공백으로 보장하지 않는다.
처음 등장하는 미생성 쪽도 차례가 오기 전에는 빈 공간이다. 모든 쪽의 저→중→고 렌더나
저화질 placeholder 실험은 추가하지 않았다.

## 검증

- Studio 전체: **1,547 total / 1,546 pass / 1 skip / 0 fail**.
- `npx tsc --noEmit`, `npm run build`, `git diff --check` 통과. 기존 bundle 크기 경고는 남는다.
- 추가/보정 계약: 1·2·8 visible 비동기 분할, 빈 쪽 우선, soft budget 양보, 새 입력 취소,
  strict flush, actual/target 예약, 동일 DPR의 구 zoom, decode 완료 경합, preview main/layer
  좌표, 동일 크기 resize 및 줌 중 높이 변경. 기존 scroll fast path/settled DPR 테스트도 통과.
- Rust/WASM 변경 없음. WASM SHA-256:
  `24d3d2ffe0c1b43d4f53c23762820112b8298c2a081c9dd4842406819f4130fc`.
- Chrome 152, Canvas2D, DPR 2, 브라우저 viewport 1232×863, 자동 배치. 실제 편집 영역은
  scrollbar에 따라 1197/1212 폭, 686/701 높이였다. 위 브라우저 크기와 혼동하지 않는다.
- 이전과 같은 진단용 `--no-opt` WASM이다. 시간은 버튼 smoke 단일 표본이며 제품 개선율,
  실제 손가락 핀치, 최적화 배포판 또는 compositor paint 결과로 해석하지 않는다.

| 문서 | 34 / 50 / 100% 열 수 | 정착 호출 내부 main raster | 전환 전체 releaseAll | 최종 결과 |
| --- | --- | --- | --- | --- |
| kps-ai, 77쪽 | 4 / 3 / 1 | 모두 0 | 0 / 0 / 0 | 세 배율 모두 visible 누락·pending image·관찰 오류 0 |
| KTX, 실제 4-layer 1쪽 | 1 / 1 / 1 | 모두 0 | 0 / 0 / 0 | 세 배율 완료, 지도·그림·표 확인 |
| 4쪽 보건소 실문서 | 4 / 3 / 1 | 모두 0 | **1 / 1 / 0** | 세 배율 완료, 아래 폭 변경 예외 잔존 |
| exam_kor, 20쪽 | 3 / 2 / 1 | 모두 0 | 0 / 0 / 0 | 세 배율 모두 visible 누락·pending image·관찰 오류 0 |

4종 × 3배율의 known-work 완료를 확인했다. kps 34%와 KTX/4쪽 문서 100%는 패널을 접고
스크린샷도 직접 확인했다. 별도 baseline 픽셀 diff나 모든 상태의 시각 검증을 뜻하지 않는다.
새 main raster는 retained도 포함해 kps 34%에서 12회였다. 이를 visible 8쪽과 혼동하거나
이미지 후처리까지 포함해 '쪽당 언제나 한 번만 렌더'한다고 주장하지 않는다.

## 실측 중 발견한 resize 우회와 남은 한계

초기 B1의 kps 34→50%에서는 가로 scrollbar가 생겨 편집 영역 높이가 701→686으로 바뀌었다.
기존 `onViewportResize`가 preview 도중 전체 해제·visible 동기 렌더를 실행했다. 진단 반복에서
이 resize span은 81ms였다. 따라서 큐에 넣는 수정만으로 모든 정착 burst를 제거하지 못했다.

폭이 같은 줌 중 높이 변경은 geometry/앵커 계산을 즉시 유지하되 quiet/animation을 취소하지
않고 최종 줌 큐에 렌더를 맡겼다. 보정 후 kps 34→50%는 releaseAll 0, resize span 약 0.1ms,
main 9회였다. 서로 다른 단일 반복이므로 UX 개선율로 계산하지 않는다.

**실제 폭 변경은 기존 resize 계약을 유지한다.** 4쪽 문서는 세로 scrollbar의 등장·소멸로
1197↔1212px 폭이 바뀌었다. 34/50% 전환에서 각각 releaseAll 1회와 최대 82.2/88.1ms resize
작업이 남았다. 이 두 경우를 B의 무공백/무전역해제 수용 통과로 세지 않는다. 폭 변경을 큐로
넘기려면 자동 열·중앙 앵커·scroll clamp·resize 취소 회귀를 검증하는 다음 보정이 필요하다.

단일 main raster 최대도 문서별 약 36.4ms(kps), 64.7ms(KTX), 68.3ms(exam)였다. 페이지별
스케줄링은 이런 개별 작업을 쪼개지 못하며 decode 후 재렌더도 별도 비용이다. 이번 snapshot은
transition peak memory를 측정하지 않았으므로 메모리 성능 향상도 주장하지 않는다.

## 다음 순서와 사용자 확인

1. 이 후보의 실제 연속 핀치(자동 kps 100→약34→100%)를 A의 사용자 경험과 비교한다.
   두 번에 나눠 줄여도 된다. 입력 중/뗀 직후 끊김과 새 페이지 등장 과정을 구분한다.
2. 실제 폭 변경 예외를 shared geometry/앵커 회귀와 함께 보정한다. B 수용 완료 전 필수다.
3. B1 후에도 기존 bitmap의 이미지 공백이 의미 있는 문제이면 예산 내 staging swap을 비교한다.
   신규 저화질 preview는 별도 후속 실험이며 자동으로 시행하지 않는다.
4. 실제 핀치·빠른 역방향/편집/renderer 전환·CanvasKit 실문서·peak memory 및 통제 A/B는
   B 잔여/Stage 3.3·4 게이트다. 지금 제출 준비 또는 최종 B 채택으로 올리지 않는다.

서버는 `http://127.0.0.1:4200/?renderer=canvas2d&url=/samples/exam_kor.hwp&scrollProbe=1`.
검증 탭에서 kps-ai를 열고 자동 100%로 준비한다. 기록 시작은 사용자가 조작 직전에 누른다.
원격 push·PR·코멘트는 하지 않았다.

## 2026-09-09 실제 핀치 — 분할 확인, 입력 재개 경합 발견

사용자가 서버 재시작 뒤 핀치 기록을 완료했다. 탭을 새로고침하지 않고 표시된 JSON을 확보했다.
source `3d6e3246f` / product `14a306d6c`, Canvas2D, DPR 2, 브라우저 1232×863이다.
이번 문서는 **exam_kor 20쪽**이며, 실제 기록은 약 **54.70→27.62→112.27%**, 128개 줌 입력,
열 수 1/2/3이었다. 패널 배치 select에는 '한 쪽'이 표시됐지만 원장의 실제 topology는 변했다.
select 표시만으로 실행 배치를 단일 열로 단정하지 않는다. 이전 A의 kps-ai/100% 시작 표본과
문서·배율·입력 조건이 달라 시간 개선율을 계산하지 않는다.

- 원장: stopped, knownWorkReadyAtStop=true, dropped 전부 0, 관찰 errors 0.
- 정착 진입 6회, **전역 해제 0회 / 정착 진입 안의 main raster 0회**.
- main 24회 / 총 inclusive 1,159.5ms / 단일 최대 61.1ms. image reraster 7회 / 313.4ms.
  wasm.layerRaster는 이 경계와 중첩되므로 합산하지 않는다.
- 입력 delivery 구간의 rAF gap p50 8.3ms, p95 48.6ms, max 129.4ms. 구간 전체 long task
  22회. rAF는 compositor 프레임이 아니며 이 표본만으로 체감 개선을 확정하지 않는다.
- 마지막 입력 delivery→마지막 main 종료 235.0ms. 사용자 손가락 종료/최종 paint 지연은 아니다.
- 종료 snapshot의 visible [0]은 DPR 2이고 visible/prefetch 큐 및 pending image는 모두 0.

### 수용 차단: 입력과 첫 animation callback 사이의 큐 실행

상대 시각 2555.1ms에 새 줌 입력이 도착해 generation 37, animating/inputActive/rasterPending이
true로 바뀌었다. 그러나 **2555.2ms에 page index 5의 main raster가 61.1ms 실행**됐다.
첫 geometry.zoom은 그 뒤 2626.6ms였으며, 같은 페이지 image reraster도 2634.8ms에 48.6ms
실행됐다. 상태는 span 반환 시점 표본이지만, 입력·smooth·main·geometry의 시작 순서 자체가
기록에 있어 새 입력 뒤 queue 작업이 실행됐음을 확인할 수 있다.

현재 `createPageRenderWork.isValid`는 renderWorkGeneration과 lookup key를 확인하지만
ViewportManager의 입력 대기는 확인하지 않는다. renderWorkGeneration 취소는
`onZoomChanged`의 preview 분기에서 이뤄진다. 첫 animation callback 전에는 구 zoom/key와
작업 세대가 아직 같아 기존 frame 작업이 통과할 수 있다. B1 테스트는 cancelPendingPrefetch를
직접 먼저 실행했으므로 이 입력→첫 callback 간격을 놓쳤다.

다음 보정은 이 순서를 강제로 재현하는 테스트부터 추가하고, 첫 geometry callback에 기대지 않는
입력 대기 상태의 dispatch 차단 및 최신 정착 재예약을 확인한다. 비동기 이미지 후처리는 구 surface의
소유권과 완료 복구를 보존해야 하므로 단순 cancelAll을 넣지 않는다. 이 페이지의 후처리는 앞선
잘못된 dispatch에서 시작됐지만, 기존 pending 이미지 일반 경합도 별도로 확인한다.
앞서 기록한 실제 폭 변경 예외 역시 남아 있다. 지금은 B를 최종 채택하거나 staging/저화질 실험으로
확장하지 않으며, 추가 사용자 녹화를 이 재현 테스트의 선행 조건으로 요구하지 않는다.

[최소 요약 — 로컬 보존 안내](assets/issue6040-post-stack/README.md)을 남겼다. 전체 원시는 ignored
`tmp/issue6040-b1-pinch-20260909.json`에만 보존했으며 SHA-256은
`a9c5968d28ab672f4c01de3760038a266f6554d0ab48f6705abfc01006ea6f1f`다.
이번 수집·분석에서는 제품 코드와 서버를 바꾸지 않았다.

## 2026-09-09 Stage 3.2B 보정 — 첫 frame 전 dispatch와 이미지 취소 복구

사용자의 후속 진행 승인으로 위 경합을 보정했다. 계획 커밋은 `45986f013`이다.

1. 실제 ViewportManager와 PageRenderScheduler가 공유하는 가짜 frame host에서 구 render
   callback을 먼저 예약한 뒤 ctrl-wheel을 전달했다. 첫 geometry 이벤트와 세대 변경은 아직
   없고 VM pending/animating만 true인 상태다. 수정 전에는 구 페이지 [1, 2]를 그려 재현됐고,
   수정 후에는 main/decode 0회, stale 3건 폐기, 기존 surface 유지, 다음 정착의 최신 3쪽 완료를
   확인했다. 입력 대기와 animation 각각에서 visible/retained-transition/prefetch도 검사했다.
2. CanvasView dispatch에 기존 `isZoomPreviewActive()` 검사를 추가했다. 새 입력마다 별도
   이벤트·전 문서 scan을 추가하지 않는다. 거부한 prefetch 예약은 반환하고 최종 정착이 최신
   작업을 재구성한다. 기존 surface와 이미 진행 중인 유효한 이미지 작업은 취소하지 않는다.
3. 이미지 수명 확인 중 추가 결함도 재현했다. `cancelReRender`/`cancelAll`이 pending job만
   지우고 retry key를 남겨, 같은 그림의 다음 요청이 완료된 것으로 오인되어 생략됐다.
   **실제로 취소하는 pending 쪽의 key만 제거**한다. 완료된 다른 쪽의 decode 재사용은 유지한다.
   개별/전체 취소 테스트 둘 모두 수정 전 실패, 수정 후 늦은 구 callback 무시·새 canvas/scale
   복구·정상 완료된 다른 페이지 재사용을 통과했다. 줌에 cancelAll을 추가한 것이 아니다.

### 검증 결과와 해석 경계

- 새 테스트 9개 모두 수정 전 실패를 확인했다. 집중 53개 통과, Studio 전체 **1,556 total /
  1,555 pass / 1 skip / 0 fail**. `npm run build`(TypeScript 포함), `git diff --check` 통과.
  기존 bundle 크기 경고만 남는다. Rust/WASM은 변경하지 않았다.
- 실제 브라우저 Canvas2D/DPR 2, **669×863** viewport에서 exam_kor·kps-ai·KTX·4쪽 문서의
  34/50/100% 총 12개 버튼 smoke를 확인했다. visible 누락, pending image/prefetch,
  visible/prefetch render queue, 관찰 errors는 모두 0이었다. kps/4쪽 34%는 2열, 나머지는
  1열이었다. KTX 50%에서 지도·표가 표시되는 것도 스크린샷으로 직접 확인했다.
- [최소 smoke JSON — 로컬 보존 안내](assets/issue6040-post-stack/README.md)을 남긴다.
  일부 snapshot에는 scroll-settle timer가 남아 있으므로 모든 브라우저 작업의 완전 종료를
  증명하는 자료는 아니다. 버튼 smoke이고 viewport도 이전 실제 핀치와 다르므로 성능 A/B나
  체감 개선율로 해석하지 않는다. 새 실제 핀치 입력은 아직 계측하지 않았다.
- 원장에 있던 61.1ms 작업과 후속 48.6ms를 그대로 절감 시간으로 더하지 않는다. 재현 테스트는
  해당 잘못된 dispatch 경로가 막혔음을 증명할 뿐이며, 새 제스처의 시간을 예측하지 않는다.
- DPR·예산·줌 앵커·geometry·스크롤 정책은 그대로다. 개별 raster의 긴 실행 시간, 유효한 기존
  이미지 완료 비용, 실제 폭 변경의 전체 갱신, staging/peak memory 및 통제 비교는 남는다.
  Stage 3.2B 보정이며 B 전체 완료·Stage 3.3 진입·제출 준비로 올리지 않는다.

4200 서버는 보정 코드를 제공한다. 기존 실제 핀치 원시는 보존했고 원격 push/PR/코멘트는 하지 않았다.
