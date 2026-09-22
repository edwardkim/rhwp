# Task #7280 Stage 42 — R3a 표 측정 결과 수용 판단 분리

- Issue: #7280. 이전: [Stage41](task_m100_7280_stage41.md).
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md) R3, §7.1.
- 시작 head: `df7d96038`. 고정 동작 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: 구현·집중 검증 진행 중. R3 전체 또는 제출 게이트 완료가 아니다.

## 이번 경계

`format_table`의 측정 결과 선택을 `table::fit_measured_for_host` Query로 분리한다.
입력은 기존 Paragraph/Table/MeasuredTable 참조, dpi와 읽기 전용 profile 조회다.
전체 TypesetEngine/TypesetState나 가변 문서 접근을 전달하지 않는다.
profile은 기존 조건식에서 같은 순서로 지연 조회하고 측정 helper 호출 순서를 유지한다.
원래 측정값 fallback, host 간격, 각주 수집과 FormattedTable 조립은 호출자에 남긴다.

이 작업의 독립 기준은 고정 baseline의 동작과 기존 정식 회귀 계약이다. 선언 높이 fit,
빈 호스트의 중첩/선언 꼬리 보정 등 기존 호환 조건을 이동하지만 새 조판 규칙으로 승인하지 않는다.
HWP/HWPX IR, 계산 상수·비교 연산·clone 의미와 조건의 단락 평가를 변경하지 않는다.

소비 경로: `format_table`의 원본 측정 조회 → Query의 보정 후보 →
`fitted_visible_mt.as_ref().or(mt)` → FormattedTable의 행/누적/전체 높이 →
`typeset_block_table_inner`의 fit·행 도메인 준비 → `scan_block_table_split_rows`의 컷/예약 →
기존 PageItem/continuation 배치다. 뒤 단계의 원본/유효 표 선택·높이 보정은 그대로 남긴다.
이번 절편은 시작/끝 컷·누적 예약·빈 물리 밴드·이어받기 종료 알고리즘을 수정하지 않는다.

## 검증 계획

원본 추출식과 새 Query를 dpi/profile 표기만 정규화해 대조하고 나머지 parent가 불변인지 검사한다.
기존 표 fit·성장·꼬리·TAC/float 회귀를 별도 integration suite에서 실행한다.
새 source-side test/support와 public API는 추가하지 않는다. baseline·ignore도 변경하지 않는다.
review worktree에서 suite 준비·고정 baseline 정책 검사·fmt·native Clippy·집중 nextest를 순차 실행한다.
정적 대조는 모든 분기 실행이나 한컴 시각 일치의 증거가 아니다.

§7.2의 R3 책임 묶음 전체 회귀·Native/fresh WASM 시각 비교, §7.3의 최종 제출 lint/build
묶음은 후속 게이트로 남긴다. 이번 절편에서 원격 push·PR·댓글은 수행하지 않는다.
