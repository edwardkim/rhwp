# #6040 새 Stage 3.2A — 연속 핀치의 반복 정착 억제

- 일자: 2026-09-08 KST
- 상태: **후보 구현·로컬 회귀 및 실제 사용자 왕복 핀치 분석 완료. 사용자는 이전보다 훨씬 부드럽다고 확인. quiet 최적값 확정·B 착수 승인은 별도**
- Issue: #6040
- 기준선: Stage 3.1 `0b76ecb16`, 착수 계획 `1e09531a9`
- [수행 계획](../plans/task_m100_6040.md), [구현 계획](../plans/task_m100_6040_impl.md),
  [입력 기준선](task_m100_6040_post_stack_stage3_1.md)

## 이번 변경이 해결하려는 문제

기존 경로는 배율 smoothing이 목표에 도달하면 손가락 입력이 계속 오고 있어도 최종 렌더를
실행했다. Stage 3.1의 느린 핀치 두 구간에서 전체 해제 58회, main raster 267회를 관찰했다.
목표 수렴과 입력 종료가 같은 사건이 아니라는 점을 분리한다. 이 수치는 이번 후보의 개선율이 아니다.

`ZoomInputSettle`은 입력마다 세대를 갱신하고, **마지막 유효 ctrl/meta-wheel 이후 120ms quiet와
배율 수렴이 모두 만족될 때** 최신 세대의 최종 raster를 한 번 요청한다. quiet는 물리 손가락 종료
검출이 아닌 휴리스틱이다. 120ms는 후보이며 80/160ms와 실제 입력 비교 없이 최적값으로 확정하지 않는다.

- 입력 도중과 수렴 후 quiet 대기에는 기존 surface의 CSS preview를 유지한다.
- zoom-changed의 좌표·앵커·눈금자·배율 표시는 계속 갱신한다. ready 알림에서는 geometry를
  다시 계산하지 않고 최종 렌더만 실행해 중복 앵커 보정을 피한다.
- 입력당 상태 갱신은 O(1), timer는 최대 하나다. 수렴 후 quiet 대기만을 위해 rAF를 돌리지 않는다.
  Worker, thumbnail cache, 저해상도 선렌더, DPR/예산 변경은 없다.
- 버튼·직접 배율·fit 명령은 wheel quiet를 기다리지 않는다. 일반 스크롤·편집 focus 등 명시 조작은
  현재 배율의 pending raster를 완료하거나 최신 전체 갱신으로 넘긴다. 이 경로는 동기 비용이 남는다.
- 문서 교체·reset·detach·실제 resize·보기 설정 변경은 낡은 세대를 무효화한다. 같은 크기의
  ResizeObserver 알림은 quiet를 깨지 않는다. 비동기 renderer 선택이 겹쳐도 중단한 줌의 전체
  갱신 책임은 최신 선택에 승계한다.

최종 `zoom-settled`는 여전히 기존의 전체 해제 후 **visible 동기 렌더**다. 새로 보일 페이지가
preview 중 비어 있거나 마지막 동기 렌더가 길어지는 문제까지 해결한 것은 아니다. Stage 3.2B의
페이지별 교체 및 3.3의 집중 취소 검증은 이번 구현에 포함하지 않았다.

## 로컬 검증

| 검증 | 결과 |
| --- | --- |
| 입력 정착 상태기계 | 8건 통과: 80/120/160ms, 역방향 새 입력, 취소, 두 입력 구간, 재진입 |
| 실제 ViewportManager wheel 경로 | 10건 통과: 작은/큰 입력, 일반 스크롤, 명령, min/max, meta, detach |
| Canvas preview/정착 경로 | 22건 통과: 배치·방향, stale ready, scroll 우회 방지, resize, 편집, renderer 선택 승계 |
| 실제 VM → Canvas 경계의 제어된 작은 입력 | 16ms 간격 20입력 → preview 20회, quiet 뒤 전체 해제·정착 렌더 각 1회, ready 시 geometry 추가 호출 0 |
| Studio 전체 `npm test` | **1,535건: 1,534 pass / 1 skip / 0 fail** |
| `npx tsc --noEmit`, `npm run build` | 통과, 254 modules / PWA 생성 |
| `git diff --check` | 통과 |

최초 전체 검사에서 스크롤 테스트의 가짜 ViewportManager에 새 guard가 사용하는 메서드가 없어
9건 실패했다. 해당 fixture에 비애니메이션·비pending 상태를 명시한 뒤 전체 재검사했다.
생산 코드를 optional 호출로 바꾸어 오류를 숨기지 않았다. 빌드에는 기존 CanvasKit fs/path
externalization과 chunk 크기 경고가 남는다. Rust/WASM은 수정하지 않았다.

WASM SHA-256: `24d3d2ffe0c1b43d4f53c23762820112b8298c2a081c9dd4842406819f4130fc`.
진단용 DEV/`--no-opt` 조건이므로 출시판 성능 수치로 사용하지 않는다.

## 실제 브라우저 기능 smoke

Canvas2D, DPR 2, 자동 배치, 편집 viewport **669×863**에서 패널 버튼으로 34→50→100%를
실행했다. 각 값의 정착 snapshot에서 visible 페이지의 surface 존재와 known-work 완료를 확인했다.

| 문서 | 쪽 수 | 34/50/100% 열 수 | 세 배율의 결과 |
| --- | ---: | --- | --- |
| exam_kor.hwp | 20 | 1 / 1 / 1 | 가시 쪽 누락 0, image/prefetch pending 0, 관찰 오류 0 |
| basic/KTX.hwp | 1 | 1 / 1 / 1 | 동일 |
| 21868765_별표2_보건소_분장사무.hwp | 4 | 2 / 1 / 1 | 동일 |
| kps-ai.hwp | 77 | 2 / 1 / 1 | 동일 |

kps-ai 34% 화면에서 표지·목차·본문이 두 열로 표시되고 페이지가 비어 있지 않음을 스크린샷으로
확인했다. 이는 정착 화면 기능 smoke이며 픽셀 단위 원본 화질 비교나 이동 중 시각 회귀 판정은 아니다.
버튼 경로이므로 `zoomGeneration=0`, `rasterPending=false`다. 이 결과로 wheel quiet의 실제 지연이나
성능 개선을 주장하지 않는다. 앞선 넓은 화면 기준선(1232×863)과도 통제된 시간 비교가 아니다.

## 사용자 검증 및 다음 gate

로컬 4200에서 kps-ai, 자동, 100%로 준비한다. 이전과 같은 창 크기로 설정한 뒤 패널의
**연속 핀치 시작 → 실제 느린 100→약 34% 핀치 → 손을 뗀 뒤 안정화 → 연속 핀치 종료**를 수행한다.
두 번으로 나누어도 된다. 구간 전체는 20초 이내로 하고, 종료 뒤 화면을 새로고침하지 않는다.

1. 입력 도중 좌우·상하 점프, 눈금자 불일치, 감도 변화가 없는지 확인한다.
2. 손을 뗀 뒤 preview가 최종 화질로 바뀌는 대기가 거슬리는지 별도로 판단한다.
3. 기록에서 `zoom.raster`/`page.releaseAll`, main raster 횟수, rAF gap, long task,
   마지막 입력→known-work 완료를 기준선과 비교한다. 입력 수·폭·방향이 다르면 개선율로 단정하지 않는다.
4. 빠른·역방향 핀치와 일반 스크롤/편집 개입을 추가 확인하고 필요하면 80/160ms 후보를 비교한다.
   수동 구간 종료와 known-work ready는 별개다.

[패널 안내](../manual/studio_scroll_probe_guide.md)에 inputActive/rasterPending/zoomGeneration 및
zoom.ready/raster/cancel/flush의 해석을 추가했다. 원시 계측을 무더기로 추가하지 않았으며 이번
단계에는 출시 성능 개선 수치가 없다. 실제 입력 수용 결과 승인 전에는 B 단계로 넘어가지 않는다.
원격 push·PR·이슈 코멘트는 수행하지 않았다.

## 실제 사용자 확인 — 후보 `712e892f5`

사용자는 “완료했어. 이전보다 훨씬 부드러운 것 같아.”라고 보고했다. 화면을 새로고침하거나
관찰 결과를 덮어쓰지 않고 종료 버튼이 남긴 JSON을 읽었다. 기록은 **100→32.8225→101.9822%**
왕복이며, 판독 당시 UI의 12%는 이 frozen snapshot에 포함되지 않는다. 100→34% 단방향만
측정한 것으로 오기하지 않는다. 서버·브라우저 위치·제품 코드를 변경하지 않았다.

- kps-ai 77쪽, Canvas2D, DPR 2, 편집 viewport **1232×863**, 같은 diagnostic WASM hash.
- 수동 기록 6,166.6ms, wheel 194건 중 zoom 154건·일반 scroll 40건, span 2,079건,
  frame 683건. `stopped`, `knownWorkReadyAtStop=true`, 누락·관찰 오류 0, image/prefetch pending 0.
- [최소 요약 — 로컬 보존 안내](assets/issue6040-post-stack/README.md). 원시는 ignored
  `tmp/issue6040-stage3-2a-pinch-20260908.json`에만 보존했다.
  SHA-256: `223ffe6f5373a170794da3c60e128bd933d862b3c17a62a81e38286c3f31d0ea`.

### A의 효과와 B에 남은 작업

120ms를 넘는 zoom input delivery 공백으로 나누면 6개 입력 묶음이다. 이는 계측상의 분류이며
사용자가 물리 핀치를 정확히 6번 했다는 뜻이 아니다. 각 묶음에 정착이 한 번씩 있고,
모든 `zoom.raster` 시작은 직전 zoom 입력에서 **121.1~121.5ms 뒤**였다.

| 항목 | 이번 후보의 관찰값 |
| --- | ---: |
| zoom 입력 / 최종 정착·전체 해제 | 154 / 6 |
| inputActive 또는 rasterPending 상태의 main raster | **0회** |
| 전체 main raster / inclusive 합계 | 34회 / 537.5ms |
| 그중 동기 zoom.raster 내부 main raster | 23회 |
| zoom.raster 최대 / 단일 main raster 최대 | 161.7ms / 37.7ms |
| 활성 구간 rAF gap p50 / p95 / max | 8.3 / 9.4 / 201.5ms |
| 50ms 이상 long task | 3건: 100 / 161 / 51ms |

rAF 활성 구간은 첫 zoom input부터 마지막 main raster 뒤 첫 관찰 frame까지이며 사이의
입력 공백·일반 스크롤도 포함한다. renderer span과 중첩 span을 합산하지 않는다. 40개 일반
scroll은 5번째 정착 뒤, 6번째 zoom 입력 전이다. pending 중 스크롤 강제 flush의 실사용 검증으로
볼 수 없으며 역방향도 앞선 raster 종료 뒤 시작하므로 취소 경합 검증을 대체하지 않는다.

이전 느린 기준선은 354 zoom 입력·58정착·267 main raster·3,523.5ms였다. 문서·폭·DPR은
맞지만 이번은 더 짧고 역방향 확대가 포함된 입력이다. **차이를 통제된 개선율로 환산하지 않는다.**
다만 후보의 입력 활성 구간에 반복 최종 raster가 없고 공백마다 한 번씩 실행된 것은 A의 의도와
일치하며, 사용자 체감 개선 보고를 뒷받침한다. 이전 역방향 표본은 원래 정착 5회였으므로
모든 입력 패턴에서 같은 비율의 효과가 나는 최적화라고 주장하지 않는다.

32.8225%의 4열 정착에서는 visible 8쪽 동기 처리에 **161.7ms**가 남았다. 마지막 zoom 입력부터
그 동기 처리 종료까지 약 **282.9ms**(quiet 포함), 이후 main raster까지 포함하면 **411.4ms**다.
후자는 prefetch 등을 포함하며 화면이 보이기까지 걸린 시간 또는 물리 finger-up 이후 시간과
같지 않다. 따라서 A의 반복 작업 제거는 긍정적으로 평가하지만 최종 burst와 blank 지속시간은
B의 별도 목표로 유지한다. 120ms는 현재 후보값으로 유지하며 80/160ms보다 우월하다고 확정하지 않는다.

다음 제안은 A의 현 결과를 승인받고 B의 기존 화면 유지·페이지별 교체를 진행하는 것이다.
필수 렌더의 총 완료시간뿐 아니라 첫 visible 채움, slice/page p95·max, blank 지속시간,
old+staging peak surface 비용을 분리한다. 이미 실행 중인 단일 WASM raster는 페이지 사이의
양보로 선점할 수 없으며 새 Worker·범용 scheduler·좌표계 변경은 승인 범위에 추가하지 않는다.
