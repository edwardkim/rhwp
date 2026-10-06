# 구현 계획 — Task M100 #6040

## 현재 유효한 재개 설계

2026-09-10 메모리 gate 실패의 승인된 최소 보정을 `931306732`로 구현했다.
[메모리 보고서](../working/task_m100_6040_memory_gate.md)에 동일 시나리오 2회 및 6개 새 회귀를
기록한다. 줌 정착에서 원장 초과 시 먼 offscreen active만 반환해 retained-transition의 admission
우회를 막았다. visible 읽기 화질·preview·strict 편집·현재 focus를 보존한다. 원장 비용 함수는
기존 실제 크기와 target 중 큰 값으로 공유한다. 일반 scroll 정책·예산 상수·DPR 정책은 그대로다.
사용자가 클릭 없는 읽기·줌 직후 스크롤·재확대의 기준선 대비 무회귀 요청에 “문제없어”로
응답해 정성 수용을 기록했다. 다음은 제출 범위·최종 보고서/PR 초안 정리다.
아래 Stage 4.3 제외 실험 재개나 원격 push·PR 생성·merge·이슈 종료 승인이 아니다.

2026-09-10 현재 검증 후보는 Stage 4.2까지와 최신 devel #6902를 결합한 로컬 통합이다.
[통합 검증 기록](../working/task_m100_6040_integration_20260910.md)에 충돌 해소와 추가 회귀
2개를 기록한다. 아래 Stage 4.3은 원 worktree에 보존한 제외 실험이며 이번 후보에 적용하지 않는다.

### Stage 4.3 — 고배율 동일 surface 확정의 제한 실험

2026-09-10 **조사·계획 완료, 구현 승인 대기**. 제품은 `a54d97575` 그대로다.
[조사 결과](../working/task_m100_6040_post_stack_clamp_reuse_investigation.md)에서 같은 physical
key도 구 zoom이면 raster 1회를 요청함을 확인했다. 그러나 Canvas 꺾쇠 선 폭, DOM 그림 좌표,
지연 callback은 zoom에 의존한다. 단순 exact-key skip 또는 renderedZoom 덮어쓰기는 채택하지 않는다.

#### 선택과 범위

첫 실험은 **Canvas2D의 완료된 active canvas-only surface**에 한정한다. DOM flow-image,
진행 중 이미지/RawSvg 보정, HF 편집, CanvasKit, detached LRU 복원, strict 편집에서는
기존 경로로 fallback한다. 문서·revision·backend·profile·geometry·layer·정확한 physical
scale이 모두 같아야 하며 화면 표시 품질과 예산은 기존 계약 그대로다. DPR 숫자만 같다는
이유 또는 scale의 느슨한 오차 범위로 hit를 넓히지 않는다.

배율 의존 꺾쇠를 bitmap 밖 작은 DOM/SVG 표시로 분리하는 후보를 먼저 검증한다. 선택 이유:

- 기존 전체 bitmap 유지: 현재처럼 정확하지만 같은 scale에서도 전체 raster가 필요하다.
- 꺾쇠 영역만 문서 patch replay: 내용 복원·겹침 계약이 필요하고 큰 Canvas를 수정하므로
  native 업로드 절감을 보장하지 못한다. 첫 후보로 쓰지 않는다.
- 작은 DOM/SVG 꺾쇠: 화면상 선 두께를 독립 갱신할 수 있으나 paint 순서·정렬·소유권 검증이
  추가된다. 페이지 크기 Canvas 복제 없이 시험할 수 있어 제한 후보로 택한다.

기존 bitmap에 꺾쇠가 합성돼 있으면 새 계약이 아니다. 지우기·metadata 세탁으로 바꾸지 않는다.
분리 모드로 정상 렌더한 surface에만 표시 계약/version 증거를 남기고 재사용한다. 모드 전환의
최초 1회 raster 비용도 측정에 포함한다. 필요 범위를 넘어 전체 renderer guide 구조를
리팩토링해야 한다면 먼저 중단하고 별도 계획을 받는다.

#### 파일별 구현 후보

- `page-margin-guides.ts`와 해당 tests
  - 모서리 geometry·edges·화면상 0.8–1.5px 규칙을 하나의 계산 계약으로 재사용한다.
    화면상 위치/두께와 clip 계약을 현재 Canvas 출력과 대조한다. 기존 drawing API는 유지한다.
- `page-renderer.ts`와 신규/관련 renderer tests
  - 좁은 opt-in render context로만 본문 Canvas의 guide paint를 분리한다. 일반/strict/HF/
    fallback 기본값은 기존대로다. async 재렌더에서도 같은 guide mode를 전달하고 실패 시
    일반 full raster로 복원한다. 아직 불명확한 pending image를 성공으로 간주하지 않는다.
  - 분리 guide의 z-order는 기존 flow Canvas 위·기존 front plane 아래 등 실제 paint 순서를
    보존해야 한다. 무조건 모든 layer 위에 놓지 않는다. pointer-events는 없어야 한다.
  - DOM flow-image 내부 재배치 일반화와 callback 재타기팅은 이번 후보에서 하지 않는다.
- `canvas-view.ts`와 zoom scheduler/preview/LRU/읽기 tests
  - zoom-settled의 active surface에서만 capability/완료/identity/physical/표시 계약을 검사한다.
    성공 시 bitmap dimension 할당·draw·clone 없이 기존 요소의 CSS geometry를 확정하고
    preview transform을 해제한다. renderedZoom과 requested/effective DPR 진단을 일치시킨다.
    actual surface pixels는 재사용 사실에 맞게 유지하며 정책 예산을 바꾸지 않는다.
  - guide layer 생성·position·release·detach/restore 시 잔여 DOM/이중 소유권이 없어야 한다.
    detached 자체를 최적화하지 않더라도 새 guide 계약의 cache key와 수명 처리는 필수다.
  - 한 메서드에서 skip해도 다른 refresh 경로가 full raster하는지 검사한다. strict의 완료
    계약은 유지하며 강제 편집 갱신을 캐시 최적화로 가리지 않는다.
- `page-scroll-probe.ts` 및 관찰기
  - 기본적으로 변경하지 않는다. before/after 같은 관찰 adapter·완료 조건을 사용한다.
    새 guide 수명에 필요한 완료 검사만 있다면 별도 계약 테스트와 양측 호환성을 먼저 확인한다.
- scheduler·ViewportManager·VirtualScroll
  - quiet/smoothing/frame 정책·앵커·공유 좌표·열 배치는 변경하지 않는다.

#### 재현·검증 gate

1. 기존 재현을 새 회귀 테스트로 확장한다. completed canvas-only 동일 scale에서는 성공 시
   새 main raster 0회, scale/revision/geometry/backend/profile/구성/완료 여부가 다르면
   기존 렌더로 fallback해야 한다. 300→500과 500→200은 재렌더가 필요한 대조군이다.
2. 354→500→354 반복의 CSS page box·guide 두께/위치·layer 순서·requested DPR·actual pixels,
   pointer/selection/caret·스크롤 위치를 확인한다. 완료되지 않는 preview를 재발시키지 않는다.
3. 새 입력·취소·문서/renderer 전환·편집·resize·decode/fallback·LRU 왕복의 최신 결과와
   단일 소유권을 검증한다. 실행 중인 WASM이나 브라우저 업로드를 선점한다고 주장하지 않는다.
4. 집중 검사 후 Studio 전체 test와 TypeScript 포함 build. 제품 변경 없는 조사 단계에서는
   이 전체 게이트를 실행하지 않는다. Rust 변경이 필요해지면 범위/계획을 먼저 다시 검토한다.
5. 동일 WASM/폰트/Firefox·viewport/DPR/문서/자동 배치에서 `a54d97575`와 A/B한다.
   분리 모드 첫 진입·완료 뒤 재방문·미완료 중 재입력을 각각 구분한다. 패널은 접고 관찰
   on/off 영향도 분리한다. 고배율 픽셀 비용 때문에 후보 둘을 동시에 큰 배율로 유지하지 않는다.
6. 성공 범위와 fallback 빈도를 보고한다. eligible 전환의 raster 0회만으로 채택하지 않는다.
   실제 rAF/입력 지연·visibleStable/retainedComplete·단순 pixels/peak 메모리, 가능하면
   Firefox native 복사/업로드를 비교한다. 원본 surface의 위치·배율 변경만으로도 compositor
   비용이 남을 수 있다. 새 사용자 계측은 후보가 준비된 뒤 필요한 비교만 요청한다.
7. 화질·좌표·이미지 누락·오래된 결과·strict 지연은 차단한다. 시간 중앙값 +20% 경보,
   guide overlay 비용·추가 surface/DOM 증가·첫 전환 비용을 함께 보고한다. 표본 수·측정하지
   않은 조건은 명시한다. 실익이 작거나 일반 경로를 과도하게 바꿔야 하면 제품 도입을 보류한다.

#### 승인 경계

이 계획은 제한 실험 허가를 받기 위한 것이며 최종 제품 채택안이 아니다. 최초 페이지의 cold
metadata/전체 raster, 모든 미완료 고배율 전환, GPU 전체 병목을 해결하는 약속이 아니다.
Worker·타일·저화질 placeholder·DPR 하향·예산 증액은 포함하지 않는다. Stage 4.2 사용자
결과는 보존하며 통합 수용·remote push·PR 생성·이슈 종료는 계속 보류한다. 실험 결과를
검토한 뒤 제품에 유지할지, 별도 구조 개선에서 진행할지 다시 승인받는다.

### Stage 4.2 — 줌 정착 후 frame 기회 양보 후보

2026-09-10 **사용자 승인 후 구현·자동 재측정 완료, Firefox·통합 수용 대기**.
Stage 4.1 `76e8496a4` 위에서 zoom-settled의 visible 마지막 slice와 화면 밖 dispatch 사이
경계 하나를 보정했다. 일반 스크롤·DPR 예산·화질·LRU admission은 바꾸지 않았다.
[Stage 4.2 보고서](../working/task_m100_6040_post_stack_stage4_2.md)에 exact source hash,
검사·버튼 A/B·교환 비용과 미완료 게이트를 기록했다. 아래는 승인받아 수행한 설계다.

#### 관찰과 설계 선택

`runVisibleSlice()`의 finally는 visible 큐가 비면 `ensureDeferredTask()`를 부른다.
`retained-transition`은 즉시 0ms timer가 되어, 다음 frame 관찰 전에 무거운 page raster를
시작할 수 있다. Stage 4.1 100% 본 회차 1에서는 visible가 162.3ms에 끝났으나 화면 밖 작업
166.0–226.2ms 뒤 228.9ms에 visibleStable이 관찰됐다. 이는 실행 순서의 증거이지 paint
timestamp는 아니다.

선택 후보는 **zoom visible 실행 → 다음 rAF에서 예약만 수행 → 기존 deferred task에서
화면 밖 한 쪽 실행**이다. `setTimeout(0)`만 유지하거나 microtask로 바꾸는 안은 frame 경계를
명시하지 못한다. 고정 50/150ms 대기·모든 retained의 idle 전환은 읽기 이후 전체 완료를
필요 이상 늦출 수 있어 첫 후보로 택하지 않는다. Worker·타일·staging은 이번 경보에 비해 범위가 크다.

[HTML event loop](https://html.spec.whatwg.org/multipage/webappapis.html#event-loop-processing-model)는
렌더 기회와 task 실행을 구분하며 중간 rendering 없이 timer를 합쳐 실행하는 경우도 허용한다.
[rAF 문서](https://developer.mozilla.org/en-US/docs/Web/API/Window/requestAnimationFrame)상
callback은 repaint 전이며 숨긴 탭에서는 중단될 수 있다. 따라서 후보를 `afterPaint` 보장으로
명명하지 않는다. 브라우저가 실제로 보여주는지는 별도 검증한다(자료 확인: 2026-09-10).

#### 파일별 변경 경계

- `rhwp-studio/src/view/page-render-scheduler.ts`
  - `setDesiredWork`에 기본값이 기존 동작인 명시적 opt-in 옵션을 추가한다. 후보 이름은
    `yieldAfterVisible`; zoom 여부를 scheduler 내부에서 추측하지 않는다.
  - 해당 generation에서 visible 작업을 실제 실행하고 마지막 visible 큐가 비었을 때만
    frame 대기를 한 번 예약한다. exact/cache hit만 있어 실행이 없거나 화면 밖 큐가 없으면 생략한다.
  - 화면 밖 대기 소유권은 기존 deferred task와 같은 취소 경로로 관리한다. frame/idle/timer 중
    하나만 소유하고, 대기 frame callback은 유효한 소유권·현재 generation·빈 visible 큐를
    재검사한 다음 기존 deferred 정책으로 넘긴다. 이 callback 자체에서 raster하지 않는다.
  - 새 visible 요청, cancelAll, reentrant 새 generation, 예외에서 이전 대기가 재생되지 않게 한다.
    일반 요청으로 교체되면 zoom 전용 정책도 해제하며 이후의 진짜 scroll 즉시성을 유지한다.
  - snapshot에는 이 대기도 pending으로 나타나야 한다. 기존 `idleScheduled`는 이미 timer
    대기를 포함하므로 deferred frame 포함 여부를 문서화하고, 완료 판정이 중간에 true가 되는
    공백을 만들지 않는다. 계측기 ready 조건을 완화하지 않는다.
  - host API의 기존 requestFrame/cancelFrame만 사용한다. 새 전역 timer 상수나 busy polling,
    background 전용 timer 우회를 추가하지 않는다. 화면 복귀 후 유효 작업 재개를 검증한다.
- `rhwp-studio/src/view/canvas-view.ts`
  - `setDesiredWork` 호출에서 `isZoomSettled`에만 opt-in한다. scroll/scroll-settled/strict의
    옵션 기본값, 동기 편집 완료, Stage 4.1 동일 viewport 중복 제거는 그대로 유지한다.
  - page renderer·CSS preview·ruler/anchor·DPR planner·surface cache key는 수정하지 않는다.
- `rhwp-studio/tests/page-render-scheduler.test.ts`
  - 새 rAF가 같은 frame batch에서 실행되지 않는 host를 사용해 timer-first 순서를 재현한다.
    visible → 대기 frame → deferred timer/idle을 독립적으로 진행하며 각 시점의 pending을 검증한다.
  - opt-in on/off, 1/여러 visible slice, offscreen 없음, visible 전부 invalid/exact-hit,
    다수 retained, idle 지원 유무, 새 generation·strict·cancel·실패·reentrant 교체를 검사한다.
  - rAF가 잠시 진행되지 않아도 task가 우회하지 않으며, 재개 시 한 번만 실행되는지 확인한다.
    테스트 host의 frame 완료를 브라우저 paint 완료라고 서술하지 않는다.
- `rhwp-studio/tests/canvas-view-zoom-scheduler.test.ts`
  - 실제 CanvasView의 zoom opt-in과 일반 scroll/strict opt-out을 연결한다. Stage 4.1의
    실제 DPR 경합, 새 zoom 전에 남은 큐 취소, 클릭 없이 raw 화질 회복 계약을 다시 검증한다.
- 기존 zoom/scroll 관찰 파일은 이 후보에서 변경하지 않는다. 새 상태 때문에 관찰 계약 추가가
  꼭 필요하면 before/after 공통 adapter로 분리하고 제품 개선과 계측 변경 효과를 구분한다.

#### 검증 순서와 중단 기준

1. old code에서 dispatch 순서 재현 → 최소 후보 → scheduler/zoom/preview/scroll 집중 검사.
2. Studio 전체 `npm test`(자식 프로세스 제한에 따라 sandbox 밖), TypeScript 포함 build.
3. 동시 빌드 없는 동일 WASM/폰트/viewport/DPR에서 Stage 4.1 전후 A/B. 본 회차 외 예열·실패도
   보존하고 raw counters·frame·visibleStable·retainedComplete·pixels를 함께 기록한다.
4. 통합 기준선 `56706247f` 대비 34/50/100% 경보 재평가, warm 왕복·관찰 on/off 재검증.
   원인이 다른 50% 경보를 frame 대기 하나로 해결했다고 가정하지 않는다.
5. 200/354/500% 및 역방향 입력에서 clamp 추가 raster와 요청 DPR/physical scale을 구분한다.
   현 후보와 비교해서 대기/단일 raster 비용이 악화되면 수용 보류한다. 실제 Firefox 확인도 필요하다.
6. 별도 Stage 4.2 결과 보고서와 제품·테스트를 commit한다. 모든 확대 경보가 해소돼도 남은
   CanvasKit/문서·편집 매트릭스/peak 메모리 미완료를 통과로 바꾸지 않는다.

계획 commit에서는 문서만 작성했고, 후속 승인으로 위 범위의 제품·테스트와 자동 측정을 수행했다.
원격 push·PR 생성·이슈 종료는 포함하지 않는다. 실제 Firefox·미실행 통합 게이트는 대기한다.
scheduler 바깥의 화질·staging·renderer 구조 수정이 필요하면 계획부터 다시 검토한다.

### 이전 단계 결정 기록

- 2026-09-10 [Stage 4.1 결과](../working/task_m100_6040_post_stack_stage4_1.md): 실제 경합
  재현·동일 viewport scroll 중복 제거·구 zoom exact surface 확정을 구현했다. 전체 1,607 pass /
  1 skip, build 통과. 18회 재측정에서 최종 visible DPR 2·오류 없음과 100% main raster 2회를
  확인했지만 visibleStable은 50/100% +25.6/+44.9% 경보가 남는다. retained-transition의 0ms
  task가 다음 frame 전에 실행되는 경계를 다음 승인 대상으로 좁힌다. 이번 단계에서 timer 정책을
  바꾸지는 않는다. 고배율 clamp의 추가 raster 비용과 나머지 통합 게이트는 미완료다.

- 2026-09-09 Stage 4 경보 후 사용자 승인: 확대 중복 raster를 수용 보정(Stage 4.1)으로
  재현한다. 실제 CanvasView·scheduler·DPR planner를 연결해 zoom-settled → 같은 viewport의
  지연 scroll → scroll-settled 순서와 page 1의 DPR/래스터 횟수를 확인한다. 줌 정착에서 이미
  계획한 x/y/zoom/viewport 크기와 같은 scroll만 CanvasView에서 중복 제거하는 최소 보정을
  검토한다. 위치·크기·배율이 다르거나 strict/문서/resize로 상태가 바뀌면 기존 경로를 유지한다.
  viewport-scroll 이벤트 자체는 억제하지 않아 눈금자·입력 소비자를 보존한다. 새 timer·Worker·
  DPR 하향·prefetch 정책 변경은 없다. 양축 실제 이동, 같은 위치 지연 알림, strict와 새 zoom,
  raw 읽기 화질을 테스트하고 34/50/100% 재측정으로 +20% 경보의 해소 여부를 판정한다.
  해결되지 않은 경보·미실행 CanvasKit/메모리 게이트는 그대로 남긴다.
  실문서 보정 중 physical scale만 같은 구 zoom surface의 metadata/preview가 남는 경계도
  확인했다. 완료 조건을 느슨하게 바꾸지 않고 visible/retained 큐가 구 zoom을 한 번 확정하도록
  한다. bitmap 무래스터 재사용의 CSS·이미지 callback 계약 일반화는 별도 작업으로 남긴다.

- 2026-09-09 Stage 4 [중간 결과](../working/task_m100_6040_post_stack_stage4.md):
  Canvas2D 동일 조건 A/B에서 34% max rAF는 감소했지만 50/100% visibleStable이 +20% 경보선을
  넘었다. 100%의 화면 밖 page 1 중복 raster를 추가 추적했다. 전체 수용을 보류하고 줌 유발
  scroll/정착 DPR 재판정 경합을 결정적 테스트로 재현할 최소 보정 범위를 먼저 검토한다.
  제품은 `ed268e373` 그대로이며 나머지 시각 매트릭스·peak 메모리를 통과로 세지 않는다.

- 2026-09-09 Stage 3.3 결과 후 Stage 4 승인: 제품 `56706247f`(4201)와 `ed268e373`(4200),
  같은 WASM·adapter·fonts·viewport·DPR로 비교한다. 이전 후보 4202는 변경하지 않는다.
  먼저 Canvas2D exam_kor 자동 배치에서 100→34→50→100% 3회, A/B 순서를 번갈아 측정하고
  첫 회를 별도 warm-up으로 표시하되 실패도 보존한다. 후속 warm 왕복 20이동과 관찰 on/off를
  양쪽에서 실행한다. 표본 수가 작으므로 통계적 개선이나 실제 핀치 개선율은 주장하지 않는다.
  공통 raster/전체 release·visibleStable·rAF·최종 backing pixels·cache·화질을 분리한다.
  known-work 미완료/오류/visible DPR 부족은 차단, 시간 중앙값이 20% 이상 악화하면 재조사
  경보로 삼되 2회 본 표본만으로 무회귀 확정하지 않는다. DOM/screenshot은 시간 기록 후 확인한다.
  후보 4문서×Canvas2D/CanvasKit의 줌·읽기·배치 기능을 smoke하고 미실행 조합은 명시한다.
  정착 snapshot은 peak RSS/GPU 증거가 아니며 peak 계측이 없으면 메모리 게이트를 완료로
  세지 않는다. 실제 Firefox 핀치 및 광범위 수동 조작은 자동 smoke와 분리해 보고한다.

- 2026-09-09 폭 resize 사용자 확인 뒤 Stage 3.3 진행 승인: 현재 B1의 in-place 경로를
  기준으로 빠른 역방향 zoom 큐, strict 응답 전 완료, 문서 reset, renderer revision 교체,
  decode·fallback 완료 순서를 결정적 테스트로 검증한다. 아직 없는 staging/Worker/타일은
  추가하지 않는다. `scheduleReRender` 직후 취소·교체 시 미실행 microtask가 불필요한
  layer 조회를 시작하는지 먼저 재현하고, 확인되면 기존 job/token 검사만 진입점에 적용한다.
  이미 실행한 동기 WASM이나 decode를 선점한다고 주장하지 않는다. 최신 작업의 retry와
  fallback은 유지한다. 집중·전체 Studio 검사 후 결과와 실브라우저 미검증 범위를 분리해
  보고하며, Stage 4 전체 A/B·CanvasKit·메모리 수용 완료로 확대하지 않는다.

- 2026-09-09 권장 순서 진행 승인: cold metadata 개선은 #6040의 완료 필수 조건으로 추가하지
  않는다. 우선 B의 기존 수용 차단인 zoom 중 폭 resize 우회를 보정한다. `onViewportResize`의
  높이 전용 지연을 **현재 animation/quiet가 유효한 동안의 폭·높이 변경**으로 확장한다.
  기존 중심 앵커·공유 VirtualScroll geometry는 즉시 갱신하고 stale 큐만 취소한다. zoom 세대를
  취소하거나 전역 surface 해제·동기 raster를 하지 않고 마지막 zoom 정착이 최신 visible을
  예약한다. scrollbar 폭을 15px 등으로 추측하지 않는다. 실제 창 resize도 zoom 중에는 같은
  경로이며, idle resize·strict 편집·문서/renderer 전환의 기존 계약은 유지한다.
  테스트에서 quiet/animation, 양축 이동·전체 배치·열 변경과 최종 예약을 확인하고 실문서
  scrollbar 전환을 재현한다. 정착 후 늦은 resize 등 남은 우회가 발견되면 수용 완료로 세지 않는다.
  이 결과와 사용자 확인 전에는 전체 B 완료·새 구조 구현으로 넘어가지 않는다.

- 2026-09-09 첫 cold/warm JSON 분석 뒤 사용자 승인: B 잔여 수용 조사 안에서
  `scrollProbeBudget=1` DEV 정밀 계측을 추가한다. 제품 메서드를 분리하거나 렌더 정책을 바꾸지 않고
  기존 메서드 wrapper로 layer count, overlay/tree WASM 조회, descriptor, 예산 원장/LRU 정리를
  관찰한다. page 번호와 중첩 inclusive 시간을 남기며 중첩 구간의 합집합을 제외한 잔여 시간은
  **순수 planner 시간으로 단정하지 않는다**. 캐시를 데우는 추가 조회·DOM readback은 하지 않는다.
  테스트와 브라우저 cold smoke 뒤 원인 범위·남은 Firefox 확인을 보고한다. 구조 보정·Worker·타일
  구현은 이번 계측 승인에 포함하지 않는다. 기존 before/after 제품 SHA와 WASM은 고정한다.

- 2026-09-09 사용자 승인: cold/warm A/B 서버와 동일 계측을 준비한다. 비교 제품은
  `56706247f` / `0345107c5`로 고정하고 Rust/Cargo 차이가 없는지 확인한 뒤 같은 진단 WASM,
  fonts, dependencies를 사용한다. DEV 패널만 양쪽에 같은 adapter로 적용한다. 구 기준선에 없는
  zoom pending API는 관찰 capability로 명시하고 없는 경계를 제품에 주입하지 않는다.
  새 문서를 열고 `document-view-loaded`에서 자동으로 연속 기록을 시작해 수동 버튼 지연을 줄인다.
  기록은 파일 fetch/파싱 전체가 아닌 초기 화면 구성 직후부터이며 이 경계를 JSON에 남긴다.
  rAF에서 cached visible/bitmap 준비 상태만 제한적으로 표본화하고 DOM bounds/readback은 하지
  않는다. warm은 현재 문서의 수동 시작, 종료 뒤 JSON 다운로드로 보존한다. 관찰 오버헤드와
  브라우저 실제 paint 시점의 부재를 명시한다. 제품 source 변경이나 성능 개선 판정은 하지 않는다.

- 2026-09-09 추가 사용자 승인: 읽기·편집 화질을 최우선으로 하여 기존 64M 정착 승격 gate를
  폐기한다. 현재 visible 집합은 정착/default(zoom·resize·편집 포함)에서 raw DPR을 lock하며,
  스크롤 중에는 기존 active surface의 DPR lock을 유지한다. 클릭 여부로 읽기 화질이 바뀌지
  않아야 한다. 화면 밖 planner/LRU/prefetch 예산, 편집 focus 보호, Canvas 크기 clamp는 유지한다.
  전 페이지 최고화질 캐시·타일링·Worker·저화질 preview는 추가하지 않는다.
  CanvasView의 별도 material-overlap scan과 미사용 64M resolver를 제거하고, 100/200/211/300%,
  여러 visible·backend·DPR·정착/default·focus 변경의 raw 보호를 테스트한다. 실제 exam_kor에서
  클릭 전후 DPR/scale·좌표와 할당 픽셀 증가를 확인한다. 비용 증가를 성능 개선으로 주장하지 않는다.
  이는 Stage 3.2B 사용자 수용 보정이며, 폭 resize·staging·성능 비교 잔여 게이트를 대체하지 않는다.

- 2026-09-09 사용자 승인: Stage 3.2B의 실제 핀치에서 발견한 입력→첫 animation callback
  경합을 보정한다. 실제 ViewportManager와 render scheduler의 callback 순서를 테스트로 재현한
  뒤, 작업 dispatch 때 pending/animating이면 이전 visible·retained·prefetch 작업을 거부한다.
  다음 정착에서 최신 작업이 완료되고 prefetch 예약이 반환되는지 확인한다. 기존 surface와
  유효한 이미지 완료는 유지하며 cancelAll·추가 Worker·DPR/예산/좌표 변경은 하지 않는다.
  이미지 작업의 교체·취소·재시도도 검증한다. 실제 폭 변경과 staging 비교는 잔여 게이트다.

- 기준: `upstream/devel@56706247f4950286117496c41f5b2c4b1cdbddc5`, 2026-09-07 KST
- 브랜치: `codex/issue-6040-post-stack-zoom`
- 현재 승인 범위: 2026-09-08 Stage 3.1 결과 `0b76ecb16` 보고 후 작업지시자가 **Stage 3.2A
  구현·검증**을 승인했다. 화질·예산·앵커·감도를 유지하며 반복 정착을 먼저 억제한다.
  Stage 3.2B 페이지별 교체와 3.3 집중 취소 검증은 A 결과 승인 뒤 별도로 진행한다.
- 현재 결과: [Stage 3.2A 후보 구현·검증](../working/task_m100_6040_post_stack_stage3_2a.md) 완료.
  실제 왕복 핀치에서 사용자가 체감 개선을 확인했고 입력 중 main raster는 0회였다.
  입력 조건이 달라 개선율·quiet 120ms 최적값은 확정하지 않았다. B 착수 승인은 별도다.
- 2026-09-08 후속 비교 논의와 “우선 계획대로 B를 진행해줘”로 B 구현을 승인받았다.
  아래의 이전 승인 대기 기록보다 다음 순서가 우선한다. 먼저 전역 해제 없는 페이지별 in-place
  갱신(B1)을 비교 후보로 검증한다. 기존 CSS preview를 개별 갱신까지 유지하고, 미생성 visible을
  먼저 처리하며 obsolete 큐를 취소한다. DPR·예산·좌표·스크롤 정책은 유지한다.
  별도 staging 교체는 B1에 비해 실제 blank/화질 회복 이득이 있는지와 중복 메모리·decode 수명
  비용을 비교해 결정한다. B1만으로 완성 전 구 bitmap 보존을 보장한다고 주장하지 않는다.
  새 저화질 preview는 이번 구현에서 제외하며 B 후 빈 화면이 여전히 문제일 때만 별도 실험한다.
- 2026-09-07 사용자가 #6821 등록·기준선 연결 후 #6040 계획대로 진행을 승인했다.
  이후 연속 핀치 계측 → 화면 유지·페이지별 교체 → 최신 요청 취소 순서를 요청했다.
  저해상도 선렌더는 보류한다. 아래 역사적 Canvas 전용 snapshot 설계는 폐기 유지.
- B1 결과: `14a306d6c`, [검증·잔여 보고](../working/task_m100_6040_post_stack_stage3_2b.md).
  줌 visible 큐·구 preview·actual 예약·pending decode 재연결을 구현했다. 줌 중 높이만 바뀌는
  scrollbar resize는 geometry만 즉시 갱신하고 최종 큐로 렌더한다. 실제 폭 변경은 기존 동기
  resize를 유지해 4쪽 문서 두 전환에서 전체 해제가 남았다. 이 예외 보정·실제 핀치·staging 비교
  전에는 B 전체 완료로 판정하지 않는다.

### Stage 1.3: 실제 경로와 관찰 경계

현재 `ViewportManager`는 smooth zoom의 animation event들과 최종 settled event를 발행한다.
`CanvasView.onZoomChanged()`는 각 event에서 앵커 계산과 `recalcLayout()`을 수행한다.

- `recalcLayout()` → `VirtualScroll.setPageDimensions()`가 전체 페이지 폭/높이·행을 갱신한다.
- animation 때는 기존 `updateRenderedPageZoomPreview()`로 CSS 크기를 바꾸며 최종 품질 raster를
  시작하지 않는 분기가 이미 있다. 따라서 CSS preview의 신규 도입이 과제가 아니다.
- animation preview는 `recalcLayout()` 안과 `onZoomChanged()`에서 호출될 수 있다. 실제 호출 횟수를
  기록한 뒤 중복 제거 가능성을 판단한다.
- settled 때는 `releaseAllRenderedPages()` → `cancelAll()` → `updateVisiblePages('zoom-settled')`로
  진행한다. 이 이유는 scroll/scroll-settled가 아니므로 visible은 응답 전에 동기 렌더하고 인접
  prefetch만 scheduler에 보낸다. #6042가 줌의 visible raster까지 분할한 것은 아니다.

측정 파일은 기존 `rhwp-studio/src/dev/page-scroll-probe.ts`만 최소 확장한다. 복원 가능한 기존
`observeBoundary`에 `recalcLayout`, `setPageDimensions`, `updateRenderedPageZoomPreview`,
`releaseAllRenderedPages` 네 경계를 추가한다. 기존 trace의 token/시간·bounded buffer 계약을 사용하고
새 Worker·timer·DOM scan·제품 분기를 hot path에 추가하지 않는다. `scrollProbe=1` 없는 DEV와
production은 계측을 설치하지 않는다. 관찰 off에서 원 메서드를 복원하는 기존 테스트를 재검증한다.

계측은 diagnostic code 변경이며 제품 정책 수정이 아니다. exact base와 adapter diff를 구분해 보고한다.
inclusive 시간은 중첩되므로 `geometry.zoom + geometry.layout + geometry.dimensions`를 CPU 총시간으로
합산하지 않는다. 기존 패널의 `visibleStable`은 관찰 가능한 렌더 완료이지 compositor 표시 시각이 아니다.

### 새 Stage 2: 중복 preview 최소 보정

| 파일 | 검토할 책임 | 유지할 경계 |
| --- | --- | --- |
| `canvas-view.ts` | `onZoomChanged()`의 두 번째 preview 호출 제거 | `recalcLayout()` 내부 첫 preview, 앵커 복원·알림·취소 순서 유지 |
| `canvas-view-zoom-preview.test.ts` | 실제 CanvasView + VirtualScroll의 preview 호출·DOM 쓰기·최종 좌표 계약 | 자동/고정/가로 배치, 줌 왕복·resize·settled·빈 문서 |
| 기존 smooth-zoom test | 중복 호출 위치에 묶인 source assertion을 보정 | animation 중 raster 금지와 실제 동작 테스트 병행 |

첫 preview는 새 VirtualScroll 좌표에서 기존 main/overlay 요소의 top/left/transform을 갱신한다.
그 뒤 앵커 복원은 viewport scroll만 바꾸고, `applyZoomPreviewBox()`는 scroll 값을 읽지 않는다.
따라서 같은 zoom event의 두 번째 순회는 현재 요소에 같은 값을 다시 쓴다. 첫 호출을 유지하면
`recalcLayout()`의 resize·문서 갱신 호출자도 기존처럼 정합한 요소 위치를 얻는다.

검증은 실제 메서드를 실행하여 animation event당 preview 1회, 활성 요소당 네 style 쓰기,
최신 VirtualScroll 기반 main/overlay 위치와 renderedZoom별 scale을 확인한다. 기존의 추가 preview를
시험에서 한 번 더 실행한 뒤 위치가 바뀌지 않는지도 비교한다. 직접 resize 재계산은 animation 중
1회 preview를 유지하고, settled에서는 preview 0회와 기존 release/cancel/visible 동기 호출 순서를
유지해야 한다. 기존 자동 열 commit·anchor·ruler·pool 소유권 회귀도 실행한다.

결정적 작업량은 `N`번 event 중 마지막이 settled이면 `2×(N−1) → N−1` preview 순회다.
이는 해당 순회·style 쓰기 감소이지 전체 줌 시간의 50% 개선이 아니다. 실문서 browser에서는 동일
WASM/viewport/DPR로 이 관계와 최종 화면을 확인하고, 장치 부하가 통제되지 않은 시간은 채택 근거로
사용하지 않는다. Stage 4의 전체 A/B·사용자 검증을 대체하지 않는다.

이번 Stage에서는 `VirtualScroll`, `ViewportManager`, `Ruler`, input overlay, DPR/LRU/scheduler,
정착 surface 수명을 바꾸지 않는다. 전체 geometry 재구축 감소는 비용 근거가 약해 보류하며,
눈금자 지연 보정은 [#6821](https://github.com/edwardkim/rhwp/issues/6821)에 남긴다.

범용 zoom-frame snapshot과 gesture 전체 topology 동결은 이 표의 확정 구현 사항이 아니다.
#6454는 기존 조사에서 도입 gate 미달로 종료됐다. 새 근거 없이 이를 선행 필수 작업으로 재개하지 않는다.

### 새 Stage 3.1: 연속 입력 기준선 (제품 정책 무변경)

현재 관찰기는 wheel마다 `ScrollObservation.begin()`을 호출하며 이전 trace를 supersede한다.
이 기록을 한 번의 물리 핀치로 합쳐 읽지 않는다. 수동으로 기록 구간을 지정하는 별도 DEV 원장을
사용하며, 기존 scroll/button trace의 완료 판정은 바꾸지 않는다.

| 파일 | 변경 | 경계 |
| --- | --- | --- |
| `src/dev/zoom-session-observation.ts` | DOM 없는 bounded 입력·span·frame 기록 | scope/시간 상한/중단/누락 기록, 임의 gesture 종료 추정 없음 |
| `src/dev/page-scroll-probe.ts` | 연속 핀치 시작/종료 UI와 기존 observer 연결 | passive capture, 제품 호출/반환/예외 보존, DEV query opt-in만 |
| `tests/zoom-session-observation.test.ts` | 입력 간 정착·후속 입력·중단·buffer·독립 snapshot | 실제 핀치 재현 또는 성능 개선으로 해석하지 않음 |
| `studio_scroll_probe_guide.md` | 실사용자의 핀치 수집·저장 절차와 한계 | 화면 크기/문서/배율/빌드 고정, paint trace는 별도 반복 |

- 구간은 최대 20초, 입력·span·frame 각각 고정 상한이다. UI가 시작한 구간만 보관하고 매 wheel의
  전체 DOM scan·JSON 직렬화·문서 조회를 하지 않는다. 구간 off/dispose/문서·content 전환에서는 중단한다.
- 입력은 delivery 시각과 event timestamp, deltaMode/delta/수정키/isTrusted를 기록한다.
  timestamp를 손가락을 뗀 시각으로 해석하지 않으며 ctrl-wheel과 물리 pinch는 사용자 설명으로 구분한다.
- 기존 observer의 geometry/zoom/release/raster/visibility/ruler 경계를 같은 시간축에 복사하고
  zoomAnimating을 기록한다. 중첩 inclusive 시간은 합산하지 않는다. 기록 종료는 렌더 완료가 아니며
  종료 시 알려진 작업의 readiness는 별도 필드다.
- 이 observer의 rAF는 실제 compositor paint가 아니다. 입력 중/정착 뒤 어느 main-thread 경계와
  frame gap이 겹치는지 먼저 조사하고, browser paint/composite 자체는 별도 profiler 반복으로 확인한다.
- 동일 문서에서 느린 100→34%, 빠른 100→34%, 방향 전환 100→34→100%를 각각 짧게 수집한다.
  실제 사용자 핀치 전에는 제품 병목의 확정 또는 개선율을 보고하지 않는다.
- 2026-09-08 세 시나리오 수집 완료. 실제 도달 배율·viewport 차이는 보고서에 명시했으며 통제
  before/after 비교가 아니다. 역방향 원장의 줌 131건과 후속 일반 scroll 38건을 분리했다.
  이전 main raster 종료 577.4ms 뒤 확대가 시작해 미완료 작업 취소 경합은 관찰하지 못했다.
  이 미측정을 추가 수동 녹화의 필수 조건으로 만들지 않고, A의 stale callback·재진입과 B/3.3의
  renderer lease·decode 경합 테스트에서 실행 순서를 강제한다.

### 새 Stage 3.2 상세 설계 — 반복 정착 억제를 먼저 분리

근거는 [Stage 3.1 실제 입력 보고](../working/task_m100_6040_post_stack_stage3_1.md)이며 결과
commit은 `30d777a0e`다. 두 물리 핀치에서 전체 해제 58회·main raster 267회, 4열의 동기 정착은
매번 8쪽이었다. 이 수치는 개선 전 진단이며, 단순히 Canvas 삭제만 없애도 작업량이 줄어든다는
근거가 아니다. **3.2A 입력 정착 경계 → 결과 승인 → 3.2B 페이지 교체** 순서로 분리한다.
아래 새 이름은 구현 후보 이름이지 현재 존재하는 API가 아니다.

#### Stage 3.2A — 입력이 이어지는 동안 최종 raster 재진입 억제

목표는 줌 감도·배율 수렴·열 배치·포인터 앵커를 바꾸지 않고 비싼 정착 횟수만 줄이는 것이다.
이 단계는 Canvas/layer/LRU 교체 방식·DPR 정책·scroll scheduler를 바꾸지 않는다. 따라서 마지막
정착의 일괄 동기 raster와 새로 보이는 쪽의 지연 등장은 남을 수 있다. 이를 완료로 과장하지 않는다.

| 파일 | 수정 후보 | 보존할 계약 |
| --- | --- | --- |
| `src/view/zoom-input-settle.ts` (신규) | DOM 없는 quiet/세대 상태와 단일 timer host | 입력당 O(1), 전체 문서 scan·Worker·추가 raster 없음 |
| `src/view/viewport-manager.ts` | 유효 ctrl/meta wheel의 입력 상태, 수렴 상태, raster-ready 알림 | 기존 zoom 계산식·16ms smoothing·epsilon·pointer anchor·fit mode 유지 |
| `src/view/canvas-view.ts` | raster 대기 중 기존 CSS preview 유지, ready에서 최종 렌더 1회 | shared VirtualScroll, 기존 눈금자/selection 이벤트, 단계 A의 동기 visible 계약 |
| `src/dev/page-scroll-probe.ts`, `zoom-session-observation.ts` | input-active / raster-pending / ready·cancel을 animating과 분리 계측 | pending을 완료로 오인하지 않음, DEV opt-in·bounded 기록 |
| `tests/zoom-input-settle.test.ts` (신규), 기존 smooth-zoom/preview/observer test | fake clock·rAF로 작은 연속 입력과 정착·취소 검증 | 임의 대기나 source 문자열만으로 성공 판정하지 않음 |

상태와 실행 순서는 다음과 같다.

1. 유효 ctrl/meta wheel이 들어오면 입력 세대와 `lastInputAt`만 갱신한다. 기존 줌 수식으로 목표를
   정하고 기존 `zoom-changed`로 geometry·배율 표시·눈금자·입력 overlay를 계속 갱신한다.
2. `isZoomAnimating()`은 실제 smoothing 상태로 유지한다. 수렴 뒤에도 true로 위장하거나
   `zoom-changed` 발행 자체를 늦추지 않는다. 별도 `isZoomRasterPending()` 후보는 입력 quiet
   대기 또는 최종 수렴 대기를 나타낸다.
3. quiet timer는 마지막 입력 이후 지정 시간 경과와 해당 세대 유효성을 확인한다. 아직 animation
   중이면 raster하지 않고 수렴 통지를 기다린다. 반대로 먼저 수렴하면 quiet 확인을 기다린다.
4. 두 조건이 맞으면 같은 세대에서 `zoom-raster-ready` 후보 이벤트를 정확히 한 번 발행한다.
   이 이벤트는 최종 렌더만 요청한다. geometry·앵커를 다시 계산하려고 가짜 `zoom-changed`를
   한 번 더 발행하지 않는다. 새 입력은 이전 timer/미실행 ready를 무효화한다.
5. ready의 최종 호출은 단계 A에서 기존 release/cancel/visible 동기 경로를 한 번 실행한다.
   단일 동기 raster를 중간에 취소할 수 있다고 가정하지 않는다.

초기 실험 quiet 후보는 **120ms**, 비교 후보는 **80/160ms**다. 확정 제품 상수가 아니라 승인 후
실문서 비교할 값이다. 120ms는 입력 종료를 정확히 검출하는 규격이 아니며 기다림 자체가 화질 회복
지연을 더할 수 있다. 한 번의 측정에서 발생한 170ms dispatch 지연을 숨기려고 그보다 긴 상수를
선택하지 않는다. 멈춤이 후보보다 길면 같은 물리 핀치에서도 정착을 허용할 수 있고, 입력 재개 시
남은 작업을 취소한다. 실제 조작에서 정착이 반드시 두 번만 발생한다고 약속하지 않는다.

회귀를 막기 위한 세부 경계:

- `smoothZoomTo()`의 epsilon 분기는 현재 public `setZoom()`을 부른다. 새 public `setZoom()`이
  wheel 상태를 취소하면 내부 수렴까지 취소될 수 있으므로, 내부 배율 적용과 명시 명령 취소 경계를
  구분한다. API 사용자의 기존 `setZoom()` 즉시 적용과 버튼/슬라이더 smooth zoom은 지연시키지 않는다.
- `recalcLayout()`은 현재 animating=false면 transform을 해제하는 `repositionActivePages()`를
  호출한다. quiet 대기에도 기존 renderedZoom 기준 preview를 사용해야 하며, 이 분기를 고치지
  않고 렌더만 미루면 큰 구 bitmap이 새 배치 좌표에 맞지 않는 점프가 생길 수 있다.
- wheel이 유발한 scroll 이벤트가 기존 `!isZoomAnimating()` guard를 우회해 새 raster를 시작하지
  않도록 같은 pending 조건을 확인한다. 실제 일반 스크롤·resize·편집 명령은 pending을 명시적으로
  정리하거나 최신 zoom으로 기존 strict 경로를 완료한다. 보통 스크롤의 150ms settle은 바꾸지 않는다.
- 새 입력에서 오래된 예약을 무효화하되 매 wheel마다 active 페이지/DOM을 추가 순회하지 않는다.
  기존 `cancelPendingPrefetch()`의 예산 재계산까지 중복 호출하는 연결을 피하고, 자료 정리는 기존
  geometry/렌더 경계와 결합한다. 취소 후에도 기존 bitmap 자체를 제거하지 않는다.
- public zoom/fit 명령, document/backend/profile 전환, detach/dispose에서 timer와 세대를 정리한다.
  viewport/배치 전환도 이전 ready가 중복 렌더하지 않게 처리한다. min/max zoom에서 실제 목표가
  변하지 않는 wheel과 delta=0은 불필요한 ready/raster를 만들지 않는다.
- timer callback이 실행되기 전에 이미 입력 task가 queue에 있을 수 있다. quiet는 heuristic이며
  OS의 제스처 종료를 알 수 없다. callback 시 최신 세대를 재확인해 중복·stale 결과를 막고, 이
  경합을 별도 테스트한다. 전체 topology를 gesture 동안 동결하는 방식은 사용하지 않는다.

단계 A의 수용 조건:

- 후보 quiet보다 짧은 간격의 작은 입력 N개를 fake clock으로 전달하면 geometry는 갱신되지만
  입력 사이 release/raster는 0회, quiet+수렴 뒤 최종 렌더는 1회다. 두 입력 구간 사이 충분한
  휴지가 있으면 2회다. timer/animation 수렴 순서를 뒤집어도 결과가 같아야 한다.
- 취소/재진입/버튼/fit/resize/편집/dispose 및 강제로 늦게 실행한 stale callback에서 이중 최종
  렌더가 없다. timer 수는 최대 1개이고 quiet 때 대기 rAF loop를 계속 돌리지 않는다.
- 대기 중 모든 active main/overlay가 최신 shared geometry를 사용한다. 자동·고정·가로 배치와
  분수 줌·DPR에서 좌우/상하 점프, 눈금자/캐럿/선택 지연과 최종 화질의 회귀가 없어야 한다.
- 같은 WASM·문서·viewport·DPR의 slow/fast/reverse에서 반복 정착과 raster를 줄였는지 측정한다.
  관찰 on/off와 기존 버튼 경로도 비교한다. 낮아진 호출 수만으로 사용자 체감 전체 개선을 확정하지
  않고 입력→preview, 마지막 delivery→첫/전체 visible 품질 회복을 각각 보고한다.
- focused test 후 Studio 전체 test/TypeScript/build와 기존 4종 실문서·사용자 핀치 검증을 수행한다.
  배포판과 비교할 때는 최적화 빌드끼리 고정한다. 진단용 --no-opt 값으로 개선율을 외삽하지 않는다.

#### Stage 3.2B — 기존 surface 유지와 페이지별 교체 (A 결과 후 별도 확정)

이 절은 코드 확인에 근거한 교체 설계와 남은 선택이다. 단계 A 승인에 포함해 자동 구현하지 않는다.

| 확인한 현재 경계 | 교체 설계에서 필요한 처리 |
| --- | --- |
| `renderScheduledPage()`는 기존 active Canvas를 제자리 재렌더한다 | queue에 넣는 것만으로 성공 전 구 화면 보존이 되지 않음 |
| `PageRenderer.applyOverlays()`는 source Canvas의 부모에서 같은 page layer를 검색·교체한다 | staging 전용 분리 부모에 새 main+layer를 묶고 실제 parent에 섞지 않음 |
| `reRenderJobs`·`prefetchRequestTokens`가 page 번호 기준이다 | 같은 쪽의 old/staging을 별도 유효 lease로 구분, 취소는 해당 lease만 소유 |
| `CanvasPool`은 page마다 active main 하나만 허용한다 | staging을 `acquire(pageIdx)`로 중복 등록하지 않고 명시적으로 commit/adopt |
| `disposeCachedPageSurface()`가 page 단위 cancel/remove를 호출한다 | old bundle 정리가 새 bundle의 layer·decode job을 지우지 않도록 소유한 참조만 폐기 |
| stale active의 예산 예약은 지금 target estimate를 사용한다 | 교체 중 실제 old 픽셀과 staging 픽셀을 모두 세어 저배율 전환의 과소 계상을 방지 |

후보 교체 흐름:

1. 최종 desired 집합과 target DPR을 기존 정책으로 계산하되
   `refreshRenderSurfacePlan(true)`의 즉시 reraster는 사용하지 않는다. 현재 exact surface는 유지하고
   LRU는 정확한 lookup key만 최종 결과로 사용한다. 다른 배율의 구 surface는 임시 preview일 뿐
   새 배율 cache hit나 최종 화질 완료가 아니다.
2. 기존 `PageRenderScheduler`의 visible 큐를 사용하고 줌의 synchronous fast path는 끈다.
   아직 bitmap이 없는 visible 쪽 → visible 중심/나머지 갱신 → retained 순으로 후보 우선순위를
   둔다. 편집 focus·strict 편집은 별도 기존 계약을 유지한다. 기존 스크롤의 정책/상수는 바꾸지 않는다.
3. 한 번에 최대 한 쪽의 staging bundle만 허용한다. 동기 PageRenderer 작업 후 준비된 최신
   main/layer를 같은 JS task 안에서 교체한다. 새 쪽이 없거나 실패/취소면 old bundle을 유지한다.
   canvas CSS 크기/위치는 physical 축에서 역산하지 않고 기존 VirtualScroll에 맞춘다.
4. image/rawSvg가 있으면 raster 반환을 이미지 완료로 오인하지 않는다. old에 그림이 있는데 new가
   decode 대기이면 구 그림을 지우는 교체를 피해야 한다. 기존 이미지 retry/fallback과 lease의
   완료/실패 전달 계약을 먼저 명시하고, 무한 대기·page 하나가 visible 큐 전체를 막는 상황을 검증한다.
   CanvasKit은 readiness/fallback 및 canvas replacement를 별도 검증한다.

메모리 원장은 다음 항목을 **고유 surface/예약별 한 번씩** 센다.

- active/preview의 실제 backing pixels
- staging의 실제 backing pixels 또는 아직 할당하지 않은 예약
- 미생성 mandatory target 예약(이미 staging인 쪽은 중복 예약하지 않음)
- detached LRU와 승인된 선택 prefetch 예약

LRU/비가시 선택 surface 회수 후에도 구 visible+새 한 쪽이 들어가지 않는 경우가 있다. 기존
mandatory가 예산을 초과하는 큰 한 쪽/4-layer에서 **무조건 구 화면 보존·동일 예산·언제나 즉시
완료를 동시에 보장할 수는 없다**. 예산을 키우거나 DPR을 내리는 변경은 허용하지 않는다.
이 경우 같은 쪽의 기존 in-place 경로로 제한적으로 돌아갈지, staging 대상만 더 제한할지는
**B 착수 전 사용자에게 한계와 함께 별도 제안**한다. 이 선택 전에는 전체 범위의 무공백 교체를
확정한 계획으로 간주하지 않는다. 아직 렌더한 적 없는 쪽의 즉시 표시는 별도 preview 없이는 보장하지 않는다.

##### 2026-09-08 B 착수 점검 — 예산 부족 예외 제안 (선택 승인 대기)

#6040 최신 본문·전체 댓글과 A 이후 코드를 다시 확인했다. `CanvasPool.adopt/replace`는 단일
active 소유권을 제공하지만, `PageRenderer.scheduleReRender`는 page 번호로 기존 job을 취소한다.
`isPageSurfaceComplete`도 page job 및 부모의 이미지 상태에 의존한다. 따라서 큐만 비동기로
바꾸는 구현은 안전한 swap이 아니며, bundle별 취소·완료·해제 책임을 먼저 분리해야 한다.

권고 정책은 **예산 내 staging + 예산 부족 시 해당 한 쪽만 in-place fallback**이다.

1. 이미 화면에 있는 main/layer의 실제 backing pixels와 아직 없는 mandatory target 예약을
   중복 없이 합산한다. staging은 한 쪽에 한정하고 새 main/layer 전체 예상 비용을 추가한다.
   stale active를 더 작은 target 크기로 세어 여유가 생긴 것처럼 판단하지 않는다.
2. 비가시 선택 surface·LRU를 회수한 뒤에도 기존 retained 예산 안에 staging이 들어가지 않으면
   그 페이지는 staging 대상에서 제외한다. DPR tier/retained 예산 자체를 변경하지 않는다.
3. 제외된 한 쪽만 기존 in-place 갱신으로 진행한다. 다른 visible의 preview는 유지하고
   전역 `releaseAllRenderedPages()`로 돌아가지 않는다. 이미 필수 surface 자체가 예산을 넘는
   경우에도 추가 old+new 복제를 하지 않으며, 기존 mandatory 초과를 해소했다고 주장하지 않는다.
4. 이 예외는 최신 화질 갱신의 진행을 보장하기 위한 절충이다. 해당 한 쪽의 일시적 blank 및
   단일 raster의 긴 실행은 남을 수 있다. 무조건 무공백을 위해 예산을 늘리거나 저화질로
   낮추는 정책은 선택하지 않는다. 예외 발생 수·사유·peak surface 비용을 계측한다.
5. 충분한 메모리에서의 정상 swap은 main/layer·필수 이미지 준비와 최신 lease 확인 후 실행한다.
   비동기 decode 실패/취소가 단일 staging slot을 영구 점유하거나 뒤 visible을 막지 않도록
   기존 retry/fallback의 완료·실패를 명시적 결과로 연결하는 테스트부터 작성한다.

이 정책을 승인받으면 예산 admission/예외 테스트 → bundle 수명과 실패·취소 테스트 → 기존
scheduler의 zoom 페이지별 실행 → 계측·실문서 검증 순서로 B를 구현한다. scheduler의 4ms는
페이지 경계 soft budget이고 단일 WASM 실행을 선점하지 못한다. 기존 scroll 상수와 A의
quiet120ms를 변경하지 않는다. 총 완료시간 감소만으로 판정하지 않고 입력 기회·blank·최종 화질
복구·메모리를 각각 보고한다. 이번 설계 점검에서는 소스·서버·브라우저 및 원격 상태를 바꾸지 않았다.

#### Stage 3.3 — 최신 요청만 commit하는 수명 계약

취소 안전성의 최소 부분은 A/B 구현 때부터 필수다. 잘못된 결과 표시를 3.3까지 허용한다는 뜻이 아니다.
3.3에서는 빠른 재입력·역방향·문서 변경·이미지 완료 경합을 집중 검증하고 스케줄링을 보정한다.

- `zoomGeneration + renderWorkGeneration + document/content identity + backend/profile + rasterKey
  + surface lease`를 작업 생성/실행/commit·decode callback에서 확인한다. 큐를 Map으로 교체하는
  기존 `setDesiredWork()`를 재사용하고 매 입력 전체 문서 작업을 새로 만들지 않는다.
- 새 입력은 대기 작업/미완성 staging의 commit 권한을 없애고, 화면의 기존 preview는 유지한다.
  직접 전달된 최신 입력 이전에 이미 시작한 동기 WASM/page raster는 선점할 수 없다.
- 취소 정리는 자기 lease의 resource/timer/예약만 해제한다. page 번호가 같다고 현재 active의
  layer나 다음 세대의 이미지 작업을 취소하지 않는다. 오류는 기록하되 즉시 재시도 loop로 숨기지 않는다.
- strict 편집·문서/renderer 전환은 오래된 문서 화면 보존의 예외다. 이전 content를 새 문서로
  표시하지 않고 기존 무효화 계약을 우선한다.

공통 회귀 gate는 KTX 실제 4-layer, kps-ai, exam_kor, 4쪽 문서의 auto/한 쪽/두 쪽/맞쪽/여러 쪽,
Canvas2D/CanvasKit, 저·고배율, scroll/resize/편집/인쇄다. surface 복제 비용·peak pixels·최종 화질·
warm 스크롤도 함께 보고한다. 시간과 시각 검증은 분리하며 저장하는 원시는 최종 비교에 필요한
최소 대표 구간만 남긴다.

### 검증·기록 경계

- Stage 1.3: 기존 자동 배치/zoom-anchor/smooth-zoom/ruler focused test, DEV 계측 계약 테스트,
  TypeScript·build, 실문서 browser smoke. 새 제품 개선이나 전체 회귀 통과를 미리 주장하지 않는다.
- Stage 2 이후: 변경 파일 focused, Studio 전체 test, TypeScript, build, 동일 조건 browser A/B와
  수동 트랙패드·스크롤·선택 검증. Rust source 변경이 없으면 Rust lint 전체 묶음은 생략한다.
- 계획 commit → 진단·보고 commit → 사용자 결과 승인 → 다음 Stage 순서를 유지한다.
- 원시 결과는 최종 기준선과 재집계에 필요한 최소 JSON만 보존한다. 매 실험 screenshot/log를 PR에
  누적하지 않는다. 개발 패널 안내는 기존 `studio_scroll_probe_guide.md`를 사용한다.

---

## 과거 구현 계획 원문 (이력, 현재 실행 지시 아님)

- **이슈**: [#6040](https://github.com/edwardkim/rhwp/issues/6040)
- **브랜치**: `codex/issue-6040-zoom-topology`
- **PR 기준 commit**: `upstream/devel` `b9d408f0d`
- **최초 구현 기준**: `upstream/devel` `2deb3dd61`
- **문서 성격**: 구현 전 파일·상태 전이 설계
- **계획 승인**: 2026-08-30 작업지시자 승인, Stage 1 진행
- **현재 상태**: Stage 2·3 구현 폐기, Stage 1·1.1 유지, Stage 1.2 live 자동 열 commit 보정. 아래 줌
  상태 전이 설계는 역사적 폐기안이며 새 공유 좌표 계획 승인 전에는 구현하지 않는다.

## 자동 열 계약

### 후보 계산

자동 열 후보는 이미 zoom이 적용된 표시 폭을 입력으로 받아 다음 의미를 갖는다.

```text
fitColumns = floor((viewportWidth + pageGap) / (maxDisplayedPageWidth + pageGap))
candidate = clamp(fitColumns, 1, pageCount)
```

- 페이지가 없으면 1열의 빈 레이아웃, 한 쪽이면 1열을 반환한다.
- invalid/0 viewport와 invalid page width는 안전하게 1열로 수렴한다.
- 50%라는 절대 배율은 입력과 분기에서 제거한다.
- zoom과 resize는 동일 순수 계산을 호출한다.
- 같은 경계의 미세한 왕복 입력에는 현재 commit 열 수와 CSS px 단위 히스테리시스 여유를 함께 전달해
  candidate가 즉시 되돌아가지 않게 한다.

### 점유 그리드 중앙 정렬

- `auto`의 commit 열 수는 page count보다 클 수 없으므로 3쪽 문서의 첫 행이 3쪽이면 점유 열도 3이다.
- 중앙 정렬 폭은 `occupiedColumns × maxPageWidth + gaps`를 사용한다.
- 페이지별 폭이 다른 경우 각 slot 안의 기존 중앙 정렬을 유지한다.
- `double`, `facing`, `multiple`은 지정 topology 자체가 의미이므로 의도적인 빈 slot을 유지한다.
- 마지막 미완성 행은 이 작업에서 별도 가운데 정렬하지 않고 기존 `pageIdx % columns` 규칙을 보존한다.

## 줌 상태 전이

> **폐기된 설계**: 이 절의 Canvas 전용 preview는 눈금자·캐럿·선택·hit-test와 다른 좌표계를 만들어
> 회귀를 일으켰다. 현재 branch는 최신 PR 기준 대비 `CanvasView`·`ViewportManager`·caret/input 경로를
> 변경하지 않으며, `Ruler`는 Stage 1.1의 끝 라벨 경계 처리만 포함한다. 후속 설계는 모든 소비자가 하나의
> authoritative preview geometry를 사용하거나, 기존처럼 매 프레임 같은 `VirtualScroll`을 소비해야 한다.

```text
idle
  └─ 첫 animation zoom event → preview(snapshot)
preview
  ├─ animation frame → committed topology 유지 + active element CSS preview
  └─ settled event → commit(final candidate, one layout pass, anchor restore)
commit
  └─ visible active Canvas를 새 품질로 점진 교체 → idle
```

snapshot에는 최소한 다음 값을 둔다.

- 시작 zoom과 committed topology key/columns
- 기준 페이지와 그 페이지 안의 정규화된 x/y 앵커
- 시작 viewport와 scrollLeft/scrollTop
- active page별 기존 rendered zoom/epoch

애니메이션 프레임은 전체 page array의 `setPageDimensions()`를 다시 호출하지 않는다. committed
VirtualScroll 좌표와 snapshot으로 현재 active Canvas·overlay에만 preview box를 적용하고 배율 표시는 계속
갱신한다. 정착 event는 final zoom의 candidate로 `recalcLayout()`을 정확히 한 번 호출하고 새 page box에서
정규화 앵커를 복원한다.

## 파일별 구현

### `rhwp-studio/src/view/virtual-scroll.ts`

- `GRID_ZOOM_THRESHOLD`와 그 분기를 제거한다.
- auto candidate/commit에 필요한 순수 helper 또는 작은 value object를 추가한다.
- `VirtualScroll` 인스턴스가 auto의 이전 committed columns를 보존하고 다음 `setPageDimensions()` 계산에
  전달한다. horizontal·고정 배치·문서 교체에서는 이를 reset하며 fixed arrangement 호출 계약은 바꾸지
  않는다.
- `getLayoutTopologyKey()`에 commit된 auto columns만 반영해 preview 후보가 topology 변경으로 보이지 않게
  한다.
- auto의 grid width와 margin은 실제 점유 열을 사용한다.

### `rhwp-studio/src/view/canvas-view.ts`

- **Stage 1.2 실제 변경**: `reset()`이 `VirtualScroll.resetAutoColumnCommit()`을 호출해 이전 문서의
  경계 상태를 제거한다. settled zoom의 기존 전체 반환·재할당 경로는 변경하지 않는다.

아래 항목은 폐기된 Stage 2 설계이며 현재 code candidate에 포함되지 않는다.

- `ZoomLayoutPreviewSession` 상태와 render epoch를 관리한다.
- `onZoomChanged()`를 animation preview와 settled commit 경로로 분리한다.
- preview 경로에서는 `recalcLayout()`, `renderPage()`, `releaseAllRenderedPages()`를 호출하지 않는다.
- settled 경로는 final candidate를 commit한 뒤 한 번만 레이아웃을 계산하고 snapshot의 기준 페이지/정규화
  앵커로 scroll을 복원한다.
- resize는 같은 candidate resolver를 사용하되 독립적인 단일 commit으로 처리한다.
- fixed arrangement에서는 topology key가 바뀌지 않으므로 좌표 재배치와 품질 교체만 수행한다.

### `rhwp-studio/src/view/canvas-pool.ts`

- 현재 page→Canvas 소유권을 유지한 채 교체 후보 Canvas를 준비하고 성공 시 swap하는 최소 API를 둔다.
- swap 전까지 기존 Canvas는 DOM과 pool의 active entry로 남는다.
- 취소·실패한 후보는 DOM에 노출하지 않고 available pool로 돌린다.
- page 결과를 viewport 밖에 보존하는 LRU/eviction은 추가하지 않는다.

### `rhwp-studio/src/view/page-renderer.ts`

- 필요한 경우 staging Canvas 렌더가 기존 페이지 layer를 먼저 제거하지 않도록 render/commit 경계를
  분리한다.
- 최신 render epoch의 성공 결과만 Canvas와 부속 layer를 commit한다.
- render scale 계산은 현재 `zoom × rawDpr`와 `clampRenderScale()`을 그대로 사용한다. tier·surface 예산은
  #6041에서 구현한다.

### 진단 경로

- 기존 DEV 진단 패턴을 따라 제스처당 preview frame, layout commit, candidate change, full release,
  page replacement 횟수와 anchor CSS px 오차를 관찰 가능하게 한다.
- production 동작이나 공개 문서 모델에는 진단 상태를 저장하지 않는다.

### `rhwp-studio/src/engine/input-handler.ts`

- **Stage 1.2 실제 변경**: 기존 zoom overlay 갱신을 공통 메서드로 모으고, viewport resize에서는
  CanvasView가 레이아웃을 확정한 다음 tick에 캐럿·필드·텍스트/셀 선택·그림/표 선택을 다시 투영한다.
- 별도 preview 좌표를 만들지 않고 기존 cursor/VirtualScroll authoritative geometry만 소비한다.

## 테스트

### `virtual-scroll-page-arrangement.test.ts`

- 51%에서도 폭이 두 쪽만 허용하면 2열, 세 쪽을 허용하면 3열
- 50% 전후가 동일 geometry에서 동일 candidate
- 3쪽 27%·17%에서 columns=3이고 묶음 중심 오차 ≤1 CSS px
- page count보다 큰 fitColumns의 cap
- invalid/0 viewport의 1열 fallback
- single/double/facing/multiple과 horizontal movement의 topology 불변
- 페이지 폭이 다른 auto slot의 내부 중앙 정렬

### 줌 preview/commit focused test

- animation N frame에서 전체 layout commit 0, settled event에서 1
- animation frame에서 final-quality `renderPage`와 `releaseAllRenderedPages` 0
- 경계 하나를 넘는 제스처의 topology commit 1 이하
- 기준 페이지/정규화 앵커 오차 ≤2 CSS px
- zoom in/out 왕복과 resize가 같은 최종 candidate를 선택
- stale render epoch가 새 Canvas를 덮어쓰지 않고 실패 시 기존 Canvas 유지

### 기존 회귀

- #685, #689, #2560의 grid hit/current-page/row navigation
- #3244, #3245, #3246, #3591의 zoom sensitivity/anchor/horizontal pan
- page arrangement transaction이 최종 `recalcLayout()` 한 번만 수행하는 계약
- Canvas2D와 CanvasKit의 기존 render 성공·fallback 경로

## Stage별 예상 변경 경계

1. Stage 1: `virtual-scroll.ts`와 자동 배치 focused test, working 보고서
2. Stage 1.1: 수평 눈금자 끝 라벨 경계 처리와 실제 브라우저 검증
3. Stage 1.2: live auto commit·문서 reset·resize overlay 재투영과 actual CanvasView/CanvasPool test
4. Stage 2: **폐기**, Canvas 전용 줌 상태 전이와 앵커 설계를 현재 branch에서 제거
5. Stage 3: **폐기**, 점진 Canvas 교체와 계측을 현재 branch에서 제거
6. Stage 4: Stage 1~1.2 통합 test·build·browser evidence와 최종 보고서

각 Stage 결과 승인 뒤 해당 source·test·보고 문서를 하나의 검토 가능한 commit으로 고정한다. 실제 조사에서
파일 책임이 달라지면 source를 수정하기 전에 이 구현 계획과 승인 기록부터 갱신한다.

## native stacked PR 연결 계획

1. #6040의 네 Stage와 최종 승인을 마치고 bottom head를 고정한다.
2. 그 head에서 `codex/issue-6041-adaptive-render-scale`을 만들고 #6041 수행·구현 계획 승인을 받는다.
3. #6041 완료 head에서 `codex/issue-6042-page-virtualization`을 만들고 #6042 수행·구현 계획 승인을 받는다.
4. `gh stack view`에서 `devel ← #6040 ← #6041 ← #6042` 순서와 각 layer diff를 확인한다.
5. 세 head의 범위별·누적 검증, 한국어 PR 제목·본문·stack map을 사용자에게 보고하고 게시 승인을 받는다.
6. `gh stack submit --remote upstream`은 push와 PR 생성을 함께 수행하므로 승인 뒤 한 번 실행한다.
7. GitHub native stack public preview의 cascading rebase 뒤에는 영향받은 상단 head를 다시 동기화·검증한다.

## 커밋 경계 후보

1. `docs(studio): #6040 자동 줌 토폴로지 계획을 기록한다`
2. `fix(studio): 자동 페이지 열과 점유 그리드 정렬을 바로잡는다`
3. `perf(studio): 줌 preview와 토폴로지 commit을 분리한다`
4. `perf(studio): 줌 정착 Canvas를 점진 교체한다`
5. `docs(test): #6040 통합 검증 결과를 기록한다`

#6041의 render scale tier와 #6042의 LRU·scheduler 변경은 이 branch에 포함하지 않는다.
