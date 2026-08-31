# Task M100 #6042 Stage 1 — 기준선·관찰 계약

- Issue: [#6042](https://github.com/edwardkim/rhwp/issues/6042)
- 작성일: 2026-08-31 KST
- 상태: **Stage 1 완료 — Stage 2 승인 대기**. 제품 최적화는 미착수.
- 승인: 계획 커밋 `da1c4d9d6` 뒤 작업지시자의 `진행해줘`.
- 직접 기준선: #6467 `ba68cd655aed5fd94804f725c033cf615231ce4b`.
- 계획: [수행](../plans/task_m100_6042.md), [구현](../plans/task_m100_6042_impl.md).

## 1. 이번 단계의 경계

제품 `src/view/`·Rust·DPR·geometry·zoom anchor·눈금자 scheduling은 바꾸지 않았다.
`main.ts`의 DEV/query 조건부 loader와 `src/dev/`의 관찰 도구만 추가했다. 기본 URL과 production에는
관찰기를 설치하지 않는다. 기존 #6521 LOD 코드도 가져오지 않았다.

무수정 #6467 서버(4188)와 계측 서버(4186)를 분리하고 같은 WASM·JS·font 자산을 사용했다.
처음 연 exam_kor의 main Canvas dimensions·위치·전체 dataset은 두 서버에서 동일했다.
[초기 DOM 대조 원본](assets/issue6042/initial-parity.json)은 픽셀 단위 screenshot 동일성 증명은 아니다.

## 2. 관찰 계약과 발견한 계측 결함

- wrapper는 원 `this`·인수·반환 값·동일 Promise·예외를 보존한다. off는 wrapper 내부의 boolean
  분기만 바꾸는 것이 아니라 원 own descriptor/prototype 경계를 복원한다.
- 최대 128 interaction, interaction당 512 span/512 frame interval로 기록량을 제한했다. DOM bounds,
  모든 surface dimension 열거, JSON 직렬화는 명시적인 결과 읽기에서만 수행한다. DOM flow-image의
  참조 수집은 layer 생성 경계에서 하고, frame에서는 보유 참조의 완료 속성만 읽는다.
- document generation / renderer epoch / 관찰된 content epoch / interaction id / image request token을
  구분한다. 오래된 callback, 중단, timeout, 알려진 decode 실패를 성공 표본으로 바꾸지 않는다.
- `setScrollTop`은 DOM 좌표만 동기로 바꾸고 visibility는 scroll rAF에서 갱신한다. 첫 관찰기는 그
  사이의 **이전 visible 집합**에 완료 mark를 찍었다. 목표 좌표·zoom·scope의 이벤트 반영 확인을 추가했다.
  [제외 표본](assets/issue6042/discarded-pre-ack-exam-scroll.json)은 최종 통계에서 제외했다.
- ruler와 geometry callback 사이에서 이미 일치한 preview를 다음 observer rAF가 놓칠 수 있었다.
  일치한 호출 경계에서 바로 mark하도록 보정했다. [중간 표본](assets/issue6042/intermediate-ack-exam-scroll.json)도
  최종 집계에서 제외했다. 이들은 **제품 회귀가 아니라 관찰 도구 결함**이다.
- 178→4쪽 교체에서 파싱 완료(`document-loaded`)를 화면 구성 완료로 잘못 구독해 관찰 runner가
  12초 timeout을 냈다. 실제 visible/retained 4쪽과 focus 0은 이미 정상이었다.
  `document-view-loaded`로 교정했고, [실패 기록](assets/issue6042/discarded-document-switch-178-to-4.json)은
  보존했다. 성공한 scroll 표본도 최종 관찰기에서 다시 수집했다.
- fallback timer가 job을 제거해도 아직 pending인 decoder는 완료로 승격하지 않는다. 관찰된 실패는
  interrupted, 계속 pending이면 timeout이다. 관찰을 켜기 전 끝난 decode처럼 증거가 없는 경우는
  unknown으로 구분하고 focused-sharp는 기록하지 않는다.

측정값의 뜻:

| 지표 | 관찰한 경계 | 주장하지 않는 것 |
| --- | --- | --- |
| preview | geometry와 ruler가 같은 zoom으로 갱신된 첫 호출 경계 | compositor가 실제 표시한 시각 |
| visible-first | 최종 zoom·목표 scroll 반영 후 준비된 가시 Canvas 첫 확인 | 첫 픽셀 scanout |
| focused-sharp | 가시 focus의 최종 raster와 관찰된 decode/캐시/no-image 근거 | 비가시 focus의 완료, 미관찰 decode 추정 |
| visible-stable | 최종 가시 Canvas·알려진 이미지 작업·관찰된 DOM image 완료 | 모든 backend 자원의 완전한 품질 인증 |
| retained-complete | 기존 working set의 알려진 작업 완료 | 화면 밖 전체 문서 완료 |
| knownWorkNextFrameMs | 목표 scroll 반영 및 알려진 작업이 연속 두 rAF에서 안정 | input handler CPU 시간, GPU/RSS, 실제 dropped frame 수 |

시간들은 상위·하위 inclusive span이 중첩된다. `raster.main + wasm.layerRaster`처럼 더해서 전체 시간을
만들지 않는다. `syncMs`는 거의 0ms인 scroll setter 시간이며, 렌더가 빨라졌다는 근거로 쓰지 않는다.

## 3. 재현 방법

1. 고정 #6467과 이번 worktree에서 같은 lockfile/runtime/generated pkg/fonts로 Vite를 실행한다.
2. 계측 서버는 `?renderer=canvas2d&scrollProbe=1&url=/samples/exam_kor.hwp`로 연다.
3. `기준선 문서` → `실문서 열기`, `검증 배치`, 배율 버튼으로 조건을 정한다.
4. `왕복 20회`는 시작 위치를 2행 높이로 맞춰 안정화를 기다린 후 5행/2행 위치를 20회 번갈아 요청한다.
   문서 끝에서는 실제 scrollTop으로 clamp된다. 실제 y는 raw sample에 저장된다.
   이 시간 시나리오는 20/178쪽 문서에 사용했다. 짧은 문서에서 두 위치가 같아지는 no-op은 유효 성능
   표본으로 쓰지 않는다. KTX·4쪽 문서는 레이어·줌·문서 교체 확인용이다.
5. `관찰 비용 A/B`는 매 round off/on 순서를 교대한다. 12 round 중 첫 2 round를 버리고 같은 실행을
   두 번 반복해 조건별 off 20 / on 20, 총 20쌍을 집계한다. off에도 같은 runner의 이벤트 ack·두 rAF
   안정화 확인이 남으므로 **완전히 관찰 없는 사용자 입력과의 비교가 아니라 wrapper 증분 비용**이다.
6. screenshot/readback·컴파일·테스트는 최종 시간 표본과 분리한다. 원본은 [자산 폴더](assets/issue6042/)에
   보존하고 `node mydocs/working/assets/issue6042/summarize.mjs`로 재집계한다.
   [집계 JSON·자산 hash](assets/issue6042/summary.json)도 함께 보존했다.

## 4. 결과

이 단계는 최적화 전 기준선이며 성능 개선률을 주장하지 않는다.

### 환경

Canvas2D, 1280×720 CSS px, 실제 DPR 2, 같은 36개 font manifest를 사용했다.
Node 24.15.0 / npm 11.12.1 / Vite 8.2.2. Browser UA는 각 raw JSON에 기록했다.
[환경·source/WASM/lock/font hash](assets/issue6042/environment.json)를 고정했다.
`Σ(canvas.width×canvas.height)`는 현재 main·overlay·idle pool backing store 합이며,
RGBA 환산은 여기에 4를 곱한다. WASM heap·DOM image decoder·실제 GPU/RSS 메모리는 포함하지 않는다.

### 계측 자체의 증분 비용

같은 4열 34% 시나리오, 앞 2 round 제외, on/off 각각 20표본이다. 아래 시간은 모두 ms다.

| 문서 | off 중앙값 / p95 | on 중앙값 / p95 | 중앙값 차이 |
| --- | ---: | ---: | ---: |
| exam_kor 20쪽 | 155.45 / 158.70 | 154.25 / 163.20 | −1.20ms (−0.77%) |
| hwpspec 178쪽 | 320.05 / 372.80 | 320.45 / 334.90 | +0.40ms (+0.12%) |

쌍별 on−off 차이의 중앙값/p95는 시험지 −1.3/7.0ms, hwpspec −2.2/13.1ms다.
hwpspec off의 최대 449.2ms 같은 이상치를 포함해 원본·표준편차를 보존했다. 관찰 on이 빠른 표본은
**관찰기가 제품을 최적화한 결과가 아니라 반복 잡음**이다. 이 두 조건에서는 큰 계측 부담이 관측되지
않았다는 제한적인 판단이며, 모든 문서/backend에서 무시 가능한 비용이라고 일반화하지 않는다.

### 제품 최적화 전 스크롤 기준선

조건당 20회, 총 **120회 모두 complete / 관찰 오류 0건**이다. 최초 로드·서버 startup 시간은 제외했다.
행 위치를 건너뛰는 왕복이므로 서로 다른 배율의 숫자만 비교해 줌별 우열을 주장할 수 없다.

| 문서 | 배치·배율 | visible-stable 중앙값 | 알려진 작업+두 rAF 중앙값 / p95 | 한 interaction의 main raster 수 |
| --- | --- | ---: | ---: | ---: |
| exam_kor 20쪽 | 1열·100% | 63.75ms | 182.35 / 196.80ms | 3 |
| exam_kor 20쪽 | 2열·50% | 124.45ms | 332.45 / 366.70ms | 6 |
| exam_kor 20쪽 | 4열·34% | 152.55ms | 266.55 / 383.60ms | 2 또는 6 |
| hwpspec 178쪽 | 1열·100% | 21.60ms | 56.15 / 68.80ms | 3 |
| hwpspec 178쪽 | 2열·50% | 38.65ms | 95.15 / 118.60ms | 6 |
| hwpspec 178쪽 | 4열·34% | 171.00ms | 257.45 / 322.90ms | 12 |

방향에 따른 비용이 다르다. 시험지 4열은 하행/복귀 중앙값이 **152.10/377.85ms**, hwpspec 4열은
**318.55/191.25ms**다. 통합 중앙값 하나만으로 왕복 체감을 설명하지 않는다. 각 방향 10회, variance·
frame interval·inclusive counters·scope·실제 y는 raw JSON과 재집계 스크립트에 보존했다.

시험지 4열의 마지막 active 픽셀은 **39,090,060px**, hwpspec 4열은 **7,838,640px**다.
같은 문서의 1열 측정 종료에서는 시험지 **49,503,546px 중 idle pool 10,726,560px**, hwpspec
**16,063,224px 중 idle pool 5,363,280px**가 잡혔다. 이는 해당 테스트 순서 끝의 snapshot이며
독립 cold-start 메모리 비교나 최대 peak 메모리 수치가 아니다.

### 문서·줌·이미지 확인

계획의 6개 실제 fixture는 현재 WASM에서 각각 **20 / 178 / 77 / 1 / 4 / 21쪽**으로 열렸다.
KTX는 main/background/behind/front **실제 Canvas 4장**, 21쪽 실문서의 첫 4쪽은 각 2-layer이며
새 문서 scope에서 image prefetch **decoded** 상태가 관측됐다. KTX는 이 조건에서 비동기 이미지
작업 없는 `none` 경로이므로 KTX만으로 image decode를 검증했다고 주장하지 않는다.

KTX의 50→34 / 34→50 / 50→100% 줌을 각각 한 번 관찰했다. 네 지표는 raw JSON의
`preview / focusedSharp / visibleStable / retainedComplete`로 분리된다. 예를 들어 50→34%는
**13.0 / 134.3 / 134.3 / 134.3ms**, 50→100%는 **13.9 / 162.8 / 162.8 / 162.8ms**였다.
이는 완료 경계 확인을 위한 n=1 smoke이며 배율별 성능 비교·개선률이 아니다. 한 쪽이며 비동기 이미지가
없어서 후반 세 경계가 같은 rAF에서 관측된 것이다.

- [KTX 처음 열기](assets/issue6042/ktx-loaded.json)
- [KTX 50→34%](assets/issue6042/ktx-zoom-50-to-34.json), [34→50%](assets/issue6042/ktx-zoom-34-to-50.json),
  [50→100%](assets/issue6042/ktx-zoom-50-to-100.json)
- [77쪽 문서](assets/issue6042/kps-ai-loaded.json), [21쪽 다층·decode 확인](assets/issue6042/multi-layer-loaded.json)
- [21쪽 다층 100→34% 줌](assets/issue6042/multi-layer-zoom-100-to-34.json): preview 14.6ms,
  focused-sharp/visible-stable 184.8ms, retained-complete 263.1ms. 기존 4쪽은 cached, 새 prefetch 2쪽은
  decoded를 관측했다. 이 역시 n=1 완료 계약 확인이며 성능 개선 자료가 아니다.
- [178쪽 두 쪽 보기 JPEG](assets/issue6042/hwpspec-double-50.jpg): 배치 확인용, 화질 판정용 아님.

### 해석과 다음 단계의 수용 기준 제안

1. 20쪽·178쪽 모두 한 interaction에 같은 선형 조회가 **2회**, 조사량은 각각 **40·356 page**다.
   두 조회를 한 snapshot으로 묶을 여지는 분명하다. 그러나 이 환경의 조회 CPU는 대부분 0.1ms 이하다.
   0ms 기록은 시간 분해능 미만이지 무료라는 뜻이 아니다. **행 인덱스만으로 체감 속도 향상을 약속하지 않는다.**
2. 주 비용은 main 및 layer raster다. 새 visible뿐 아니라 `refreshRenderSurfacePlan(true)`의 기존
   Canvas 재렌더와 한 idle callback 안의 여러 prefetch render가 포함된다. Stage 4에서는 두 경로를
   함께 다뤄야 하며, 상위 inclusive time과 하위 WASM time을 더해 비용을 과장하지 않는다.
3. 시험지 4열 저배율처럼 active surface가 이미 40M 근처인 경우 여유 LRU는 작다. 반면 hwpspec
   4열은 표면 비용 여유가 크다. **문서별 LRU 효익을 분리**하고 cache 때문에 DPR을 더 내리지 않는다.
   같은 배율에서도 visible↔prefetch의 실제 DPR이 달라지면 정확한 cache key는 miss가 정상이다.
4. idle pool이 보유한 픽셀은 기존 planner 예산 밖에 남을 수 있다. Stage 3 ledger는 main·모든 layer·
   detached LRU·idle pool·예약을 중복 없이 합산해야 한다. DOM에서 제거됐다고 메모리 0으로 보고하지 않는다.
5. 다음 반복 비교의 **제안 경보선**은 중앙값 `max(10ms, before의 5%)`, p95 `max(25ms, before의 10%)`
   증가다. 초과하면 동일 조건 교대 반복으로 원인을 조사한다. 이는 통계적 유의성 선언이나 그 이하 회귀를
   허용한다는 뜻이 아니다. 위치·화질·호출 계약 회귀는 시간과 무관하게 실패다. Stage 2 승인 때 함께 확정한다.
6. 4ms visible slice / 최대 2 page / idle 1 page는 계속 **실험 후보**다. 이미 한 page WASM 호출만으로
   수십 ms를 쓰므로 4ms hard cap을 지킬 수 없다. 이 단계에서 값을 제품 상수로 도입하지 않았다.
   Stage 4에서는 page 경계 양보·최신 visible 진행성을 먼저 검증하고, 한 page 내부 지연은 별도 한계로 남긴다.

## 5. 아직 통과를 주장하지 않는 범위

### 문서 교체 page-info 경고: 원본에서도 재현

관찰기 내부 `errors=[]`는 브라우저 콘솔 전체가 깨끗하다는 뜻이 아니다. 별도로 콘솔을 읽으니
문서 교체 중 `[CanvasView] 페이지 N 정보가 없습니다`가 확인됐다. 다음처럼 원인을 분리했다.

- **완전 무수정 #6467, 4188 서버**: exam_kor 20쪽에서 `파일 → 새로 만들기`로 바꾸자 0·1쪽에
  같은 error 로그가 발생했다. [원본 서버 로그](assets/issue6042/document-reset-console-unmodified.json).
- **관찰 wrapper off**: KTX → hwpspec → 4쪽 문서 교체에서도 같은 계열 로그를 확인했다.
  [off 로그](assets/issue6042/document-switch-console-off.json), [on 로그](assets/issue6042/document-switch-console-on.json).
- 최종 on의 178쪽 focus index 5 → 4쪽 교체는 focus 0·새 scope·새 viewport로 정상 종료했다.
  [교체 snapshot](assets/issue6042/document-switch-178-to-4.json). 이것이 기존 중간 error를 해결했다는 뜻은 아니다.

`renderCanvas()`는 WASM 예외가 아니라 `this.pages[pageIdx]`가 없을 때 위 로그를 낸다. 소스상
`reset()`은 pages를 비우지만 VirtualScroll의 이전 geometry와 ViewportManager의 scroll rAF를 함께
무효화하지 않는다. placeholder가 scroll 영역을 줄이면서 생기는 scroll 갱신은 이전 가시 page를
계산할 수 있다. **원본에도 존재하는 문서 수명 경계 문제로 분리**하며, 정확한 이벤트 interleaving과
empty snapshot 계약을 Stage 2 실행 기반 회귀 테스트에 포함한다. 이번 관찰 단계에서 제품을 고치지는 않았다.

### 후속 검증 경계

- DPR 1, CanvasKit·auto fallback, 네이티브 휠/트랙패드 연속 입력, 가로 이동·resize·맞쪽 경계의
  전체 실문서 A/B는 Stage 5의 미완료 검증 항목이다. 현재 시간 자료는 setter가 기존 native scroll
  event/rAF 경로를 타는 통제 시나리오이며 사람의 휠 입력 지연 자료가 아니다.
- 실제 backend fallback 주입, 무응답 decoder/취소 후 callback, 편집·undo·redo·dispose의 **전체 실제
  renderer 통합** 검증은 Stage 3~4에서 cache 소유권·scheduler와 함께 수행한다. 여기서 음성 조건은
  관찰 helper·Promise/세대 계약 테스트로 확인했으며, 그 사실을 backend 전체 검증으로 확대하지 않는다.
- 최종 완료 값은 알려진 작업의 경계다. compositor presentation·실측 dropped frames·최종 화질의 자동
  판정은 아니다. screenshot은 실문서가 열린 상태의 시각 증거이며 원본/수정본 픽셀 동등성을 대체하지 않는다.
  Browser 캡처는 실제 파일 형식을 확인한 **1280×720 JPEG**로, 배치 확인에만 사용한다. Stage 5의
  화질 비교에 필요한 무손실 PNG를 확보한 것으로 보고하지 않는다.
- 제품 최적화, 새 PR·push, #6467 Ready 전환, #6444 처리, merge는 수행하지 않았다.

## 6. 최종 검증과 인계

- 집중 Node 테스트 **98/98 통과**, `tsc --noEmit` 통과, production build 통과.
  [실행 로그](assets/issue6042/validation.log)(줄끝 공백만 정리). build의 CanvasKit `fs/path` externalization·chunk-size
  경고는 기록했으며, 성공 종료를 경고가 없다는 뜻으로 보고하지 않는다.
- production JS에서 `scrollProbe`·`page-scroll-probe`·관찰 UI 문자열이 검출되지 않았다.
  기준선 worktree의 기존 build와 이번 build의 JS/CSS/WASM asset SHA-256도 모두 일치했다.
  [asset별 비교](assets/issue6042/production-assets-parity.json). 이는 runtime asset 대조이며 PWA manifest
  전체·서버 설정까지 비교했다는 뜻은 아니다.
- 제품 view/Rust 파일 변경 0. 전체 Studio suite·Rust lint는 이번 단계에서 실행하지 않았으며
  Stage 6 제출 게이트를 통과한 것으로 보고하지 않는다.
- `git diff --check`, 문서 상대 링크, fixture/source hash, fixed-base ancestry와 변경 범위를 확인한다.
  원본·관찰기 오류 자료·최종 표본을 구분했고, JSON 재집계 스크립트와 환경 hash를 함께 남긴다.

다음 승인은 **Stage 2: 기존 좌표 의미를 보존하는 행/X 인덱스·visibility snapshot 및 빈 문서 전환 경계**다.
위 시간 경보선도 함께 검토하되 4ms scheduler 후보는 아직 채택하지 않는다. Stage 3 LRU나 Stage 4
scheduler를 미리 구현하지 않고, push·PR 생성·Ready 전환 없이 여기서 멈춘다.
