# Task M100 #1443 Stage 15 작업 기록

- 이슈: #1443
- 브랜치: `local/task_m100_1443`
- 시작일: 2026-06-20
- 선행 커밋: `d00fde1e task 1443: 표 외곽 구조를 한컴 PDF 기준으로 보정`

## 1. 목표

`samples/셀보호2.hwp`에서 좌우 안 여백이 10mm로 지정된 셀의 텍스트 배치와 줄바꿈을 한컴 PDF 기준과 맞춘다.

Stage 14에서 전체 표 bbox와 내부 grid는 한컴 PDF와 맞췄지만, 안 여백이 큰 셀의 텍스트 표시가 아직 다르게 보인다.

## 2. 확인할 기준

- 한컴 기준 PDF: `pdf/셀보호2-2024.pdf`
- 입력 샘플: `samples/셀보호2.hwp`, `samples/셀보호2.hwpx`
- 비교 대상 셀:
  - 마지막 행 첫 번째 셀
  - 셀 폭: 약 21mm
  - 안 여백: 왼쪽 10mm, 오른쪽 10mm, 위/아래 0mm
  - 내용: `12345`

## 3. 작업 방향

- 한컴 PDF와 rhwp 렌더 PNG에서 해당 셀을 확대 비교한다.
- 현재 줄바꿈/정렬/클리핑 계산이 셀 안 여백을 어떻게 반영하는지 확인한다.
- 셀 내부 가용 폭이 매우 좁을 때 한컴처럼 표시되도록 레이아웃 계산을 보정한다.

## 4. 검증 계획

- `pdftoppm -png -r 96 pdf/셀보호2-2024.pdf ...`
- `cargo run --bin rhwp -- export-svg samples/셀보호2.hwp ...`
- `rsvg-convert ...`
- 확대 crop으로 마지막 행 첫 번째 셀의 텍스트 배치를 비교한다.
- 관련 회귀 테스트:
  - `cargo test --test issue_493_cell_attrs -- --nocapture`
  - `cargo test -p rhwp --lib renderer::layout::table_layout::row_cut_tests::test_shrink_cell_padding_preserves_explicit_cell_margin -- --nocapture`

## 5. 확인 결과

Stage 14 커밋 이후 다시 렌더링해 비교했다.

- 한컴 PDF 확대 crop: `output/poc/task_m100_1443_stage15/hancom_bottom_left_text_zoom.png`
- rhwp HWP 확대 crop: `output/poc/task_m100_1443_stage15/rhwp_bottom_left_text_zoom.png`
- rhwp HWPX 확대 crop: `output/poc/task_m100_1443_stage15_hwpx/rhwp_hwpx_bottom_left_text_zoom.png`

결과:

- 한컴 PDF는 마지막 행 첫 번째 셀에서 `12`, `34`, `5` 형태로 줄바꿈된다.
- rhwp HWP 렌더도 같은 `12`, `34`, `5` 형태로 표시된다.
- rhwp HWPX 렌더도 HWP와 동일하게 표시된다.

따라서 안 여백 10mm 셀의 텍스트 배치 문제는 Stage 14의 전체 표 구조/높이 보정 이후 함께 해소된 것으로 판단한다. Stage 15에서는 추가 소스 수정 없이 시각 정합 확인만 수행했다.

## 6. 검증 결과

- `cargo run --bin rhwp -- export-svg samples/셀보호2.hwp -o output/poc/task_m100_1443_stage15 -p 0`: 통과
- `cargo run --bin rhwp -- export-svg samples/셀보호2.hwpx -o output/poc/task_m100_1443_stage15_hwpx -p 0`: 통과
- `rsvg-convert`: HWP/HWPX PNG 변환 통과
