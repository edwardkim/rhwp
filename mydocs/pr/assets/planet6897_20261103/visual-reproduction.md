# 전쪽 시각 검증의 재실행 방법

이 문서는 로컬 `output/`의 임시 실행기를 보유하지 않아도 같은 입력·Print PDF·조판 세대·글꼴 계약으로 실행할 수 있게 한다.
기록된 c183/이전 제품 TSV는 해당 producer의 실제 결과이며 최신 head 재출력으로 표시하지 않는다.
아래 명령을 다른 제품 코드에서 실행하면 별도 검증 결과다. 원본과 독립 한컴 Print PDF는 변경하거나 다시 출력하지 않는다.

## 제품과 실행 환경

- Native는 `--features native-skia`로 빌드한 CLI, fresh WASM은 `scripts/wasm-pack-locked.sh --target web`의 최적화 산출물이다.
- Cargo는 기본 작업공간의 `target/pr-review`를 재사용하고 빌드 잡은 작업지시자의 지정인 8개다. Cargo 빌드끼리는 겹치지 않는다.
- 시각 측정은 시스템 글꼴 전체와 아래 표의 명시 대체 설정을 사용한다. 최소 글꼴 환경의 Rust 회귀와 구분한다.
- 기록된 producer의 Native/WASM·입력/PDF·전체 쪽수·TSV 해시는 각 검증 Markdown에서 확인한다. 글꼴 프로그램은 배포하지 않는다.

```bash
CARGO_BUILD_JOBS=8 cargo build --locked --profile release-test \
  --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia --bin rhwp
CARGO_BUILD_JOBS=8 CARGO_TARGET_DIR=/home/tsjang/rhwp/target/pr-review \
  sh scripts/wasm-pack-locked.sh --target web --out-dir pkg
```

검증할 revision을 먼저 고정하고 위 두 빌드의 실제 source·바이너리/WASM 해시를 기록한다.
최종 head의 실행 성공을 과거 producer의 재현으로 바꾸어 표시하지 않는다.

## Native와 fresh WASM 명령

아래 `INPUT`, `PRINT_PDF`, `COMPAT`는 같은 행의 값으로 바꾼다. `KEY`와 출력 디렉터리는 실행별로 구분한다.
글꼴 계약이 있는 행에만 `--font-environment <아래 설정 경로>`를 두 명령 모두에 추가한다.

```bash
venv/bin/python scripts/visual_sweep.py --silhouette-only --resume \
  --hwp INPUT --pdf PRINT_PDF --compat COMPAT --dpi 96 \
  --rhwp-bin /home/tsjang/rhwp/target/pr-review/release-test/rhwp \
  --key KEY --out output/pr-review/검증명/native

venv/bin/python scripts/visual_sweep.py --silhouette-only --resume \
  --hwp INPUT --pdf PRINT_PDF --compat COMPAT --dpi 96 \
  --rhwp-bin /home/tsjang/rhwp/target/pr-review/release-test/rhwp \
  --wasm-pkg pkg --page-workers 1 \
  --key KEY --out output/pr-review/검증명/wasm
```

`--resume`은 저장된 입력·제품·도구·글꼴·Git 출처가 같은 실행만 이어받는다. 출처가 달라 거부되면
기존 manifest를 덮어쓰거나 기대 해시를 바꾸지 말고 별도 새 출력 위치에서 실행한다.
출력의 `silhouette.tsv`가 있는지만 보지 않고 모든 페이지·누락·쪽수·해시와 종료 상태를 확인한다.

## 80개 입력·기준 PDF와 조판 세대

`compat`는 엔진의 조판 세대 값이다. `2022`를 한컴 제품 버전으로 부르지 않는다. 실제 Print 버전·출처는 각 입력의 기존 PDF 증거를 따른다.

| 원 PR | 입력 | 독립 Print PDF | compat | 글꼴 계약 |
| --- | --- | --- | ---: | --- |
| PR #8422 | [30026-safety-reform-toc.hwp](<../../../../samples/issue8418/30026-safety-reform-toc.hwp>) | [30026-safety-reform-toc-2022.pdf](<../../../../pdf/issue8418/30026-safety-reform-toc-2022.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8429 | [business_plan.hwpx](<../issue_8424/inputs/after/business_plan.hwpx>) | [business_plan-2024.pdf](<../../../../pdf/issue-8424/business_plan-2024.pdf>) | 2024 | 시스템 전체; 별도 대체 없음 |
| PR #8429 | [cellwrap.hwpx](<../issue_8424/inputs/after/cellwrap.hwpx>) | [cellwrap-2024.pdf](<../../../../pdf/issue-8424/cellwrap-2024.pdf>) | 2024 | 시스템 전체; 별도 대체 없음 |
| PR #8431 | [30396-childcare-leave-recommendation.hwp](<../../../../samples/issue8423/30396-childcare-leave-recommendation.hwp>) | [30396-childcare-leave-recommendation-2022.pdf](<../../../../pdf/issue8423/30396-childcare-leave-recommendation-2022.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8433 | [levels.hwpx](<../issue_8425/probe/levels.hwpx>) | [levels-2024.pdf](<../../../../pdf/issue-8425/levels-2024.pdf>) | 2024 | 시스템 전체; 별도 대체 없음 |
| PR #8434 | [156690773-declared-zero-textbox.hwp](<../../../../samples/issue8432/156690773-declared-zero-textbox.hwp>) | [156690773-declared-zero-textbox-2022.pdf](<../../../../pdf/issue8432/156690773-declared-zero-textbox-2022.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8434 | [156761223-declared-zero-textbox.hwp](<../../../../samples/issue8432/156761223-declared-zero-textbox.hwp>) | [156761223-declared-zero-textbox-2022.pdf](<../../../../pdf/issue8432/156761223-declared-zero-textbox-2022.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8437 | [156618114-logo-textbox-offset.hwpx](<../../../../samples/issue8435/156618114-logo-textbox-offset.hwpx>) | [156618114-logo-textbox-offset-2022.pdf](<../../../../pdf/issue8435/156618114-logo-textbox-offset-2022.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8437 | [156638890-logo-textbox.hwpx](<../../../../samples/issue8435/156638890-logo-textbox.hwpx>) | [156638890-logo-textbox-2022.pdf](<../../../../pdf/issue8435/156638890-logo-textbox-2022.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8437 | [156770078-logo-table.hwpx](<../../../../samples/issue8435/156770078-logo-table.hwpx>) | [156770078-logo-table-2022.pdf](<../../../../pdf/issue8435/156770078-logo-table-2022.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8444 | [156578788-tac-logo-line.hwp](<../../../../samples/issue8441/156578788-tac-logo-line.hwp>) | [156578788-tac-logo-line-2022.pdf](<../../../../pdf/issue8441/156578788-tac-logo-line-2022.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8444 | [156669499-tac-logo-line.hwpx](<../../../../samples/issue8441/156669499-tac-logo-line.hwpx>) | [156669499-tac-logo-line-2022.pdf](<../../../../pdf/issue8441/156669499-tac-logo-line-2022.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8444 | [156715172-tac-table-picture.hwpx](<../../../../samples/issue8441/156715172-tac-table-picture.hwpx>) | [156715172-tac-table-picture-2022.pdf](<../../../../pdf/issue8441/156715172-tac-table-picture-2022.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8448 | [156569782-reset-line-logo-empty.hwp](<../../../../samples/issue8446/156569782-reset-line-logo-empty.hwp>) | [156569782-reset-line-logo-empty-2022.pdf](<../../../../pdf/issue8446/156569782-reset-line-logo-empty-2022.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8448 | [156667426-reset-line-logo.hwp](<../../../../samples/issue8446/156667426-reset-line-logo.hwp>) | [156667426-reset-line-logo-2022.pdf](<../../../../pdf/issue8446/156667426-reset-line-logo-2022.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8461 | [156759019-text-line-tac-logo.hwp](<../../../../samples/issue8458/156759019-text-line-tac-logo.hwp>) | [156759019-text-line-tac-logo-2022.pdf](<../../../../pdf/issue8458/156759019-text-line-tac-logo-2022.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8465 | [156552561-offpage-textbox.hwpx](<../../../../samples/issue8462/156552561-offpage-textbox.hwpx>) | [156552561-offpage-textbox-2022.pdf](<../../../../pdf/issue8462/156552561-offpage-textbox-2022.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8465 | [156633674-offpage-logo.hwpx](<../../../../samples/issue8462/156633674-offpage-logo.hwpx>) | [156633674-offpage-logo-2022.pdf](<../../../../pdf/issue8462/156633674-offpage-logo-2022.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8469 | [rowbreak_content_fit_f1200_h2660_d14.hwpx](<../../../../samples/issue8464/rowbreak_content_fit_f1200_h2660_d14.hwpx>) | [issue8464_rowbreak_content_fit_f1200_h2660_d14-2020.pdf](<../../../../pdf/issue8464_rowbreak_content_fit_f1200_h2660_d14-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8469 | [rowbreak_content_fit_f1200_h2660_d16.hwpx](<../../../../samples/issue8464/rowbreak_content_fit_f1200_h2660_d16.hwpx>) | [issue8464_rowbreak_content_fit_f1200_h2660_d16-2020.pdf](<../../../../pdf/issue8464_rowbreak_content_fit_f1200_h2660_d16-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8469 | [rowbreak_content_fit_h2505.hwpx](<../../../../samples/issue8464/rowbreak_content_fit_h2505.hwpx>) | [issue8464_rowbreak_content_fit_h2505-2020.pdf](<../../../../pdf/issue8464_rowbreak_content_fit_h2505-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8469 | [rowbreak_content_fit_h2570.hwpx](<../../../../samples/issue8464/rowbreak_content_fit_h2570.hwpx>) | [issue8464_rowbreak_content_fit_h2570-2020.pdf](<../../../../pdf/issue8464_rowbreak_content_fit_h2570-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8469 | [rowbreak_content_fit_h3180.hwpx](<../../../../samples/issue8464/rowbreak_content_fit_h3180.hwpx>) | [issue8464_rowbreak_content_fit_h3180-2020.pdf](<../../../../pdf/issue8464_rowbreak_content_fit_h3180-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8473 | [36412338-approval-footer-split.hwpx](<../../../../samples/issue8467/36412338-approval-footer-split.hwpx>) | [36412338-approval-footer-split-2022.pdf](<../../../../pdf/issue8467/36412338-approval-footer-split-2022.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8473 | [36416725-approval-footer-split.hwpx](<../../../../samples/issue8467/36416725-approval-footer-split.hwpx>) | [36416725-approval-footer-split-2022.pdf](<../../../../pdf/issue8467/36416725-approval-footer-split-2022.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8475 | [vertical_fit.hwpx](<../../../../samples/issue8470/vertical_fit.hwpx>) | [vertical_fit-2020.pdf](<../../../../pdf/issue8470/vertical_fit-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8475 | [vertical_oversize.hwpx](<../../../../samples/issue8470/vertical_oversize.hwpx>) | [vertical_oversize-2020.pdf](<../../../../pdf/issue8470/vertical_oversize-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8476 | [pushed_none.hwpx](<../../../../samples/issue8471/pushed_none.hwpx>) | [pushed_none-2020.pdf](<../../../../pdf/issue8471/pushed_none-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8477 | [cell_picture_row_split.hwp](<../../../../samples/issue8472/cell_picture_row_split.hwp>) | [issue8472_cell_picture_row_split-2020.pdf](<../../../../pdf/issue8472_cell_picture_row_split-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8486 | [156454114-p4g-opening-remarks.hwp](<../../../../samples/issue8468/156454114-p4g-opening-remarks.hwp>) | [156454114-p4g-opening-remarks-2022.pdf](<../../../../pdf/issue8468/156454114-p4g-opening-remarks-2022.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8486 | [line_reserve_F20_L30_A30_R12.hwp](<../../../../samples/issue8468/line_reserve_F20_L30_A30_R12.hwp>) | [line_reserve_F20_L30_A30_R12-2020.pdf](<../../../../pdf/issue8468/line_reserve_F20_L30_A30_R12-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8486 | [line_reserve_F20_L30_A40_R12.hwp](<../../../../samples/issue8468/line_reserve_F20_L30_A40_R12.hwp>) | [line_reserve_F20_L30_A40_R12-2020.pdf](<../../../../pdf/issue8468/line_reserve_F20_L30_A40_R12-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8487 | [missing_lines.hwpx](<../../../../samples/issue8479/missing_lines.hwpx>) | [issue8479_missing_lines-2020.pdf](<../../../../pdf/issue8479_missing_lines-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8489 | [section_top_spacing-2020.hwp](<../../../../samples/issue8474/section_top_spacing-2020.hwp>) | [section_top_spacing-2020.pdf](<../../../../pdf/issue8474/section_top_spacing-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8489 | [section_top_spacing.hwpx](<../../../../samples/issue8474/section_top_spacing.hwpx>) | [section_top_spacing-2020.pdf](<../../../../pdf/issue8474/section_top_spacing-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8489 | [top_tac_cases-2020.hwp](<../../../../samples/issue8474/top_tac_cases-2020.hwp>) | [top_tac_cases-2020.pdf](<../../../../pdf/issue8474/top_tac_cases-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8489 | [top_tac_cases.hwpx](<../../../../samples/issue8474/top_tac_cases.hwpx>) | [top_tac_cases-2020.pdf](<../../../../pdf/issue8474/top_tac_cases-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8496 | [table_tail.hwp](<../../../../samples/issue8493/table_tail.hwp>) | [issue8493_table_tail-2020.pdf](<../../../../pdf/issue8493_table_tail-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8497 | [span_page_bottom-2020.hwp](<../../../../samples/issue8478/span_page_bottom-2020.hwp>) | [span_page_bottom-2020.pdf](<../../../../pdf/issue8478/span_page_bottom-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8498 | [156602236-press-logo.hwpx](<../../../../samples/issue8491/156602236-press-logo.hwpx>) | [156602236-press-logo-2022.pdf](<../../../../pdf/issue8491/156602236-press-logo-2022.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8498 | [156736331-press-logo.hwpx](<../../../../samples/issue8491/156736331-press-logo.hwpx>) | [156736331-press-logo-2022.pdf](<../../../../pdf/issue8491/156736331-press-logo-2022.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8505 | [child_product_safety_rule.hwp](<../../../../samples/issue8501/child_product_safety_rule.hwp>) | [issue8501_child_product_safety_rule-2020.pdf](<../../../../pdf/issue8501_child_product_safety_rule-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8508 | [36316493-approval-body.hwpx](<../../../../samples/issue8502/36316493-approval-body.hwpx>) | [36316493-approval-body-2022.pdf](<../../../../pdf/issue8502/36316493-approval-body-2022.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8509 | [1190000-200700045-nearfit-table-indent.hwp](<../../../../samples/issue8503/1190000-200700045-nearfit-table-indent.hwp>) | [1190000-200700045-nearfit-table-indent-2022.pdf](<../../../../pdf/issue8503/1190000-200700045-nearfit-table-indent-2022.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8509 | [1490000-201000053-nearfit-table-indent.hwp](<../../../../samples/issue8503/1490000-201000053-nearfit-table-indent.hwp>) | [1490000-201000053-nearfit-table-indent-2022.pdf](<../../../../pdf/issue8503/1490000-201000053-nearfit-table-indent-2022.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8512 | [156500952-k-startup-league.hwpx](<../../../../samples/issue8510/156500952-k-startup-league.hwpx>) | [156500952-k-startup-league-2022.pdf](<../../../../pdf/issue8510/156500952-k-startup-league-2022.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8514 | [wanju_goods_forms.hwp](<../../../../samples/issue8511/wanju_goods_forms.hwp>) | [issue8511_wanju_goods_forms-2020.pdf](<../../../../pdf/issue8511_wanju_goods_forms-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8519 | [table_host_top.hwp](<../../../../samples/issue8518/table_host_top.hwp>) | [issue8518_table_host_top-2020.pdf](<../../../../pdf/issue8518_table_host_top-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8530 | [rowbreak_exact_fit.hwpx](<../../../../samples/issue8529/rowbreak_exact_fit.hwpx>) | [issue8529_rowbreak_exact_fit-2020.pdf](<../../../../pdf/issue8529_rowbreak_exact_fit-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8486 | [line_reserve_F20_L30_A46_R12.hwp](<../../../../samples/issue8468/line_reserve_F20_L30_A46_R12.hwp>) | [line_reserve_F20_L30_A46_R12-2020.pdf](<../../../../pdf/issue8468/line_reserve_F20_L30_A46_R12-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8522 | [field_fit.hwpx](<../../../../samples/issue8507/field_fit.hwpx>) | [field_fit-2020.pdf](<../../../../pdf/issue8507/field_fit-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8522 | [field_fit_rb.hwpx](<../../../../samples/issue8507/field_fit_rb.hwpx>) | [field_fit_rb-2020.pdf](<../../../../pdf/issue8507/field_fit_rb-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8522 | [field_over1414.hwpx](<../../../../samples/issue8507/field_over1414.hwpx>) | [field_over1414-2020.pdf](<../../../../pdf/issue8507/field_over1414-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8522 | [field_over1414.hwpx](<../../../../samples/issue8507/field_over1414.hwpx>) | [field_over1414-2024.pdf](<../../../../pdf/issue8507/field_over1414-2024.pdf>) | 2024 | 시스템 전체; 별도 대체 없음 |
| PR #8522 | [field_over19.hwpx](<../../../../samples/issue8507/field_over19.hwpx>) | [field_over19-2020.pdf](<../../../../pdf/issue8507/field_over19-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8522 | [field_over19.hwpx](<../../../../samples/issue8507/field_over19.hwpx>) | [field_over19-2024.pdf](<../../../../pdf/issue8507/field_over19-2024.pdf>) | 2024 | 시스템 전체; 별도 대체 없음 |
| PR #8522 | [field_over19_rb.hwpx](<../../../../samples/issue8507/field_over19_rb.hwpx>) | [field_over19_rb-2020.pdf](<../../../../pdf/issue8507/field_over19_rb-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8522 | [field_over19-saved-2020.hwp](<../../../../samples/issue8507/field_over19-saved-2020.hwp>) | [field_over19-2020.pdf](<../../../../pdf/issue8507/field_over19-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8522 | [tac_adjacent-saved.hwpx](<../../../../samples/issue8507/tac_adjacent-saved.hwpx>) | [tac_adjacent-saved-2020.pdf](<../../../../pdf/issue8507/tac_adjacent-saved-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8522 | [tac_adjacent_generator.hwpx](<../../../../samples/issue8507/tac_adjacent_generator.hwpx>) | [tac_adjacent_generator-2020.pdf](<../../../../pdf/issue8507/tac_adjacent_generator-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8531 | [band_host_spacing_22.hwpx](<../../../../samples/issue8525/band_host_spacing_22.hwpx>) | [band_host_spacing_22-2020.pdf](<../../../../pdf/issue8525/band_host_spacing_22-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8531 | [band_host_spacing_23.hwpx](<../../../../samples/issue8525/band_host_spacing_23.hwpx>) | [band_host_spacing_23-2020.pdf](<../../../../pdf/issue8525/band_host_spacing_23-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8531 | [band_host_spacing_23.hwpx](<../../../../samples/issue8525/band_host_spacing_23.hwpx>) | [band_host_spacing_23-2024.pdf](<../../../../pdf/issue8525/band_host_spacing_23-2024.pdf>) | 2024 | 시스템 전체; 별도 대체 없음 |
| PR #8531 | [band_host_spacing_24.hwpx](<../../../../samples/issue8525/band_host_spacing_24.hwpx>) | [band_host_spacing_24-2020.pdf](<../../../../pdf/issue8525/band_host_spacing_24-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8428 | [issue2004_cell_image_stack.hwp](<../../../../samples/issue2004_cell_image_stack.hwp>) | [issue2004_cell_image_stack-hwp-2020.pdf](<../../../../pdf/issue2004_cell_image_stack-hwp-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8428 | [issue2004_cell_image_stack.hwpx](<../../../../samples/issue2004_cell_image_stack.hwpx>) | [issue2004_cell_image_stack-hwpx-2020.pdf](<../../../../pdf/issue2004_cell_image_stack-hwpx-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8433 | [levels.hwpx](<../issue_8425/probe/levels.hwpx>) | [levels-2020.pdf](<../../../../pdf/issue-8425/levels-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8535 | [scaffold_table_footnote.hwpx](<../../../../samples/issue8520/scaffold_table_footnote.hwpx>) | [scaffold_table_footnote-2024.pdf](<../../../../pdf/issue8520/scaffold_table_footnote-2024.pdf>) | 2024 | 시스템 전체; 별도 대체 없음 |
| PR #8422 | [156487948-marine-pollution-law.hwp](<../../../../samples/issue6737/156487948-marine-pollution-law.hwp>) | [issue6737-156487948-2020.pdf](<../../../../pdf/issue6737-156487948-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8433 | [issue3307_outline_number.hwpx](<../../../../samples/task3307/issue3307_outline_number.hwpx>) | [issue3307_outline_number-2020.pdf](<../../../../pdf/task3307/issue3307_outline_number-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8475 | [42565_입법예고문.hwp](<../../../../samples/issue8470/42565_입법예고문.hwp>) | [42565_입법예고문-2020.pdf](<../../../../pdf/issue8470/42565_입법예고문-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8546 | [leading_shape_line-2020.hwp](<../../../../samples/issue8543/leading_shape_line-2020.hwp>) | [leading_shape_line-2020.pdf](<../../../../pdf/issue8543/leading_shape_line-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8546 | [leading_shape_line-2020.hwp](<../../../../samples/issue8543/leading_shape_line-2020.hwp>) | [leading_shape_line-2024.pdf](<../../../../pdf/issue8543/leading_shape_line-2024.pdf>) | 2024 | 시스템 전체; 별도 대체 없음 |
| PR #8547 | [gangseo_exempt_certificates.hwp](<../../../../samples/issue8532/gangseo_exempt_certificates.hwp>) | [issue8532_gangseo_exempt_certificates-2020.pdf](<../../../../pdf/issue8532_gangseo_exempt_certificates-2020.pdf>) | 2022 | 시스템 전체; 별도 대체 없음 |
| PR #8408 | [editable-2010.hwpx](<../issue_8381/integration_20261103/editable-2010.hwpx>) | [21_언어_기출_편집가능본-2010.pdf](<../../../../samples/21_언어_기출_편집가능본-2010.pdf>) | 2024 | 시스템 전체; 별도 대체 없음 |
| PR #8408 | [editable-2020.hwpx](<../issue_8381/integration_20261103/editable-2020.hwpx>) | [21_언어_기출_편집가능본-2020.pdf](<../../../../samples/21_언어_기출_편집가능본-2020.pdf>) | 2024 | 시스템 전체; 별도 대체 없음 |
| PR #8408 | [editable-plain.hwpx](<../issue_8381/integration_20261103/editable-plain.hwpx>) | [21_언어_기출_편집가능본.pdf](<../../../../samples/21_언어_기출_편집가능본.pdf>) | 2024 | 시스템 전체; 별도 대체 없음 |
| PR #8443 | [pr8443-emc-edit.hwpx](<../issue_8440/integration_20261103/pr8443-emc-edit.hwpx>) | [emc_edit-2024.pdf](<../../../../pdf/issue-8440/emc_edit-2024.pdf>) | 2024 | 시스템 전체; 별도 대체 없음 |
| PR #8540 | [scaffold_outline_a.hwpx](<../scaffold_outline_numbering/scaffold_outline_a.hwpx>) | [scaffold_outline_a-2024.pdf](<../../../../pdf/issue8540/scaffold_outline_a-2024.pdf>) | 2024 | 시스템 전체; 별도 대체 없음 |
| PR #8540 | [scaffold_outline_b.hwpx](<../scaffold_outline_numbering/scaffold_outline_b.hwpx>) | [scaffold_outline_b-2024.pdf](<../../../../pdf/issue8540/scaffold_outline_b-2024.pdf>) | 2024 | 시스템 전체; 별도 대체 없음 |
