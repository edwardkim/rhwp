# Stage 154: PageHide blank-page owner 지문 퇴역

## 목적

`hwp5_origin_redundant_pagehide_break_marker` 안의 `hwpx_appendix_blank_page`가 사용하는
103x2/206-cell 표 지문을 제거한다. PageHide blank page를 실제로 소유하는 source 구조를
정의하고, 표 shape가 아닌 형제 문단의 marker·break·non-inline object 관계로 판정한다.

## 분석 범위

- section marker의 `PageHide`와 Group shape가 가진 blank-page 소유권
- 다음 `PageBreak` 문단의 non-inline table이 그 blank page 뒤의 본문을 시작하는 관계
- HWP5-origin 중복 PageHide marker와 native HWPX의 유효 blank page 구분

## 금지 조건

- table row/column/cell count, width/height, physical page, paragraph index를 구현 gate로 사용하지 않는다.
- Stage 152/153의 page count와 ColumnBreak carrier 소유권을 되돌리지 않는다.
- 분석 문서만 커밋하지 않는다.

## 완료 기준

- 기존 103x2/206-cell selector의 원본 구조 근거를 기록한다.
- PageHide blank-page owner를 일반 source relation으로 대체한다.
- HWP/HWPX 2025 편람 page count를 다시 확인하고 결과를 문서에 기록한다.

## 원본 구조 분석

HWPX dump의 section 11은 아래 순서로 blank page를 기록한다.

1. section marker는 `Section` break, `PageHide`, 장식용 `Group` shape를 함께 가진다.
2. 바로 다음 문단은 control 없는 빈 문단이다.
3. 그 다음 문단은 `PageBreak`와 `PageHide` 하나만 가진다.
4. 다음 문단은 `PageBreak`와 non-inline table 하나로 본문을 시작한다.

따라서 blank page의 소유권은 표의 행/열 수나 크기가 아니라, 장식 Group을 가진 section
marker와 PageHide marker, 다음 page-starting non-inline table의 형제 관계에 있다. 기존
103x2/206-cell 조건은 이 일반 관계를 특정 표의 shape로 잘못 축소한 구현 지문이었다.

## 구현

`hwp5_origin_redundant_pagehide_break_marker`의 stored-layout HWPX 예외를
`hwpx_pagehide_blank_page_owner`로 교체했다.

- 제거: `row_count == 103`, `col_count == 2`, `cells.len() == 206`
- 보존: stored-layout HWPX, section marker의 `Group + PageHide`, 다음 `PageBreak`의
  non-inline table 관계
- 결과: 해당 PageHide marker는 blank page를 실제로 소유할 때만 HWP5-origin 중복 marker
  제거 대상에서 제외된다.

## 하드코딩 감사

`src`와 현재 working report를 대상으로 `HWPX_QA_*`, `HWP5_ORIGIN_QA_*`,
`NATIVE_HWP5_QA_*`, 두 줄 tail allowance, `hwpx_appendix_blank_page`,
103-row/206-cell selector를 검색했다. 실행 코드에는 남은 항목이 없고, 이전 Stage 문서의
역사적 기록만 남는다.

## 검증 결과

1. `cargo build --target-dir target/stage154`: 성공
2. `target/stage154/debug/rhwp export-svg 'samples/2025 행정업무운영 편람(최종).hwp' ...`:
   `pageCount=383`, `renderedCount=383`
3. `target/stage154/debug/rhwp export-svg 'samples/2025 행정업무운영 편람(최종).hwpx' ...`:
   `pageCount=383`, `renderedCount=383`

export 과정에는 기존 `LAYOUT_OVERFLOW`/`LAYOUT_TABLE_OVERLAP` 진단이 출력됐으나, 명령은
성공했고 이번 변경 전후 HWP/HWPX 페이지 수는 모두 383으로 유지됐다.

## 상태

완료. 표 shape 지문을 source 소유권 관계로 교체했고, 양 형식의 383페이지 보존을 확인했다.
