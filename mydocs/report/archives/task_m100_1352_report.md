# 완료 보고서 — Task M100-1352

- 이슈: https://github.com/edwardkim/rhwp/issues/1352
- 제목: HWPX 표 셀 TAC 이미지/텍스트 세로 정렬 한컴 정합
- 브랜치: `local/task_m100_1352`
- 작성일: 2026-06-18

## 1. 결과 요약

`samples/hwpx/hy-001.hwpx`의 첫 표 셀에서 TAC picture와 텍스트 `광부`가 같은 줄에 있을 때
picture가 셀 하단으로 밀려 잘리는 문제를 수정했다.

수정 전 render tree 기준 첫 셀은 `y=79.35, h=41.55`이고, 텍스트는 `y=81.23`, 이미지는
`y=102.56`에 배치되어 이미지 하단이 셀 하단을 넘어갔다. 수정 후 텍스트와 이미지가 모두
`y=81.23`에 놓여 셀 안 중앙 배치가 한컴 PDF 기준과 맞아졌다.

## 2. 변경 사항

| 파일 | 내용 |
|---|---|
| `src/renderer/layout/paragraph_layout.rs` | 표 셀 안 visible text가 있는 TAC picture 라인에서는 label extra 보정을 적용하지 않도록 보정 |
| `tests/issue_1352_table_cell_tac_picture_text.rs` | `hy-001.hwpx` 첫 셀의 TAC picture/text 세로 위치와 셀 내부 배치를 고정하는 회귀 테스트 추가 |
| `mydocs/plans/task_m100_1352.md` | 수행 계획서 |
| `mydocs/plans/task_m100_1352_impl.md` | 구현 계획서 |
| `mydocs/working/task_m100_1352_stage1.md` | Stage 1 구현/검증 보고서 |

## 3. 원인과 수정

기존 `tac_picture_label_extra_px` 보정은 TAC picture가 line label처럼 들어가는 경우를 위해
picture의 y 위치를 아래로 보정한다. 하지만 표 셀 안에서 같은 줄에 실제 텍스트가 있는 경우에도
이 보정이 적용되어, picture만 텍스트보다 아래로 밀리고 셀 clip 밖으로 내려갔다.

이번 수정은 `CellContext`가 있고 줄에 visible text가 있는 경우 label extra를 0으로 제한한다.
셀 밖의 기존 TAC-only 라인과 whitespace-only 라인은 기존 보정 경로를 유지한다.

## 4. 회귀 유입 분석

회귀 추적 결과, 문제 로직은 `321aee69 task 1139: 12쪽 글자처럼 취급 그림 흐름 보정`
커밋에서 유입된 것으로 판단했다.

근거:

- `upstream/devel` 현재 기준 `d8b40eff`에서 `samples/hwpx/hy-001.hwpx`와
  `samples/hwpx/hancom-hwp/hy-001.hwp`를 각각 SVG로 내보내면 첫 이미지가 모두
  `y=102.56`에 배치되어 같은 bad 좌표를 보였다.
- `ccb5c7bc fix(layout): align TAC host spacing cell text`와 그 부모 `20191ed4`도
  첫 이미지가 `y=102.56`이므로 #1376/#1417 계열 변경이 최초 회귀 지점은 아니다.
- `321aee69`에서 `tac_picture_label_extra_px`가 도입되며 TAC picture의 y 계산이
  기존 `(y + baseline - pic_h).max(y)` 경로에서 `y + label_extra` 경로를 탈 수 있게 됐다.
- 이 보정은 12쪽 미주 그래프 라벨처럼 TAC picture가 line label 역할을 하는 경우를 위한
  보정이었지만, 표 셀 안에서 실제 텍스트가 같은 줄에 있는 `hy-001` 첫 셀에도 적용되었다.
- 결과적으로 `광부` 텍스트는 `y=81.23`에 남고 picture만 `y=102.56`으로 내려가
  셀 clip 아래로 잘리는 회귀가 발생했다.

참고: `321aee69`와 직전 커밋은 현재 toolchain/트리 조합에서 `layout_column_item`
인자 수 불일치로 직접 SVG 렌더가 되지 않았다. 따라서 직접 렌더 가능한 후속 커밋들의 bad
좌표와 `321aee69` diff를 함께 근거로 유입 지점을 판정했다.

## 5. 검증

실행한 검증:

```bash
cargo test --test issue_1352_table_cell_tac_picture_text -- --nocapture
cargo test --test issue_1285_tac_sequence_right_align -- --nocapture
cargo test --test issue_1161_copy_picture_in_cell -- --nocapture
cargo test --lib test_hy001_textbox_inline_pictures_render_for_hwp_and_hwpx -- --nocapture
cargo build
cargo build --release
cargo test --release --lib
cargo test --profile release-test --tests
cargo fmt --check
cargo clippy --all-targets -- -D warnings
git diff --check
wasm-pack build --target web --out-dir pkg
```

결과: 모두 통과.

신규 테스트 출력:

```text
[issue_1352] cell=[y=79.35, h=41.55] text=[y=81.23, h=37.79] image=[y=81.23, h=37.79]
```

## 6. 시각 비교

한컴 PDF 기준은 GitHub media URL에서 받은 `pdf-large/hwpx/hy-001.pdf` 실제 PDF를 사용했다.
로컬의 PDF 파일은 Git LFS pointer라 비교 기준으로 쓰지 않았다.

시각 비교 산출물:

```text
output/issue1352_hy001_verify_20260618/rhwp_png/hy-001_001.png
output/issue1352_hy001_verify_20260618/pdf_png/pdf-1.png
output/issue1352_hy001_verify_20260618/compare_header_cell_crop.png
```

수정 전/후 비교 산출물:

```text
mydocs/report/assets/task_m100_1352_visual_compare/README.md
mydocs/report/assets/task_m100_1352_visual_compare/compare/before_after_header_cell_compare.png
mydocs/report/assets/task_m100_1352_visual_compare/compare/before_after_page1_compare.png
mydocs/report/assets/task_m100_1352_visual_compare/compare/before_header_cell_crop.png
mydocs/report/assets/task_m100_1352_visual_compare/compare/after_header_cell_crop.png
```

비교 기준:

- 수정 전: `upstream/devel` (`d8b40eff`)
- 수정 후: `task 1352` 커밋 (`829a6a3e`)
- 핵심 SVG 좌표: 수정 전 image `y=102.56`, 수정 후 image `y=81.23`

판정:

- 수정 전: rhwp 출력에서 first cell의 logo/text가 셀 하단으로 밀려 잘림
- 수정 후: rhwp 출력에서 logo/text가 셀 안 중앙 높이로 들어오며 Hancom PDF reference와 정합

## 7. PR 준비

예정 PR 제목:

```text
task 1352: 표 셀 TAC 그림과 텍스트 세로 정렬 보정
```

예정 PR 본문:

```markdown
## 요약

- 표 셀 안에서 TAC picture와 visible text가 같은 줄에 있을 때 picture가 아래로 밀리는 문제를 보정했습니다.
- `hy-001.hwpx` 첫 셀의 TAC picture/text 세로 정렬을 고정하는 회귀 테스트를 추가했습니다.
- 한컴 PDF 기준 crop과 rhwp crop을 비교해 셀 중앙 배치 정합을 확인했습니다.

## 검증

- `cargo build --release`
- `cargo test --release --lib`
- `cargo test --profile release-test --tests`
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `wasm-pack build --target web --out-dir pkg`

Closes #1352
```

## 8. 남은 작업

- 작업지시자 최종 확인 후 stage 변경분 커밋
- 작업지시자 push/PR 생성 승인 후 `edwardkim/rhwp` base `devel`로 Open PR 생성
