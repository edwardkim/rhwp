> **Stack 3/3** — base는 의도적으로 `codex/issue-6041-budget-first-render-scale`입니다.
>
> 1. #6458 (`devel` ← `codex/issue-6040-zoom-topology`)
> 2. #6467 (`codex/issue-6040-zoom-topology` ← `codex/issue-6041-budget-first-render-scale`)
> 3. **이 PR** (`codex/issue-6041-budget-first-render-scale` ← `codex/issue-6042-page-virtualization`)
>
> 세 PR은 GitHub native stack #6640으로 연결되어 있습니다. 리뷰는 #6467 대비 이 layer의 diff를 기준으로 부탁드립니다.

## 변경 요약

- 전체 페이지를 반복 조사하지 않고 기존 geometry에서 만든 row/X index와 immutable visibility snapshot을
  공유합니다.
- mandatory visible 쪽은 viewport 중심부터 page 단위 rAF slice로 처리하고, optional prefetch는 visible
  완료 뒤 idle에서 한 쪽씩 실행합니다. 새 입력은 generation으로 오래된 작업을 취소합니다.
- 같은 문서·revision·backend·scale의 완성 main/overlay page bundle을 physical-pixel 예산 안에서 exact
  LRU로 재사용합니다.
- 스크롤 중에는 현재 surface DPR을 유지하고, 입력 종료 150ms 뒤 충분히 노출된 쪽을 center-first로
  승격합니다. 64M surface-pixel gate를 넘으면 #6467 planner 결과를 유지합니다.
- 초기 표시, 1·2쪽 fast path, zoom·resize·편집·undo/redo·strict render의 기존 동기 계약과 #6040의 배치,
  #6467의 DPR/surface budget은 바꾸지 않습니다.

## 관련 이슈

Closes #6042

## 해결하려는 사용자 문제

#6467까지는 한 번에 유지할 Canvas의 physical-pixel 예산과 쪽별 DPR을 정하지만, **언제 어느 쪽을
판정·그림·재사용할지**는 기존 구조를 따릅니다. 페이지가 많은 문서를 저배율 다중 쪽 보기로 열고 새
구간으로 이동하면 다음 일이 scroll 입력과 같은 메인 스레드 구간에 연달아 일어날 수 있었습니다.

- 전체 페이지의 가시 영역을 반복해서 조사
- 새 visible 쪽 여러 장을 연속 raster
- 화면 밖 인접 쪽까지 한꺼번에 prefetch
- 방금 본 쪽을 분리한 뒤 다시 돌아왔을 때 같은 bitmap을 재생성

사용자에게는 긴 문서에서 scrollbar thumb를 크게 옮기거나 PageDown으로 새 행에 들어갈 때 첫 화면이
늦게 보이고, 다중 쪽을 빠르게 왕복할 때 스크롤이 잠깐 멈추는 현상으로 나타납니다. 반대로 작업을 너무
강하게 미루면 스크롤 후 보이는 비포커스 쪽이 클릭할 때까지 낮은 DPR로 남는 문제도 있습니다.

## 사용자가 보게 되는 변화

| 실제 행동 | 기존 | 변경 후 |
| --- | --- | --- |
| 178쪽 문서 34%·4열에서 scrollbar로 먼 행 이동 | visible 여러 쪽 raster가 scroll callback을 오래 점유 | viewport 중심 쪽부터 page 단위로 양보하며 표시 |
| 같은 구간을 위아래로 왕복 | 예산 안의 bitmap도 release/pool 경로에 따라 다시 raster 가능 | 문서·revision·backend·scale이 정확히 같으면 완성 page bundle을 LRU에서 재부착 |
| 빠르게 아래→위 방향 반전 | 이미 효용이 사라진 prefetch가 뒤늦게 실행될 수 있음 | 새 generation이 낡은 작업을 취소하고 진행 방향 후보만 제한적으로 prefetch |
| cursor가 없는 두 쪽으로 스크롤하고 멈춤 | #6467 예산 결과의 DPR 1이 클릭 전까지 유지될 수 있음 | 이동 중 surface는 유지하고 150ms 정착 뒤 클릭 없이 읽기 화질 회복 |
| zoom·resize·편집·문서 교체 | 기존 즉시 정합 계약 | 기존 동기 경로와 focus/caret/ruler 의미 유지 |

이 PR은 “모든 페이지를 미리 그려 영구 보존”하는 방식으로 돌아가지 않습니다. visible은 반드시 처리하고,
완성 bitmap 재사용과 선택 prefetch만 현재 physical-pixel headroom 안에서 수행합니다. 짧은 문서의
fully-warm 전체 상주나 장치 성능별 자동 확장은 별도 문제로 남깁니다.

## 주요 설계 결정

- **visibility 결과 공유:** geometry 산식은 바꾸지 않고 row/X index와 immutable snapshot만 추가해
  기존 AABB·gap·facing 의미를 보존했습니다.
- **page bundle 단위 exact cache:** main만 보존하지 않고 background/behind/front overlay와 image 완료
  상태까지 함께 소유합니다. 불완전 결과는 cache hit가 아닙니다.
- **visible 우선, optional prefetch 후순위:** mandatory visible은 예산 때문에 생략하지 않습니다. 예산을
  넘을 때 중단되는 것은 보존 이득이 없는 선택 prefetch뿐입니다.
- **page-boundary scheduling:** 한 쪽 raster 내부를 선점할 수 있다고 주장하지 않습니다. 한 쪽을 끝낸
  뒤 다음 쪽 전에 메인 스레드에 양보합니다.
- **scroll 중 품질 고정, 정착 뒤 승격:** 계속 움직이는 동안 DPR thrash를 피하고 사용자가 읽는 시점에
  화질을 복구합니다. 편집 focus를 viewport로 옮기지 않습니다.
- **고정된 안전 경계:** 장치 성능 추측이나 열 수 규칙을 넣지 않고 #6467의 32M/40M planner와 별도의
  64M settled-visible gate만 사용합니다. 출력/high-quality profile은 기존 raw 경로를 유지합니다.

## 성능 영향 및 측정 결과

Chromium 151, 1280×720 CSS px, 실제 DPR 2, Canvas2D에서 Stage 3과 A/B했습니다. p50/p95 회귀 경보선은
결과 확인 전에 `max(10ms, 5%)` / `max(25ms, 10%)`로 고정했습니다.

### 178쪽 `hwpspec.hwp`, 4열·34% cold jump, 20쌍

| 지표 p50 / p95 | before | after | 변화 |
| --- | ---: | ---: | ---: |
| 첫 visible | 271.2 / 278.9ms | 56.5 / 59.0ms | -79.2% / -78.8% |
| visible 전체 안정 | 271.2 / 278.9ms | 250.9 / 259.7ms | -7.5% / -6.9% |
| retained 전체 완료 | 377.0 / 396.3ms | 351.6 / 359.7ms | -6.7% / -9.2% |
| `visibility.update` | 243.8 / 247.7ms | 19.1 / 19.6ms | -92.2% / -92.1% |

main raster는 12회로 동일하고, 같은 일을 입력 callback 밖의 page slice로 옮겼습니다. 이 표본의 long
task는 40건·합계 6,887ms에서 0건으로 줄었습니다. 한 쪽 raster 자체는 선점할 수 없으므로 모든 장치의
compositor frame 개선으로 일반화하지 않습니다.

178쪽 warm 왕복은 16.9/17.7ms → 16.6/17.7ms이고 양쪽 모두 raster 0입니다. Stage 5에서 발견한
`exam_kor` 역방향 cache thrash는 target-state 예약과 prefetch gate로 수정해 before/after 모두 raster
3회, cache take 4회로 복구했습니다. retained 증가는 p50/p95 +4.2/+4.6ms로 사전 경보선 안입니다.

scroll-settled 화질 회복은 별도의 의도한 비용이 있습니다. 두 쪽·100%에서 scroll callback은 양쪽 모두
0/0.1ms지만, 입력 종료 뒤 DPR 1→2 raster 때문에 최종 known work는 p50/p95 33.3/54.1ms 늦어집니다.
64M gate로 이 비용과 메모리 상한을 제한합니다.

Canvas2D·CanvasKit 확장 matrix의 성능 표본은 revision당 260개였고 모두 complete/error 0입니다.
`exam_kor` 두 쪽·50% retained 완료는 331.6/342.8ms → 346.2/362.2ms로 p50/p95 +14.6/+19.4ms였지만,
결과 확인 전에 정한 경보선 +16.6/+34.3ms 안이었습니다. 첫 visible은 123.2/132.1ms → 63.3/71.2ms였습니다.
이 수치를 모든 두 쪽 문서의 일반 성능 배수로 주장하지 않습니다.

### 성능 수치 해석

- 첫 visible 감소는 사용자가 새 구간의 첫 내용을 더 빨리 보는 개선입니다.
- retained 완료는 background prefetch까지 포함하므로 첫 표시보다 늦을 수 있습니다.
- long-task 수는 PerformanceObserver 관찰값이며 compositor dropped frame 수가 아닙니다.
- scroll-settled DPR 승격은 최적화가 아니라 읽기 화질 복원을 위한 의도된 후처리 비용입니다.
- LRU는 메모리를 더 보존해 CPU 재작업을 줄이는 교환입니다. 전체 메모리 감소를 주장하지 않습니다.

자세한 판정과 원시는 [최종 보고](https://github.com/edwardkim/rhwp/blob/codex/issue-6042-page-virtualization/mydocs/report/task_m100_6042_report.md)와
`mydocs/working/assets/issue6042*`에 있습니다.

## 시각 비교

아래 이미지는 같은 viewport의 배치·누락 확인용입니다. JPEG pixel diff를 glyph 화질 정량값으로 쓰지
않았고, DOM·surface snapshot과 실제 DPR/physical dimension을 주 근거로 사용했습니다.

| Stage 3 before | scheduler 보정 after |
| --- | --- |
| ![Canvas2D auto 34% before](https://github.com/edwardkim/rhwp/blob/codex/issue-6042-page-virtualization/mydocs/working/assets/issue6042-stage5-expanded/canvas2d-exam-auto-34-stage3.jpg?raw=true) | ![Canvas2D auto 34% after](https://github.com/edwardkim/rhwp/blob/codex/issue-6042-page-virtualization/mydocs/working/assets/issue6042-stage5-expanded/canvas2d-exam-auto-34-corrected.jpg?raw=true) |

cursor가 없는 쪽으로 스크롤했을 때는 이동 중 surface를 유지하고, 정착 뒤 클릭 없이 읽기 화질을
회복합니다.

| 정착 전/기존 DPR 1 | 정착 후 DPR 2 |
| --- | --- |
| ![DPR 1](https://github.com/edwardkim/rhwp/blob/codex/issue-6042-page-virtualization/mydocs/working/assets/issue6042-stage5-scroll-quality/exam-double-100-before-dpr1.png?raw=true) | ![DPR 2](https://github.com/edwardkim/rhwp/blob/codex/issue-6042-page-virtualization/mydocs/working/assets/issue6042-stage5-scroll-quality/exam-double-100-after-dpr2.png?raw=true) |

## 검증

- [x] `npm test`: 1,422 total / 1,421 pass / 1 skip / 0 fail
- [x] `npx tsc --noEmit --pretty false`
- [x] `npm run build`
- [x] `npm run e2e:manifest-check`: 126/126
- [x] scheduler/LRU/CanvasView/budget/visibility/zoom 집중 suite: 71/71
- [x] 실제 DPR 1 Chrome ruler/viewport resize: 28/28 snapshots
- [x] browser image decode 실패 3회 → fallback render 3회, 잔여 queue/error 0
- [x] Canvas2D·CanvasKit·auto fallback, 34/50/100/200%, 가로·대면·마지막 행, 문서 교체,
  edit/undo/redo
- [x] Stage 집계기 4종, raw JSON parse, ancestry, `git diff --check`
- [x] Rust source/test/fixture/Cargo/CI 변경 없음 — Rust gate N/A

사용자가 Stage 3/현재 로컬 서버를 직접 비교했고, scroll-settled 화질 보정 뒤 앞서 제보한 버벅임이
사라진 것 같다고 확인했습니다. 이를 정량 개선률로 확대하지 않고 자동화 계측의 보조 근거로만 사용합니다.

## 리뷰 시작점

- visibility 계약: `rhwp-studio/src/view/virtual-scroll.ts`
- page bundle 소유·예산: `page-surface-lru.ts`, `canvas-pool.ts`, `canvas-view.ts`
- 실행 순서·취소: `page-render-scheduler.ts`
- DPR lock·settled-visible gate: `render-surface-budget.ts`, `canvas-view.ts`
- browser 수명 회귀: `e2e/page-virtualization-image-failure.test.mjs`, `e2e/ruler-resize.test.mjs`
- 최종 판정: `mydocs/report/task_m100_6042_report.md`

diff의 큰 부분은 반복 A/B raw evidence입니다. 리뷰 UI가 raw JSON을 접어도 `summary.json`과 위 제품
파일부터 보면 구조와 판정을 확인할 수 있습니다.

## 범위 밖

장치 성능별 자동 확장, 짧은 문서 전체 surface 상주, worker/OffscreenCanvas, raster 내부 선점,
#6521의 저배율 DPR 정책, viewport를 편집 focus로 바꾸는 동작은 포함하지 않습니다.
