# #6040 새 Stage 3.1 — 연속 핀치 계측과 실제 입력 기준선

- 일자: 2026-09-07~08 KST
- 상태: **느린·빠른·역방향 사용자 핀치 분석 완료. 화면 폭·입력 패턴이 달라 통제 비교는 아님. Stage 3.2A 제품 구현 승인 전**
- 제품 head: `f0cddc052`, 선행 계획 commit: `bed464a18`
- [수행 계획](../plans/task_m100_6040.md), [구현 계획](../plans/task_m100_6040_impl.md)

## 승인과 범위

사용자는 Stage 2가 이전 기준선보다 악화되지는 않았지만, 페이지가 많은 문서에서 핀치 도중과
뗀 직후 끊김·페이지 지연 등장이 남는다고 확인했다. 연속 핀치 계측 → 기존 화면 유지·페이지별 교체
→ 오래된 배율 작업 취소 순서를 우선하기로 했다. 저해상도 선렌더와 별도 thumbnail cache는 보류한다.

이번 구현은 DEV 계측뿐이다. ViewportManager/CanvasView/VirtualScroll/Ruler/PageRenderer와 제품
예산·화질·scheduler는 변경하지 않았다. Stage 2의 CanvasView와 WASM hash는 그대로다.

- CanvasView SHA-256: `d42b846764819ce158a1c8ceaaf87d1c69fad20df4dd00d7359b4b7d0e585595`
- WASM SHA-256: `24d3d2ffe0c1b43d4f53c23762820112b8298c2a081c9dd4842406819f4130fc`

이 브랜치는 `56706247f` 제품 기준을 유지한다. 로컬 upstream ref에는 이후 #6788/#6814의
`08bf41c69`가 추가됐지만 진단 중 기준선을 임의로 교체하지 않았다. 최종 제품 제출 전 최신 devel
정합과 exact-head 검증은 별도 gate다.

## 구현

- `ZoomSessionObservation`: DOM 없는 수동 구간 원장. wheel 입력마다 끊기는 기존 `ScrollObservation`
  trace와 분리한다. 입력, 실제 관찰 메서드 span, rAF 간격, 지원 시 long task를 같은 시간축에 남긴다.
- 패널에 **연속 핀치 시작 / 종료**를 추가했다. `scrollProbe=1`과 DEV일 때만 설치되며 수동 기록은
  최대 20초다. 실제 손가락 종료를 추정하는 timer나 제품 debounce는 추가하지 않았다.
- 입력·span 각각 8,192개, frame 2,048개, long task 256개 상한과 누락 counter를 둔다.
  정상 제품 경로에 DOM scan·JSON 직렬화·새 Worker·새 raster 작업은 추가하지 않았다.
- 관찰 off/문서·content 전환/dispose에서는 중단한다. 수동 종료 상태와 known-work readiness를
  분리하고, 종료 버튼을 다시 눌러도 첫 종료 판정을 덮어쓰지 않는다.
- [사용 안내](../manual/studio_scroll_probe_guide.md#연속-트랙패드-핀치-수집-6040)에 수집·해석 절차를 추가했다.

## 검증

| 검증 | 결과 |
| --- | --- |
| 신규 원장 + 기존 observer unit | 16/16 통과 |
| Studio 전체 `npm test` | 1,507건: 1,506 pass / 1 skip / 0 fail |
| TypeScript / production build | 통과, Vite 253 modules 및 PWA 생성 |
| 실제 브라우저의 제어된 ctrl-wheel 두 입력 | 한 수동 구간에 입력 2건·zoom.smooth 2회 보존, 누락 0·관찰 오류 0 |
| 애니메이션 중 수동 종료 | `stopped`, `knownWorkReadyAtStop=false`로 구분 |
| 관찰 off | `interrupted` / `observation toggled`, on 재설치 성공 |
| 자동 배치 100→50% 버튼 smoke | `stopped`, readiness true, animation/settled 양쪽 span·release 1회·main raster 4회, 누락 0·오류 0 |

브라우저 자동화가 만든 ctrl-wheel은 물리 트랙패드가 아니다. 위 결과는 계측 배선 확인이며 사용자
증상의 재현이나 시간 개선 증거가 아니다. 시간·스크린샷을 새 성능 baseline으로 커밋하지 않았다.
개발 서버 연결이 중간에 끊겨 재시작했으며, 이전 연결 오류 탭의 재탐색도 실패해 새 검증 탭을 사용했다.
소스 HMR 직후 checkbox smoke 한 번이 실패했고, 새 DOM의 초기화 완료 확인 후 동일 UI 동작은 통과했다.

## 실제 사용자 핀치 — 2026-09-07

사용자가 직접 트랙패드로 조작하고 수동 종료한 패널 JSON을 화면 변경·재시작 없이 읽었다.
사용자는 한 번에 목표까지 축소할 수 없어 **물리 핀치를 두 번으로 나눴다**고 설명했다.
첫 핀치와 둘째 핀치 사이의 정상 정착을 오류로 세지 않는다.

- 계측 head: `432e5c59c`, 제품 head: `f0cddc052`, 위 source/WASM hash 유지
- 실제 화면: `kps-ai.hwp — 77페이지`, Canvas2D, 자동 배치, DPR 2, inner viewport 1232×863
- 브라우저: macOS 사용자 에이전트의 Chrome 152.0.0.0. 진단용 `--no-opt` WASM/DEV 서버다.
- 수동 구간 12,528.8ms, 입력 354건, span 5,556건, rAF 1,019건, long task 44건
- `stopped`, `knownWorkReadyAtStop=true`, 누락 counter 전부 0, 관찰 오류 0,
  종료 snapshot의 pending image/prefetch 0
- 목표는 34% 근처였고 실제 종료값은 **31.501%**다. 정확히 34%를 측정했다고 보고하지 않는다.

최대 입력 delivery 간격은 input[226]→input[227]의 665.3ms다. 사용자 설명과 함께 이곳을 두
조작의 경계로 해석했다. long task가 capture보다 0.1ms 먼저 시작하는 경우도 있으므로 통계의
분류 경계는 그 공백의 중간인 session 상대 5141.75ms로 잡았다. 이는 제품 gesture 종료 규칙이나
손가락 종료 시각의 자동 검출이 아니다.

### 관찰 결과

| 항목 | 첫 번째 핀치 | 두 번째 핀치 |
| --- | ---: | ---: |
| 배율·열 | 100→53.64%, 1→2열 | 53.64→31.50%, 2→4열 |
| wheel 입력 | 227 | 127 |
| `geometry.zoom`의 non-animating 정착 / `page.releaseAll` | 34 / 34 | 24 / 24 |
| 전체 `raster.main` 호출 | 102 | 165 |
| 그중 동기 줌 정착 내부 호출 | 96 | 160 |
| `raster.main` 관찰 시간 합계 | 1083.8ms | 2439.7ms |
| 활성 구간 rAF gap p95 / max | 60.5 / 123.6ms | 170.8 / 313.6ms |
| 50ms 이상 long task 개수 / max | 20 / 78ms | 24 / 181ms |

활성 구간은 각 첫 input delivery부터 마지막 main raster 이후 처음 관찰한 rAF까지다.
수동 종료 버튼을 누르기까지의 idle 시간을 p95에 섞지 않았다. 분위수는 nearest-rank다.
`raster.main` 267회 중 정착 내부는 256회이고 나머지는 11회다. 267쪽을 고유하게 렌더했다는
뜻이 아니라 같은 쪽을 서로 다른 배율에서 반복 호출한 수다.

4열의 정착 처리 11회는 매번 visible 8쪽을 동기로 그렸다. 해당 `geometry.zoom`의 중앙값은
135.1ms, 최대 144.6ms였다. 가장 긴 rAF gap 313.6ms 구간에는 134.2ms와 143.0ms의 두
정착 처리가 겹쳐 있다. long task 최대 181ms와 rAF gap 최대 313.6ms는 서로 다른 지표다.

### 코드와 연결한 원인

1. `ViewportManager.smoothZoomTo()`는 target−current가 `0.001` 이하면 곧바로 `setZoom()`을
   부른다. 그보다 큰 입력도 smoothing 중 목표에 수렴하면 `zoomAnimating=false`를 발행한다.
   **목표 수렴은 다음 물리 입력이 오지 않을 것이라는 증거가 아니다.**
2. 실제 첫 입력 6건은 약 16ms 간격인데도 각각 non-animating 정착을 만들었다. 이 사례는 두
   핀치 사이의 665.3ms 공백과 무관하다. 전체 구간의 직접 `zoom.set`은 19회이며 non-animating
   zoom 처리는 58회다. 작은 연속 입력 사이에서도 비싼 경로로 반복 진입하는 것이 관찰됐다.
3. `CanvasView.onZoomChanged()`의 non-animating 분기는 매번 모든 surface/layer/LRU를
   해제한 다음 visible을 동기로 렌더한다. 4열에서는 한 번에 8쪽이라 작업 묶음이 커졌다.
   이후 입력의 delivery도 지연될 수 있고, 긴 프레임 뒤 smoothing은 다시 목표에 수렴할 수 있다.
   이 피드백의 정확한 기여율은 아직 분리하지 않았다.

이 구간에서 `geometry.layout`은 384회 합계 29.8ms, 최대 0.4ms였고, `geometry.preview`는
326회 합계 9.7ms였다. `geometry.dimensions`의 13.6ms는 layout에 중첩된다. 반면 main raster는
합계 3523.5ms였으며 그 하위 `wasm.layerRaster` 3466.7ms와도 중첩된다. 따라서
`geometry.zoom` 3552.0ms를 순수 geometry 비용으로 읽거나 이 값들을 전부 더하면 안 된다.
이번 표본은 전체 geometry/열 계산 재설계보다 반복 정착·동기 raster 경계를 먼저 고칠 근거다.

### 남는 한계

- 제품 수정 전 단일 사용자 구간이다. 성능 개선율·배포판 대비 회귀·다른 장치의 결과는 증명하지 않는다.
- DEV observer의 부가 비용과 브라우저 paint/composite 비용을 분리하지 않았다. rAF gap은 실제
  화면 프레젠테이션 간격이 아니며, 긴 작업의 전 시간을 WASM 비용으로 단정하지 않는다.
- 마지막 input delivery→마지막 main raster 종료는 첫 조작 86.2ms, 둘째 266.5ms다. 물리 손가락
  종료 시각·최종 화질 표시 완료·전체 image decode 완료로 해석하지 않는다.
- 빈 공간이 compositor에 실제로 표시된 시점은 이 JSON만으로 확인할 수 없다. animation preview가
  기존 active 쪽만 다룬다는 코드 경로와 사용자 관찰은 별도 증거다. 기존 surface 보존만으로 아직
  한 번도 렌더하지 않은 새 visible 쪽까지 즉시 나타나게 만들 수는 없다.

### 보존한 증적

- [최소 요약 JSON — 로컬 보존 안내](assets/issue6040-post-stack/README.md): 환경, 집계 방법,
  원 counter, 58개 정착 시점, 첫 입력 표본, long task와 최대 frame gap. 전체 원시의 대체물이 아닌
  파생 요약이며 before/after 성능 자료도 아니다.
- 표시된 JSON 전체는 Git 제외 경로 `tmp/issue6040-pinch-20260907.json`에 로컬 보존했다.
  SHA-256: `870d480ac04eae5831ff578cbab8b20500679390d6d77e89ed089b706f2a29c8`.
  이 2MB 원시와 과거 자동 smoke는 PR 증적에 추가하지 않는다.

## 빠른 사용자 핀치 — 2026-09-08

사용자가 요청한 빠른 핀치 기록을 종료한 뒤 같은 탭의 표시 JSON을 먼저 저장했다. 브라우저 연결
핸들을 복구했지만 페이지는 reload하지 않았다. 제품 source/WASM은 앞 표본과 같고 문서 head는
`d9aeb2f08`이다. scope `2/2/4`, 77쪽과 패널 `kps-ai` 선택으로 같은 문서임을 확인했다.

중요한 조건 차이는 **inner viewport가 1232×863에서 669×863으로 좁아진 것**이다. DPR은 2로
같지만 최대 표시 열이 4열에서 2열로 달라졌다. 배율도 실제 **100.351→37.890%**다. 이를
정확한 100→34% 표본이나 느린 입력 대비 속도 효과, 제품 개선으로 해석하지 않는다.

| 지표 | 결과 |
| --- | ---: |
| 수동 기록 구간 / 입력 delivery 구간 | 6358.3 / 2373.8ms |
| 입력 / span / rAF 기록 | 116 / 1626 / 701건 |
| non-animating 정착 / 전체 해제 | 23 / 23회 |
| 전체 main raster / 정착 내부 동기 raster | 68 / 58회 |
| main raster 관찰 시간 합계 | 597.8ms |
| 활성 구간 rAF gap p95 / max | 29.7 / 99.6ms |
| 50ms 이상 long task 개수 / max | 9 / 64ms |
| 마지막 delivery→마지막 main raster 종료 | 92.5ms |

frame 집계는 session 상대 896.6ms의 첫 delivery부터 마지막 main raster 이후 첫 rAF인
3369.1ms까지다. 앞 표본과 같은 nearest-rank이며 idle 종료 대기 시간은 제외했다. `stopped`,
known-work readiness true, 누락 counter 전부 0, 오류 0, pending image/prefetch 0이었다.
물리 손가락 종료나 compositor 표시 완료는 여전히 계측하지 않았다.

408.7ms와 552ms의 입력 공백이 있었지만 사용자가 물리 조작 횟수를 명시하지 않았으므로 이를
제스처 경계로 확정하지 않았다. 그 공백을 제외한 입력 묶음 안에서도 정착이 반복된다. 2열 구간의
정착 9회는 각각 4쪽을 동기로 raster했고, 정착 처리 중앙값 42.1ms / 최대 47.9ms였다.
2열에서도 반복 정착 문제가 나타난다는 추가 근거이며 4열일 때만 생기는 문제는 아니다.

`geometry.layout`은 119회 합계 8.3ms / 최대 0.2ms였고 main raster는 합계 597.8ms였다.
geometry.zoom 528.2ms는 하위 raster를 포함하므로 순수 배치 비용으로 읽지 않는다.
정착 억제를 먼저 분리하는 Stage 3.2A의 우선순위를 유지하되, quiet 120ms를 이 표본으로
확정하거나 예측 개선율을 계산하지 않는다. 단일 before 표본이며 계측 on/off 비용도 미분리다.

- [빠른 핀치 최소 요약 — 로컬 보존 안내](assets/issue6040-post-stack/README.md): 조건 차이·집계법·원 counter와 정착 시점
- 원시: Git 제외 `tmp/issue6040-fast-pinch-20260908.json` 약 0.96MB. SHA-256:
  `b4491e0eb6ee2c3676d3dba3bf996b995e5e977299d9936519d6d716a5082c9c`

저장 후 같은 탭·문서·자동 배치를 100%로 준비했다. 현재 viewport 669×863과 scroll x=468/y=0을
그대로 기록하고 viewport를 강제 변경하지 않았다. 다음 수집은 한 수동 구간의 100→34 근처→100%
역방향 왕복이며, 축소/확대에 버튼을 사용하지 않는다. 이후 before/after 비교는 동일 viewport·시작
scroll·문서·입력 조건으로 다시 고정한다. 제품 소스 변경·서버 재시작·원격 게시를 하지 않았다.

## 역방향 사용자 핀치 — 2026-09-08

사용자가 축소 후 확대를 마치고 수동 종료한 JSON을 같은 탭에서 읽었다. 문서 head는 `47a852d56`,
제품 source/WASM은 앞의 두 표본과 동일하다. `kps-ai.hwp` 77쪽, Canvas2D·자동 배치·DPR 2,
scope `2/2/4`를 유지했다. viewport는 앞선 빠른 표본의 669×863에서 **1232×863**으로 바뀌었다.
실제 배율은 **100→34.469→113.123%**, 열은 1→2→3→4열을 거쳐 다시 1열이다.
정확한 100→34→100%나 제품 수정 전후의 통제 비교로 보고하지 않는다.

| 지표 | 결과 |
| --- | ---: |
| 수동 기록 구간 | 5680.8ms |
| 전체 wheel / 줌 입력 / 후속 일반 scroll | 169 / 131 / 38건 |
| 줌 축소 / 확대 입력 | 68 / 63건 |
| span / rAF 기록 | 1849 / 642건 |
| non-animating 정착 / 전체 해제 | 5 / 5회 |
| 전체 main raster / 정착 내부 동기 raster | 30 / 20회 |
| main raster 관찰 시간 합계 | 397.5ms |
| 활성 구간 rAF gap p95 / max | 9.4 / 182.2ms |
| 50ms 이상 long task 개수 / max | 3 / 182ms |
| 마지막 줌 delivery→마지막 main raster 종료 | 85.9ms |

원장은 일반 wheel도 기록한다. `ctrlKey || metaKey`인 131건만 줌 입력으로 분리했으며,
수정키 없는 38건은 줌·raster 완료 뒤 session 상대 4619.8~4928.1ms에 발생했다.
frame 통계는 첫 **줌** delivery 954ms부터 최종 main raster 이후 첫 rAF인 4462ms까지다.
방향 전환 전 휴지는 포함하지만 후속 일반 scroll과 수동 종료 대기는 포함하지 않았다.
정착이 5회로 적었다고 제품 개선으로 해석하지 않는다. 입력 delta·간격도 앞 표본과 다르며
직접 `zoom.set`은 이번에 0회였다. 관찰 후 상태가 animating=true인 main raster span도 없었다.

정착은 약 58.08%, 34.47%, 54.59%, 87.99%, 113.12%에서 발생했다. **34.47% 정착은
visible 8쪽을 동기로 처리하며 141.5ms**가 걸렸다. 반복 횟수가 적어도 최종 묶음 비용이 남는
사례이므로, 반복 정착 억제(A)와 페이지별 교체(B)의 수용 결과를 분리해야 한다.

역방향 경계의 마지막 축소 delivery는 1837.3ms, 첫 확대 delivery는 2687.4ms로 간격이
850.1ms였다. 그 전에 시작한 main raster는 2110ms에 모두 끝났고, 첫 확대 입력은 그보다
**577.4ms 뒤**였다. 따라서 이전 raster가 남은 상태의 방향 전환·취소 경합을 관찰한 것은 아니다.
종료 snapshot의 scheduler counter는 누적값이므로 `staleDropped=0`도 이번 구간의 취소 필요성이나
안전성 증명에 사용하지 않는다. A의 fake-clock stale callback·재진입 테스트와 이후 B/3.3의
renderer lease·decode 경합 테스트로 미실행/지연 완료를 강제로 재현해야 한다.

종료는 `stopped`, known-work readiness true, 누락 counter 전부 0, 관찰 오류 0,
pending image/prefetch 0이었다. 이는 물리 손가락 종료나 compositor 표시 완료 시각은 아니다.

- [역방향 핀치 최소 요약 — 로컬 보존 안내](assets/issue6040-post-stack/README.md): 줌/일반 scroll 분리,
  조건·집계법·원 counter·5개 정착·역방향 진입 시점
- 원시: Git 제외 `tmp/issue6040-reverse-pinch-20260908.json` 약 0.96MB. SHA-256:
  `4e2e80f0295a19ab07bf76919c0b761eeaadd9fe3628353a0024b0e5a0c57e78`

최소 요약을 원시에서 재집계하고 링크·`git diff --check`를 확인했다. 이번 변경은 증적·계획 문서뿐이며
제품 test/build를 새로 실행한 것으로 보고하지 않는다. 탭의 종료 화면은 유지하고 reload·새 수집 준비·
서버 재시작·제품 수정·원격 게시를 하지 않았다.

## 아직 판정하지 않은 것

- 이전 작업이 실제로 남은 상태의 역방향 재진입·취소 경합
- 제품 수정 후 같은 조건에서 반복 정착·raster·프레임 지연이 감소하고 최종 화질이 유지되는가
- 새 visible 페이지의 미생성과 기존 surface 교체 중 어느 구간에서 빈 화면이 실제로 표시되는가
- compositor paint/composite 시간과 관찰 on/off 오버헤드

`spans`는 종료 순서로 쌓이므로 시작 시각으로 정렬하고 nested inclusive 시간을 합산하지 않는다.
긴 작업·입력 timestamp·trusted의 해석 한계는 사용 안내를 따른다. source에서 가능한 경로라는 사실을
실제 사용자 입력에서 발생한 병목으로 단정하지 않는다.

## 권장 다음 순서와 사용자 gate

1. 저해상도 선렌더는 보류 유지. 현재 2회 물리 조작에 58번 발생한 비싼 정착의 재진입부터 줄인다.
   목표 배율의 수렴과 연속 입력의 quiet 상태를 분리하는 후보를 상세 설계에 포함할 것을 제안한다.
   이번 기록만으로 고정 debounce 값을 정하거나 physical gesture end를 안다고 가정하지 않는다.
2. 공유 geometry·기존 pointer anchor를 유지하며 기존 surface는 새 쪽 렌더가 준비될 때 교체한다.
   단순 surface 보존만으로 58회의 작업 자체가 사라지는 것은 아니므로 위 입력 상태와 함께 설계한다.
3. 최신 요청을 우선하고 오래된 대기/완료 결과를 무효화한다. 이미 실행 중인 단일 동기 raster의
   선점이나 새 visible 쪽의 무조건 즉시 표시는 약속하지 않는다. 임시 surface도 기존 예산에 포함한다.

느린·빠른·역방향 세 표본 수집을 마쳤다. 정확히 34%나 손가락 종료 시각에 맞추기 위한 추가 수동
녹화는 A 착수 조건으로 요구하지 않는다. 다음 단계는 이미 작성한 파일별 상세 설계의
**Stage 3.2A 구현 승인**이다. 승인 후 입력 quiet와 배율 수렴을 분리하고, 조작 감도·앵커·화질·예산은
유지한 채 반복 정착 억제를 먼저 검증한다. A에서도 마지막 정착의 동기 묶음은 남을 수 있다.
B 페이지 교체와 3.3의 집중 취소 검증은 A 결과 승인 뒤 진행하며, 이번 수집 완료를 제품 구현
승인으로 해석하지 않는다. 다음 단계 문서는 이 단계 결과 commit 뒤 시작한다.

원격 게시·push·PR 생성·이슈 종료는 하지 않았다.
