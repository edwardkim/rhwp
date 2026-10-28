# Stage 150: 2025 편람 p47 표 fragment와 footer owner 분석

## 목적

2025 행정업무운영 편람의 한컴 기준 PDF 47쪽(display 39)과 현재 rhwp HWP·HWPX 출력을 대조해, 표 fragment와 페이지 번호가 충돌하거나 사라지는 owner 경로를 분리한다.

## 정답지와 재현

- 원본 HWP: `samples/2025 행정업무운영 편람(최종).hwp`
- 원본 HWPX: `samples/2025 행정업무운영 편람(최종).hwpx`
- 한컴 기준 PDF: `pdf/2025 행정업무운영 편람(최종)-hwp-2020.pdf`
- 재현 명령: `target/stage149/debug/rhwp export-svg <원본> -o <외부 산출 경로> --json`

## 초기 시각 대조

- PDF p47은 `3. 외국어보다 쉽게 다듬은 말 사용하기` 표의 끝과 `4. 한자어보다 쉽게 다듬은 말 사용하기` 표를 display page number `39` 위에 정상 배치한다.
- HWP p47은 같은 두 표를 출력하지만, 두 번째 표의 마지막 행이 footer/page number `39` 영역과 겹친다.
- HWPX p47은 완전히 빈 페이지다. HWP와 HWPX의 p46은 display `38`, p48은 display `40`으로 동일하므로, HWPX는 물리 p47의 표·페이지 번호 owner를 잃었다.

## 분석 범위

- HWP 경로: RowBreak table fragment가 body 하단과 footer/page-number 영역을 분리하지 못한 이유를 render tree와 row cut 결과로 확인한다.
- HWPX 경로: p47의 빈 page가 source section/page-break, table fragment cursor, 또는 master-page owner 중 어느 단계에서 생기는지 확인한다.
- 두 경로의 입력 구조가 같은지 HWP/HWPX source table과 page-break marker를 비교한다.

## 금지 조건

- physical page `47`, display page `39`, 표 행 번호, 표 ID, 표 문구를 코드 조건으로 사용하지 않는다.
- HWP와 HWPX의 서로 다른 결함을 페이지 수 조정이나 빈 페이지 삭제로 상쇄하지 않는다.
- 이 Stage에서는 분석 근거 없이 코드나 layout 상수를 수정하지 않는다.

## 상태

시각 증거 확보 완료. 다음 단계는 render-tree 및 source cursor 대조다.

## 분석 결과

- `문단 216`의 36×4 RowBreak 표는 HWP와 HWPX에서 동일한 구조다. `textWrap=Square`, `treatAsChar=false`, `flowWithText=false`, `vertRelTo=Para`, `vertOffset=2,829HU(10mm)`를 가진다.
- HWP와 HWPX의 p47 SVG는 수정 전에도 byte-identical이었다. 따라서 HWPX owner 소실이나 SVG 직렬화 차이가 아니라 공통 pagination/partial-table 경로의 결함이다.
- 수정 전 render tree에서 `문단 216`의 제목 TextLine은 `y=781.1px`, 표 header는 `y=771.8px`였다. 문단 기준 양수 offset을 적용하지 않아 표가 자신의 visible host 제목을 직접 덮었다.
- `Table 214`의 p47 fragment가 PDF보다 길어 후속 `문단 216`의 anchor도 아래로 밀린다. 이 행 metric/fragment-owner 차이는 별도 원인이므로 이 Stage에서 함께 보정하지 않는다.

## 구현

- 첫 RowBreak fragment의 `vertical_offset` 계약을 `TopAndBottom`만이 아니라 `treatAsChar=false`, `vertRelTo=Para`, 양수 offset을 가진 모든 문단 기준 표에 적용했다.
- pagination의 first-fragment budget과 partial-table paint가 같은 offset을 차감·가산한다. wrap mode, 표 ID, 문구, 행 번호, 페이지 번호는 조건으로 사용하지 않는다.

## 결과

- `cargo build --target-dir target/stage150` 성공.
- HWP와 HWPX 각각 `export-svg` 결과가 **383쪽**으로 유지됐다.
- 수정 후 HWP·HWPX p47은 동일하다. `문단 216` 표 header가 `y=809.5px`로 내려가 제목을 직접 덮지 않으며, p48에서 남은 행이 정상 continuation으로 이어진다.
- p47의 footer는 `y=888.2px`이고 `문단 216` 표 bottom은 아직 `944.1px`이다. PDF 기준의 footer-safe 결과와는 여전히 다르다. 원인은 앞선 `문단 214` fragment가 551.0px를 차지해 다음 제목 anchor를 PDF보다 아래로 민 상태이며, 다음 Stage에서 이 표의 row metric과 cursor owner만 분석한다.
- 전체 test suite는 실행하지 않았다.

## 근거 산출물

- 기준 PDF raster: `/tmp/rhwp-3820-stage150-pdf-p47.png`
- 수정 후 HWP raster: `/tmp/rhwp-3820-stage150-offset-hwp-final-p047.png`, `/tmp/rhwp-3820-stage150-offset-hwp-final-p048.png`
- 수정 후 HWPX raster: `/tmp/rhwp-3820-stage150-offset-hwpx-final-p047.png`, `/tmp/rhwp-3820-stage150-offset-hwpx-final-p048.png`
- 수정 후 render tree: `/tmp/rhwp-3820-stage150-offset-hwp-tree/render_tree_047.json`
