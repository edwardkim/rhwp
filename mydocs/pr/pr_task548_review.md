# Task #548 핀셋 cherry-pick 리뷰 — 셀 inline TAC Shape margin_left + indent

**원 PR**: [#551 (closed)](https://github.com/edwardkim/rhwp/pull/551) (참고용 — 본 cherry-pick 은 별도 사이클)
**작성자**: @planet6897 (Jaeuk Ryu) — `9dc40ddb` Task #544 v2 Stage 3 (Phase C #548)
**처리 결정**: ✅ **Task #548 단독 cherry-pick + test fixup**
**작성일**: 2026-05-04
**관련 이슈**: #548 (재오픈 → closes)

## 1. 본질

페이지 8 보기 표 (pi=167) 셀 5 (3-col 병합 본문 셀) 의 첫 줄 시작에 있는 [푸코] inline rectangle Shape (treat_as_char=true) 의 좌측 위치가 cell 좌측 가장자리 (x=131.04 px) 에 렌더 → PDF (한컴 2010, x≈155.6 px) 와 -24.6 px 시프트.

**root cause**: `src/renderer/layout/table_layout.rs` Shape 분기의 `inline_x` 산출에 paragraph 의 `margin_left + first_line_indent` 미반영.

| 경로 | x 산식 | 셀 5 line 0 결과 |
|------|--------|----------------|
| paragraph_layout (텍스트 경로) | col_x + effective_margin_left + ... | 131.04 + 24.56 = 155.60 ✓ |
| table_layout (shape 경로, 수정 전) | inner_area.x | 131.04 ✗ |
| table_layout (수정 후) | inner_area.x + line_margin | 155.60 ✓ |

## 2. 본 devel 측정 (현재 발현)

페이지 8 셀 5 line 0 [푸코] rect:
- **x=131.04** px (PDF 기대값 155.6 와 -24.6 px 시프트) ❌
- y=698.43, w=30.23, h=18.89

→ 이슈 #548 본문의 broken state 와 정확 일치. 본 devel cherry-pick 필요.

## 3. cherry-pick 대상

`9dc40ddb` (Task #544 v2 Stage 3 — Phase C #548 from @planet6897 fork)

**변경**:
- `src/renderer/layout/table_layout.rs` (+41 LOC):
  1. `effective_margin_left_line` 헬퍼 추가 (paragraph_layout 의 line_indent 산식과 동일)
  2. `inline_x` 산출 3 분기 (Left/Justify) 에 `line_margin` 가산 (Picture target_line reset, Shape target_line reset, paragraph 시작)
  3. `para_margin_left_px` / `para_indent_px` 추출 (ParaShape 에서)
- `src/renderer/layout/integration_tests.rs` (-1 LOC):
  - `test_548_cell_inline_shape_first_line_indent_p8` 의 `#[ignore]` 제거 → 활성화

## 4. cherry-pick 절차 + conflict resolution

```bash
git checkout -b local/task548 devel
git cherry-pick 9dc40ddb
# table_layout.rs Shape 분기 (line 1648-1693): incoming 채택
# integration_tests.rs: auto-merge
```

**conflict resolution**:
- `src/renderer/layout/table_layout.rs` Shape 분기 — HEAD 빈 영역 + incoming block 삽입. surrounding context 불일치로 auto-merge 실패. incoming 채택 (의미는 단순 삽입).
- 다른 파일은 auto-merge 정상.

## 5. test fixup

contributor fork 의 test_548 가 y 범위 [685, 690] 검사. 본 devel 측정 y≈698.43 (Task #479 미적용 모델로 셀 y 위치 차이) → y 범위 [690, 710] 으로 조정.

```rust
// 본 devel 의 #479 미적용 trailing-ls 모델로 셀 y 위치가 컨트리뷰터 fork
// (y≈685-690) 대비 다름. 본 devel 측정 y≈698.43 → 범위 [690, 710] 으로 조정.
if (w - 30.23).abs() < 0.5
    && (h - 18.89).abs() < 0.5
    && y > 690.0 && y < 710.0
```

## 6. 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib --release` | 1121 passed / 0 failed / 2 ignored — baseline +1 GREEN (test_548) |
| `cargo clippy --release --lib` | 신규 결함 0건 (pre-existing 2건 동일 baseline) |
| 페이지 8 [푸코] rect | x=131.04 → **155.60** (PDF 155.6 ±0.0) ✅ |
| 광범위 sweep (6 샘플 73 페이지) | 13 differ / 60 byte-identical |

**광범위 차이의 본질** (sample exam_kor_005):
- 셀 안 작은 box 의 shape rect + text +2 px 시프트 (셀 paragraph 의 margin/indent 적용)
- visible-stroke + paragraph margin 이 있는 셀 안 inline TAC Shape 만 영향

## 7. 작업지시자 결정 사항

1. cherry-pick 채택 (`9dc40ddb` + test fixup)
2. 작업지시자 시각 판정 (페이지 8 [푸코] + 광범위 sweep)
3. 통과 시 devel merge / push / 이슈 #548 close + 새 PR 등록
4. 본 cherry-pick 사이클은 PR #551 잔존이지만 `feedback_no_pr_accumulation` 메모리 적용 → **새 PR 등록**

## 8. 메모리 룰 정합

- [feedback_no_pr_accumulation] — 본 cherry-pick 은 PR #551 잔존이지만 새 PR/이슈 (Task #548) 로 등록
- [feedback_pdf_not_authoritative] — 한컴 2010 PDF 정합 + 작업지시자 시각 판정 게이트
- [feedback_essential_fix_regression_risk] — 6 샘플 73 페이지 광범위 회귀 검증 (13 differ, 본질 분석 완료)
- [feedback_rule_not_heuristic] — `effective_margin_left_line` 단일 룰 (paragraph_layout / table_layout 동일 산식)
- [feedback_visual_regression_grows] — 작업지시자 직접 시각 판정 게이트
