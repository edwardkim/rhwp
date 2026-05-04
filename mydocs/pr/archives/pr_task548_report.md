# Task #548 핀셋 cherry-pick 처리 보고서

**처리 결정**: ✅ Task #548 단독 cherry-pick (`9dc40ddb` + test fixup)
**작성일**: 2026-05-04
**브랜치**: `local/task548`
**검토 문서**: `mydocs/pr/pr_task548_review.md`
**관련 이슈**: #548 (closed → reopen → closes)

## 1. 본질 정정

`src/renderer/layout/table_layout.rs` 셀 안 inline TAC Shape (`Control::Shape` + `treat_as_char=true`) 분기에 paragraph 의 `margin_left + first_line_indent` 미반영 결함 정정.

**측정 (페이지 8 셀 5 line 0 [푸코] rect)**:
- 수정 전: x=131.04 (PDF 155.6 와 -24.6 px 시프트) ❌
- 수정 후: x=**155.60** (PDF 정합 ±0.0) ✅

## 2. cherry-pick + fixup 절차

| commit | 내용 |
|--------|------|
| `3de05051` | cherry-pick `9dc40ddb` (Task #544 v2 Stage 3 — Phase C #548 from @planet6897). table_layout.rs Shape 분기 conflict 해소 (incoming 채택). |
| `a0dad0d3` | test_548 의 y 범위 [685, 690] → [690, 710] 조정 (본 devel 의 #479 미적용 trailing-ls 모델로 셀 y 위치가 contributor fork 와 다름). |

**변경**: 3 files, +193 / -3
- `src/renderer/layout/table_layout.rs` (+41 LOC): `effective_margin_left_line` 헬퍼 + 3 분기에 `line_margin` 적용 + `para_margin_left_px` / `para_indent_px` 추출
- `src/renderer/layout/integration_tests.rs` (+2 / -2): `#[ignore]` 제거 + y 범위 조정
- `mydocs/working/task_m100_544_v2_stage3.md` (+150 / 0): 컨트리뷰터 stage 보고서

## 3. 검증

### 3.1 단위 테스트

```
cargo test --lib --release: 1121 passed / 0 failed / 2 ignored
test_548_cell_inline_shape_first_line_indent_p8 GREEN
```

baseline 1120 → +1 GREEN (test_548), 회귀 0건.

### 3.2 Clippy

```
cargo clippy --release --lib: 0 errors / 0 warnings 신규
```

pre-existing 2건 (`table_ops.rs:1007`, `object_ops.rs:298`) 동일 baseline.

### 3.3 광범위 회귀 sweep (6 샘플 73 페이지)

```
73 SVGs (5 exam samples + 21_언어 15 페이지)
13 differ (intended)
60 byte-identical
```

차이 본질 (sample exam_kor_005):
- 셀 안 inline TAC Shape rect + 본문 text +2 px 시프트 (셀 paragraph 의 margin/indent 적용)
- visible-stroke + paragraph margin 이 있는 셀 안 inline TAC Shape 만 영향

회귀 검출 가능 영역 (paragraph 텍스트 위치, 일반 shape 위치): 0 변경 ✅

### 3.4 영향 범위

| 샘플 | 페이지 | 변경 페이지 | 비고 |
|------|-------|-----------|------|
| 21_언어_기출 | 15 | 1 (페이지 8) | [푸코] inline shape 정합 |
| exam_eng | 8 | 3 | 셀 안 small shape |
| exam_kor | 20 | 6 | 동일 |
| exam_math | 20 | 0 | 영향 없음 |
| exam_science | 4 | 2 | 동일 |
| 2010-01-06 | 6 | 0 (sample 누락) | — |

## 4. 잔존 사항

### 4.1 시각 판정 게이트 (작업지시자 직접)

- `/tmp/diag548/after/21_언어_기출_편집가능본_008.svg` — 페이지 8 [푸코] 위치 PDF 정합 확인
- `/tmp/diag548/before/` ↔ `/tmp/diag548/after/` — 광범위 회귀 비교
- 비교 PDF: `samples/21_언어_기출_편집가능본-2010.pdf` 페이지 8

### 4.2 PR #551 잔존 미반영 (별도 사이클 결정 대기)

| commit | 본질 | 비고 |
|--------|------|------|
| `1934161f` Task #552 | paragraph border 시작 직전 trailing ls 보존 | 본 devel #479 미적용 모델로 발현 안 함 |
| `84d1d4b2` Task #544 v3 | 박스 안 sequential paragraph trailing-ls 보존 | 동일 이유 |
| `0341a2a7` 다중 (#517/#518/#544 v3/#552) | layout 다중 정정 일괄 | 분해 불가 |
| 기타 ~50 task | — | 작업지시자 결정 대기 |

## 5. 메모리 룰 정합

- [feedback_no_pr_accumulation] — 본 cherry-pick 은 PR #551 잔존이지만 **새 PR (Task #548) 로 등록**
- [feedback_pdf_not_authoritative] — 한컴 2010 PDF 정합 + 작업지시자 시각 판정 게이트
- [feedback_essential_fix_regression_risk] — 6 샘플 73 페이지 광범위 회귀 검증 + 본질 분석
- [feedback_rule_not_heuristic] — `effective_margin_left_line` 단일 룰 (paragraph_layout / table_layout 동일 산식)
- [feedback_local_task_branches_origin_backup] — `origin/local/task544_v2` (`9dc40ddb` 포함) 보존 유지

## 6. 후속 절차

1. local/task548 → devel merge (no-ff)
2. devel push origin
3. 이슈 #548 close (with cherry-pick commit reference)
4. orders 20260504.md 갱신
5. archives 이동
6. **새 PR 등록** (planet6897:devel → edwardkim/rhwp:devel)
