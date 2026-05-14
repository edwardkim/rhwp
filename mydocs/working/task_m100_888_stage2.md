# Task #888 Stage 2 커밋 준비 보고서

## 1. 승인 상태

Stage 1 자동 검증 보고서를 작업지시자가 승인했다.

## 2. 정리 작업

### 2.1 `section.rs` diff 축소

초기 이식 후 `src/parser/hwpx/section.rs` 에 `rustfmt` 기반 포맷 변경이 많이 섞여 있었다.

정리:

- `section.rs` 를 최신 `local/devel` 상태로 되돌림
- HWPX `pageBorderFill` 파싱과 `PageBorderFill` import 만 재적용
- diff 규모: `531 lines` 수준에서 `130 lines` 수준으로 축소

### 2.2 테스트명 정리

#888에서 추가한 회귀 테스트는 `task888_*` prefix 로 정리했다.

대상:

- `task888_basic_table_materializes_hancom_table_attrs`
- `task888_expense_report_materializes_tac_table_ctrl_attrs`
- `task888_expense_report_normalizes_transparent_paragraph_border_fill`
- `task888_expense_report_parses_page_border_fills`
- `task888_expense_report_page_border_fills_survive_hwp_save_reload`

## 3. 최종 자동 검증

```text
cargo test --test hwpx_to_hwp_adapter
```

결과:

```text
30 passed; 0 failed
```

```text
cargo test --lib --quiet
```

결과:

```text
1246 passed; 0 failed; 2 ignored
```

```text
git diff --check
```

결과: 오류 없음.

## 4. 커밋 후보 파일

코드:

- `src/document_core/converters/common_obj_attr_writer.rs`
- `src/document_core/converters/hwpx_to_hwp.rs`
- `src/parser/hwpx/section.rs`

테스트/샘플:

- `tests/hwpx_to_hwp_adapter.rs`
- `samples/hwpx/basic-table-01.hwpx`
- `samples/hwpx/expense_report.hwpx`

문서:

- `mydocs/plans/task_m100_888.md`
- `mydocs/working/task_m100_888_stage0.md`
- `mydocs/working/task_m100_888_stage1.md`
- `mydocs/working/task_m100_888_stage2.md`

## 5. 다음 단계

이 상태로 중간 커밋을 생성한다. 이후 작업지시자 시각 판정 결과가 추가되면 Stage 3 문서에 반영하고 PR 또는 `local/devel` 반영 절차를 진행한다.
