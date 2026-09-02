# Task M100 #6042 처리 결과 — 다중 페이지 스크롤 가상화

- Issue: [#6042](https://github.com/edwardkim/rhwp/issues/6042)
- 완료일: 2026-09-02 KST
- 상태: **native stack #6640 등록 — cascading rebase·Full CI 대기**
- branch: `codex/issue-6042-page-virtualization`
- 직접 base: #6467 `23b5bcf73f6e8659a90b25ebfde1311e1965364f`
- 수행 계획: [task_m100_6042.md](../plans/task_m100_6042.md)
- 구현 계획: [task_m100_6042_impl.md](../plans/task_m100_6042_impl.md)
- 제출 감사: [Stage 6](../working/task_m100_6042_stage6.md)
- 제출 PR: [#6637](https://github.com/edwardkim/rhwp/pull/6637)
- self-review: [review](../pr/archives/pr_6637_review.md),
  [구현 검토](../pr/archives/pr_6637_review_impl.md)

## 1. 결과

다중 페이지 문서의 일반 스크롤에서 매번 전체 페이지를 선형 판정하고, 화면에 들어온 쪽과 인접 쪽을 한
callback 안에서 연속 raster하던 구조를 **공유 visibility snapshot + 완성 page surface LRU + page 단위
scheduler**로 바꿨다. 줌·resize·초기 표시·편집의 기존 동기 계약과 #6040/#6467의 배치·DPR 정책은
유지한다.

사용자가 체감하는 변화는 긴 문서에서 새 구간으로 이동할 때 첫 쪽이 더 빨리 나타나고, 같은 구간을
왕복할 때 완성 surface를 재사용하며, 여러 쪽 raster가 하나의 긴 scroll callback을 만들지 않는 것이다.
스크롤 중에는 이미 표시 중인 surface를 유지하고, 멈춘 뒤 읽을 visible 쪽은 메모리 안전 gate 안에서
원래 화면 DPR로 회복한다.

## 2. 구현

### visibility

`VirtualScroll`이 기존 page geometry에서 세로 row index와 가로 X index를 만들고,
`geometryRevision + viewport` 키의 immutable snapshot을 제공한다. 기존 AABB 교차, mixed-size, facing
표지 slot, 마지막 미완성 행, gap fallback과 current-page 의미는 바꾸지 않았다.

### surface lifetime

완료된 main·overlay canvas를 하나의 page bundle로 소유하는 exact-key LRU를 추가했다. key에는 문서·뷰
scope, page, backend, layer, 실제 render scale이 포함된다. active, detached LRU, anonymous pool,
pending reservation의 physical pixel을 따로 추적하고 image/RawSvg 후속 작업이 끝나지 않은 bundle은
cache hit로 인정하지 않는다.

### scheduling

일반 scroll은 focused/viewport-center visible을 먼저 page 단위 rAF slice로 처리하고, 선택 prefetch는
visible이 끝난 뒤 idle callback당 한 쪽만 처리한다. 새 interaction은 generation으로 stale work를
기각하고 방향·속도는 prefetch 우선순위에만 사용한다. 1·2쪽 initial/strict/zoom-settled/resize/mutation은
기존 동기 fast path를 유지한다.

Stage 5에서 active DPR 전환이 끝나기 전에 old surface 크기로 LRU를 trim해 역방향 cache thrash가 생기는
것을 발견했다. active target-state를 먼저 예약하고, 실제 retained 여유가 없는 선택 prefetch를 dispatch
전에 거르는 보정으로 exact cache 복귀를 되살렸다.

### scroll-settled quality

active scroll은 DOM에 실제로 붙은 surface DPR을 interaction lock으로 사용해 이동 중 재래스터를 막는다.
마지막 입력 150ms 뒤 양 축 8 CSS px 이상 노출된 visible 집합을 center-first로 승격한다. 전체 raw 비용이
64M surface pixel 이하면 화면 DPR, 초과하면 DPR 1.5를 시도하고, 그것도 초과하면 #6467 planner 결과를
유지한다. focus/caret/ruler 의미는 바꾸지 않았다. fractional DPR에서도 main·overlay CSS 크기는 항상
`PageInfo × zoom` 논리 geometry를 사용한다.

## 3. 성능 결과

주 수치는 Chromium 151, 1280×720 CSS px, 실제 DPR 2, Canvas2D의 로컬 A/B다. 경보선은 결과 확인 전에
p50 `max(10ms, 5%)`, p95 `max(25ms, 10%)`로 고정했다.

### 178쪽 `hwpspec.hwp`, 4열·34% cold jump, 20쌍

| 지표 p50 / p95 | Stage 3 | scheduler | 변화 |
| --- | ---: | ---: | ---: |
| known work next frame | 390.3 / 400.7ms | 352.2 / 360.2ms | -9.8% / -10.1% |
| 첫 visible | 271.2 / 278.9ms | 56.5 / 59.0ms | -79.2% / -78.8% |
| visible 전체 안정 | 271.2 / 278.9ms | 250.9 / 259.7ms | -7.5% / -6.9% |
| retained 전체 완료 | 377.0 / 396.3ms | 351.6 / 359.7ms | -6.7% / -9.2% |
| `visibility.update` | 243.8 / 247.7ms | 19.1 / 19.6ms | -92.2% / -92.1% |

main raster는 양쪽 모두 12회다. scheduler는 작업량을 숨기지 않고 callback 밖의 page slice로 옮겼다.
long task는 Stage 3의 40건·합계 6,887ms에서 scheduler의 0건으로 줄었다. 이 브라우저 표본을 모든 장치의
compositor dropped frame 개선으로 일반화하지 않는다.

### warm·회귀 보정

- 178쪽 warm 왕복 retained p50/p95: 16.9/17.7ms → 16.6/17.7ms, main raster 0/0.
- `exam_kor.hwp` 역방향: Stage 3과 보정 모두 main raster 3회, cache take 4회.
- 역방향 retained p50/p95: 221.2/233.6ms → 225.4/238.2ms. +4.2/+4.6ms로 경보선 안이다.
- Canvas2D·CanvasKit 자동화 성능 표본은 revision당 260개, 모두 complete/error 0이며 사전 경보선 안이다.

### 정착 화질 비용

`exam_kor.hwp` 두 쪽·100%의 scroll callback p50/p95는 before와 after 모두 0/0.1ms다. 클릭 없이 visible
DPR은 1→2로 회복하지만, 그 raster 때문에 정착 포함 known work는 332.3/349.4ms에서 365.6/403.5ms로
33.3/54.1ms 늦어진다. visible 두 쪽 surface는 10,699,944→42,786,300px로 늘었다. 이는 scroll hot-path
회귀가 아니라 입력 종료 뒤 읽기 화질을 회복하기 위해 명시적으로 지불하는 비용이며 64M gate로 제한한다.

## 4. 정확성·시각·수명 결과

- Canvas2D, CanvasKit, auto→Canvas2D fallback 통과.
- 자동 1↔2↔3열, 고정 한 쪽·두 쪽·4열, facing, 가로 이동, 마지막 미완성 행 통과.
- 34/50/100/200% zoom, 빠른 방향 반전, 문서 교체, 편집·undo·redo 통과.
- 실제 DPR 1에서 28개 viewport resize snapshot과 ruler/grid/overflow 정합 통과.
- 실제 browser image decode 3회 실패 주입 뒤 fallback render 3회, 완료 서명·queue·error 잔류 0.
- 사용자가 수정 전후 로컬 서버를 직접 조작해 scroll-settled 화질 보정 뒤 앞서 제보한 버벅임이 사라진
  것 같다고 확인했다. 이를 정량 성능 개선률로 확대 해석하지 않고 자동화 결과의 보조 근거로만 사용한다.

자동 34% Canvas2D와 KTX 4-layer CanvasKit screenshot의 차이는 probe 숫자·ruler/footer를 중심으로
0.0592%, 0.1955%였다. JPEG 차분은 glyph 화질 정량값이 아니라 큰 배치 이동·누락이 없다는 보조 자료다.
주 근거는 동일 viewport의 DOM·surface snapshot과 integer physical dimension이다.

## 5. 검증

최종 `4ea694ff3`에서 다음을 통과했다.

- Studio: 1,422 total / 1,421 pass / 1 skip / 0 fail
- TypeScript `--noEmit`, Vite production build
- E2E manifest: 126/126
- 핵심 scheduler·LRU·CanvasView·budget·visibility·zoom suite: 71/71
- headless Chrome DPR 1 ruler resize: 28/28
- headless Chrome image failure/fallback
- Stage 집계기 4종, 모든 `issue6042*` JSON parse, ancestry, `git diff --check`

Rust source/test/fixture와 CI 설정은 바꾸지 않아 Rust lint bundle 대상이 아니다. 자세한 명령과 범위 감사는
[Stage 6](../working/task_m100_6042_stage6.md)에 있다.

## 6. 한계와 후속 경계

한 쪽 raster는 여전히 선점할 수 없다. 장치 성능별 적응, 짧은 문서 전체 surface 상주, worker/
OffscreenCanvas는 검토했지만 사용자 지시에 따라 포함하지 않았다. #6521의 저배율 화질 저하안도 폐기
상태를 유지한다.

원격 #6458 → #6467 → #6637은 trunk `devel`의 native stack #6640으로 등록됐다. 현재 #6458은 최신
`devel`과 conflict이므로 전체 stack을 Ready로 바꾸기 전에 cascading rebase로 선형성을 회복해야 한다.
native stack에서는 중간·상단 PR도 trunk 기준 protection과 Actions를 적용받으며, top 후속 push에서
실제 CI가 시작됐다. cascading rebase 뒤 갱신된 각 exact head의 Full CI, Ready, merge는 후속 경계다.

## 7. 근거 문서

- [Stage 2 — visibility index](../working/task_m100_6042_stage2.md)
- [Stage 3 — surface LRU](../working/task_m100_6042_stage3.md)
- [Stage 4 — scheduler](../working/task_m100_6042_stage4.md)
- [Stage 5 실패와 회귀 포착](../working/task_m100_6042_stage5.md)
- [Stage 4 보정](../working/task_m100_6042_stage4_correction.md)
- [Stage 5 확장 matrix](../working/task_m100_6042_stage5_expanded.md)
- [scroll-settled 화질 보정](../working/task_m100_6042_stage5_scroll_quality_correction.md)
- [Stage 5 종료](../working/task_m100_6042_stage5_complete.md)
