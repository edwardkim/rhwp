# Task 1274 구현 계획서

## 원칙

- 문서별 특례가 아니라 레이아웃 공통 로직을 우선 수정한다.
- 페이지별 결함은 자동 측정과 PNG 시각 확인을 함께 사용한다.
- SVG를 PNG로 바꿀 때는 ImageMagick 대신 `rsvg-convert`를 사용한다.
- 사용자가 WASM 시각 검증을 수동으로 한다는 기존 원칙을 유지하며, 이번 task의 Codex 검증은 네이티브 SVG/PDF PNG 비교에 집중한다.

## 1단계: 비교 파이프라인

- 대상 파일 목록을 고정한다.
- `target/debug/rhwp export-svg` 또는 `cargo run --bin rhwp -- export-svg`로 페이지별 SVG를 생성한다.
- PDF 페이지 수와 SVG 페이지 수를 비교한다.
- `pdftoppm -r 96`과 `rsvg-convert`로 PNG를 생성한다.
- Python/Pillow를 사용해 좌우 비교 PNG와 간단한 bbox/overflow 측정 보고서를 만든다.

## 2단계: 1차 sweep

- `3-09월_교육_통합_2022.hwp`부터 전체 페이지를 훑는다.
- 로그의 `LAYOUT_OVERFLOW`/`LAYOUT_OVERFLOW_DRAW`를 먼저 잡고, PNG에서 수식 겹침과 미주 간격 차이를 확인한다.
- 결함 페이지는 `mydocs/working/task_m100_1274_stageN.md`에 원인, 수정, 검증 산출물을 기록한다.

## 3단계: 수정과 회귀 고정

- 결함 유형별로 기존 테스트 파일에 케이스를 추가하거나 새 통합 테스트를 만든다.
- 미주 간격과 page overflow는 `tests/issue_1139_inline_picture_duplicate.rs`, `tests/issue_1082_endnote_multicolumn_drift.rs`의 기존 패턴을 우선 활용한다.
- 수식 겹침은 TAC/paragraph layout 경로에서 실제 bbox나 dump-pages 기준 회귀 테스트를 추가한다.

## 4단계: 반복 검증

- 각 수정 후 타깃 페이지 SVG/PNG를 다시 만든다.
- 관련 테스트와 `cargo fmt -- --check`, `git diff --check`를 실행한다.
- 수정 단위가 독립적으로 검증되면 `task 1274:` 커밋을 만든다.
- 다음 페이지/문서로 계속 진행한다.

## 산출물 경로

- SVG: `output/task1274/<문서키>/svg/`
- rhwp PNG: `output/task1274/<문서키>/rhwp_png/`
- PDF PNG: `output/task1274/<문서키>/pdf_png/`
- 비교 PNG: `output/task1274/<문서키>/compare/`
- 단계별 문서: `mydocs/working/task_m100_1274_stageN.md`
