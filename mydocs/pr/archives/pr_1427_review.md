# PR #1427 리뷰 기록

## PR 정보

- PR: https://github.com/edwardkim/rhwp/pull/1427
- 제목: `task 1282: 회전 표 셀 그림 리사이즈 정합 개선`
- 연결 이슈: https://github.com/edwardkim/rhwp/issues/1282
- base: `edwardkim/rhwp:devel`
- head: `jangster77/rhwp:task_m100_1282`
- 상태: Open, Draft 아님
- 작성 시점: 2026-06-17 20:13 KST

## 변경 범위

- 표 셀 내부 picture 리사이즈 후 소유 셀 높이를 회전 visual hull 기준으로 동기화한다.
- 회전각 변경 시 기존 bbox 중심을 유지하도록 picture offset을 재계산한다.
- picture 속성창의 회전각 단위와 signed offset 표시를 한컴 동작에 맞게 정리한다.
- `쪽 영역 안으로 제한` on/off 샘플에서 제한 여부에 따른 table flow와 clipping 동작을 보정한다.
- `TableCellNode.clip` 의미를 복원해 rowbreak table 회귀를 해소한다.
- HWP 스펙 errata와 task 보고서, 시각 비교 자료를 함께 갱신한다.

## 로컬 검증

통과 확인:

```text
cargo build --release
cargo test --release --lib
cargo test --profile release-test --tests
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --test issue_1282_rotated_cell_picture_resize
cargo test --test issue_1279_picture_rotation_save
wasm-pack build --target web --out-dir pkg
cd rhwp-studio && node e2e/table-picture-resize-1282.test.mjs --mode=headless
cd rhwp-studio && npm run build
```

비고:

- `cargo clippy --all-targets -- -D warnings`는 작업지시자가 로컬 터미널에서 직접 재실행해 통과를 확인했다.
- 리뷰 문서 추가 커밋은 문서 전용 변경이므로 `git diff --check`와 변경 범위 확인으로 검증한다.

## 시각 검증 자료

PR diff에 포함된 주요 증적:

- `mydocs/report/assets/task_m100_1282_stage11/comparison_restrict.png`
- `mydocs/report/assets/task_m100_1282_stage11/comparison_no_restrict.png`
- `mydocs/report/assets/task_m100_1282_stage11/pdf_restrict.png`
- `mydocs/report/assets/task_m100_1282_stage11/pdf_no_restrict.png`
- `mydocs/report/assets/task_m100_1282_ta_pic_001_r_stage6_after.png`
- `mydocs/report/assets/task_m100_1282_ta_pic_001_r_stage6_rotation0.png`

판정:

- 회전된 표 셀 그림의 드래그 리사이즈 후 셀 높이와 clipping이 한컴 PDF 기준 비교와 맞는다.
- `쪽 영역 안으로 제한` on 상태에서는 셀 경계를 침범하지 않고, off 상태에서는 no 샘플과 같은 배치가 된다.
- 회전각 0도 변경 후 bbox 중심 보존과 셀 높이 감소가 확인됐다.

## 리뷰 결론

로컬 필수 검증과 시각 비교 기준은 충족했다. 이 문서와 오늘할일 커밋을 PR head에 포함해 GitHub Actions를 다시 확인한 뒤, 모든 required check가 통과하면 merge 가능으로 판단한다.
