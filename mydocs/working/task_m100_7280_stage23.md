# Task #7280 Stage 23 — R2u 빈 호스트 float 표 조회·예약 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2t](task_m100_7280_stage22.md), 시작 head `705cb9a90`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R2u 구조 이동 완료, 고정 제품 SHA 집중 검증 예정.

## 1. 책임 경계와 보존 계약

`controls/empty_float.rs`가 빈 호스트 문단의 float 표 수용 여부와 수평 범위·예약 높이를
조회한다. 필요한 페이지 관측값과 읽기 전용 lane만 받고, 엔진이나 가변 TypesetState를 받지 않는다.
`EmptyFloatPlacement`는 확정할 네 값만 전달하며 typeset 내부에서만 사용한다.

`controls::try_place_empty_para_float_table`은 조회 성공 시 상태 명령을 호출한다.
`state.rs`는 페이지 관측값·각주 반영 예산 조회와 **항목 추가 → lane 예약 → 흐름 높이 갱신**을
담당한다. 이 순서를 새로운 일괄 트랜잭션이나 다른 계산법으로 바꾸지 않는다.

### 실제 호출과 소비

- 기존 `try_typeset_empty_para_float_table` 진입점과 일반 표 문단의 호출·실패 폴백은 보존한다.
- 빈 문단, TopAndBottom/Square, 형제 수, 선언 크기, 저장 앵커, profile 분기를 그대로 옮긴다.
  기존 heuristic의 타당성 재승인이나 수정은 하지 않는다.
- 페이지 snapshot은 읽기 전용 필드 관측뿐이며, 각주 예산은 closure로 **기존 수평 범위 계산 뒤**에
  조회한다. 조기 탈락한 후보는 예산 함수를 호출하지 않는다.
- 현재 단 영역, 본문·용지 너비, 문단 들여쓰기/여백으로 수평 범위를 계산한다.
  저장 vpos/문단 원점 선택, 형제 block 표의 점유와 lane 소유 판별, 예약 높이와 0.5px 비교를 보존한다.
- `None`만 기존 `false`에 대응한다. 형제 탐색 closure의 `false`는 그대로 유지한다.
- 확정 때 `lanes.place`를 원래처럼 호출한다. query가 계산한 `lane_top`을 원점으로 재사용하지 않고
  원래 `raw_top`을 전달하므로 lane 내부 밀기 계산 횟수와 의미도 보존한다.
- 기존 저장 앵커 helper와 선언 높이 판별 helper는 부모/공통 모듈에 남는다.
  부모 의존 전체 해소, 모든 float 경로 이동 또는 상태 필드의 완전 캡슐화는 이번 종료 조건이 아니다.

IR/API, 조판 산식·상수·평가 순서, 테스트 source/assertion/ID, baseline/golden/ignore와 CI 정책은
바꾸지 않는다. 일반 block 표·분할·각주 등록 및 다른 컨트롤 배치 경로도 그대로 둔다.

## 2. 검증 계획과 한계

`output/7280/stage23/verify-empty-float.mjs`가 query·예산·상태 명령을 원래 함수 본문으로
복원해 비교한다. coordinator 인자, 반환 필드, 기존 부모 경로/테스트/상태 명령 불변도 확인한다.
정적 복원 비교는 실행 검증을 대체하지 않는다.

별도 review worktree에서 파생 suite 준비, 고정 baseline 대비 manifest/unit-tier 검사,
fmt, native Clippy, 집중 nextest를 순차 실행한다. R2t 230건에 다음 기존 계약을 추가한다.

- #6946 5건: 앞 형제의 block 배치와 lane 소유, 표 겹침/본문 경계.
- #7203 두 모듈 8건: 저장 앵커 좌표, 비대칭 바깥여백, TAC/offset 반례, 후속 표 경계.
- #5585 형제 앵커 2건: 실제 문서의 형제 오프셋 중복 소비와 작은 꼬리 조각 보호.

모든 입력 조합·분기 실행을 새로 추적하지 않는다. 기존 계약의 통과를 직접 시각 판독이나
한컴 전체 일치로 격상하지 않는다. 전체 회귀·WASM/workspace lint·workspace build·Native Skia·
fresh Docker WASM 및 직접 시각 대조는 통합 게이트에 남긴다. 원격 push·PR·댓글은 수행하지 않는다.
