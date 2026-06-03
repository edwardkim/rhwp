# Task #962 Stage 5 — 시각 검증 + 최종 작업 정리

## 1. 시각 검증 (작업지시자 확인)

- 시험지 page 2 문14 <보기> textbox:
  - Before: ㄱㄴㄷ + 본문 + 수식이 시각상 overlap (duplicate emit)
  - After: ㄱ. h(1)=3 / ㄴ. 함수 h(x)는... / ㄷ. 함수 g(x)가... 정상 표시 ✓ 한컴 PDF 정합
- 문12, 문15 의 textbox content 도 정상 표시 ✓

## 2. 최종 변경 요약

### 2.1 코드 변경
`src/renderer/layout/shape_layout.rs:1609-1675`:
- **Fix B**: 두번째 loop 의 Equation branch 에서 paragraph_layout 등록 확인 후 emit
- `tree.get_inline_shape_position` 으로 cell_ctx 매칭 — 등록된 경우 inline_x advance + emit skip
- 미등록 (legacy fallback) 시 기존 emit 분기 유지

### 2.2 문서 추가
- `mydocs/plans/task_m100_962.md` (수행 계획서)
- `mydocs/plans/task_m100_962_impl.md` (구현 계획서)
- `mydocs/plans/task_m100_962_impl_v2.md` (Fix B 구현 계획)
- `mydocs/working/task_m100_962_stage1.md` (Root cause)
- `mydocs/working/task_m100_962_stage4.md` (회귀 검증)
- `mydocs/working/task_m100_962_stage5.md` (시각 검증)
- `mydocs/report/task_m100_962_report.md` (최종)
- `mydocs/orders/20260517.md` 갱신

## 3. PR 구성

samples/pdfs 는 PR #956 등 이전 PR 에서 추가되므로 **본 PR 에 포함 안 함**.
layout.rs 변경은 PR #958/#961 영역 — 본 PR 에 포함 안 함.

- base = upstream/devel
- head = jangster77:local/task962
- 변경 단일 파일: `src/renderer/layout/shape_layout.rs`
