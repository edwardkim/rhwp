# Task M100-1352 구현 계획서

## 1. 목표

표 셀 안에서 TAC picture와 텍스트가 같은 줄에 있을 때, image가 line baseline/label 보정 때문에 아래로 밀려 cell clip에 잘리는 문제를 해소한다.

## 2. Stage 구성

### Stage 1 — 재현 테스트 고정

대상:

- `tests/issue_1352_table_cell_tac_picture_text.rs`

작업:

- `samples/hwpx/hy-001.hwpx`를 로드한다.
- 1페이지 render tree에서 `광부` text run이 들어 있는 첫 표 셀을 찾는다.
- 같은 text line의 image bbox를 찾는다.
- image 하단이 cell bbox 안에 들어오는지 단언한다.
- image y가 `광부` text run y보다 과도하게 아래에 있지 않은지 단언한다.

기대:

- 현재 `devel`에서는 실패한다.
- 수정 후 통과해야 한다.

### Stage 2 — 셀 안 TAC picture y 보정

대상 후보:

- `src/renderer/layout/paragraph_layout.rs`

작업:

- `tac_picture_label_extra_px` 적용 지점을 점검한다.
- `cell_ctx.is_some()`이고 같은 line에 visible text가 있는 경우, label extra로 image y를 아래로 미는 경로를 제한한다.
- 기존 본문/글상자/빈 문단 TAC picture의 #974, #1151 동작을 건드리지 않도록 조건을 좁힌다.

판단 기준:

- cell context가 있는 visible text + TAC picture line에서 image y는 line top 또는 baseline 기반 자연 위치에 남아야 한다.
- text가 없는 TAC-only line, equation-only TAC line, sibling TopAndBottom table 보정은 기존 의미를 유지한다.

### Stage 3 — 회귀 검증

focused:

```text
cargo test --test issue_1352_table_cell_tac_picture_text
```

관련 회귀:

```text
cargo test --test issue_1285_tac_sequence_right_align
cargo test --test issue_1161_copy_picture_in_cell
cargo test --release --lib
```

시각:

```text
target/debug/rhwp export-svg samples/hwpx/hy-001.hwpx -o output/issue1352_hy001/svg
rsvg-convert -f png -o output/issue1352_hy001/rhwp_page1.png output/issue1352_hy001/svg/hy-001_001.svg
pdftoppm -r 96 -png output/issue1352_hy001/hancom_hy001.pdf output/issue1352_hy001/pdf
```

### Stage 4 — 전체 검증 및 PR 준비

검증:

```text
cargo build --release
cargo test --release --lib
cargo test --profile release-test --tests
cargo fmt --check
wasm-pack build --target web --out-dir pkg
```

문서:

- `mydocs/working/task_m100_1352_stage1.md`
- `mydocs/report/task_m100_1352_report.md`

PR 준비:

- 커밋 제목은 `task 1352:`로 시작한다.
- PR 제목과 본문은 한국어로 작성한다.
- PR 본문에 `Closes #1352`를 포함한다.
- `gh pr create`는 작업지시자 별도 승인 후에만 실행한다.

## 3. 위험과 완화

- 위험: `tac_picture_label_extra_px` 제한이 다른 TAC picture label 보정 케이스를 되돌릴 수 있다.
  - 완화: `cell_ctx.is_some()` + visible text line 조합으로 좁힌다.
- 위험: `hy-001`은 #974 guard도 겸하므로 글상자 내부 picture 정합이 깨질 수 있다.
  - 완화: 기존 `hy-001` 관련 unit/wasm 테스트와 SVG 비교를 함께 수행한다.
- 위험: 한컴 PDF는 Git LFS pointer로만 내려올 수 있다.
  - 완화: 검증 시 실제 PDF가 없으면 GitHub media URL로 받아 `output/`에만 둔다.

## 4. 승인 게이트

이 문서 승인 후 Stage 1 소스/테스트 수정을 시작한다.
