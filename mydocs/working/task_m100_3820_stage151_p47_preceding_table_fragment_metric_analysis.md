# Stage 151: 2025 편람 p47 선행 RowBreak 표 fragment metric 분석

## 목적

한컴 기준 PDF 47쪽의 footer-safe 표 배치를 위해, `문단 214` RowBreak 표가 p47에서 후속 `문단 216`의 제목과 표 anchor를 아래로 미는 원인을 분석한다.

## Stage 150에서 고정한 사실

- `문단 216`의 Square 표는 문단 기준 양수 `vertical_offset`을 첫 fragment budget과 partial paint에 반영해야 한다.
- 그 수정으로 제목-표 직접 겹침은 해소됐고 HWP·HWPX 모두 383쪽을 유지한다.
- 그러나 수정 후 p47에서 `문단 214` 표 fragment는 `y=172.7..723.7px`(551.0px)을 차지한다.
- 그 결과 `문단 216` 제목과 표는 PDF 기준보다 아래로 밀리고, 표 bottom `944.1px`이 footer 시작 `888.2px`를 넘는다.

## 분석 범위

- PDF p47의 `3. 외국어보다 쉽게 다듬은 말 사용하기` 표 종료 위치와 HWP/HWPX의 같은 source fragment를 비교한다.
- `문단 214`의 실제 row heights, measured row heights, cut row heights, repeat header, cell padding, row cursor를 같은 좌표계에서 기록한다.
- 표가 한 행을 과소/과대 측정하는지, 또는 fragment owner가 PDF와 다른지 분리한다.

## 금지 조건

- p47, display 39, 문단 214, 표 ID, 행 번호, 표 문구를 구현 조건으로 사용하지 않는다.
- Stage 150의 문단 기준 offset 계약을 되돌리거나 page count만 맞추기 위한 reserve를 추가하지 않는다.
- 분석 근거가 완료되기 전에는 코드와 테스트를 수정하지 않는다.

## 완료 기준

- PDF와 rhwp 사이의 첫 table fragment bottom delta를 수치로 기록한다.
- 차이가 행 metric인지 fragment cursor owner인지 결론을 분리해 기록한다.
- 구현 범위가 공통 row/table layout 계약으로 표현 가능한 경우에만 다음 단계에서 코드 변경을 시작한다.

## 분석 결과

- `문단 214`는 16행, `cell_spacing=3.4px`인 저장 RowBreak 표다.
- 저장 `common.height`와 PDF의 첫 표 외곽 높이는 모두 약 `500px`이다.
- 수정 전 renderer는 행 높이 합을 `common.height=500px`에 맞춘 뒤, row 좌표를 만들 때 15개의 행 간격 `15 x 3.4px = 51px`을 다시 더했다. 그 결과 p47 첫 표가 `551px`로 paint됐고, 뒤의 빈 문단과 `문단 216`을 함께 아래로 밀었다.
- 따라서 원인은 row cursor owner나 문서별 저장 reset이 아니라, 저장 표의 outer height와 내부 행 간격을 다른 좌표계로 해석한 공통 row metric 결함이다.

## 구현

- `src/renderer/layout/table_layout.rs`의 `fit_row_heights_to_common_height`가 행 합계 목표를 `common.height - (row_count - 1) * cell_spacing`으로 계산하도록 수정했다.
- `common.height`는 저장 표의 outer height이고 `row_y`는 행 간격을 별도로 더하므로, 이 공제로 같은 gap의 이중 계상을 막는다.
- 이 함수는 whole table과 partial table 양쪽의 `resolve_row_heights`가 공유한다. 문단 번호, 페이지, 표 ID, 행 번호를 조건으로 사용하지 않았다.

## 결과

- `cargo build --target-dir target/stage151` 완료.
- 원본 HWP와 HWPX를 각각 전체 SVG로 다시 내보냈고, 둘 다 `383`쪽이다.
- 두 입력의 p47 render tree는 동일하다.
- `문단 214` 표: `y=172.7px`, `height=500.0px`.
- `문단 216` 첫 fragment: `y=758.5px`, bottom border `y=891.6px`.
- page number `39`의 TextLine: `y=915.1px`. 표 border와 `23.5px` 떨어져 있어 두 파일 모두 페이지 번호와 겹치지 않는다.
- SVG raster 대조에서도 HWP와 HWPX의 p47 표와 page number가 분리됐으며, PDF의 첫 표 외곽 높이와 일치한다.

## 남은 범위

- `문단 216` border는 body bottom `888.2px`을 `3.4px` 넘는다. page number와는 겹치지 않지만, PDF 대비 제목과 두 번째 표의 상단 간격 차이는 아직 남아 있다.
- 이 간격은 첫 표 높이와 별개인 `문단 215`의 빈 line/anchor 소비 계약을 다음 짧은 Stage에서 분석한다. 이 Stage에서는 footer 침범을 막기 위한 reserve나 문서별 예외를 추가하지 않는다.

## 상태

완료. 공통 저장 표 row metric 보정과 HWP/HWPX p47 페이지 번호 분리까지 확정.
