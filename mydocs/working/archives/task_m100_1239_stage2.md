# Stage 2 완료 보고서 — Task #1239: 인라인 수식 줄 배정 수정

- **이슈**: #1239 (M100)
- **브랜치**: `feature/issue-1239-equation-multiline-merge`
- **단계**: Stage 2 / 3
- **작성일**: 2026-06-02

## 수정 내용

`src/renderer/layout/paragraph_layout.rs`:

- `equation_only_tac_line_assignment()` 추가 — 모든 줄이 수식만(빈 runs)이고 char_start 가
  비구분(연속 동일/감소)일 때, **같은 char_start 의 줄들에 같은 char position 의 연속 TAC 를
  순서대로 분배**해 `tac_idx → line_idx` 매핑 생성.
- TAC 줄 배정(`tac_on_line`)을 기존 #1221 의 줄수==tac수 1:1 (`index_based_tac`) 에서, 위
  분배 매핑(**m:n 일반화**)으로 교체. None(비대상) 이면 기존 char 기반 유지.

→ 연속 인라인 수식(사이 텍스트 char 없음)이 한컴 LINE_SEG 로 별도 줄에 배치된 경우(문20
"S=…=…=…"), 같은 position 의 두 수식이 같은 줄로 병합되던 문제 해소.

## 검증

| 항목 | 결과 |
|------|------|
| 문20 S= 블록 | 병합 해소 → **5줄 분리** (PDF 정합), 빈 공백 제거 |
| 시각 | `output/poc/task1239/mun20_after.png` |
| 골든 스냅샷 | 8 passed |
| **전체 `cargo test`** | **1933 passed, 0 failed** |

#1221(셀 z-표 1:1)은 분배 매핑의 특수 케이스로 동작 보존(전체 통과 확인).

## 다음 단계

Stage 3: 인라인 수식 다(多)문서 시각 점검(exam 등) + 최종 보고서.
