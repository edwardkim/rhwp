# #6040 통합 후보 메모리 게이트 — 비가시 보존 보정

## 현재 판정 — 2026-09-10 보정 후

### 사용자 수용 확인

2026-09-10 작업지시자가 아래 두 항목의 Firefox 최종 비교 요청에 **“문제없어”**로 응답했다.
검증 요청의 after 제품은 `931306732`, 문서 기록을 포함한 로컬 HEAD는 `7d378e663`이었다.

1. 다른 페이지로 스크롤하고 멈춘 뒤 클릭 없이 선명해지는 읽기 화질.
2. 줌 아웃 직후 스크롤·빠른 재확대가 `37bd46a72` 기준선보다 악화되지 않는지.

해당 두 항목은 **사용자 정성 수용 통과**로 기록한다. 새 프로파일/계측 JSON은 받지 않았으므로
지연 percentile·Firefox 버전·전체 native/GPU peak를 추가로 검증한 것으로 해석하지 않는다.
기준선에도 남은 고배율 raster/native 전달 멈춤은 이번 보정의 미해결 회귀로 취급하지 않으며,
기존 별도 구조 개선 로컬 초안으로 구분한다. 후속 이슈 생성·실험은 승인하지 않은 상태다.

다음은 제출 범위·누적 diff·최종 보고서/PR 초안 정리다. 이번 수용 응답은 remote push,
PR 생성·merge 또는 #6040 종료 승인이 아니다. 소스 변경 없이 수용 기록만 갱신한다.

사용자 승인 후 제품 보정을 로컬 `931306732b9c59b88063dd54caaf921143410d13`으로 고정했다.
**아래 동일 시나리오의 소유 surface 회귀는 해소**했다. 이 계측 자체는 RSS/GPU peak 또는
Firefox 수용의 증거가 아니며, 사용자 수용은 위 응답을 별도 근거로 한다. 기존 실패 기록은 보존한다.

- [보정 후 최소 증거 — 로컬 보존 안내](assets/issue6040-post-stack/README.md): 동일 환경 2회,
  checkpoint별 pixel 값 동일, 관찰 오류 0, 종료 시 visible/prefetch queue와 pending 작업 정착.
- 전체 Studio **1,643 pass / 2 skip / 0 fail**, TypeScript·Vite·PWA build 통과.
- 6개 회귀 추가: 확대 시 visible 보호와 offscreen admission, 먼 쪽 우선 초과분 반환,
  축소 시 구 preview 실제 비용, offscreen 편집 focus 보존·strict 재진입,
  예산 안 exact warm 재사용, 미완료 layer backing 반환. 앞 4개는 보정 전 실패를 확인했다.

| 체크포인트 | devel before | 보정 전 후보 | 보정 후 후보 |
| --- | ---: | ---: | ---: |
| 첫 200% | 114,071,400 | 135,464,550 | 114,071,400 |
| 500% 최고 관찰값 | 268,450,552 | 402,101,695 | 268,450,552 |
| 500→100% | 33,874,172 | 33,874,172 | 28,524,200 |
| 34% | 23,103,360 | 15,677,280 | 15,677,280 |
| 34→100% | 55,267,322 | 55,267,322 | 34,169,202 |
| 비편집 읽기 200% | 48,125,352 | 221,018,100 | 85,553,550 |
| 재열기 100% | 33,874,172 | 33,874,172 | 28,524,200 |

500%에서 제거한 **133,651,143 pixel**은 비가시 page 1이며 visible page 0의 268,450,552는
그대로다. 비편집 읽기 200%에서는 비가시 두 쪽 **135,464,550 pixel**을 제거하고 visible의
85,553,550은 유지했다. before보다 여전히 큰 37,428,198은 클릭 없이 읽는 화질을 보호한 비용이다.
페이지 번호는 0-based다. RGBA 단순 환산과 브라우저 실제 메모리 사용량은 다르다.

### 구현 범위와 남는 절충

`zoom-settled`에서 기존 원장을 확인해 초과할 때만 화면 밖 active surface를 먼 쪽부터 반환한다.
기존 preview/다음 target 중 큰 비용을 공통 함수로 산정해 admission과 회계가 어긋나지 않게 했다.
반환한 페이지는 신규 prefetch와 같은 예산 검사에 다시 들어간다. visible의 preview·raw DPR,
편집 focus·strict 완료 계약, 일반 scroll의 스케줄 정책, 예산 상수는 변경하지 않았다.
완성 bundle은 기존 disposal 경로, 미완료 surface는 작업 취소·반환 및 backing 축 0화를 사용한다.
단일 retained 집합을 정렬·순회하며 제거마다 전체 원장을 반복 재계산하지 않는다.

큰 preview가 아직 남아 있는 축소 정착에서는 그 실제 비용 때문에 prefetch가 거절될 수 있다.
따라서 500→100%와 재열기 100%에 offscreen 한 쪽이 이전처럼 미리 그려지지 않았다. 이는
화면 누락이 아니라 선택적 미리 그리기의 보수적 admission이다. 다음 visibility 갱신/strict는
정상 복원한다. visible 완료 뒤 prefetch 자동 재시도 정책을 추가하지 않았으므로, 모든 cold
스크롤 지연이 개선된다고 주장하지 않는다. Firefox에서 재진입·줌 직후 스크롤 체감 확인이 남는다.

고배율 visible 단일 쪽 자체는 여전히 크다. 이번 보정은 전체 Canvas raster/native 전달의
선점 불가능한 비용을 해결하지 않으며 타일/Worker 등 별도 구조 실험을 포함하지 않는다.

### 작은 문서 warm 재사용 대조

[warm 최소 증거 — 로컬 보존 안내](assets/issue6040-post-stack/README.md)는 동일 환경의 4쪽 실문서,
한 쪽 보기 100%에서 DEV 왕복 20회 버튼을 before/after 각각 실행한 것이다. 메모리 관찰은 끄고,
첫 준비 뒤 20회 및 34→100% 줌 복귀 뒤 20회를 비교했다. 각 시나리오는 왕복 전에 방문 구간을
준비하므로 cold 최초 로딩 측정이 아니다.

- 네 실행 모두 기록한 20회 구간의 main/image **추가 raster 0회**, 모든 trace complete, 오류 0.
- 두 변형 모두 전체 보존 14,266,592 pixel, LRU eviction 0. 작은 working set을 강제로 비우지 않았다.
- cache hit는 첫 실행 종료 누적 10, 두 번째 종료 누적 20으로 동일했다. miss/invalidations는
  로딩·줌을 포함한 누적 값이므로 warm 구간의 miss 수로 해석하지 않는다.
- 이 스모크는 캐시 재사용 보존 근거다. Firefox 연속 입력 지연이나 모든 문서의 frame p95
  무회귀를 증명하지 않으며 실행 시간의 성능 향상 수치는 제시하지 않는다.

추가 화질 스모크: `exam_kor` 100%에서 다음 행으로 이동 후 200% 확대, 문서 클릭 없이
focus=0 / visible=[1]을 확인했다. visible DPR=2, main/background/behind 3개 surface 각
4491×6350, 합계 85,553,550 pixel, pendingImages=0, 오류 0이었다. 브라우저 스크린샷에서도
본문·표·한자 표시를 확인했다. 이는 DOM/page 좌표와 읽기 표시 스모크이며 한컴 출력 정답지
대조나 최종 Firefox 수용을 대체하지 않는다.

## 보정 전 기록

- 날짜: 2026-09-10 KST
- 단계: Stage 4 후속 검증. **메모리 게이트 실패, Firefox 최종 수용 요청 보류**.
- before 제품: `37bd46a72f9fd9ffd709e35244df79c00e789780`
- after 제품: `6ee35a19bb5b2e35b02229c48a8086655479de35`
- 후속 변경: DEV 전용 관찰 코드·테스트만 추가. 제품 view/렌더 정책은 변경하지 않았다.
- [최소 계측 증거 — 로컬 보존 안내](assets/issue6040-post-stack/README.md)

## 환경과 계측 범위

동일 devel의 before worktree와 통합 after를 각각 loopback 4206/4205로 실행했다.
WASM·package-lock·설치 의존성 및 관찰 adapter 5개 파일을 동일하게 맞췄다. before의
`src/view`는 수정하지 않았다. before에 새 관찰 adapter만 덮은 로컬 overlay는 제품 baseline과
구별하며 원격 제출하지 않는다.

- Canvas2D, 인앱 Chromium, viewport 1280×720, DPR 2, `exam_kor.hwp` 20쪽.
- WASM SHA-256: `6ee336d08591621b5e5da087014f5a3eebc999325eba726c1ddab8a2d943e719`.
- DEV `scrollProbe=1&scrollProbeMemory=1`의 **메모리 검사** 버튼을 사용.
- 순서: 문서 재열기·자동 배치·100% 준비 → 200→500→100→34→100% → 다음 행·200% →
  재열기·100%. 각 단계에서 known renderer/scheduler 작업 정착을 기다린다.
- before와 after 각 2회. 해당 checkpoint의 pixel 합계는 각 변형 안에서 두 번 모두 같았다.

관찰은 렌더/할당/반환 등 기존 메서드의 완료 경계와 명시 snapshot에서 active Canvas·overlay,
idle pool, detached LRU를 합산한다. 연속 핀치 기록을 켰을 때는 그 rAF에서도 표본을 수집한다.
최고 관찰값은 **소유 surface pixel peak의 관찰 하한**이다. 메서드 안의 짧은 임시 surface,
WASM 내부 surface, GPU 복사본, GC 대기 detached 객체, RSS를 포괄하지 않는다. DOM 열거 비용이
있으므로 memory-on 자료의 시간 수치로 성능 개선을 주장하지 않는다. 일반 URL/production에는
이 관찰이 적용되지 않는다. 전체 기록을 쌓지 않고 count·peak·last만 보존한다.

## 결과

| 체크포인트 | before 소유 pixel | after 소유 pixel |
| --- | ---: | ---: |
| 첫 200% | 114,071,400 | 135,464,550 |
| 500% (최고 관찰값) | 268,450,552 | 402,101,695 |
| 500→100% 복귀 | 33,874,172 | 33,874,172 |
| 34% 다중 페이지 | 23,103,360 | 15,677,280 |
| 34→100% | 55,267,322 | 55,267,322 |
| 비편집 쪽 읽기 200% | 48,125,352 | 221,018,100 |
| 문서 재열기 100% | 33,874,172 | 33,874,172 |

500%의 차이는 **133,651,143 pixel(+49.79%)**이며 after에만 남은 화면 밖 page 1이다.
보이는 page 0은 양쪽 모두 268,450,552 pixel이다. after 합계의 RGBA 단순 환산은
1,608,406,780 bytes이며 실제 RSS 측정값이 아니다.

200% 비편집 읽기의 증가 172,892,748 pixel은 다음 둘로 분리된다.

1. visible page 1 화질 복원: 48,125,352 → 85,553,550, 증가 **37,428,198**.
2. after에 남은 offscreen page 0(편집 focus)과 page 2: **135,464,550**.

읽는 쪽이 클릭 전 흐리던 문제를 되살려 총량을 낮추는 것은 수용하지 않는다. 그러나 화면 밖
보존 비용까지 최종 읽기 화질의 필수 비용이라고 설명할 수도 없다.

34%에서 after가 작고, 재열기 뒤 원래 크기로 돌아온다는 사실은 보존 정책이 상태에 의존함을
보여준다. 전체 메모리 누수 없음·브라우저 즉시 회수의 증거는 아니다.

## 원인과 해제 경계

- `buildPrefetchRenderWork()`는 active Canvas가 있으면 `retained-transition`, 없으면
  `prefetch`로 분류한다. `tryReservePrefetchSurface()`는 후자에만 적용된다.
- before는 줌 정착 때 전역 해제하여 새 offscreen 페이지가 prefetch 예산 검사에 걸린다.
- after는 기존 화면을 유지하므로 이미 있던 offscreen Canvas가 retained-transition으로
  남는다. 그것을 확대 배율로 다시 그리는 데 같은 admission이 적용되지 않는다.
- `reconcilePageSurfaceBudget()`는 active의 목표 비용을 mandatory 예약에 합산한다.
  초과 원장을 기록하고 LRU를 퇴거시켜도 이미 active인 offscreen 전환 자체는 막지 못한다.
- CanvasPool의 반환과 LRU 퇴거는 backing width/height를 0으로 만든다. 일반
  `removePageLayers()`/`removeAllPageLayers()`는 DOM 제거만 수행한다. 이 경로의 실제 메모리
  회수 시점은 GC/native 수명에 달려 있어 DOM census만으로 즉시 회수를 주장할 수 없다.

소유권/LRU disposal 기존 회귀와 새 관찰 테스트를 포함한 전체 Studio 결과는
**1,637 pass / 2 skip / 0 fail**. after와 공통 adapter를 올린 before 모두 TypeScript·Vite·PWA
build를 통과했다. 이 검사는 위 메모리 회귀를 통과로 바꾸지 않는다.

## 당시 최소 보정 제안 — 이후 승인·구현됨

이 문제는 B의 전역 해제 제거와 기존 retained admission 사이의 통합 결함이므로 #6040 안에서
보정한다. Worker·타일·장치별 자동 예산·전 페이지 preload로 범위를 넓히지 않는다.

1. 줌 정착에서 visible의 preview 유지와 최종 화질을 보장한다. strict 편집 경로도 유지한다.
2. offscreen의 기존 Canvas도 새 배율로 전환할 때 비용·필요성을 재심사한다. 예산이 없으면
   불필요한 확대 raster를 시작하지 않는다. 미렌더 상태를 무한 정착 대기로 남기지 않도록
   active 소유권·기존 surface/LRU 반환·known-work 판정을 함께 정리한다.
3. offscreen에 caret이 있다고 해서 사용자가 보는 화질을 낮추지 않는다. 비가시 surface를
   해제하는 경우에도 편집 focus 자체를 바꾸지 않고, 다시 보이거나 strict 편집하면 현재 계약에
   맞게 복원한다. 실제 구현에서 이 조건 보존 가능성을 회귀로 먼저 고정한다.
4. 100→500 확대·500→100 축소·visible 진입·strict/취소/LRU 회귀와 같은 메모리 시나리오를
   재실행한다. 작은 문서 warm 왕복의 raster 증가도 확인한다. 예산 상수 증액으로 우회하지 않는다.

이 제안은 작성 당시 미구현이었으며, 이후 승인에 따라 위 보정·검증을 수행했다.
메모리 회귀를 후속 구조 이슈로 넘겨 #6040을 닫지 않는다. Firefox 비교는 별도 수용 게이트다.

## Firefox 비교 준비와 기존 기록

before 4206 / after 4205에 `scrollProbe=1&scrollProbeAB=1`을 사용하면 동일한 수동 계측 UI를
이용할 수 있다. after 제품은 이제 `931306732`다. **성능 측정에는 `scrollProbeMemory`를 넣지 않는다.** 양쪽 모두 같은 문서,
화면 크기·배치·배율을 명시적으로 맞추고 한 번에 한 탭만 조작한다. 이번 보정의 자동 검사 뒤
클릭 없는 읽기·줌 직후 스크롤·재핀치의 최종 사용자 수용 판정을 요청한다.

기존 사용자 cold JSON은 Firefox 155, 1512×845, `before-56706247f`/
`after-0345107c5`, `cold-scroll-v1`였다. 시나리오와 브라우저 기준 자료로 재사용하되 최신
통합 head의 무회귀 증거로 대체하지 않는다. 원본은 사용자의 Downloads에 그대로 보존했다.
