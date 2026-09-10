---
kind: guide
status: active
canonical: mydocs/manual/studio_scroll_probe_guide.md
last_verified: 2026-09-10
---

# Studio 스크롤·줌 개발 측정 패널

페이지가 많은 문서에서 새 구간 표시, 같은 구간 왕복, 정착 후 화질과 Canvas 재사용을 조사하는 개발
도구다. 문서·배치·줌을 반복 조작하고 이미 존재하는 렌더 경계를 관찰한다. 제품의 예산·화질 정책을
설정하는 UI나 일반 사용자를 위한 기능은 아니다.

구현은 [`page-scroll-probe.ts`](../../rhwp-studio/src/dev/page-scroll-probe.ts), 지표 계약과 테스트는
[`scroll-observation.ts`](../../rhwp-studio/src/dev/scroll-observation.ts),
[`scroll-observation.test.ts`](../../rhwp-studio/tests/scroll-observation.test.ts)를 따른다. 내부
CanvasView/LRU/scheduler에 의존하므로 다른 애플리케이션에 그대로 붙이는 독립 라이브러리는 아니다.

## 시작하기

1. [개발 환경 가이드](dev_environment_guide.md)에 따라 현재 checkout과 맞는 WASM, Studio 의존성과
   폰트를 준비한다. 다른 checkout의 WASM이나 `--no-opt` 빌드와 최적화 배포판을 섞어 성능을 비교하지 않는다.
2. 저장소 루트에서 개발 서버를 실행한다. 예시 포트가 사용 중이면 다른 포트를 선택한다.

   ```bash
   npm --prefix rhwp-studio run dev -- --host 127.0.0.1 --port 4198 --strictPort
   ```

3. [Canvas2D 측정 URL](http://127.0.0.1:4198/?renderer=canvas2d&scrollProbe=1)을 연다.
   `renderer=canvaskit`도 사용할 수 있다. 실제 선택된 backend를 별도로 확인하고 기록한다.
4. 오른쪽 위 **기준선 문서 → 실문서 열기**를 누르고 `완료`를 기다린다. 선택지는 `exam_kor`,
   `hwpspec`, `kps-ai`, `KTX 4-layer`, 4쪽 실문서, 21쪽 다중 레이어다. 파일은 저장소 `samples/`에서
   불러오며 없으면 임의의 대체 파일로 측정하지 말고 환경을 먼저 복구한다.

[`main.ts`](../../rhwp-studio/src/main.ts)의 `import.meta.env.DEV`와 `scrollProbe=1`이 모두 있어야
패널이 설치된다. production build/preview/배포 확장에는 URL을 붙여도 나타나지 않는다. DEV에서도
옵션 없이 열면 설치되지 않는다. 끄려면 쿼리를 제거하고 다시 로드한다.

아래는 과거 측정 중 패널이 보이는 예시다. 현재 성능이나 화질의 기준 이미지는 아니다.

![개발 패널과 hwpspec 다중 쪽 예시](../working/assets/issue6042-stage2/hwpspec-auto-34.jpg)

## 조작과 결과 저장

| 조작 | 실제 동작과 주의점 |
| --- | --- |
| 배치 / 34·50·100·200% 줌 | 기존 page-view 설정과 smooth zoom 경로 사용. `네 열`은 4열·2행 설정이며 자동 배치와 다르다. |
| 처음으로 / 다음 행 | 문서 시작 또는 현재 쪽 높이+gap만큼 세로 이동한다. |
| 왕복 20회 | 시작 위치를 맞춘 뒤 두 위치를 **총 20번 이동**한다. 20개의 왕복 쌍이 아니다. 각 이동의 알려진 작업이 두 프레임 연속 안정될 때까지 기다린다. |
| 관찰 비용 A/B | 같은 제품에서 **계측 on/off** 비용을 비교한다. 제품 수정 전/후 비교가 아니다. 12라운드마다 on/off 순서를 번갈아 수행하며 첫 2라운드는 warm-up으로 집계에서 제외한다. |
| 관찰 결과 | 현재 DOM/surface/queue와 최근 trace의 snapshot을 JSON으로 표시한다. 명시적 snapshot은 DOM bounds도 읽으므로 시간 표본과 분리한다. |
| 기록 초기화 | trace/error/long-task 기록을 비운다. 문서를 다시 로드하거나 LRU·브라우저 cache를 비우지는 않는다. |
| 연속 핀치 시작 / 종료 | 최대 20초의 수동 기록 구간을 `pinchSession`에 보존한다. 입력 사이에 제품이 정착해도 구간을 끊지 않는다. 종료 시 JSON을 표시하지만 렌더 완료를 뜻하지는 않는다. |

`왕복 20회`와 `관찰 비용 A/B`가 끝나면 `Stage 1 JSON` 영역에 결과가 표시된다. 이는 남아 있는 UI
이름이며 현재 구현 전체의 관찰 결과다. **일반 패널에는 다운로드 버튼이 없다.** 아래 A/B opt-in을
사용하지 않는 경우 JSON 전체를 복사해
UTF-8 `.json` 파일로 저장한다. 일괄 결과를 저장하기 전에 `관찰 결과`를 누르면 `samples`가 있는
일괄 결과 대신 현재 snapshot으로 바뀌므로 먼저 복사한다.

시간 표본 수집 중에는 화면 크기·줌·편집 focus를 바꾸거나 다른 패널 버튼을 누르지 않는다. 스크린샷,
DevTools profiling, DOM snapshot은 별도 반복에서 수집한다. 관찰을 켠 채 wheel·PageUp/PageDown을
직접 조작할 수도 있지만 빠른 연속 입력은 기존 trace를 interrupt하므로 개별 제스처 전체 시간으로
해석하지 않는다. 패널의 프로그램 이동은 실제 scroll/rAF 경로를 쓰지만 wheel 관성까지 재현하지 않는다.

### 연속 트랙패드 핀치 수집 (#6040)

기존 `traces`는 wheel마다 새 기록이 되므로 연속 입력의 원인 분석에는 별도 `pinchSession`을 사용한다.
이 기능도 DEV query opt-in이며 제품의 줌 종료 판정·화질·렌더 순서는 수정하지 않는다.

1. 원하는 문서를 열고 **자동** 배치와 **100%**를 설정한다. 문서 로딩과 초기 렌더를 기다린다.
2. **연속 핀치 시작**을 누른 뒤 페이지 영역에서 약 3~5초에 걸쳐 100%→34% 근처로 핀치한다.
   손을 뗀 뒤 페이지가 나타나는 과정까지 기다리고 **연속 핀치 종료**를 누른다. 총 구간은 20초 미만이다.
3. 표시된 JSON을 복사해 저장한다. 같은 브라우저 탭을 공유하는 작업에서는 종료 후 그대로 두고
   수집 완료를 알리면 에이전트가 표시된 JSON을 읽을 수 있다. 다음 시작/초기화/새로고침 전에 저장한다.
4. 빠른 줌아웃, 100→34→100% 방향 전환도 별도 구간으로 수집한다. 첫 입력 전후의 정지 시간까지
   기록되므로 전체 구간 길이를 줌 완료 시간으로 계산하지 않는다. 다른 설정 변경/편집은 섞지 않는다.
5. 프로파일러·동영상·스크린샷은 별도 반복에서 수집한다. 패널의 rAF 간격은 compositor의 실제
   paint 시각이 아니며, 계측 on/off의 비용도 동일 입력 조건에서 따로 확인한다.

`inputs[].at`, `spans[].at`, `frames[].at`, `longTasks[].at`는 같은 `startedAt` 기준의 ms다.
span 배열은 호출 **종료** 순서로 추가되므로 원인 분석에서는 시작 시각으로 정렬하고, 중첩된
inclusive 시간을 합산하지 않는다. span의 zoom/animating/columns는 해당 호출 반환 시점의 상태다.
Stage 3.2A 후보부터는 `inputActive`(quiet 대기), `rasterPending`(quiet 또는 수렴 대기),
`zoomGeneration`도 기록한다. `animating=false`만으로 최종 렌더 횟수를 세지 않고 `zoom.raster`와
`page.releaseAll`을 확인한다. `zoom.ready`는 지연 정착 진입, `zoom.cancel`은 실제 pending을 취소한
호출만 센다. `zoom.flush`는 일반 wheel 등 명시 조작의 완료 요청이며 no-op 호출도 포함한다.
패널 최상위 `zoomInput`에서도 상태를 확인할 수 있고 pending이면 known-work 완료로 판정하지 않는다.
Stage 3.2B 페이지별 갱신 후보에서 `zoom.raster`는 **최종 raster 예약 진입**이지 전체 렌더 시간이
아니다. `raster.main` span, scheduler의 visible 실행·잔여 큐, `visibleStable`을 함께 본다.
전역 해제가 없으므로 `page.releaseAll=0`만으로 raster가 없었다고 해석하지 않는다. 구 zoom
preview는 physical scale이 clamp로 같아도 최신 visible 완료로 세지 않는다.
`zoom.smooth`는 공통 smoothing 적용 경계로서 wheel과 버튼을 모두 포함한다.
원장의 wheel에는 일반 스크롤도 포함되므로 줌 분석은 `ctrlKey || metaKey`로 분리한다.
`inputs[].eventTimestamp`는 원 이벤트 timestamp이며 손가락을 뗀 시각 또는 OS 입력 시각의 보증이
아니다. `trusted=true`도 브라우저 자동화로 생성될 수 있으므로 실제 트랙패드인지 사용자 설명을
함께 남긴다. 프로그램 버튼과 제어된 wheel은 도구 smoke이지 물리 핀치 성능 표본이 아니다.

- `status=stopped`는 수동 종료이며 `knownWorkReadyAtStop`은 기존 runner가 아는 작업의 readiness다.
  이미지·compositor 전체 완료 또는 최종 화면의 품질을 보증하지 않는다.
- `timeout`/`interrupted`는 성공으로 세지 않는다. 관찰 off, 문서/renderer/content 전환은 중단한다.
  원시 frame 중 긴 작업이 있으면 20초 상한 감지도 그 작업 다음 callback에서 실행된다.
- 입력·span은 각각 8,192개, frame은 2,048개, long task는 256개까지 보관한다. `dropped`가 0이
  아니면 해당 상세 시간축은 불완전하며, 더 짧은 구간으로 재수집한다. counter 합계는 상세 누락 후에도
  유지되지만 시간축이 완전한 것으로 해석하지 않는다.
- `longTasksSupported=false`면 빈 long-task 배열을 “긴 작업 없음”의 증거로 쓰지 않는다.
- 이 계측으로 먼저 **입력 사이에 release/raster가 반복되는가**를 확인한다. low-resolution 선렌더나
  별도 미리보기 cache를 활성화하는 기능은 아니다.

### 첫 진입·왕복 A/B 기록 (#6040)

DEV URL에 `scrollProbe=1&scrollProbeAB=1&abVariant=before` 또는 `after`를 붙이면 아래 버튼과
추가 rAF 상태 표본이 활성화된다. `abVariant`는 사용자 지정 이름이지 실제 source SHA의 증명이
아니다. 서버별 exact SHA/WASM/adapter hash는 별도로 고정한다.

1. `exam_kor`와 같은 검증 배치를 고른다. 자동 축소 재현에는 `자동`을 선택하며, `네 열`과 혼동하지
   않는다. 두 브라우저 창의 크기·브라우저 자체 확대/축소도 같게 유지한다.
2. **새 문서 + 첫 진입 기록**은 100%로 설정하고 선택한 문서를 새로 연다. 초기 동기 화면 구성 뒤
   `document-view-loaded`에서 자동 기록을 시작한다. first-view 이후의 cold surface 진입이며,
   최초 WASM 로드·파일 fetch/파싱 전체나 OS/network cold를 재는 기능이 아니다.
3. 첫 화면이 나오면 즉시 같은 배율로 핀치하고 스크롤한다. **연속 핀치 종료 → JSON 저장**으로
   기록한다. 20초 제한을 지키며 `timeout`/`interrupted`도 실패·미완료 표본으로 보존한다.
4. 기존에 지나간 구간으로 돌아가 안정화한 뒤 **현재 구간 기록**을 누르고 같은 구간을 왕복한다.
   `warm-manual`이라는 이름이 실제 cache hit나 fully-warm을 보증하지는 않는다. cache/queue 지표를
   함께 판정한다. 종료·저장 후에만 다음 측정을 시작한다.
5. 실제 스크롤·핀치 표본과 버튼 자동화 smoke, profiler·스크린샷 반복을 구분한다. 같은 adapter라도
   관찰 오버헤드가 같다고 단정하지 않으며 on/off 대조는 별도 수행한다.

`pinchSession.frames`에 scrollY, visiblePages, missingBitmapPages, currentRasterPages,
visibleQueued/prefetchQueued/pendingImages를 추가한다. cached 상태만 읽으며 DOM bounds 또는
Canvas readback을 매 frame 수행하지 않는다. `missingBitmapPages`는 부착 Canvas가 없는 쪽이다.
Canvas 내용 자체가 빈지, 이미지 decode나 compositor 게시까지 완료됐는지는 보증하지 않는다.

구 기준선에는 없는 zoom pending API/관찰 경계를 제품에 추가하지 않는다.
`observationCapabilities.unavailableBoundaries`와 두 capability boolean으로 차이를 기록한다.
`zoom.smooth`는 기준선의 `smoothZoomTo`와 후보의 `applySmoothZoomTo`를 감싸므로 그 inclusive
시간을 같은 함수 비용으로 직접 비교하지 않는다. 공통 raster 경계·frame/input 지표를 우선한다.

### 예산 갱신 정밀 계측 (#6040)

`scrollProbe=1&scrollProbeBudget=1`을 추가하면 기존 호출에만 복원 가능한 wrapper를 설치한다.
A/B 저장 버튼도 필요하면 `scrollProbeAB=1`을 함께 사용한다. 옵션이 없으면 이 상세 경계는
설치하지 않으며 제품의 DPR·예산·스케줄링·캐시를 변경하거나 미리 조회하지 않는다.

| 경계 | 관찰 대상 |
| --- | --- |
| `budget.layerCount` | 캐시를 포함한 페이지별 surface layer 수 조회 |
| `budget.overlaySummary`, `budget.treeSummary` | 레이어 요약과 fallback tree 조회·파싱 |
| `wasm.overlayImages`, `wasm.layerTree` | JS→WASM bridge 동기 조회 전체 |
| `budget.descriptor` | 쪽별 surface descriptor 생성 |
| `budget.reconcile`, `cache.reconcile` | 예약 픽셀 원장과 LRU 예산 정리 |

`observationCapabilities.budgetDetail=nested-boundaries-v1`로 설치를 식별한다. 구 revision의
없는 경계는 `unavailableBoundaries`에 남긴다. page는 0-based이며 reconcile의 숫자 인수는
page가 아니다. `budget.refresh` 안의 자식 구간만 골라 **시간 구간 합집합**으로 중첩을 제거한다.
`layerCount → overlaySummary → wasm.overlayImages`를 더하면 같은 시간을 세 번 세게 된다.
자식 제외 잔여 시간에는 순수 planner뿐 아니라 Map/진단 갱신과 관찰 비용도 있으므로 순수 예산
산술 시간으로 단정하지 않는다. WASM 내부의 tree 생성·이미지 변환·직렬화는 이 계측만으로
더 분리되지 않는다. 0ms는 브라우저 시계의 해상도 이하라는 뜻이지 연산 없음이 아니다.

상세 wrapper 때문에 span 수와 관찰 비용이 늘어난다. 짧은 cold/재방문 구간을 별도로 수집하고
`dropped.spans=0`을 확인한다. 이 옵션의 시간값을 기본 계측만 켠 결과와 직접 개선율로 비교하지
않는다. 집계·JSON 직렬화·DOM snapshot은 종료 뒤 수행한다.

### 소유 surface 메모리 검사

DEV URL에 `scrollProbe=1&scrollProbeMemory=1`을 추가하면 **메모리 검사** 버튼이 나타난다.
선택 문서를 새로 열고 자동 배치·100%로 준비한 다음 200→500→100→34→100%, 다음 행·200%,
문서 재열기·100%를 실행한다. 각 단계는 알려진 작업의 정착을 기다리고 `checkpoints`에
최고 관찰값과 마지막 소유 상태를 남긴다. 오류/12초 정착 timeout은 실패로 표시한다.

`surfaceMemory`는 기존 관찰 경계 완료·명시 snapshot 및 켜진 핀치 기록의 rAF에서 active main/
overlay, idle pool, detached LRU의 물리 pixel을 합산한다. 기록은 count·peak·last로 제한한다.
메서드 안의 임시 Canvas, WASM 내부·GPU 복사본·GC 대기 객체는 포함하지 않으므로 **관찰 하한**이지
실제 RSS나 GPU peak가 아니다. DOM 열거 비용이 추가되어 **이 옵션은 시간 A/B에서 끈다**.
`기록 초기화`는 이 집계도 비우지만 제품 surface/cache를 지우지 않는다. 일괄 완료 결과를 복사한
뒤 다른 버튼을 누른다. [최종 요약 예시](../working/assets/issue6040-post-stack/final-summary.json)는
실행 결과의 축약 사례이며 원시 trace를 모두 커밋하는 기준이 아니다.

## 무엇을 측정하는가

시간 단위는 ms다. 동일 이름이라도 제품 revision에서 완료 정의가 바뀌었는지 먼저 확인한다.

| 필드/지표 | 의미 | 의미하지 않는 것 |
| --- | --- | --- |
| `samples[].syncMs` | scroll setter가 반환하기까지의 동기 시간 | 전체 렌더 완료 시간 |
| `knownWorkNextFrameMs` | runner가 아는 작업과 두 프레임 안정 대기 완료 | compositor 표시 완료/모든 decoder 완료 |
| `preview` | geometry·ruler 줌 값이 일치한 관찰 경계 | 모든 visible 쪽의 선명한 raster 완료 |
| `visibleFirst` / `visibleStable` | 첫 visible / visible 집합의 알려진 bitmap 준비 | 이후 정착 DPR 승격까지 완료했다는 보장 |
| `focusedSharp` | visible 편집 쪽의 관찰 가능한 이미지 완료까지 확인 | 비가시 focus의 완료 시간; 관찰 근거가 없으면 값이 없을 수 있다. |
| `retainedComplete` | 실제 materialized retained와 scheduler의 알려진 작업 완료 | 후보로만 남은 모든 쪽의 강제 raster |
| `counters` / `traces[].frames` | 호출 횟수·inclusive 시간 / 관찰 rAF 간격 | 중첩 시간을 합한 CPU 총시간 / 실제 dropped frame 수 |
| `longTasks` | 지원 브라우저의 Long Tasks 관찰 기록 | 모든 브라우저에서 동일하게 얻는 지표 |
| `pages[].surfaces`, `activePixels`, `detachedCache` | 실제 backing 크기와 추적 중인 cache·예약 비용 | 프로세스 RSS나 GPU 총메모리; pixels×4는 RGBA 단순 환산일 뿐이다. |

관찰 기록은 bounded buffer다. 장시간 조작 결과를 무한 보존하지 않으므로 작은 시나리오별로 저장한다.
`timeout`, `interrupted`, 이미지 실패, 남은 queue는 누락된 표본이 아니라 별도로 보고할 결과다. 임의로
버린 뒤 성공 표본만 비교하지 않는다. `관찰` off는 wrapper를 복원하지만 같은 runner/이벤트 구독을
사용하므로 패널이 아예 없는 production과 동등하다는 뜻도 아니다.

## 다른 변경에서 재사용하는 방법

1. before/after exact SHA와 계측 adapter revision, fixture SHA-256, 실제 backend, browser/OS/장치,
   viewport CSS px, 실제 DPR, zoom·배치, WASM/JS/lock/font hash와 빌드 옵션을 함께 기록한다.
   기존 [환경 기록](../working/assets/issue6042/environment.json)은 형식 예시이며 새 실행의 환경이 아니다.
2. 같은 조건의 별도 서버에서 A/B 순서를 번갈아 반복한다. cold 새 구간, warm 동일 구간 왕복,
   fully-warm 전체 로드 후 왕복을 구분한다. 패널의 `왕복 20회` 버튼 하나가 이 세 조건을 자동으로
   분리하는 것은 아니다. cold는 각 반복의 문서/cache 초기화 조건과 목적지를 명시한다.
3. p50/p95·표본 수·raster/cache·queue/error·physical pixels를 함께 본다. 첫 표시가 빨라도 최종
   화질 회복은 늦을 수 있다. 정착 후 화질은 별도 스크린샷/실제 DPR로 확인한다.
4. 회귀 경보선과 warm-up 제외 규칙은 결과를 보기 전에 정한다. #6042의 수치나 임계값을 다른 장치의
   절대 합격선으로 복사하지 않는다. 비교되는 양쪽에 동일한 계측 계약이 없으면 먼저 그 한계를 밝힌다.
5. PR에는 방법·환경·대표 A/B의 모든 반복·재집계 명령·불리한 결과를 포함한 요약을 남긴다. 폐기
   표본과 중간 smoke 전체를 계속 누적할 필요는 없다. 이번 PR의 선택 예시는
   [최소 증거 색인](../working/assets/issue6042/README.md)을 따른다.

패널은 원인 분리용 도구이며 사용자 조작, 브라우저 profiling과
[시각 검증 정책](verification/visual_verification_governance.md)을 대체하지 않는다.
