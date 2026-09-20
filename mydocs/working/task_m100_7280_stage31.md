# Task #7280 Stage 31 — R2ac 표 소유 문단 흐름 조정 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2ab](task_m100_7280_stage30.md), 시작 head `d32e1df64`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: 구현 완료, 고정 제품 SHA 집중 검증 준비.

## 1. 책임과 보존 범위

`typeset_table_paragraph`의 진입·컨트롤 순회·후처리를
`controls/paragraph_flow.rs`로 옮긴다. 부모는 기존 시그니처와 입력을 유지하는 얇은 진입점이다.
`TableParagraphInput`은 원본 문단·구성/측정 결과·스타일·전체 문단 슬라이스를 빌린다.
원래 사용하지 않던 `_page_def`는 부모 호환 시그니처에만 남긴다.
새 public API·IR·trait·범용 rule engine은 없다.

이 조정자는 기존 엔진의 포맷과 배치 메서드를 호출한다. 엔진 의존을 제거했다고 주장하지 않는다.
기존 순수 조회와 가변 배치 담당자를 연결하는 구조 이동이며, 포맷/분할/각주 본체는 별도 책임으로 남는다.

- 진입 진단은 배타 영역 소비보다 먼저 수행한다. `RHWP_DIAG_TAC` 조회 조건·로그 인자도 같다.
- `state::prepare_table_paragraph_column`은 기존 `apply_visible_float_exclusions(0.0)` 뒤
  현재 단 너비를 읽는다. width fallback과 평가 순서를 바꾸지 않는다.
- 포맷 → stored TAC 조기 반환 시도 → 일반 TAC 준비 → `ensure_page` 순서를 보존한다.
- `table_paragraph_flow_position`은 그 뒤의 높이/완료 쪽 수를 읽는다. 원래 연속으로 같은
  `current_height`를 읽던 `height_before`/`para_start_height`는 동일 값이다. 사이에 쓰기가 없으므로
  한 번 읽은 높이를 두 변수로 사용해도 문단 시작점과 정산 기준은 같다.
- 컨트롤 순서, 장식 표의 continue, 일반 표의 host 텍스트 소유권 표시,
  배치 → 각주 → break, 비표 컨트롤 배치를 그대로 연결한다.
- 문단 단위 float lane과 host 텍스트 플래그의 초기화 범위는 유지한다.
- 루프 뒤 host 텍스트 발행 → TAC 높이 정산 순서를 유지한다. 마지막 조건의 쪽 수는
  기존 state 읽기 전용 메서드로 같은 시점에 조회한다. 앞 조건의 short circuit은 그대로다.
- profile 조회 callback은 미리 계산하지 않고 원래 호출 지점에서 평가한다.

기존 분기·수치·알고리즘 타당성은 변경하거나 새로 승인하지 않는다.
Rust 테스트·assertion·baseline·golden·ignore·CI는 변경하지 않는다.

## 2. 검증 계획과 한계

`output/7280/stage31/verify-paragraph-flow.mjs`는 이동한 coordinator에 state 식을 다시 펼쳐
시작 head의 전체 문단 메서드와 대조한다. 부모 입력 연결, 나머지 부모/기존 state·controls 불변,
진단·너비 소비/관측 순서와 높이 snapshot의 식을 확인한다.
정적 대조는 실행 증명이나 출력 시각 판정을 대체하지 않는다.

별도 review worktree에서 suite 준비 → 고정 baseline 대비 manifest/unit-tier → fmt →
native Clippy → R2ab와 동일한 313건 집중 nextest를 순차 수행한다.
기존 선택은 stored TAC/일반 TAC/float/장식 표/혼재 컨트롤/각주 계약을 포함하지만,
모든 분기 조합을 실행했다는 의미는 아니다. 원본 사례 검사와 합성·변형 계약의 한계는
[R2aa](task_m100_7280_stage29.md#2-검증-계획과-한계)와 R2ab 기록을 따른다.

전체 회귀·WASM/workspace lint·workspace build·Native Skia·fresh Docker WASM 및 직접 시각 대조는
구현계획 §7의 통합 게이트에 남긴다. 원격 push·PR·댓글은 이번 승인 범위가 아니다.

## 3. 고정 head 검증

검증 완료 후 제품 SHA·명령·결과·증적을 기록한다.

## 4. 후속

남은 개별 지연 표 배치 조정과 부모 문단/컨트롤 진입점의 경계를 점검한다.
표 포맷·분할·이어받기 본체와 각주 등록 알고리즘, 최종 상태 캡슐화·통합 검증은 남아 있다.
