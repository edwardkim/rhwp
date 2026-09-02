Closes #6042

이 PR은 GitHub native stack의 세 번째 PR입니다.

`#6458 (#6040) → #6467 (#6041) → 이 PR (#6042)`

리뷰는 #6467 대비 diff를 기준으로 부탁드립니다. 현재 bottom #6458은 최신 `devel`과 conflict가 있어,
전체 stack의 Ready 전환 전 bottom부터 restack하고 위 PR을 다시 쌓아 재검증할 예정입니다.

## 무엇을 바꾸나

다중 페이지 문서의 일반 스크롤에서 전체 페이지 판정과 연속 raster가 입력 callback을 오래 점유하던
구조를 다음 세 층으로 분리합니다.

1. 기존 page geometry로 row/X index와 immutable visibility snapshot을 만듭니다.
2. 같은 문서·revision·backend·실제 scale의 완성 main/overlay page bundle을 physical-pixel 예산 안에서
   exact LRU로 재사용합니다.
3. visible을 focused/viewport-center 순서의 page 단위 rAF slice로 처리하고, 선택 prefetch는 visible 완료
   뒤 idle callback당 한 쪽만 실행합니다. 새 입력은 generation으로 stale work를 기각합니다.

초기 표시, 1·2쪽 fast path, zoom-settled, resize, 편집/undo/redo, strict render는 기존 동기 의미를
유지합니다. #6040의 배치와 #6467의 32M/40M DPR·surface budget도 바꾸지 않습니다.

스크롤 중에는 현재 surface DPR을 고정해 재래스터를 피합니다. 마지막 입력 150ms 뒤 충분히 노출된 visible
쪽만 center-first로 화면 DPR까지 회복합니다. 전체 raw 비용이 64M surface px를 넘으면 DPR 1.5, 그것도
넘으면 #6467 planner 결과를 유지합니다. 이 64M은 cache 예산이 아니라 정착 visible 품질의 safety gate입니다.

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

## 범위 밖

장치 성능별 자동 확장, 짧은 문서 전체 surface 상주, worker/OffscreenCanvas, raster 내부 선점,
#6521의 저배율 DPR 정책, viewport를 편집 focus로 바꾸는 동작은 포함하지 않습니다.
