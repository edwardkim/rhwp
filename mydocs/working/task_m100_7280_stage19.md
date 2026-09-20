# Task #7280 Stage 19 — R2q 일반 TAC 문단 배치 전 판단 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2p](task_m100_7280_stage18.md), 시작 head `dfb08ead3`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R2q 구현·정적 복원 비교 완료, 고정 제품 SHA의 집중 검증 예정.

## 1. 책임과 보존 범위

저장 TAC 경로가 수용하지 않은 표 문단의 카운트·첫 줄 높이·편집 후 성장 높이·소유 줄
상자·저장 하단·리셋 전 접두 줄의 fit와 배치 전 페이지 전이 판단을 분리한다.
새로운 조판 규칙이나 예외를 추가하지 않으며 기존 heuristic의 타당성 승인도 아니다.

- `controls/tac_flow.rs`: `TacFlowQuery`가 TAC 흐름 참여와 저장 줄 소속 판별을 공유한다.
  엔진 전체 대신 DPI와 private profile 참조만 받는다. 생성 시 profile을 미리 읽지 않고
  원래 조건 위치에서 `Cell::get`을 수행하며 상태 변경 메서드는 노출하지 않는다.
- `controls/tac_fit.rs`: 좁은 페이지 관측값과 기존 `FormattedParagraph`/실측을 읽어
  카운트, 성장 높이, 페이지 전이 여부를 반환한다. 상태 쓰기는 하지 않는다.
- `controls::prepare_tac_paragraph`: 판단이 끝난 뒤 기존 `advance_column_or_new_page`를 호출한다.
- `state::tac_fit_page`: profile, 현재 높이, 저장 원점, 현재 항목의 불변 참조만 제공한다.

페이지 state profile과 엔진 profile은 기존 코드대로 구분한다. 가용 높이는 진단을 포함하므로
값을 미리 계산하지 않고 closure를 통해 원래 저장 하단 판정 및 최종 fit 위치에서 조회한다.
저장 하단 판정을 리셋 이전 높이 보정 앞으로 옮기지 않으며, `8.0` 성장 임계값·여백 계산·
부등호·find 순서·short-circuit 순서를 그대로 유지한다.

### 결과 소비와 좌표

`format_paragraph`의 줄 높이/advance와 `MeasuredTable.total_height` → `tac_fit` 판단 →
기존 페이지 전이 → `ensure_page` → 기존 컨트롤 배치 순서다.
`session_grown_tac_total`은 기존 후반부의 저장 높이 cap 하한에도 쓰이므로 결과로 함께
반환한다. 후반부에서 다시 계산하거나 선언 높이로 대체하지 않는다.
컨트롤 순서·좌표·metadata·paint 소비 경로와 cap의 기존 산식은 변경하지 않는다.

다른 기존 호출부는 엔진의 얇은 위임 메서드로 공통 TAC 조회를 사용한다. 이 위임은 후속
컨트롤 경로 분리 때 제거할 이행 경계이며 공통 판별을 복제하지 않는다.
부동/지연 컨트롤 배치와 표 컷/continuation, state의 최종 캡슐화는 이번 범위가 아니다.
IR/API, 테스트 source/assertion/ID, baseline/golden/ignore, CI 정책도 변경하지 않는다.

## 2. 검증 계획과 정적 증거

`output/7280/stage19/verify-tac-fit.mjs` / `extraction-proof.json`:

- 판단 블록을 원래 지역변수·메서드·상태 전이로 복원해 전체 계산과 평가 순서 비교 통과.
- 공유 TAC 판별 3개 함수 본문, private 관측값 매핑, 위임 및 조정 순서 비교 통과.
- 부모 파일의 나머지 경로/테스트, 기존 state 명령과 저장 TAC 조정자 불변 확인.
- 짧아진 이름에 따른 rustfmt의 단순 match/closure 중괄호 변환은 복원 후 같은 formatter로 비교.

별도 review worktree에서 파생 suite 준비 → manifest/unit-tier의 고정 baseline 비교 →
fmt → native Clippy → 집중 nextest를 순차 수행한다. 기존 typeset/composer/float_placement,
#7103/#7150/#6601/#6812/maintainer_nested_table_lines에 더해 #5700/#5807/#7049/#7062와
`tac_group_page_bottom_overflow`를 선택한다. 실제 파생 suite 소속은 준비 후 재확인한다.

기존 실물·축소 입력의 좌표 계약과 합성 경계 계약은 구분한다. 정적 동작 보존 및 기존
계약 실행이 편집 세션 성장 임계값 등 모든 분기의 실행 추적이나 한컴 직접 시각 판정은 아니다.
전체 회귀, WASM/workspace lint, workspace build, Native Skia, fresh Docker WASM/직접 시각
대조는 통합 게이트에 남긴다. 원격 push·PR·댓글은 이번 승인 범위에 포함하지 않는다.

## 3. 후속

고정 제품 head 집중 검증 결과를 기록한 후 일반 TAC/float의 컨트롤 순서와 배치 조정 분리를
계속한다. R2 전체 및 최종 통합 게이트는 아직 완료하지 않았다.
