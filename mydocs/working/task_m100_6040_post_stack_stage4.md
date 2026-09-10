# #6040 Stage 4 — 통합 수용 중간 결과와 확대 경보

- 일자: 2026-09-09 KST
- 상태: **A/B 자동 측정 완료, 확대 경보로 통합 수용 보류**
- 제품 기준선 `56706247f`, 후보 `ed268e373`, 측정 계획 `73b72fd2f`
- [수행 계획](../plans/task_m100_6040.md), [구현 계획](../plans/task_m100_6040_impl.md)
- [최소 계측 증거 — 로컬 보존 안내](assets/issue6040-post-stack/README.md)

## 판정

34% 다중 쪽 축소에서 긴 단일 프레임 간격은 줄었다. 그러나 50%, 100% 확대의
visibleStable 중앙값은 사전 경보선인 +20%를 넘었다. 추가 50→100% 추적에서 화면 밖
두 번째 쪽의 main raster가 두 번 발생했다. 따라서 전체 성능 개선·무회귀·제출 준비
완료로 판정하지 않는다. 이 경보를 해소하기 전에 전체 문서/backend 시각 매트릭스를
완료 처리하거나 별도 cold metadata 구조 작업으로 넘어가지 않는다.

이번 단계는 계측·조사·문서만 수행했다. 제품을 보정하거나 화질·예산을 낮추지 않았다.

## 비교 조건과 한계

- 인앱 Chromium 152, Canvas2D, exam_kor 20쪽, viewport 1280×720, DPR 2.
  관찰된 문서 viewport는 양쪽 모두 1260×558이었다.
- 4201 기준선 / 4200 후보. 별도 이전 후보 4202는 변경하지 않았다.
- 두 제품 사이 Rust/Cargo/fonts/package-lock 차이가 없고 동일 진단용 **no-opt WASM**과
  동일 DEV 계측 adapter를 사용했다. WASM SHA-256과 제품 SHA는 JSON에 남겼다.
  최적화된 확장 배포판의 절대 시간이나 실제 Firefox 핀치 개선율은 아니다.
- 자동 배치 100→34→50→100%를 3회. 회차 0은 warm-up으로 별도 보존,
  본 회차 1은 후보→기준선, 회차 2는 기준선→후보 순서였다. 각 회차 첫 위치로 복귀했다.
  각 배율당 본 표본은 2개뿐이므로 통계적 유의성이나 일반적인 개선율을 주장하지 않는다.
- 배율별 열 수 3/2/1, visible 집합과 최종 backing 조건은 일치했다. 일부 최종 y 좌표에는
  약 0.5 CSS px 차이가 있어 픽셀 단위 동일 화면 비교라고 주장하지 않는다.
- 초기 zoom round의 cold 준비 비용과 후속 warm 결과는 분리했다. DOM snapshot은 시간
  기록이 끝난 뒤 읽었으며 screenshot으로 브라우저 paint 완료를 계측하지 않았다.

## 버튼 줌 결과

아래는 본 회차 두 값의 중앙값(ms)이다. max rAF 열도 **각 회차 최대값의 중앙값**이며
전체 표본의 p95가 아니다. visibleStable은 계측기가 아는 최신 visible 렌더 완료 경계이지
브라우저 합성·GPU 업로드 완료나 모든 decoder 완료의 증명은 아니다.

| 목표 배율 | visibleStable 이전 → 후보 | 변화 | max rAF 이전 → 후보 | main raster 호출/회 이전 → 후보 |
| --- | ---: | ---: | ---: | ---: |
| 34% | 442.25 → 452.50 | +2.3% | 344.80 → 67.15 | 9 → 6 |
| 50% | 192.10 → 237.40 | **+23.6% 경보** | 120.20 → 114.10 | 4 → 4 |
| 100% | 151.25 → 211.95 | **+40.1% 경보** | 62.75 → 119.30 | 2 → 3 |

34%의 visibleFirst는 442.25→155.55ms였다. 페이지별 진행으로 먼저 완료되는 쪽이 생겼지만
전체 visible 최종 완료가 빨라진 것은 아니다. 기존 bitmap 유지와 처음부터 미생성인 페이지의
공백도 구분해야 한다. 18개 zoom trace는 모두 complete, 최종 visible DPR 2, pending image 0,
관찰 오류 0이었다. 각 줌의 전체 surface 해제는 기준선 1회, 후보 0회였다.

정착 후 totalAllocatedPixels는 34% 23,103,360→15,677,280, 50%
23,183,212→28,133,932, 100% 33,874,172→33,874,172였다. visible 밖 보존/예약 집합이
다르므로 동일 작업량의 순수 renderer 효율 비교가 아니다. **최종 snapshot이지 peak RSS,
GPU 메모리나 임시 surface 최고 사용량이 아니다.** 메모리 수용 게이트는 미완료다.

## 반복 스크롤과 관찰 비용

34% 자동 배치에서 개발 패널의 `왕복 20회`를 실행했다. 실제로는 20이동(10왕복)이며
각 이동 뒤 known-work 정착을 기다린다. 첫 두 이동은 보존하되 warm 요약에서 제외했다.
동일 y=1091.5↔2728.5 구간의 18이동 결과는 다음과 같다.

| 지표 | 기준선 | 후보 |
| --- | ---: | ---: |
| 이동→known-work 다음 frame 중앙값 | 285.90ms | 282.85ms |
| rAF 간격 p95 (nearest-rank) | 62.00ms | 61.90ms |
| rAF 최대 간격 | 63.10ms | 65.20ms |
| main raster 호출 합계 | 90 | 90 |

두 버전 모두 18 trace complete다. 이 표본에서는 +20% 경보가 없으나, 매 이동 5회의
raster가 남았으므로 **완전 캐시된 무렌더 fully-warm 스크롤**이 아니다. LRU hit/miss/
eviction snapshot은 누적값이며 회차별 증가분으로 제시하지 않는다. 기존 배포판과의
fully-warm UX 비교를 대체하지 않는다.

관찰 on/off는 12회 순서를 번갈아 실행하고 첫 2회를 제외했다. 10개씩의 중앙값은
기준선 off/on 274.75/270.70ms, 후보 off/on 277.60/277.15ms였다. 음의 차이를 계측의
성능 개선이라고 해석하지 않는다. 이번 대조에서 뚜렷한 양의 오버헤드는 관찰하지 못했으나
timer/frame 정렬 잡음이 있고 panel runner 자체는 남아 있으므로 production 동등성은 아니다.
이 대조는 스크롤용이며 확대 경보에 대한 observer-off 직접 대조는 아직 없다.

## 확대 경보 추가 추적

본 측정과 별개로 후보의 50→100%를 다시 추적했다. 동일 visible [0]에서 다음 순서를 보았다
(JSON diagnostic, 시간은 조작 시작 상대값).

1. 97.5ms: zoom raster-ready의 visible 예약. 해당 호출 자체는 약 0.3ms.
2. 98.0ms: 다른 visibility 갱신이 이어지고 scheduler.desired 안에서 page 0 main raster
   45.6ms가 동기 실행됨.
3. 147.4ms: 화면 밖 page 1 main raster 63.2ms.
4. 213.9ms: visibleStable 관찰. page 1 작업이 첫 visible 완료 관찰도 늦출 수 있음.
5. 365.1ms: page 1 main raster 59.1ms가 다시 실행됨.

소스상 `updateVisiblePages('zoom-settled')` 직후 scroll 갱신은 새 generation/계획을 만든다.
`scrolling`은 active surface의 이전 DPR을 잠그고, `scroll-settled`는 visible raw 보호와
화면 밖 예산 판정으로 되돌린다. 표준 스크롤의 이 정책 자체는 의도된 계약이다.
하지만 줌이 생성한 scroll이 끼어들면, 보존된 page 1을 이전 DPR로 새 줌에 다시 그리고
정착 후 화면 밖 DPR로 또 그리는 경합이 생길 수 있다.

**page 1 중복 raster와 실행 순서는 관찰 사실**, 정확한 visibility reason/DPR 전환이
원인이라는 설명은 소스와 시점에 기반한 가설이다. 기존 계측은 각 raster의 요청 DPR과
visibility reason을 모두 보존하지 않아 확정하지 않는다. 50% 경보까지 이 원인 하나로
설명하지 않는다. 다음 보정에서는 이 순서를 결정적 테스트로 재현하고, 줌 유발 scroll과
실제 사용자 scroll을 구분하는 최소 변경 가능성을 먼저 검증한다. 실제 스크롤 즉시성,
최종 visible raw 화질, 클릭/편집 strict 계약을 희생하지 않아야 한다.

## 게이트와 다음 단계

- 현재 제품 Studio 전체 재실행: **1,597 total / 1,596 pass / 1 skip / 0 fail**.
- `npm run build`(tsc 포함) 통과. 기존 CanvasKit fs/path externalization·bundle 크기 경고 유지.
- Rust source 변경 없음. 새로운 Rust/WASM 빌드를 성능 비교 중 실행하지 않았다.
- CanvasKit 및 나머지 4문서/배치/편집 시각 매트릭스, 실제 Firefox 핀치 최종 A/B,
  peak 메모리는 **이번 단계 미실행/미완료**. 이전 단계 smoke와 사용자 확인을 새 exact-head
  통합 수용 통과로 승격하지 않는다.
- 제품 코드를 추가로 수정하지 않았다. 경보 재현·최소 보정 범위를 승인받은 뒤 기존
  Stage 4 조건으로 재측정하고, 잔여 매트릭스를 이어간다. 이 중간 결과를 먼저 커밋한다.
- 원격 push·PR 생성·GitHub 게시·이슈 종료는 하지 않았다.
