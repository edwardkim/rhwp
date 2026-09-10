# #6040 — cold 예산 갱신 내부 정밀 계측

- 날짜: 2026-09-09 KST
- 단계: Stage 3.2B 잔여 수용 조사. Stage 4 완료 또는 새 구조 실험이 아니다.
- 사용자 승인: 네 Firefox JSON 분석 뒤 제안한 상세 계측 진행.
- 작업 head: `ec2c956ed` 위 DEV 변경. 제품 기준선 `56706247f`, 후보 `0345107c5` 유지.
- 결론: cold의 긴 `budget.refresh`는 단순 예산 산술이 아니라 페이지별 layer 정보 조회의
  WASM overlay 경로가 지배한다. 제품 수정·성능 개선·원격 게시 없음.

## 1. 사용자 Firefox 표본

원본은 사용자가 첨부한 Downloads의 다음 파일이며 저장소에 복제하지 않았다.

- `rhwp-before-56706247f-cold-1788897293436.json`
- `rhwp-before-56706247f-warm-manual-1788897312414.json`
- `rhwp-after-0345107c5-cold-1788897338260.json`
- `rhwp-after-0345107c5-warm-manual-1788897349618.json`

공통 Firefox 155, DPR 2, viewport 1512×845, exam_kor 20쪽, 종료 시 4열. 모두 stopped,
knownWorkReadyAtStop=true, 오류·누락 0. LongTasks API 미지원이므로 빈 배열은 긴 작업 없음이 아니다.

| 지표 | before cold | after cold | before warm | after warm |
| --- | ---: | ---: | ---: | ---: |
| 최대 rAF 간격 ms | 585 | 590 | 36 | 18 |
| 50ms 이상 간격 수 | 16 | 11 | 0 | 0 |
| main raster 수 | 36 | 28 | 12 | 0 |
| budget.refresh 최대 ms | 584 | 587 | 1 | 1 |

before 최종 zoom=0.307847676, after=0.281322572다. after 페이지 면적이 약 16.5% 작아
캐시 예산에 유리하며 before warm은 편집 focus도 변경됐다. 전체 elapsed는 조작/정지 시간까지
포함한다. 한 쌍으로 성능 개선율이나 B 회귀 부재를 선언하지 않는다. 비트맵 없는 rAF 표본 수는
긴 동기 작업 중 표본 자체가 생기지 않는 영향을 받으므로 빈 화면 지속 시간과 동일하지 않다.

## 2. 추가 관찰과 제품 무변경 경계

`budget-probe-boundaries.ts`의 8개 기존 메서드 wrapper를 `scrollProbeBudget=1`에서만 설치한다.
새 호출/캐시 warm-up, WASM 재빌드, CanvasView/PageRenderer/Rust 수정은 하지 않는다.
`observeBoundary`의 this·인수·동일 반환/예외·복원 계약을 재사용한다. snapshot capability와
0-based page 번호로 경계를 구분한다. 기존 기본 observer에도 page 라벨 분류만 확장했다.

same adapter를 detached before(4201)/after(4202)에 적용했다. 개발 서버 URL에 옵션을 붙여
reload해야 상세 경계가 설치된다. 기존 query만 사용하면 기본 계측을 유지한다.

| 파일 | SHA-256 |
| --- | --- |
| page-scroll-probe.ts | `3be22edcfa82e8eb90002c8ce69ef07f09fb50d98306acd4528c5c385487e96f` |
| budget-probe-boundaries.ts | `e329b92fa6e5bbfee5a156dc6aea6e6b51c9eb57fd259b04a4ddcb5342059a98` |

WASM/fixture/lock/fonts와 다른 adapter hash는 [A/B 준비 기록](task_m100_6040_post_stack_ab_setup.md)
그대로다. 진단 `--no-opt` 빌드이며 배포 성능 수치로 사용하지 않는다.

## 3. 인앱 브라우저 cold 원인 재현

1280×720, DPR 2, Canvas2D, exam_kor, **고정 네 열**, 100→34% 버튼, 다음 행 5회.
사용자 Firefox의 자동 배치/실제 핀치가 아니라 동일 코드 경로의 분리 smoke다. 각 기록 중에는
추가 DOM snapshot을 수집하지 않았다. UI 호출 사이 제어 지연이 있으므로 입력 성능 비교는 아니다.

| 동일 cold 시나리오의 긴 구간 | before ms | after ms |
| --- | ---: | ---: |
| 첫 budget.refresh | 248.5 | 245.2 |
| 그 안의 layerCount 합집합 | 248.1 | 245.0 |
| 가장 긴 budget.refresh | 562.4 | 564.4 |
| 그 안의 layerCount 합집합 | 562.2 | 563.7 |
| 그 안의 WASM overlay 조회 합집합 | 561.7 | 562.5 |
| 그 안의 budget.reconcile | 0.2 | 0.0 |

가장 긴 후보 구간의 18/19/20쪽 layer 조회는 각각 82.1/135.8/343.2ms였다.
20쪽의 WASM overlay 조회는 342.2ms. layerCount·overlaySummary·WASM은 중첩되므로 더하지 않는다.
해당 구간의 layerCount 호출은 순차이며 자식 구간 밖 refresh 잔여는 약 0.7ms다. 남은 시간에는
planner/Map/diagnostics/observer가 섞이므로 순수 planner 측정이라고 하지 않는다.

소스 경로:

1. `CanvasView.updateVisiblePages`가 스케줄러 `setDesiredWork` 전에 `refreshRenderSurfacePlan` 호출.
2. retained 후보 각각에 `PageRenderer.getCanvasSurfaceLayerCount` 호출.
3. JS layer count cache miss이면 overlay summary → `WasmBridge.getPageOverlayImages`.
4. Rust `get_page_overlay_images_native`가 `build_page_layer_tree`를 호출하고 요약을 직렬화한다.
   내부 tree cache hit은 clone, miss는 tree 생성을 포함한다. 이 WASM 내부에서 tree/이미지/직렬화
   각각의 비중까지 측정한 것은 아니다.

따라서 이 표본의 큰 멈춤은 렌더 큐 이전에 이미 발생한다. 기존 page scheduler의 soft budget이나
yield만 조정하면 이 앞단 동기 호출을 분할하지 못한다. 개별 raster 134.3ms도 별도로 존재해,
예산 준비를 분리해도 모든 버벅임이 사라진다고 약속할 수 없다.

## 4. 재방문과 한계

같은 후보 문서 정착 후 현재 구간 기록 → 처음으로 → 다음 행 5회. layerCount 152회는 모두
시계 해상도상 0ms, budget.refresh 최대 0.4ms였다. 하지만 **main raster는 26회/총 1190.3ms**,
LRU cumulative eviction은 5→21, misses는 22→38이었다. 이 34%/고정 네 열/작은 viewport는
사용자 28.13% 표본의 무래스터 재방문과 다르다. tree metadata warm과 Canvas surface warm은
별개이며, 캐시 thrashing 가능성은 여전히 남긴다. 재방문 전체가 무비용이라고 주장하지 않는다.

세 기록 모두 stopped·knownWorkReadyAtStop=true·errors=[]·dropped=0. 최소 재검토 근거는
[smoke-summary.json — 로컬 보존 안내](assets/issue6040-post-stack/README.md)에 환경·전체 boundary
counter·긴 예산 구간의 중첩 집계·page 비용·LRU 상태만 남겼다. 전체 DOM·문서 내용·개별 frame
원시는 추가하지 않았다. JSON의 sums는 서로 중첩된 inclusive 값이며 전체 합을 계산하면 안 된다.

## 5. 검증과 다음 결정

- wrapper/capability/session focused **23 pass**.
- Studio 전체 **1561 pass, 1 skip, 0 fail**.
- 작업 후보 및 구 기준선 TypeScript 검사 통과, git diff --check 통과.
- 실브라우저 before/after cold, after warm의 상세 capability·오류·누락 확인.
- 인앱에서 관찰 off → 이동 → on → 이동 모두 오류 0. 검증용으로 만든 두 탭만 닫고 기존 사용자
  탭과 서버는 보존했다. 이는 wrapper 기능 검사이지 on/off 성능 오버헤드 계측은 아니다.
- Rust/product source 변경 없음. 기존 WASM 재사용이며 Rust lint/rebuild 생략.
- 추가 wrapper 관찰 오버헤드는 정량 미측정. 이 수치를 기존 기본 계측과 직접 개선율로 비교하지 않는다.

다음은 **budget planning에서 cold metadata 준비를 어떻게 분리할지 설계**하는 단계다.
1) 경량 summary API/기존 metadata 재사용 가능성, 2) cold 준비를 page 작업 단위로 옮길 때
실제 layer 예산을 모르는 상태의 안전한 예약·실패·재입력 계약을 비교해야 한다. 임의 layer 수를
과소 추정하거나 전체 문서를 미리 준비하는 방식은 채택하지 않는다. 아직 보정 방향을 확정하지
않았으며 새 구조 이슈·Worker·타일 구현이나 B 완료 판정으로 넘어가지 않는다.

Firefox 587ms의 내부 비중 자체를 확정하려면 동일 옵션으로 짧은 cold 한 번이 추가로 필요하다.
다만 양쪽 인앱 재현으로 source 경로의 cold 병목은 확인했으므로 사용자 수집을 반복 요구하기 전에
이 근거로 보정 범위와 별도 이슈 여부를 검토할 수 있다. 제품 변경 전/후 검증은 별도로 필요하다.
