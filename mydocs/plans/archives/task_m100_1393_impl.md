# Task M100 #1393 구현계획서 — 표 pageBreak 게이트 동승

- 수행계획서: `mydocs/plans/task_m100_1393.md` (승인 완료)
- 브랜치: `local/task1393`
- 작성일: 2026-06-14
- 단계: 2단계

## 0. 사전 조사 확정 (수행계획서 측정 완료)

- serializer 방출은 PR #1405(`ad55059f`) 정정 완료 — 전수 43파일 pageBreak 멀티셋
  불일치 0. 매핑 정합(CELL↔RowBreak, TABLE↔CellBreak) 확인.
- `diff_documents`는 표 `page_break` 미비교 (사각) — 게이트 동승만 남음.
- 비교 지점: `diff_paragraph_char_shapes`의 Table arm (`roundtrip.rs:909`). 셀 문단
  재귀가 셀 내 중첩 표까지 들어가므로, Table arm에 page_break 비교 1줄 추가 시
  최상위·중첩 표 모두 동승.

## 1단계 — 게이트 동승 + 테스트

### 1.1 variant

- `IrDifference::TablePageBreak { section, paragraph, path, detail }` 추가
  + Display (`…tbl page_break: expected={:?} actual={:?}`).

### 1.2 비교 추가

- Table arm(char_shapes 재귀)에서 `ta.page_break != tb.page_break` 시 push.
  path는 `…/ctrl[{ci}]tbl`. (linesegs 재귀 쪽은 page_break 무관 — char_shapes 한 곳.)

### 1.3 단위 테스트

- page_break 변형 주입(RowBreak vs CellBreak) → `TablePageBreak` 검출.
- 실샘플 form-002 roundtrip → 게이트 0 (이미 보존).

## 2단계 — 전수 검증 + 문서

1. `hwpx-roundtrip --batch samples/hwpx` 전수 → `output/poc/task1393/`
   (page_break 보존 + 게이트 동승 후 IR_DIFF 0)
2. baseline (B=0 유지, 신규 xfail 0) + CI급 (release-test + fmt + clippy)
3. 매뉴얼 갱신 (#1393 해소 + 게이트 항목)
4. 최종 보고서 (PR #1405 방출 해소 + 본 타스크 게이트 동승 역할 분리 기록)

## 위험 관리

| 위험 | 단계 | 대응 |
|------|------|------|
| 게이트 동승이 숨은 차이 노출 | 1·2 | 전수 멀티셋 0 확인 완료 — 신규 xfail 0 예상, 배치 재확인 |
| 중첩 표 누락 | 1 | 셀 문단 재귀가 중첩 표 page_break 도달 — Table arm 1지점으로 충분 |
| 방출 회귀와 충돌 | — | 방출 무변경, 게이트만 추가 |
