# Stage 152: 저장 frame tail 하드코딩 퇴역

## 목적

기존 `HWPX_QA_*`, `HWP5_ORIGIN_QA_*`, `NATIVE_HWP5_QA_*` allowance와
6x5/15-cell/높이값 조건을 전부 제거하고, HWP/HWPX가 저장한 공통 구조로
RowBreak 표의 tail 소유자를 결정한다.

## 분석 결과

기존 Q&A 보정은 서로 다른 16px, 32px, 48px, 64px, 65px, 96px 값을 사용했지만,
원본 자료에서는 다음 세 구조로 환원된다.

1. 셀 내부 `LINE_SEG` vpos rewind 앞의 source frame tail
2. 마지막 가시 응답 행 뒤에 source-empty spacer가 있는 terminal response
3. normal capacity cut이 stored-frame 직전의 짧은 tail만 남기는 native HWP5 giant cell

이들은 표 행수, 열수, 셀 수, 표 높이, 문단 수, physical page, fixture 이름과 무관한
`CellUnit`/`MeasuredTable` 구조다.

## 구현

- `stored_frame_cut_for_row`는 normal orphan 정책을 거치지 않고
  `CellUnit.stored_frame_break_before`까지의 정확한 source frame을 계산한다.
- `paragraph_tail_cut_for_row`는 terminal response의 normal cut이 문단 중간에서 멈춘 경우,
  stored frame을 넘지 않는 범위에서 그 문단 끝까지만 확장한다.
- `typeset.rs`는 마지막 visible response와 source-empty spacer를 공통 gate로 사용한다.
  선택된 source tail의 overflow는 고정 픽셀이 아니라 실제 painted candidate와 body area의 차이로 계산한다.
- native HWP5 row-cut에는 stored-frame 직전의 48px 이하 sliver를 흡수하는 공통 경로를 연결했다.
  HWPX의 vpos reset은 local reset 의미도 가지므로 이 물리-frame 흡수 경로에서는 제외하고,
  동일한 terminal-response source contract를 사용한다.

## 제거한 구현 지문

- `HWPX_QA_*`, `HWP5_ORIGIN_QA_*`, `NATIVE_HWP5_QA_*` 상수와 terminal-row inline 상수
- Q&A의 `row_count == 6`, `col_count == 5`, `cells.len() == 15`
- Q&A의 `common.height == 13_042|11_382|11_315|19_355|15_224|18_084|23_988|29_772|15_385|47_726`

정적 검색에서 위 QA 상수와 지문은 결과가 없었다. 남아 있는 `row_count == 6` 한 건은
다른 `hwpx_appendix_design_table_trailing_column_break` 규칙의 6x3 표 계약이며, 다음 Stage에서
별도로 분석한다.

## 검증 결과

- `cargo build --target-dir target/stage152`: 성공
- `export-svg samples/2025 행정업무운영 편람(최종).hwp`: 383쪽
- `export-svg samples/2025 행정업무운영 편람(최종).hwpx`: 383쪽
- 47쪽 PNG: HWP/HWPX 모두 표 하단과 page number `39`가 분리되어 겹치지 않는다.

## 상태

완료. 다음 Stage는 남아 있는 별도 문서 지문 selector의 구조적 대체를 분석한다.
