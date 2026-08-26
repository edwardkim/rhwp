# Stage 149: HWPX 저장 frame 기반 표 fragment 공통 계약 분석

## 목적

2025 행정업무운영 편람의 103x2 병렬 규정 표에 행 번호, 조문 문구, 특정 페이지, 고정
reserve를 적용했던 Stage 143~148 보정을 폐기했다. 이 Stage는 HWPX 저장 layout을 가진
모든 RowBreak 표에 적용할 수 있는 fragment 계약을 찾는다.

## 문제 정의

RowBreak 표는 페이지를 넘을 때 다음 세 좌표계가 반드시 같아야 한다.

- 저장 좌표계: 셀 문단의 `lineSeg` 순서, `vertical_pos`, 행 선언 높이
- 분할 좌표계: `advance_row_cut`의 `start_cut`, `end_cut`, `row_cut_content_height`
- paint 좌표계: 반복 머리행, 표 border, footer를 제외한 visible body area

현재 관찰된 p321 형상은 paint 좌표계의 footer가 `y=646.3`에서 시작하는데 partial
table border가 `y=759.1`까지 생성될 수 있음을 보였다. 원본별 reserve를 더하지 않고,
이 세 좌표계를 공통 predicate로 정합해야 한다.

## 분석 절차

1. HWPX RowBreak 표에서 fragment가 사용할 수 있는 body 높이를 header/footer/각주와
   반복 머리행까지 포함해 하나의 `visible_fragment_budget`으로 정의할 수 있는지 확인한다.
2. complete row와 partial row 모두 `row_cut_content_height`가 이 budget을 초과하면,
   `advance_row_cut` 재시도 또는 다음 fragment 이월 중 어느 처리가 paint 결과와
   정합하는지 기존 일반 fixture로 검증한다.
3. HWPX의 저장 `lineSeg` reset은 단순 문단 시작과 실제 physical frame 경계를 구분할
   수 있을 때만 fragment owner 신호로 사용한다. 표 ID, 행 번호, 조문 텍스트는 사용하지
   않는다.
4. 공통 predicate가 확정된 경우에만 renderer 코드, 일반화 회귀 fixture, 결과를 같은
   커밋에 포함한다.

## 금지 조건

- `HWPX_PARALLEL_REGULATION_*` 같은 문서·행 번호 전용 상수를 추가하지 않는다.
- 문구 검색, 특정 페이지 번호, 표 ID, 셀 개수로 layout 동작을 분기하지 않는다.
- page count만 맞추기 위한 reserve 상쇄 또는 owner break를 추가하지 않는다.

## 상태

분석 시작. 코드 변경 전이며, 새 릴리스 준비 중이므로 merge, push, PR 생성 또는 원격
변경은 금지한다.

## 분석 결과

- 원본 PDF는 383쪽이고, 이전 HWPX 출력은 section 12의 `문단 16` RowBreak 표에서 저장 frame 하나를 앞쪽 fragment에 흡수해 382쪽으로 끝났다.
- 문제 셀은 표의 `r5,c1`이며, lineSeg가 `18,720HU + 900HU`에서 `0HU`으로 reset한 뒤 `5,760HU + 900HU`까지 이어진다. 두 저장 frame의 line end 합계 `26,280HU`는 선언 셀 높이 `29,176HU`의 약 90%다.
- 따라서 이 reset은 문단 내부의 작은 local viewport 재시작이 아니라, 선언된 셀 기하를 두 조각으로 보존한 저장 frame 경계다. body 높이의 일정 비율, 표 ID, 행 번호, 문구, 페이지 번호로 판정할 근거는 없다.
- 병렬 규정 표에서도 같은 종류의 저장 frame reset이 있었으며, 기존의 `HWPX_PARALLEL_REGULATION_*` 행별 reserve는 이 공통 계약을 문서별 상수로 잘못 표현한 것이었다.

## 구현

- `table_layout.rs`의 direct HWPX RowBreak 셀은 다음 중 하나일 때만 저장 frame 후보로 승격한다.
  - synthetic lineSeg를 제외한 reset이 둘 이상이다.
  - reset이 하나이고, reset 전 frame의 line end와 reset 후 frame의 line end 합이 선언 셀 높이의 80% 이상이다.
- 이 판정은 nested table이 없는 direct cell에만 적용한다. 작은 nested cell의 local viewport reset은 기존처럼 물리 page frame으로 승격하지 않는다.
- 저장 frame으로 판정된 unit은 direct/nested 구분 없이 row cut에서 strict boundary로 처리한다. `typeset.rs`의 RowBreak row 스캔도 내부 저장 reset이 있는 row에는 landscape short-row bleed를 허용하지 않는다.
- 이전의 HWPX 병렬 규정 표 row별 reserve와 표 식별 조건은 제거했다.

## 결과

- `cargo build --target-dir target/stage149` 성공.
- `target/stage149/debug/rhwp export-svg 'samples/2025 행정업무운영 편람(최종).hwpx'` 결과는 **383 SVG 페이지**다. PDF 기준 `pdf/2025 행정업무운영 편람(최종)-hwp-2020.pdf`의 383쪽과 일치한다.
- HWPX 372쪽은 `4. 항목란`의 종료로 끝나며, `5. 표`는 373쪽의 새 저장 fragment에서 시작한다. HWPX 373쪽과 374쪽의 `5. 표`·`6. 선`·`7. 칸`, `8. 글자`·`9. 한글과 함께 적는 외국 글자` 흐름은 PDF 373쪽과 374쪽의 순서에 맞는다.
- 끝부분도 HWPX 382쪽의 display 374와 HWPX 383쪽의 발간면이 각각 PDF 382쪽과 383쪽에 대응한다.
- 이 Stage에서는 전체 test suite를 실행하지 않았다. 다음 Stage에서 이 공통 판정의 최소 회귀 테스트와 다른 HWPX fixture 스윕을 별도 범위로 수행한다.

## 근거 산출물

- 원본 lineSeg dump: `/tmp/rhwp-3820-stage149-hwpx-sec12-p16.dump`
- 조판 진단: `/tmp/rhwp-3820-stage149-hwpx-diag-7.log`
- HWPX SVG: `/tmp/rhwp-3820-stage149-hwpx-common-frame`
- 시각 대조 PNG: `/tmp/rhwp-3820-stage149-common-frame-p372.png`, `/tmp/rhwp-3820-stage149-common-frame-p373.png`, `/tmp/rhwp-3820-stage149-common-frame-p374.png`, `/tmp/rhwp-3820-stage149-common-frame-p382.png`, `/tmp/rhwp-3820-stage149-common-frame-p383.png`
