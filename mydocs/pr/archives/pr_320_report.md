# PR #320 최종 보고서 — Task #313/#314/#317/#318 (Epic #309 마무리 + 회귀 수정)

## 결정

✅ **Merge 승인 (admin merge)**

## PR 정보

- **PR**: [#320](https://github.com/edwardkim/rhwp/pull/320)
- **이슈**: #313, #314, #317, #318 (모두 CLOSED)
- **작성자**: @planet6897
- **처리일**: 2026-04-25
- **Merge commit**: `d2d34fd`

## 처리 절차

1. ✅ PR 브랜치 체크아웃 (`task318`)
2. ✅ devel 머지 → `mydocs/orders/20260425.md` 1구간 충돌 해결
3. ✅ 검증: 992 lib + 25 어댑터(0 ignored) + issue_301 1 + svg_snapshot 6 + tab_cross_run 1 모두 통과
4. ✅ 4샘플 페이지 수 확인: 21_언어=15 (PDF 일치), exam_math=20, exam_kor=24, exam_eng=9
5. ✅ Task #291 (어제 핀셋) 효과 보전: KTX.hwp pi=31/32 = 518.16/517.95 유지
6. ✅ WASM Docker 빌드 → 브라우저 시각 검증 (작업지시자 판정 성공)
7. ✅ `planet6897/task318` 에 push (`96ce6d6..9bb593b`)
8. ✅ 재승인 + admin merge → 이슈 #313/#314/#317/#318 모두 CLOSED

## 승인 사유

1. **#316 의 z-table 회귀 정식 수정** — `tests/issue_301.rs` `#[ignore]` 제거 후 통과
2. **어댑터 격리 회수** — `hwpx_to_hwp_adapter` 25/0/0 (3 ignored 회수)
3. **Epic #309 핵심 목표 달성** — 21_언어 19→15쪽 (PDF 정확 일치)
4. **#301 가드 패턴 일관 적용** — `already_rendered_inline`, `is_wrap_host`
5. **Task #291 효과 보전** — KTX.hwp 표 좌표 그대로 유지
6. **CLAUDE.md 절차 완전 준수** — 4 task 모두 계획서/보고서 완비

## 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ 992 passed / 0 failed / 1 ignored |
| `cargo test --test issue_301` | ✅ **1 passed (z-table 회귀 해소)** |
| `cargo test --test hwpx_to_hwp_adapter` | ✅ **25 / 0 / 0** (격리 회수) |
| `cargo test --test svg_snapshot` | ✅ 6 passed (golden 유지) |
| `cargo test --test tab_cross_run` | ✅ 1 passed |
| `cargo clippy / wasm32 check` | ✅ clean |
| CI (원본) | ✅ 전부 SUCCESS |
| 4샘플 페이지 수 | ✅ 21_언어=15 / exam_math=20 / exam_kor=24 / exam_eng=9 |
| WASM Docker 빌드 + 브라우저 시각 | ✅ 작업지시자 판정 성공 |

## 변경 내역 (4개 task 통합)

### Task #313 — TypesetEngine main 전환
- `src/document_core/queries/rendering.rs::paginate()` — Paginator → TypesetEngine default
- `RHWP_USE_PAGINATOR=1` env fallback
- TYPESET_VERIFY 검증 코드 제거

### Task #314 — HWPX 어댑터 normalize (부분 완료)
- `src/document_core/commands/document.rs::normalize_hwpx_paragraphs` 함수 추가
- char_shapes 빈 → default `[(0, 0)]`
- control_mask 재계산
- 셀 paragraphs 재귀 처리

### Task #317 — 어댑터 +1쪽 잔존 origin 보강
- `src/document_core/converters/hwpx_to_hwp.rs::adapt_table`
- raw_ctrl_data attr 영역(offset 0..4) 0 강제
- `table.attr=0` 보존
- typeset is_tac 비대칭 해소

### Task #318 — issue_301 회귀 수정 (가장 중요)
- `src/renderer/layout/table_partial.rs:766` — `already_rendered_inline` 가드 (#301 동일 패턴)
- `src/renderer/layout.rs::layout_column_item` — PartialParagraph 분기에 `is_wrap_host` 가드 (FullParagraph 동일 패턴)
- `tests/issue_301.rs` `#[ignore]` 제거

## Epic #309 — 마무리

본 PR 머지로 Epic #309 의 모든 sub-issue (#311, #312, #313, #314, #317, #318) 완료. **21_언어 PDF 정확 일치 (15쪽) 달성**.

## 후속 사안 (작성자 명시)

> TypesetEngine 이 wrap=Square 호스트 paragraph 에 대해 PartialParagraph 를 emit 하는 동작이 정상인지 재평가. 현재 layout 측 가드로 회피하지만, 의도가 명확하면 PartialParagraph 자체를 emit 하지 않는 것이 더 깔끔.
