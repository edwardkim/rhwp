# #6040 Stage 1.3 — stack 병합 후 줌 기준선

- 일자: 2026-09-07 KST
- 제품 기준: `upstream/devel@56706247f4950286117496c41f5b2c4b1cdbddc5`
- 브랜치: `codex/issue-6040-post-stack-zoom`
- 계획 commit: `d48bdd550`
- 상태: **자동 기준선·사용자 수동 검증 통과, 새 Stage 2 진행 승인**
- [수행 계획](../plans/task_m100_6040.md), [구현 계획](../plans/task_m100_6040_impl.md)

## 결론

새 제품 개선은 아직 구현하지 않았다. #6458·#6467·#6637이 병합된 제품 그대로 기존 줌 경로를
확인했고, DEV 패널에 복원 가능한 네 관찰 경계만 추가했다.

1. 4개 실문서의 `100→50→34→100%` 12개 전환이 모두 관찰 가능한 렌더 완료에 도달했다.
   timeout/interrupted와 관찰 오류는 0건이다.
2. 각 전환은 `N`회 zoom/layout/dimensions 갱신과 `2×(N−1)`회 preview를 기록했다.
   animation의 같은 preview가 `recalcLayout()`과 `onZoomChanged()`에서 두 번 실행되는 경로와 일치한다.
3. **모든 전환에서 전체 surface 해제가 1회 발생**했다. 첫 새 main raster는 전부 이 해제 이후였다.
   animation 중 최종 품질 raster를 새로 시작하지 않는 기존 분기를 유지하고 있다.
4. 줌 정착은 #6042 scroll용 visible 분할 경로가 아니다. `zoom-settled`에서는 visible을 동기 렌더하고
   인접 prefetch만 scheduler에 전달한다. 따라서 scheduler 도입만으로 줌 정착 비용이 해결된 것은 아니다.
5. 이 표본의 전체 `recalcLayout` 합은 전환당 0.5~1.8ms였다. main raster의 합은 19.4~463.3ms였다.
   시간은 장치 부하가 통제되지 않은 진단값이므로 개선율·성능 수용 근거로 쓰지 않는다. 다만 기존
   [#6454 STOP 결론](https://github.com/edwardkim/rhwp/issues/6454#issuecomment-5503939025)을 뒤집어
   범용 공유 좌표계가 필요하다고 주장할 근거도 이번에 얻지 못했다.

## 코드 변경 경계

- `rhwp-studio/src/dev/page-scroll-probe.ts`: `geometry.layout`, `geometry.dimensions`,
  `geometry.preview`, `page.releaseAll` 네 기존 메서드 wrapper 등록만 추가했다.
- `CanvasView`, `VirtualScroll`, `ViewportManager`, DPR·budget·LRU·scheduler 제품 코드는 변경하지 않았다.
- 새 Worker·queue·timer·DOM readback은 추가하지 않았다. `scrollProbe=1`을 주지 않으면 관찰기는
  설치되지 않는다. production build에서 패널과 새 boundary 문자열이 없는 것도 확인했다.
- 관찰 off 이후에도 KTX의 50% 줌이 정상 완료되고 observer 오류가 없었다. prototype descriptor,
  this/인수/반환/Promise/예외·wrapper 복원 계약은 기존 관찰기 테스트 12건으로 확인했다.

## 환경과 재현

- macOS 26.5.2 arm64, Node 24.15.0, Rust 1.93.1
- Codex in-app Chromium 152, 1280×720 CSS px, 실제 DPR 2, 자동 배치, Canvas2D
- KTX의 실제 Canvas cache key `backend:canvas2d`, `layers:4`와 main/background/behind/front
  네 backing surface를 확인했다. 사용한 파일은 **`samples/basic/KTX.hwp` 1쪽**이며 동명의 다른 파일과
  혼용하지 않는다.
- Docker daemon이 꺼져 있어 다음 native 진단 경로로 정확한 제품 base의 WASM을 새로 빌드했다.

  ```bash
  CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg --no-opt
  npm ci --prefix rhwp-studio --no-audit --no-fund
  npm --prefix rhwp-studio run dev -- --host 127.0.0.1 --port 4200 --strictPort
  ```

- Rust release는 최적화됐지만 `wasm-opt` 후처리는 없는 빌드다. 배포판과의 성능 비교 자료가 아니다.
- [개발 패널 안내](../manual/studio_scroll_probe_guide.md)의 `scrollProbe=1` → 실문서 열기 → 자동 →
  100%에서 시작 → 기록 초기화 → 50% → 관찰 결과 → JSON 저장 → 같은 방식 34%, 100%로 반복했다.
- 문서마다 첫 load 완료를 기다렸고 시간 관찰과 screenshot을 분리했다. warm-up·반복 통계·계측 on/off
  시간 calibration은 이번 단계에서 수행하지 않았다. 다른 사용자 작업의 Rust 빌드가 같은 장치에서
  실행 중이었으므로 실제 frame/완료 시간은 채택 판단에 사용하지 않는다.

## 호출 수 기준선

괄호 없는 세 숫자는 각각 `100→50 / 50→34 / 34→100%` 순서다. zoom event에는 최종 settled 1회가
포함돼 있다. main raster 수에는 새 visible과 인접 prefetch, 별도 surface scale 변경이 포함될 수 있어
visible 페이지 수와 동일하다고 해석하지 않는다. 이미지 후속 raster는 별도 counter다.

| fixture | 쪽 수 | 도착 열 수 | zoom/layout 횟수 | preview 횟수 | 전체 해제 | main raster 횟수 |
| --- | ---: | --- | --- | --- | --- | --- |
| 4쪽 실문서 | 4 | 3 / 4 / 1 | 12 / 9 / 12 | 22 / 16 / 22 | 1 / 1 / 1 | 4 / 4 / 3 |
| `exam_kor.hwp` | 20 | 2 / 3 / 1 | 12 / 9 / 12 | 22 / 16 / 22 | 1 / 1 / 1 | 4 / 9 / 4 |
| `kps-ai.hwp` | 77 | 3 / 4 / 1 | 7 / 9 / 12 | 12 / 16 / 22 | 1 / 1 / 1 | 6 / 12 / 3 |
| `basic/KTX.hwp` | 1 | 1 / 1 / 1 | 12 / 9 / 12 | 22 / 16 / 22 | 1 / 1 / 1 | 1 / 1 / 1 |

각 event의 횟수는 rAF cadence/CPU 부하에 따라 달라질 수 있다. 이 결과는 7 또는 12프레임을 제품
불변 상수로 고정하라는 뜻이 아니라, **실제로 발행된 N번에 대한 1:1 layout·2회 preview·최종 1회
release 관계**를 보여준다. 전체 해제의 자체 시간만 큰 병목이라는 뜻도 아니다. 그 뒤의 cache miss와
visible 동기 raster를 포함한 수명 경로가 후속 조사 대상이다.

## 증거와 한계

[최소 기준선 JSON — 로컬 보존 안내](assets/issue6040-post-stack/README.md)에 12개 표본의 모든 counter·milestone·rAF
표본, zoom/release/raster 관련 span, 오류·queue, surface와 최종 DOM box를 보존했다. 중간 화면 PNG와
일회성 서버 로그는 추가하지 않았다. 원시 기록의 `spansDropped`·`framesDropped`는 모두 0이다.
JSON은 관련 span만 선별 보존했으므로 모든 nested call을 담은 전체 profiler trace는 아니다.

- `visibleStable`·`retainedComplete`는 관찰 가능한 작업 완료이고 compositor 표시 완료가 아니다.
- 자동 배치에서 34→100% 뒤 편집 쪽이 viewport 밖이면 `focusedSharp`가 없는 것이 올바르다.
  focus 쪽으로 강제로 이동시키지 않았다.
- 4쪽 34%, kps-ai 34%, KTX 34%의 실제 screenshot을 열어 점유 묶음·문서·layer 합성을 확인했다.
  animation의 좌우/상하 흔들림과 눈금자 동기화는 이후 사용자 연속 입력 검증을 통과했다.
  별도로 관찰된 가로 스크롤 눈금자 지연은 아래 #6821에서 조사하며, 지연이 없었다고 기록하지 않는다.
- 기존 27%·17%의 actual-page cap, 50% 위 두 열 선택, anchor/resize/horizontal pan 계약은 focused
  테스트로 확인했다. 모든 고정 배치의 browser matrix·CanvasKit smoke는 Stage 4에서 수행하며,
  이번 12개 자동/Canvas2D 전환으로 대체하지 않는다.
- bundled font tree와 projection hash는 남겼지만 외부 webfont response bytes는 동결하지 않았다.
  한컴 typography fidelity나 구 배포판 대비 선명도 우열을 주장하지 않는다.

## 실행한 로컬 검증

| 검증 | 결과 |
| --- | --- |
| `virtual-scroll-page-arrangement`, `canvas-view-page-arrangement`, `viewport-manager-smooth-zoom`, `zoom-anchor`, `ruler-scale`, `ruler-label-geometry`, `virtual-scroll-horizontal-pan`, `zoom-fit` | 60/60 통과 |
| `scroll-observation.test.ts` | 12/12 통과 |
| `npm run build` (Studio) | TypeScript + Vite 253 modules + PWA build 통과 |
| 정확한 source native WASM | release + `--no-opt`, 3m 53s, 성공 |
| 실제 browser | 4개 문서 × 3개 전환, 12/12 complete, observer errors 0 |
| `git diff --check` | 통과 |

Stage 1.3의 source 변경은 DEV boundary 등록뿐이다. Studio 전체 test·Rust 전체 lint/회귀·전체 visual
matrix는 실행하지 않았고, 이 기록은 PR 제출 전체 gate 통과 선언이 아니다.

## 사용자 확인과 다음 gate

계측 없는 [로컬 exam_kor 기준선](http://127.0.0.1:4200/?renderer=canvas2d&url=/samples/exam_kor.hwp)을
제공한다. 서버는 정확한 제품 base + 비활성 DEV 계측 코드이며 새 줌 최적화 후보가 아니다.

1. 쪽 모양 `자동`에서 100% 부근부터 트랙패드/기존 wheel 줌으로 천천히 50%·34%까지 내리고 다시
   올린다. 화면 이동이 사용자가 승인했던 병합 버전과 같은지, 눈금자가 쪽과 동시에 움직이는지 확인한다.
2. 슬라이더로 빠르게 확대/축소 방향을 번갈아 바꾼다. 정착 후 쪽이 예상 밖으로 좌우·상하 점프하거나
   쪽/레이어가 빈 상태로 남지 않는지 확인한다. 자동 열 재배치 자체는 기존 동작이므로 결함으로 보지 않는다.
3. 문서 중간으로 스크롤한 뒤 잠시 정지하고 클릭하지 않아도 화질이 회복되는지, 클릭한 곳에 캐럿과
   선택 영역이 맞는지 확인한다. 편집 커서 쪽이 항상 중앙에 남아야 한다는 새 조건은 추가하지 않는다.

2026-09-07 작업지시자가 위 수동 검증에 대해 "전부 통과했어"라고 확인하고, 별도 눈금자 이슈 등록·연결
후 #6040 계획대로 진행을 승인했다. 기준선 진단 head는 `9762ea755`다.

### 별도 관찰 — #6821

- 가로 트랙패드 스크롤에서 눈금자가 페이지를 미세하게 뒤따른다는 사용자 관찰을
  [#6821](https://github.com/edwardkim/rhwp/issues/6821)에 등록했다.
- 세로도 같은 예약 경로를 사용하므로 조사 범위에 포함하지만 동일 현상이 확인된 것은 아니다.
- 원인 후보는 `ViewportManager.onScroll`의 rAF 알림과 `Ruler.scheduleUpdate`의 추가 rAF다.
  실제 지연량·compositor 표시 시각·다른 렌더 작업 영향은 계측하지 않았다.
- 이 예약 구조는 연작 이전 코드에도 있다. 이번 DEV 관찰 변경이 만든 회귀로 분류하지 않는다.
- #6040의 선행 필수 작업으로 두지 않는다. 새 Stage 2에서는 눈금자 예약 경로를 변경하지 않고,
  기존 지연을 악화시키지 않는 회귀 검증을 유지한다.

다음 단계는 공통 좌표계를 건드리지 않는 중복 preview 제거 가능성과 결정적 테스트부터 검토한다.
효과가 작은 전체 geometry 개편은 보류하고, 새 Stage 3에서 정착 visible raster/surface 수명을
별도로 다룬다. Stage 2 결과 승인 없이 Stage 3까지 진행하지 않는다.

## 원격 상태

- #6040의 완료/잔여 범위와 재개 단계 본문을 게시하고 API로 한글·내용·OPEN 상태를 확인했다.
- `zoom-settled`의 visible 동기 렌더 문구 추가 정정과 결과 알림 코멘트는 추가 승인 확인에서 잠시
  보류했다. 사용자에게 구체적 승인을 받아 본문을 정정하고
  [결과 코멘트](https://github.com/edwardkim/rhwp/issues/6040#issuecomment-5561853814)를 게시했다.
- push·PR 생성·Ready·merge·issue close는 수행하지 않았다.
- 사용자 승인에 따라 #6821을 생성하고 #6040 본문과
  [기준선 승인·연결 코멘트](https://github.com/edwardkim/rhwp/issues/6040#issuecomment-5561975759)를 갱신했다.
