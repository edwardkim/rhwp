# Task M100 #1486 Stage 5: PR 전 회귀와 마지막 쪽 그림 위치 보정

- 이슈: #1486
- 브랜치: `local/task_m100_1486`
- 작성일: 2026-06-24
- 방법론: Hyper-Waterfall
- 선행 커밋:
  - `9dbd32cc task 1486: HWPX 분할 표 TAC 배치 1차 보정`
  - `f27d39cb task 1486: RowBreak 잔여 조각 페이지 보정`
  - `ee17cb39 task 1486: RowBreak 병합행 분할 순서 보정`
  - `ab6c9f3c task 1486: 13쪽 footer와 19쪽 그림 잘림 보정`

## 배경

PR 준비를 위해 `upstream/devel` 최신 기준으로 rebase 한 뒤 로컬 검증을 시작했다.
`cargo build --release`와 `cargo test --release --lib`는 통과했지만,
`cargo test --profile release-test --tests`에서 기존 `issue_1105` 통합 테스트가 실패했다.

작업지시자 시각 확인으로 `hwpx_sample2.hwpx` 마지막 29쪽의 LH 그림 위치도 한컴 PDF 기준과
맞지 않음이 추가 확인되었다.

## 해결 대상

1. `tests/issue_1105.rs` 회귀
   - 실패 샘플: `samples/k-water-rfp-2024.hwp`
   - 실패 1: 페이지 수가 한컴 PDF 기준 27쪽이 아니라 28쪽으로 증가
   - 실패 2: `pi=52` 첫 대형 rowspan RowBreak 표의 5쪽 분할이 기대와 달라짐
   - 기대 조건: `PartialTable pi=52 ci=0 rows=0..4`, `end_cut=[3, 4, 2, 4, 4, 2, 20]`

2. #1486 샘플 마지막 29쪽 LH 그림 위치
   - 한컴 PDF 기준보다 rhwp SVG의 그림/텍스트 배치가 어긋남
   - 29쪽 side-by-side, render tree, PDF raster/text bbox로 위치 차이를 확인한다.

## 분석 계획

- `RHWP_TABLE_DRIFT=1 cargo test --profile release-test --test issue_1105 -- --nocapture`로
  `pi=52` 분할 로그를 수집한다.
- `hwpx_sample2.hwpx` 29쪽 render tree와 PDF 29쪽 PNG/PDF 텍스트 bbox를 비교해 LH 그림 노드의
  bbox 차이를 수치화한다.
- Stage 3/4에서 추가한 RowBreak rowspan-block 보정이 기존 hard-break 블록 컷을 과도하게
  바꾼 부분을 좁힌다.
- 마지막 29쪽 그림 위치는 해당 개체의 anchor/wrap/vertical relation이 어떤 렌더 경로를 타는지
  확인한 뒤, 다른 쪽의 그림 배치에 영향이 적은 조건으로 보정한다.

## 검증 계획

- `cargo test --profile release-test --test issue_1105 -- --nocapture`
- `cargo test --release --test issue_1486_hwpx_partial_tac_table -- --nocapture`
- `cargo build --release`
- `hwpx_sample2.hwpx` 29쪽 PDF/SVG PNG side-by-side 재생성
- 필요 시 전체 29쪽 visual sweep 재실행

## 구현 결과

- `paragraph_layout.rs`
  - TAC 그림과 실제 텍스트가 같은 줄에 있는 경우 `tac_picture_label_extra` 보정을 일반 문단에서도
    적용하지 않도록 했다.
  - 기존 #1352 보정은 표 셀 내부만 가드했으나, 29쪽 LH 로고는 셀 밖 일반 문단의 TAC 그림+텍스트 줄이라
    같은 과보정이 남아 있었다.

- `table_layout.rs` / `typeset.rs`
  - RowBreak rowspan 블록 내부 hard-break가 처음 나타나는 셀의 시작 행을 판정하는 helper를 추가했다.
  - 행 offset 기반 블록 컷은 hard-break가 블록 첫 행에서 시작하는 경우에만 사용하도록 제한했다.
  - #1486의 13/14쪽처럼 아래 행 셀을 다음 조각에 남겨야 하는 경우는 유지하고, #1105의 `pi=52`처럼
    hard-break가 뒤 행 셀 안에 있는 블록은 기존 블록 컷을 사용해 `end_cut` 회귀를 복구했다.

- `tests/issue_1486_hwpx_partial_tac_table.rs`
  - 29쪽 LH 로고 TAC 그림 bbox를 `para_index=218`, `control_index=0`으로 찾아 한컴 PDF 기준
    `x=38.4`, `y=417.6`, `w=115.6` 근처에 있는지 확인하는 회귀 테스트를 추가했다.

## 검증 결과

- `cargo test --release --test issue_1486_hwpx_partial_tac_table -- --nocapture`
  - 통과: 6 passed
- `cargo test --profile release-test --test issue_1105 -- --nocapture`
  - 통과: 14 passed
  - 참고: 기존 진단 로그 `LAYOUT_OVERFLOW` 1건은 출력되지만 실패하지 않는다.
- `cargo test --profile release-test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 통과: 85 passed
- `cargo build --release`
  - 통과

## 시각 확인 산출물

- 최신 release 바이너리로 `hwpx_sample2.hwpx` 29쪽 SVG를 재생성했다.
- 한컴 PDF 기준 파일: `pdf/hwpx_sample2-2024.pdf`
- 비교 산출물:
  - `output/poc/task1486/stage5_page29_compare/page_029_side_by_side.png`
  - `output/poc/task1486/stage5_page29_compare/page_029_logo_crop_side_by_side.png`

작업지시자가 지적한 마지막 29쪽 LH 그림의 세로 위치는 이전처럼 텍스트 줄 아래로 처지지 않고
한컴 PDF 기준 줄 높이로 올라온 것을 확인했다.
