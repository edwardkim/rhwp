# Task M100-1352 Stage 1 보고서

## 1. 범위

#1352의 표 셀 안 TAC picture + text 세로 정렬 결함을 `hy-001.hwpx`로 재현하고, 셀 안 visible text 라인에서 picture가 label 보정 때문에 아래로 밀리지 않도록 보정했다.

변경 파일:

- `src/renderer/layout/paragraph_layout.rs`
- `tests/issue_1352_table_cell_tac_picture_text.rs`
- `mydocs/plans/task_m100_1352.md`
- `mydocs/plans/task_m100_1352_impl.md`

## 2. 재현 테스트

새 테스트:

```text
tests/issue_1352_table_cell_tac_picture_text.rs
```

검증 대상:

- `samples/hwpx/hy-001.hwpx` 1페이지
- `광부` 텍스트가 있는 첫 표 셀
- 같은 셀 안의 TAC picture bbox

초기 실패 수치:

```text
cell=[y=79.35, h=41.55]
text=[y=81.23, h=37.79]
image=[y=102.56, h=37.79]
image_bottom=140.35, cell_bottom=120.89
```

즉 image가 셀 하단보다 약 19.5px 아래로 내려가 잘렸다.

## 3. 수정 내용

`paragraph_layout.rs`에 `tac_picture_label_extra_for_line` helper를 추가했다.

핵심 조건:

- `cell_ctx.is_some()`
- line에 공백이 아닌 실제 텍스트가 있음

이 조건에서는 `tac_picture_label_extra_px`를 적용하지 않는다. 한컴 PDF 기준에서 `hy-001` 첫 셀의 picture와 `광부` 텍스트는 같은 세로 위치에 있어야 하므로, label 보정은 TAC-only line 쪽에 남기고 셀 안 visible text line에서는 제한했다.

영향을 좁힌 이유:

- #974의 글상자 내부 TAC picture/space guard 보존
- #1151 계열의 셀 안 inline picture cell context 보존
- 본문/빈 TAC guide line의 기존 label 보정 보존

## 4. 검증 결과

focused:

```text
cargo test --test issue_1352_table_cell_tac_picture_text -- --nocapture
```

결과:

```text
image=[y=81.23, h=37.79]
test result: ok. 1 passed
```

관련 회귀:

```text
cargo test --test issue_1285_tac_sequence_right_align -- --nocapture
cargo test --test issue_1161_copy_picture_in_cell -- --nocapture
cargo test --lib test_hy001_textbox_inline_pictures_render_for_hwp_and_hwpx -- --nocapture
cargo build
```

결과:

- `issue_1285_tac_sequence_right_align`: 2 passed
- `issue_1161_copy_picture_in_cell`: 4 passed
- `test_hy001_textbox_inline_pictures_render_for_hwp_and_hwpx`: 1 passed
- `cargo build`: 통과

참고:

- `cargo test test_hy001_textbox_inline_pictures_render_for_hwp_and_hwpx -- --nocapture`는 통합 테스트 전체 바이너리 컴파일까지 잡혀 범위가 과도해 중단했다.
- 정확한 명령인 `cargo test --lib test_hy001_textbox_inline_pictures_render_for_hwp_and_hwpx -- --nocapture`로 재실행해 통과 확인했다.

## 5. 시각 비교

생성 산출물:

```text
output/issue1352_hy001_verify_20260618/rhwp_png/hy-001_001.png
output/issue1352_hy001_verify_20260618/pdf_png/pdf-1.png
output/issue1352_hy001_verify_20260618/compare_header_cell_crop.png
```

한컴 기준 PDF는 로컬 파일이 Git LFS pointer라서 GitHub media URL에서 실제 PDF를 받아 `output/` 아래에만 두고 비교했다.

판정:

- 수정 전: rhwp crop에서 로고/텍스트가 셀 하단으로 밀려 잘림
- 수정 후: rhwp crop에서 로고/텍스트가 셀 안에 들어오며 한컴 PDF 기준과 같은 중앙 높이로 배치됨

## 6. 남은 작업

- 전체 로컬 검증 완료
- 최종 보고서 작성 완료: `mydocs/report/task_m100_1352_report.md`
- 커밋 준비
- 작업지시자 별도 승인 후 PR 생성
