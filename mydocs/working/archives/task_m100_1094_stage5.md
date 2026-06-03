# Task M100-1094 Stage 5 작업 기록

## 1. 단계 목표

작업지시자 판정:

```text
Stage 4 후보는 2페이지가 성공했다.
하지만 10페이지에서 "< 상용화 기술(제품)의 성능 목표> " 다음 표가
한컴 에디터에서는 정상 위치에 배치되지 않고 이상해진다.
rhwp-studio는 정답 HWP와 동일하게 배치한다.
```

Stage 5는 10페이지 제목 바로 뒤 표의 HWPX source 속성과 HWP5 TABLE record attr를 비교해,
한컴 에디터가 다르게 해석하는 표 나눔 contract를 복구한다.

## 2. 대상 표

원본 HWPX `samples/hwpx/aift.hwpx` Section 2에서 제목 뒤에 오는 표:

```text
제목:  < 상용화 기술(제품)의 성능 목표>
표: rowCnt=13, colCnt=10
HWPX table id = 1854032471
```

HWPX 원본 table 속성:

```text
pageBreak="TABLE"
repeatHeader="1"
rowCnt="13"
colCnt="10"
cellSpacing="0"
borderFillIDRef="6"
noAdjust="0"
```

기존 parser는 `pageBreak="TABLE"`을 처리하지 않아 `TablePageBreak::None`으로 떨어뜨렸다.

## 3. 정답 HWP와 Stage 4 생성 HWP의 차이

대상 표는 HWP5 Section 2, `body_order=123`, `record_index=885`에 해당한다.

정답 HWP TABLE attr:

```text
0x04000005
```

Stage 4 생성 HWP TABLE attr:

```text
0x04000004
```

차이:

```text
bit 0 누락
```

HWPX `pageBreak="TABLE"`은 한컴 HWP5 저장 결과에서 TABLE attr bit 0으로 직렬화된다.
반면 기존 구현은 `CELL`만 읽고, `CellBreak`을 다시 `RowBreak`으로 바꾸는 보정까지 적용하고 있었다.

## 4. 구현 내용

수정 파일:

```text
src/parser/hwpx/section.rs
src/document_core/converters/hwpx_to_hwp.rs
```

구현 규칙:

```text
1. HWPX pageBreak="TABLE" -> HWP5 TABLE attr bit 0
   - IR: TablePageBreak::CellBreak

2. HWPX pageBreak="CELL" -> HWP5 TABLE attr bit 1
   - IR: TablePageBreak::RowBreak

3. adapter에서 CellBreak을 RowBreak으로 무조건 변환하던 보정을 제거
```

정정 사항:

```text
이전 보정:
  HWPX CELL -> CellBreak -> adapter에서 RowBreak 변환

정정 후:
  HWPX TABLE -> CellBreak 보존
  HWPX CELL  -> RowBreak으로 직접 해석
```

## 5. 생성 파일

```text
output/poc/hwpx2hwp/task1094/stage5_page10_table_break_mapping/aift-table-break-mapping.hwp
```

검증 trace:

```text
output/poc/hwpx2hwp/task1094/stage5_page10_table_break_mapping/generated_s2_heading_trace.md
output/poc/hwpx2hwp/task1094/stage5_page10_table_break_mapping/generated_s2_inventory.jsonl
output/poc/hwpx2hwp/task1094/stage5_page10_table_break_mapping/generated_page10_dump.txt
```

생성 HWP의 대상 TABLE attr:

```text
body_order=123 record=885 attr=0x04000005
```

정답 HWP와 동일하게 bit 0이 포함된다.

## 6. 판정표

| file | 한컴 판정 유형 | 셀 안쪽 여백 지정 | 2페이지 표/셀 배치 | 10페이지 제목 뒤 표 배치 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1094/stage5_page10_table_break_mapping/aift-table-break-mapping.hwp` |  |  |  |  |  |  | `pageBreak="TABLE" -> TABLE attr bit0` 후보 |

## 7. 실행한 검증

```text
cargo fmt --check
cargo build --bin rhwp
cargo test parser::hwpx::section::tests::test_parse_table_page_break_table_vs_cell_mapping
cargo test document_core::converters::hwpx_to_hwp::tests::table_axis_materializes_hancom_record_contract
cargo test document_core::converters::hwpx_to_hwp::tests::table_break_materializes_hwp5_cell_break_bit
target/debug/rhwp convert samples/hwpx/aift.hwpx output/poc/hwpx2hwp/task1094/stage5_page10_table_break_mapping/aift-table-break-mapping.hwp
target/debug/rhwp hwp5-inventory ...
target/debug/rhwp hwp5-anchor-trace ...
target/debug/rhwp dump-pages ...
```

결과:

```text
success
```

## 8. 다음 판정 기준

작업지시자 시각 판정에서 확인할 항목:

```text
1. Stage 4에서 성공한 2페이지가 유지되는가
2. 한컴 에디터에서 10페이지 "< 상용화 기술(제품)의 성능 목표> " 다음 표가 정상 배치되는가
3. 마지막 페이지 출력과 메모 표시는 유지되는가
```

## 9. 페이지 번호 컨트롤 확인

작업지시자 판정으로 Stage 5의 10페이지 표 배치 문제는 해결되었다.
추가로 페이지 번호 관련 컨트롤 처리 상태를 확인했다.

### HWPX 입력 확인

`samples/hwpx/aift.hwpx`에는 다음 컨트롤이 존재한다.

```text
section0.xml
  pageNum: pos=BOTTOM_CENTER, formatType=DIGIT, sideChar=-
  pageHiding: hidePageNum=1 포함

section1.xml
  newNum: num=1, numType=PAGE
  pageHiding: hidePageNum=1 포함

section2.xml
  pageHiding: hidePageNum=1 포함
  newNum: num=1, numType=PAGE
```

### 구현 경로

`hp:pageHiding`은 `Control::PageHide`로 파싱되고, HWP5 저장 시 `CTRL_PAGE_HIDE`로
직렬화된다. `hidePageNum`은 HWP5 attr bit `0x20`으로 저장된다.

`hp:newNum`은 `Control::NewNumber`로 파싱되고, HWP5 저장 시 `CTRL_NEW_NUMBER`로
직렬화된다. `numType="PAGE"`일 때 렌더러는 해당 문단이 처음 등장하는 페이지에서만 새
페이지 번호를 적용하고, 이후 페이지는 단조 증가시킨다.

관련 경로:

```text
src/parser/hwpx/section.rs
src/model/control.rs
src/serializer/control.rs
src/serializer/body_text.rs
src/renderer/page_number.rs
src/renderer/typeset.rs
src/renderer/pagination/engine.rs
src/renderer/layout.rs
```

### 실행한 검증

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

결론:

```text
페이지번호 감추기(pageHiding hidePageNum)와 새 페이지번호 시작(newNum PAGE)은
현재 파싱, IR, HWP5 저장, 렌더링 페이지 번호 계산 경로가 모두 연결되어 있다.
```
