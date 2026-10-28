# Stage 153: 문서 지문 selector 전수 감사

## 목적

Stage 152에서 제거한 Q&A allowance 외에 renderer에 남아 있는 행수, 열수, 셀 수, 표 크기,
문단 수, 저장 좌표 조합 기반 selector를 전수 분류한다. 각 selector를 source-format 공통 계약,
일반 레이아웃 계약, 또는 제거 대상 문서 지문으로 구분하고 제거 대상은 공통 로직으로 대체한다.

## 범위

- `src/renderer/typeset.rs`와 `src/renderer/layout/table_layout.rs`의 표 shape/size 기반 분기
- profile 이름과 함께 특정 행·열·높이·문단 수를 검사하는 helper 및 inline 분기
- Stage 152에서 남은 `hwpx_appendix_design_table_trailing_column_break`의 11x2/6x3 selector

## 금지 조건

- fixture 이름, 물리 페이지, paragraph index, table height, 행수/열수/셀 수 조합을 새 구현 gate로 추가하지 않는다.
- Stage 152의 source-frame, source-paragraph, source-empty spacer 공통 계약을 되돌리지 않는다.
- 분석 문서만 커밋하지 않는다. 구현과 결과 검증을 같은 Stage 커밋에 포함한다.

## 완료 기준

- 후보 selector마다 현재 역할과 source-structure 대체 가능성을 기록한다.
- 제거 가능한 selector는 공통 계약으로 대체하고 관련 HWP/HWPX page count를 확인한다.
- 제거할 수 없는 format semantic은 문서 지문이 아닌 이유와 독립된 입력 계약을 문서에 남긴다.

## 후보 분류

### 문서 지문으로 판정한 후보

`hwpx_appendix_design_table_trailing_column_break`는 빈 `ColumnBreak` 문단의
`vpos == 28030`, 앞표의 11x2/22-cell/29491x36625, 뒷표의
6x3/10-cell/29254x35894를 동시에 확인했다. 이 조합은 일반 포맷 의미가 아니라
특정 2025 편람 별표의 물리 배치를 식별하는 구현 지문이다.

원본 HWPX dump에서는 빈 carrier 문단 12.17이 앞의 non-inline `RowBreak` 표와
다음의 non-inline `PageBreak` 표 사이에 있다. 다음 표의 PageBreak가 새 physical page의
유일한 owner이므로 carrier의 ColumnBreak는 별도 page를 열지 않는다.

### format semantic으로 보존한 조건

- `RowBreak`, `treat_as_char`, `TextWrap`, `VertRelTo`, row-span 여부는 입력 포맷이
  정의한 배치 의미이므로 유지한다.
- 1x1 RowBreak 및 stored frame reset 조건은 문서 크기가 아니라 cell fragment owner를
  결정하는 일반 구조이므로 Stage 152 공통 계약의 일부로 유지한다.

## 구현

- 함수를 `empty_table_carrier_column_break_before_page_table`로 변경했다.
- 빈 control-less `ColumnBreak`의 직전 형제와 다음 형제가 각각 non-inline table이고,
  다음 형제가 `PageBreak`일 때만 carrier로 억제한다.
- 표 행수·열수·셀 수·폭·높이와 carrier의 저장 vpos를 모두 제거했다.

## 검증 결과

- `cargo build --target-dir target/stage153`: 성공
- `export-svg samples/2025 행정업무운영 편람(최종).hwp`: 383쪽
- `export-svg samples/2025 행정업무운영 편람(최종).hwpx`: 383쪽
- 정적 검색: 이전 함수명과 `29_491`, `36_625`, `29_254`, `35_894`,
  `vertical_pos == 28_030`은 `src/renderer/typeset.rs`에 남지 않았다.

## 상태

완료. 다음 Stage에서는 남은 source-format별 수치 tolerance를 문서 지문과 구분해,
입력 구조로 계산 가능한 경우부터 퇴역한다.
