# #6040 Stage 4.2 — 줌 visible 뒤 frame 기회 양보

- 일자: 2026-09-10 KST
- 상태: **최소 보정·자동 검사·버튼 재측정 완료. 사용자 읽기 화질 통과, 연속 핀치 멈춤 잔존으로 통합 수용 보류**
- 계획: `850005bce`, 직전 제품 `76e8496a4`, 통합 기준선 `56706247f`.
- 후보: 이 보고서와 같은 commit의 제품 소스. exact source SHA-256은 아래 JSON에 고정한다.
- [수행 계획](../plans/task_m100_6040.md), [구현 계획](../plans/task_m100_6040_impl.md),
  [직전 결과](task_m100_6040_post_stack_stage4_1.md)
- 원시: [줌 비교 — 로컬 보존 안내](assets/issue6040-post-stack/README.md),
  [스크롤·관찰 비용 — 로컬 보존 안내](assets/issue6040-post-stack/README.md),
  [고배율 화질 상태 — 로컬 보존 안내](assets/issue6040-post-stack/README.md)

## 결과와 사용자 관점

보이는 쪽을 고화질로 그린 직후 화면 밖 쪽의 무거운 렌더가 연달아 실행되던 경계를 바꿨다.
줌 정착의 마지막 visible slice 뒤 다음 rAF에서 기존 deferred 작업을 **예약만** 한다.
화면 밖 raster 자체는 별도 task에서 실행한다. 먼저 완성한 읽기 화면을 표시할 기회를 주려는
변경이지, 품질을 낮추거나 총 렌더 연산을 줄이는 변경은 아니다.

패널을 접은 본 측정에서 50/100% visibleStable 중앙값은 직전 후보 대비 약 20.0/28.0%
줄었다. 34%에는 실질적인 차이가 없었다. 100% retainedComplete는 5.7% 늘어나는 교환이
있었다. 기존 통합 기준선 대비 visibleStable은 +6.2/+9.8/−4.4%로 이번 소표본에서는
이전 +20% 경보선 아래지만, **통계적 무회귀 또는 실제 Firefox 핀치 개선 확정은 아니다.**

예상 사용자 시나리오는 축소 화면에서 다시 확대했을 때, 읽는 쪽이 끝나도 화면 밖 렌더 때문에
그 다음 화면 갱신 기회까지 같이 늦어지는 현상을 줄이는 것이다. 이미 실행 중인 한 쪽의
동기 WASM/Canvas 렌더를 중간에 멈추지는 못한다. 확대 완료 전에 다시 핀치하는 Firefox
사례는 여전히 사용자 검증 대상이다.

## 구현·결정적 검사

- scheduler에 기본 false인 `yieldAfterVisible` 옵션을 추가했다. CanvasView는
  `zoom-settled`에만 true를 전달한다. scroll/scroll-settled/strict는 기존 정책이다.
- frame/idle/timer를 같은 deferred 취소 소유권으로 관리한다. 새 요청 버전과 소유권을
  재검사하여 취소된 callback과 같은 generation의 reentrant 요청도 구 대기를 재생하지 못한다.
- visible가 실제 실행되고 화면 밖 큐가 있을 때만 양보한다. 빈 큐·무효 visible·마지막 작업
  실패에서 무의미한 양보나 무한 retry를 만들지 않는다.
- `idleScheduled`는 기존 idle/timer 외 양보 frame도 pending으로 센다. 완료 판정이나
  계측 threshold를 완화하지 않았고 관찰 패널의 제품 소스도 변경하지 않았다.
- DPR 예산, surface cache key, LRU admission, preview, renderer, ruler, anchor는 변경하지 않았다.

기존 소스에서는 새 회귀 테스트의 `다음 frame보다 먼저 실행 가능한 0ms timer를 만들지 않는다`
단언이 timers=1로 실패했다. 보정 뒤 scheduler 14개와 CanvasView 4개 신규 사례를 포함한
집중 78개가 통과했다. FakeHost는 callback을 하나씩 명시적으로 실행하며, frame 대기와
timer/idle 순서를 검사한다. 실제 브라우저의 full frame batch/paint를 모사했다고 주장하지 않는다.
대기 중 취소·새 scroll·새 zoom, idle 유무, 다수 visible/retained, reentrant 교체, 실패를 다룬다.

## 동일 조건 A/B

Canvas2D / exam_kor 20쪽 / Chromium 152 / 1280×720 / DPR 2 / 자동 배치(34/50/100%에서
3/2/1열). 세 버전은 같은 진단용 no-opt WASM·폰트를 사용한다. 확장 배포판의 절대 성능과
비교하지 않는다. 조작은 DEV 버튼의 smooth zoom이며 실제 트랙패드 입력이 아니다.

먼저 세 버전 각 3회(첫 회 예열) 27 trace를 수집했다. 이후 JSON details가 펼쳐져 문서 일부를
덮고 있었음을 확인했다. 실행 순서 근거는 보존하되 이 표본을 최종 사용자 화면 조건으로 쓰지
않는다. 이 탐색 측정의 통합 기준선은 다른 두 버전 이후에 실행됐다는 순서 제약도 있다.

최종 표는 충분히 데워진 세 버전에서 **조작 전에 JSON을 접고**, 100→34→50→100%를 2회
추가한 18 trace다. 회차 1은 후보→직전→통합, 회차 2는 통합→직전→후보 순서로 각 배율을
실행했다. 직렬 측정하며 동시에 빌드·테스트하지 않았다. 결과 읽기는 조작 완료 뒤 수행했다.
`*-closed`가 최종 표본이며 탐색 27개와 합치지 않는다. JSON은 본/예열과 조건을 보존한다.

각 셀은 두 표본의 중앙값(ms), max rAF는 trace 최대 rAF 간격의 중앙값이다.

| 배율 | 지표 | 통합 56706247f | 직전 Stage 4.1 | 후보 Stage 4.2 |
| --- | --- | ---: | ---: | ---: |
| 34% | visibleStable | 427.25 | 455.10 | 453.95 |
| 34% | retainedComplete | 581.65 | 455.10 | 453.95 |
| 34% | max rAF | 328.95 | 62.70 | 64.10 |
| 50% | visibleStable | 195.85 | 268.90 | 215.10 |
| 50% | retainedComplete | 351.30 | 331.75 | 323.60 |
| 50% | max rAF | 123.70 | 116.50 | 67.65 |
| 100% | visibleStable | 165.25 | 219.30 | 157.95 |
| 100% | retainedComplete | 318.80 | 219.30 | 231.90 |
| 100% | max rAF | 71.25 | 118.50 | 73.95 |

본 회차 1의 100%에서 직전 후보는 visible page 0 raster 97.2–150.1ms,
offscreen page 1 raster 154.1–211.2ms 뒤 215.2ms에 visibleStable을 관찰했다.
보정 후에는 page 0 raster 96.5–154.7ms 뒤 163.5ms에 visibleStable,
page 1 raster는 164.1–245.7ms였다. **관찰 순서가 바뀐 증거이지 compositor paint timestamp가 아니다.**

최종 18 trace는 모두 complete, 오류 0, pendingImages 0, visible DPR 2였다.
main raster/회는 통합 9/4/2, 직전과 후보 모두 6/4/2다. 이번 보정으로 늘거나 줄지 않았다.

| 배율 | 최종 totalAllocatedPixels 통합 | 직전 = 후보 |
| --- | ---: | ---: |
| 34% | 23,103,360 | 15,677,280 |
| 50% | 23,183,212 | 28,133,932 |
| 100% | 33,874,172 | 39,224,144 |

통합 기준선보다 50/100% 최종 pixels가 큰 것은 기존 후보의 보존 정책 차이이며 이번 양보가
새로 만든 차이는 아니다. 정착 후 snapshot은 중간의 중복 surface나 peak RSS/GPU 메모리를
증명하지 않는다. 양보 때문에 이전 surface를 더 오래 잡을 가능성도 미검증으로 남긴다.

## 일반 스크롤·관찰 비용

34% 자동 배치의 왕복 20이동에서 처음 2이동을 제외한 18개를 사용했다.
known-work 후 다음 frame 관찰 중앙값은 통합 283.10ms, 직전 282.50ms, 후보 280.00ms.
이 구간은 세 버전 모두 main raster 90회다. 문서를 데운 뒤 왕복했더라도 cache가 전부 hit인
fully-warm 무래스터 스크롤은 아니며, 이 결과로 그런 상태의 UX를 증명하지 않는다.
trace max rAF 중앙값은 각각 61.65/61.00/60.20ms였다. 이번 zoom opt-in 밖의 큰 변화는 관찰하지 않았다.

관찰 on/off runner는 12회×2조건에서 처음 두 회를 제외한 각 10표본을 비교했다.

| 버전 | 관찰 off | 관찰 on |
| --- | ---: | ---: |
| 통합 | 268.80ms | 265.90ms |
| 직전 | 271.45ms | 270.70ms |
| 후보 | 271.80ms | 271.45ms |

관찰 on이 약간 빠른 수치는 잡음 수준이며 개선율로 해석하지 않는다. 이 검사는 스크롤 관찰
비용이지 줌 계측 off 성능이나 실제 핀치 비교가 아니다. 이 스크롤 batch는 JSON overlay를
통제하기 전 자료라 별도 조건으로 남긴다. 각 batch 종료 오류·pendingImages는 0이었다.

## 고배율·품질·남은 게이트

직전/후보 모두 200% visible DPR 2를 확인했다. 직접 배율 대화상자로 354→500%를 바꾼
검사는 두 후보 모두 visible page 1의 physical scale 6.136964072485382,
201,337,914 surface pixels에 도달했다. effective DPR은 354%에서 1.733605670193611,
500%에서 1.2273928144970765로 기존 안전 clamp가 같았다. 오류·이미지 대기·큐는 종료됐다.
이는 페이지의 3-layer surface 합이며 약 805MB 단순 RGBA 환산으로도 큰 비용이다.

이 검사는 버튼/대화상자 smoke다. 대화상자가 새 trace를 시작하지 않아 남은 구 trace는
품질 JSON에서 제외했다. **354→500% 추가 raster 비용·빠른 역방향 핀치 성능을 통과로 세지 않는다.**
최종 100% 후보 화면을 패널을 접은 상태로 육안 확인했지만 픽셀 단위 before/after diff는 하지 않았다.

- 실행 완료: 집중 78 pass; 전체 Studio 1,626개 중 **1,625 pass / 1 skip / 0 fail**;
  TypeScript 포함 build; `git diff --check`. build의 기존 CanvasKit fs/path 외부화·bundle 경고는 유지됐다.
- Rust/WASM 소스는 변경하지 않았다. 테스트와 build 뒤 제품 소스 수정은 없다.
- 대기: 사용자 Firefox 빠른 재확대·역방향 핀치와 클릭 없는 읽기, 실제 숨김/복귀 동작,
  CanvasKit/4문서/편집 매트릭스, peak 메모리, 고배율 추가 raster 비용 검증.
- 따라서 Stage 4 전체 수용, 제출, 원격 push, PR 생성, 이슈 종료는 진행하지 않는다.

## 다음 사용자 확인

4200 후보를 Firefox에서 새로고침하고 exam_kor를 연다. JSON 패널은 접는다.
100%에서 300–500%로 확대하며 선명해지기 전에 다시 확대하고, 곧바로 약 34%로 축소한다.
커서는 첫 쪽에 둔 채 다른 쪽으로 스크롤한 뒤 멈춰도 클릭 없이 선명해지는지 확인한다.
불편한 구간이 남으면 배율·순서·정착 전후를 기록하고 기존 연속 핀치 계측을 함께 받는다.
사용자 확인 후 남은 통합 매트릭스를 이어갈지 결정한다. 아직 확인하지 않은 조건을
이번 버튼 표본의 통과로 대체하지 않는다.

## 사용자 확인 후 판정 — 2026-09-10

후보 `a54d97575` 안내 뒤 사용자가 다음을 보고했다. 새 trace나 native profile은 첨부되지
않았으므로 기존 측정과 같은 실행의 인과관계로 연결하지 않는다.

- **읽기 화질 통과**: 첫 쪽에 커서를 둔 채 다른 쪽으로 스크롤하고 정지하면 클릭 없이 선명해진다.
- **고배율 응답성 미통과**: 300–500% 부근에서 선명해지기 전에 계속 확대하면 멈춘 뒤 목표
  배율로 순간이동하듯 보이는 현상이 이전처럼 남는다.
- **첫 진입 축소 응답성 미통과**: 배치가 늘어 처음 그리는 쪽이 생길 때도 멈춤이 있다.
  한 번 로딩한 구간을 다시 확대·축소하면 괜찮아 보인다는 정성 관찰이다.

현재 코드와 과거 근거를 다시 대조했다.

1. `PageRenderScheduler.runVisibleSlice`의 시간 예산 검사는 동기 `work.run()`이 반환된
   **뒤**에 있다. Stage 4.2는 마지막 visible 이후 offscreen 경계만 바꾸므로 한 쪽의 긴
   raster나 브라우저의 native 게시 비용을 선점하지 못한다.
2. [cold 상세 조사](task_m100_6040_post_stack_budget_detail.md)에서는 기준선과 후보 모두
   scheduler 예약 **앞**의 layer metadata 조회에 약 562/564ms가 들었다. 현재도
   `refreshRenderSurfacePlan`에서 retained 쪽의 `getCanvasSurfaceLayerCount`를 조회하며,
   JS cache miss 때 overlay/tree 경로를 준비한다. 재방문이 나아지는 관찰과 부합하지만,
   이번 멈춤의 전량이 metadata라는 뜻은 아니다. bitmap/cache·decode·업로드 준비도 구분해야 한다.
3. [과거 Firefox native 조사](task_m100_6040_post_stack_stage3_2b_firefox_upload.md)는
   큰 비트맵 복사·Texture uploads 지연을 보여줬다. 현재 head의 새 계측은 아니며 Firefox
   자체 결함이나 이번 수정의 회귀로 확정하지 않는다.
4. Stage 4.1은 완료되지 않는 구 zoom preview를 해결하기 위해 physical key가 같아도
   renderedZoom이 다르면 한 번 렌더하도록 했다. 이 조건은 현재 `renderScheduledPage`에
   남아 있다. Stage 4.2의 354/500% 품질 smoke는 동일한 physical scale/pixels를 확인했지만
   재래스터·업로드 비용은 검증하지 않았다. 따라서 이 경계의 안전한 재사용 가능성을 먼저
   조사할 가치가 있다. 과거 Stage 3의 exact-key raster 0건을 현재 동작으로 인용하지 않는다.
5. 줌 보간은 이미 elapsed를 최대 50ms로 제한한다. 긴 멈춤 전체를 elapsed에 넣어 즉시
   목표값으로 보내는 구현은 아니다. 화면상 점프의 정확한 경계는 입력 처리·rAF·게시 시점을
   분리해야 하며 smoothing 상수 변경으로 원인이 해결됐다고 주장하지 않는다.

**다음 제안(아직 구현 승인 아님)**: #6040 안에서는 우선 고배율의 동일 physical surface를
재래스터 없이 최신 zoom으로 안전하게 확정할 수 있는지 재현·설계한다. 단순 preview 검사 삭제는
이전 미완료/이미지 callback 회귀를 되살릴 수 있어 금지한다. content revision, layer, backend,
render profile, 실제 bitmap 준비, CSS geometry, 비동기 이미지 callback의 유효성을 함께 검증한다.
이 조건 밖의 확대나 최초 쪽 렌더까지 해결하는 안으로 확대하지 않는다.

cold metadata 준비 분리와 가시 영역/타일 등 렌더 단위 변경은 기존 결정대로 별도 구조 개선
설계 후보로 남긴다. 현재 후보가 기준선보다 악화한 부분은 #6040에서 먼저 보정해야 하며,
기존 한계를 분리한다는 이유로 현재 핀치 수용을 완료 처리하지 않는다. 사용자에게 같은 수집을
즉시 반복 요구하기보다 기존 자료와 결정적 재현으로 범위를 좁힌 후 필요한 A/B만 요청한다.

이번 피드백 반영은 문서만 수정했다. 제품 코드·서버·원격 게시를 바꾸지 않았다.
