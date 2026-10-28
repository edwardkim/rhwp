# #6040 Stage 4.1 — 줌 정착 중복 스크롤 보정과 재측정

- 일자: 2026-09-10 KST
- 상태: **중복 렌더 보정·재측정 완료, 확대 경보가 남아 통합 수용 보류**
- 계획 commit: `f68ab9df5`; 후보는 이 보고서와 같은 commit의 CanvasView 변경이다.
- [수행 계획](../plans/task_m100_6040.md), [구현 계획](../plans/task_m100_6040_impl.md)
- [이전 Stage 4](task_m100_6040_post_stack_stage4.md), [최소 계측 JSON — 로컬 보존 안내](assets/issue6040-post-stack/README.md)

## 결과와 사용자 영향

줌이 만든 지연 scroll 알림 때문에 화면 밖 쪽을 높은 DPR로 그린 뒤 다시 낮은 DPR로 그리는
중복은 제거했다. 예제 50→100%에서 후보의 main raster가 이전 Stage 4의 3회에서 2회로
줄었고, 화면에 보이는 쪽의 최종 DPR 2는 유지했다. 실제 스크롤 이벤트나 눈금자 소비자를
차단하지 않고 CanvasView의 동일한 렌더 계획 재생성만 생략한다.

그러나 **불필요한 연산 하나를 제거한 것과 확대가 더 부드러워진 것은 다르다.** 이번 재측정에서도
50/100% visibleStable은 기준선보다 +20% 이상 늦었다. 화면 밖 작업이 visible 렌더 직후
실행되어 다음 frame 관찰을 늦추는 순서가 남는다. 이 보정을 최종 성능 개선이나 무회귀로
승격하지 않으며, 제출·이슈 종료·다음 구조 실험으로 넘어가지 않는다.

## 재현과 수정 범위

실제 CanvasView, PageRenderScheduler, DPR planner를 연결한 결정적 테스트로 다음을 재현했다.

1. 50%에서 두 쪽의 기존 surface가 DPR 2로 남는다.
2. 100% 정착 계획은 visible page 0을 DPR 2, 화면 밖 page 1을 DPR 1로 선택한다.
3. 같은 viewport의 지연 scroll이 계획을 덮으면 이전 요청 DPR 2를 잠가 page 1을 다시 그린다.
4. scroll 정착에서 page 1을 DPR 1로 다시 그린다.

원래 코드의 실제 raster 순서는 `[[0, 2], [1, 2], [1, 1]]`, 보정 후에는
`[[0, 2], [1, 1]]`이다. 각 쌍은 `[0-based page index, requested DPR]`이다.
테스트 timer host는 0ms 작업을 150ms 정착보다 먼저 실행하도록 due time을 따른다.

변경은 다음에 한정했다.

- 성공적으로 예약한 zoom-settled의 x/y/zoom/viewport 크기/렌더 generation을 저장한다.
- 뒤늦은 `scroll`이 이 상태와 모두 같을 때 CanvasView 갱신만 생략한다.
- 양축 0.5px 이동, 크기·배율·generation 변경, strict 갱신은 기존 경로를 실행한다.
- 계획 실패 때는 marker를 남기지 않으며 전체 surface 해제 때는 marker를 비운다.
- 새 timer/Worker, DPR·prefetch 예산, 앵커·눈금자·입력 이벤트 정책은 바꾸지 않았다.

실문서 초기 보정에서는 물리 scale이 같은 구 배율 surface가 exact key 검사에 걸려
`rhwpRenderedZoom`과 preview가 남고 known-work trace가 끝나지 않는 경계도 발견했다.
계측기의 완료 조건을 완화하지 않고, 구 배율인 exact surface도 visible/retained 큐에서
정상 렌더 경로로 한 번 확정하게 했다. 완료 후 같은 배율의 exact surface는 다시 그리지 않는다.

이 선택은 bitmap 무래스터 재사용 최적화가 아니다. 특히 고배율 안전 clamp로 물리 크기가
같은 전환에서는 이전보다 한 번의 렌더가 추가될 수 있다. CSS·레이어·이미지 callback을
안전하게 갱신하는 무래스터 재사용은 구현하지 않았다. **354→500% 등 clamp 경계와 실제
Firefox 연속 핀치의 비용 검증이 남아 있으며, 이 위험을 통과로 간주하지 않는다.**

## 동일 조건 버튼 줌 A/B

기준선은 `56706247f`, 후보는 `f68ab9df5` 위의 이번 소스다. 소스·테스트·WASM SHA-256은
JSON에 고정했다. Canvas2D / exam_kor 20쪽 / Chromium 152 / viewport 1280×720 / DPR 2,
문서 viewport 1260×558, 자동 배치에서 100→34→50→100%를 3회 실행했다.
첫 회는 예열, 본 회차는 후보→기준선과 기준선→후보 순으로 바꿨다. 서로 다른 회차를 동시에
실행하지 않았고 측정 중 테스트·빌드도 실행하지 않았다.

중단 직전 저장하지 못한 측정값은 복구되었다고 가정하지 않고 제외했다. 아래는 재실행하여
파일에 보존한 18 trace 중 본 회차 두 표본의 중앙값이다. max rAF는 각 trace 최대값의
중앙값이다. 작은 표본이므로 통계적 유의성이나 Firefox 핀치 개선율을 뜻하지 않는다.

| 목표 배율 | visibleStable 기준선 → 후보 | 변화 | max rAF 기준선 → 후보 | main raster/회 |
| --- | ---: | ---: | ---: | ---: |
| 34% | 431.55 → 442.55ms | +2.5% | 334.75 → 60.95ms | 9 → 6 |
| 50% | 193.35 → 242.85ms | **+25.6% 경보** | 121.10 → 106.70ms | 4 → 4 |
| 100% | 155.35 → 225.15ms | **+44.9% 경보** | 62.40 → 120.20ms | 2 → 2 |

- 18 trace 모두 complete, 최종 visible DPR 2, pendingImages 0, 관찰 오류 0.
- 전체 surface 해제는 기준선 매회 1회, 후보 0회다.
- 34% visibleFirst 중앙값은 기준선 431.55ms, 후보 164.70ms다. 먼저 완성되는 쪽이 생겼다는
  뜻이며 전체 visible 완료가 더 빨라졌다는 뜻은 아니다.
- 동일 진단용 no-opt WASM을 사용했으며 확장 배포판의 절대 성능과 비교하지 않는다.
- DOM 기반 known-work/다음 frame 기회 계측이다. compositor 실제 표시·GPU 업로드·모든
  decoder 완료를 직접 측정하지 않았으며 screenshot으로 그 시점을 증명하지 않는다.
- 앞의 8 trace는 세부 span을 줄이는 과정에서 제외했지만 counters/frames/완료·화질은
  보존했다. 뒤의 10 trace는 raster/visibility/zoom 경계 span도 남겼다.

## 남은 지연의 관찰 근거

본 회차 1의 50→100%에서 다음 순서를 관찰했다(조작 시작 상대 ms).

| 경계 | 기준선 | 후보 |
| --- | ---: | ---: |
| visible page 0 main raster | 98.10–152.90 | 104.70–162.30 |
| 화면 밖 page 1 main raster | 161.30–216.80 | 166.00–226.20 |
| visibleStable 관찰 | 156.70 | 228.90 |

후보는 page 0 완료 뒤 page 1 작업이 먼저 실행되고 나서 visibleStable이 관찰됐다.
코드상 retained-transition은 visible 큐가 비면 0ms timer로 예약된다. **0ms timer는 별도 task지만
그 사이 paint/frame 기회를 보장하지 않는다.** 중복이 사라져도 두 무거운 렌더 사이의 frame
기회가 보장되지 않는 것이 다음 조사 대상이다. 이 표만으로 compositor가 실제로 언제 그렸는지,
50% 경보의 원인이 전부 같은지까지 확정하지 않는다. 계측 코드를 바꿔 점수를 낮추지 않는다.

## 최종 surface 비용

| 목표 배율 | totalAllocatedPixels 기준선 → 후보 |
| --- | ---: |
| 34% | 23,103,360 → 15,677,280 |
| 50% | 23,183,212 → 28,133,932 |
| 100% | 33,874,172 → 39,224,144 |

100% active pixels는 양쪽 모두 33,874,172다. 후보에는 5,349,972px detached cache 한 쪽이
추가로 남아 total이 높다. 40M retained 예산 안이지만 이 전환의 hit는 관찰되지 않아 재사용
효과를 주장하지 않는다. 이전 Stage 4 후보 snapshot보다도 높으며 메모리 절감으로 포장하지
않는다. 이 값은 정착 snapshot이며 **peak RSS/GPU/임시 surface 최고값이 아니다.**

## 검증과 다음 판정

- 보정 테스트 11개 추가: 실제 DPR 경합, 반복 알림, exact 구 zoom 확정, 양축·크기·배율·
  strict·generation 변경, 계획 실패 복구.
- 현재 소스의 Studio 전체 검사: **1,608 total / 1,607 pass / 1 skip / 0 fail**.
- `npm run build`(tsc 포함) 통과. 기존 CanvasKit fs/path externalization·bundle 크기 경고 유지.
  중단 전 이 exact 소스로 끝낸 검사이며 이후 제품 소스는 바꾸지 않았다.
- Rust/WASM source 변경 없음. 원격 push·PR·코멘트·이슈 종료 없음.
- warm 스크롤·관찰 on/off는 이번 소스로 재실행하지 않았다. Stage 4 기록을 새 head의
  통과로 승격하지 않는다. CanvasKit/나머지 문서/배치·편집 매트릭스, 고배율 clamp,
  실제 Firefox 핀치, peak 메모리도 미완료다.

다음 권장은 visible 완료 뒤 retained 작업의 시작 시점과 첫 frame 기회를 결정적 host 테스트로
고정하고, 실제 브라우저에서 확인하는 최소 보정이다. 임의의 긴 고정 지연, 화질 하향, 새 Worker나
staging 확대는 우선하지 않는다. 이는 이번 승인 범위의 timer 정책 불변 조건을 넘으므로,
**이번 결과를 먼저 commit하고 다음 수행·구현 범위를 승인받은 뒤** 진행한다. 확대 경보가
해소되기 전에는 Stage 4 전체 수용 완료로 처리하지 않는다.
