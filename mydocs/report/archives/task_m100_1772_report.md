# Task M100 #1772 최종 결과보고서 — HWPX 파서 표 outMargin → common.margin 동기화

## 요약

Issue #1772(11.36px 이동 군집 50건+ 등 파스 경로별 렌더 차이)의 근본 원인을
**HWPX 파서의 IR 계약 위반**(표 outMargin 을 `table.common.margin` 에 미동기화)으로
확정하고, parse_table 종료 시 동기화 4줄로 수정했다. 이슈 제목의 가설(saved-vpos
신뢰 로직 차이)은 조사 과정에서 기각되었다 — 조사 상세:
`mydocs/troubleshootings/hwpx_table_out_margin_common_margin_sync.md`.

## 변경 사항

- `src/parser/hwpx/section.rs` parse_table:
  `table.common.margin ← table.outer_margin_*` 동기화 (HWPX→HWP 어댑터
  `materialize_table_outer_margin` 와 동일 규칙 — 로드 시점에 계약 확립).
- `tests/issue_1772_table_outer_margin_sync.rs` 신설 (IR 계약 + 렌더 y 정합 2건).
- `samples/task1772/table_outer_margin_common_sync.hwpx` (결재문서 36381023) + README.
- 문서: 수행계획서(task_m100_1772.md), 조사 보고서(tech/task_m100_1772_root_cause.md).

## 검증 결과

| 항목 | 결과 |
|------|------|
| 신규 회귀 테스트 2건 (IR 계약 / 본문 첫 줄 y≈306.7 저장 lineseg 정합) | 통과 |
| 전체 `cargo test` (147 바이너리, golden SVG 포함) | 통과, 실패 1건은 기존 이슈* |
| `cargo clippy --release --lib` | 경고 없음 |
| 코퍼스 300건 render-diff(--via hwp) | OVER 13→10, 완전 해소 3, 악화 0 |
| 코퍼스 2,500건 render-diff | **OVER 90→55 (-35), PASS 2369→2404, 악화 0** |

\* `issue_852::form_01_keeps_nine_cfb_streams` — Windows CFB 경로 구분자 문제 (#1775,
origin/devel 동일 실패, 본 타스크 무관).

## 정답 근거 (한컴 정합)

재현 문서(36381023)의 저장 lineseg: 본문 pi=0 vpos=17478HU → 상단여백 75.6px +
233.0px ≈ 306.7px. 수정 전 HWPX 직파스는 295.4px(표 아래 여백 3mm 누락), 수정 후
306.7px 로 HWP5 재파스 경로·한컴 저장 위치와 일치.

## 남은 항목 (별도 이슈 권장)

1. `cell.apply_inner_margin` HWPX(false) / HWP5 재파스(true) 불일치 — 수정 후 잔여
   9.25px 군집(seoul_0601 계열 다수)의 원인. 표 높이 9.3px 차이.
2. admrul_1043 PAGE_MISMATCH — 본 수정 이후에도 잔존하는지 재확인 필요.
3. #1772 원 이슈의 나머지 카테고리(OVER 55건 잔여)는 위 1 해결 후 재집계.
