# Task M100-1094 완료 보고서

## 1. 이슈

GitHub Issue #1094:

```text
[hwpx2hwp] aift 저장 HWP의 한컴 표 높이/페이지 배치 차이
```

## 2. 목표

`samples/hwpx/aift.hwpx`를 HWP로 저장했을 때 한컴 에디터에서 정답 HWP와 다르게 나타나던
표 높이와 페이지 배치 차이를 해결한다.

비교 기준:

```text
source: samples/hwpx/aift.hwpx
oracle: samples/aift.hwp
```

선행 조건:

```text
#1092 메모 컨트롤 HWP 저장 직렬화 성공을 유지한다.
```

## 3. 문제

#1092 이후 생성 HWP는 메모 컨트롤은 정상화되었지만, 한컴 에디터에서 표 배치 차이가 남았다.

관찰된 문제:

```text
1. 2페이지 표의 특정 셀에서 "셀 안쪽 여백 지정"이 비활성화됨
2. 2페이지 표의 행/셀 높이가 정답지와 다르게 산정됨
3. 10페이지 "< 상용화 기술(제품)의 성능 목표> " 다음 표가 한컴 에디터에서 정상 위치에 배치되지 않음
4. 같은 파일을 rhwp-studio에서 열면 정상처럼 보이는 구간이 있어, 한컴 HWP5 record contract 기준 검증이 필요했음
```

## 4. 원인

### 4.1 셀 안쪽 여백 지정

대상 셀의 HWPX 원본에는 다음 속성이 있었다.

```text
hp:tc@hasMargin="1"
```

그러나 기존 HWPX parser는 이 값을 `Cell.apply_inner_margin`으로 보존하지 않았다. 정답 HWP와 생성 HWP를
record 단위로 비교한 결과, 한컴의 "셀 안쪽 여백 지정" 활성화는 TABLE attr 상위 비트만으로 결정되지
않고 셀 `LIST_HEADER`의 `width_ref/property bit0`이 필요했다.

정답 HWP:

```text
LIST_HEADER bytes 6-7 = 01 00
```

기존 생성 HWP:

```text
LIST_HEADER bytes 6-7 = 00 00
```

### 4.2 10페이지 표 나눔

10페이지 제목 뒤 표의 HWPX 원본에는 다음 속성이 있었다.

```text
pageBreak="TABLE"
repeatHeader="1"
```

기존 parser는 `pageBreak="TABLE"`을 처리하지 않아 `TablePageBreak::None`으로 떨어뜨렸다. 정답 HWP와
Stage 4 생성 HWP의 대상 TABLE attr 차이는 bit0 하나였다.

정답 HWP:

```text
TABLE attr = 0x04000005
```

Stage 4 생성 HWP:

```text
TABLE attr = 0x04000004
```

따라서 `pageBreak="TABLE"`은 HWP5 TABLE attr bit0으로 materialize해야 한다.

## 5. 수정

주요 수정 파일:

```text
src/parser/hwpx/section.rs
src/parser/control.rs
src/document_core/converters/hwpx_to_hwp.rs
src/document_core/converters/diagnostics.rs
tests/hwpx_to_hwp_adapter.rs
```

적용 내용:

```text
1. HWPX `hp:tc@hasMargin="1"`을 `Cell.apply_inner_margin = true`로 파싱
2. HWP5 parser에서 셀 LIST_HEADER `width_ref bit0`을 `Cell.apply_inner_margin`으로 복원
3. HWPX -> HWP adapter에서 `Cell.apply_inner_margin`이면 LIST_HEADER `width_ref bit0` materialize
4. HWPX `pageBreak="TABLE"`을 `TablePageBreak::CellBreak`으로 파싱
5. HWPX `pageBreak="CELL"`/`CELL_BREAK`을 `TablePageBreak::RowBreak`으로 파싱
6. adapter에서 `CellBreak`을 `RowBreak`으로 무조건 변환하던 보정 제거
7. HWPX 표의 HWP5 TABLE attr 상위 비트 contract를 기존 table-axis 규칙과 함께 유지
```

## 6. 산출물

단계 보고서:

```text
mydocs/working/task_m100_1094_stage1.md
mydocs/working/task_m100_1094_stage2.md
mydocs/working/task_m100_1094_stage3.md
mydocs/working/task_m100_1094_stage4.md
mydocs/working/task_m100_1094_stage5.md
```

최종 판정 파일:

```text
output/poc/hwpx2hwp/task1094/stage5_page10_table_break_mapping/aift-table-break-mapping.hwp
```

핵심 trace:

```text
output/poc/hwpx2hwp/task1094/stage4_cell_has_margin_width_ref/generated_anchor_trace.md
output/poc/hwpx2hwp/task1094/stage5_page10_table_break_mapping/generated_s2_heading_trace.md
```

## 7. 검증

실행한 검증:

```text
cargo fmt --check
cargo check
cargo build --bin rhwp
cargo test parser::hwpx::section::tests::test_parse_table_cell_has_margin
cargo test parser::hwpx::section::tests::test_parse_table_page_break_table_vs_cell_mapping
cargo test document_core::converters::hwpx_to_hwp::tests::stage3_cell_with_inner_margin_gets_width_ref_bit0
cargo test document_core::converters::hwpx_to_hwp::tests::table_axis_materializes_hancom_record_contract
cargo test document_core::converters::hwpx_to_hwp::tests::table_break_materializes_hwp5_cell_break_bit
cargo test --test hwpx_to_hwp_adapter task888_basic_table_materializes_hancom_table_attrs
```

페이지 번호 컨트롤 확인:

```text
cargo test --test page_number_propagation
cargo test test_634_aift_page4_pagehide_no_page_number
cargo test test_634_aift_page5_pagehide_no_page_number
cargo test test_705_aift_page2_cell_pagehide_collected
cargo test test_705_aift_page2_cell_pagehide_six_fields
cargo test test_705_aift_page3_cell_pagehide_collected
```

결과:

```text
success
```

## 8. 시각 판정

작업지시자 판정:

```text
1. 2페이지 셀 안쪽 여백 지정 문제 해결
2. 10페이지 "< 상용화 기술(제품)의 성능 목표> " 다음 표 배치 문제 해결
3. 페이지번호 감추기, 새 페이지번호 시작 컨트롤 처리 경로 확인 승인
```

## 9. 페이지 번호 컨트롤 확인

`samples/hwpx/aift.hwpx`에는 `pageHiding`, `pageNum`, `newNum`이 실제로 존재한다.

확인한 경로:

```text
HWPX hp:pageHiding -> Control::PageHide -> CTRL_PAGE_HIDE -> renderer page_hide
HWPX hp:newNum     -> Control::NewNumber -> CTRL_NEW_NUMBER -> PageNumberAssigner
HWPX hp:pageNum    -> Control::PageNumberPos -> CTRL_PAGE_NUM_POS -> page_number_pos
```

결론:

```text
페이지번호 감추기(pageHiding hidePageNum)와 새 페이지번호 시작(newNum PAGE)은
파싱, IR, HWP5 저장, 렌더링 페이지 번호 계산 경로가 모두 연결되어 있다.
```

## 10. 결론

#1094는 HWPX 표 속성을 HWP5 table/cell record contract로 materialize하지 못해 한컴 에디터에서
표 높이와 페이지 배치가 틀어지던 문제였다.

이번 작업의 핵심은 다음 두 축이다.

```text
1. hp:tc@hasMargin -> 셀 LIST_HEADER width_ref bit0
2. hp:tbl@pageBreak="TABLE" -> TABLE attr bit0
```

Stage 5 판정으로 #1092 메모 저장 성공을 유지하면서, aift 저장 HWP의 한컴 표 높이/페이지 배치 차이는
완료 처리할 수 있다.
