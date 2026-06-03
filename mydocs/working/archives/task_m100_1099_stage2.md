# Task M100-1099 Stage 2 작업 기록

## 1. 목적

Stage 1에서 확인한 1순위 파일손상 후보를 먼저 제거한다.

후보:

```text
Footer/Header 내부 Table tuple이 HWPX→HWP adapter 보강을 받지 못해
CTRL_HEADER(Table) payload가 4바이트, TABLE payload가 22바이트로 저장되는 문제
```

## 2. 원인

`src/serializer/control.rs`의 `serialize_table()`은 `table.raw_ctrl_data`가 비어 있으면
`CTRL_HEADER(Table)`에 ctrl id 4바이트만 기록한다.

```text
raw_ctrl_data 있음  → CTRL_HEADER(Table) = 4 + CommonObjAttr payload
raw_ctrl_data 없음  → CTRL_HEADER(Table) = 4
```

일반 본문 문단과 표 셀 내부 문단은 HWPX→HWP adapter의 `adapt_paragraph()`를 통과한다.
그러나 기존 구현은 `Control::Header`, `Control::Footer` 내부 문단으로 재귀 진입하지 않았다.

따라서 꼬리말/머리말 내부 표는 다음 보강을 받지 못했다.

```text
table.raw_ctrl_data
table.raw_table_record_attr
table.raw_table_record_extra
cell.raw_list_extra
```

## 3. 구현

수정 파일:

```text
src/document_core/converters/hwpx_to_hwp.rs
```

수정 내용:

```text
1. `adapt_paragraphs()` 헬퍼 추가
2. `adapt_paragraph()`에서 `Control::Header`, `Control::Footer` 내부 문단으로 재귀 진입
3. 기존 SectionDef master page / shape text box 재귀도 동일 헬퍼를 사용하도록 정리
4. Header/Footer 내부 표가 adapter materialization을 받는 단위 테스트 추가
```

추가 테스트:

```text
header_footer_nested_tables_are_materialized
```

## 4. 생성 파일

```text
output/poc/hwpx2hwp/task1099/stage2_header_footer_table_contract/exam-kor-1p-stage2.hwp
```

파일 크기:

```text
683,008 bytes
```

rhwp reload:

```text
ok, pages=1
```

## 5. Stage 1 대비 개선

Stage 1:

```text
BodyText.Section0#101~BodyText.Section0#157
  oracle    CTRL_HEADER(Table) size = 46
  generated CTRL_HEADER(Table) size = 4

BodyText.Section0#102~BodyText.Section0#158
  oracle    TABLE size = 24, row_count_hint = 1, tail_after_0x16 = 00 00
  generated TABLE size = 22, row_count_hint = 1428, tail_after_0x16 = 없음
```

Stage 2:

```text
BodyText.Section0#102~BodyText.Section0#158
  oracle    TABLE size = 24, row_count_hint = 1, tail_after_0x16 = 00 00
  generated TABLE size = 24, row_count_hint = 1, tail_after_0x16 = 00 00
```

`table_field_diff.md` 기준으로, Stage 1의 4바이트 `CTRL_HEADER(Table)` / 22바이트 `TABLE`
구조 결함은 제거되었다.

## 6. 남은 차이

Stage 2 이후에도 정답지와 다음 차이는 남아 있다.

```text
1. SectionDef subtree의 master page / footer / shape graph 순서 차이
2. DocInfo compatibility 계열 record 누락
   - FORBIDDEN_CHAR
   - COMPATIBLE_DOCUMENT
   - LAYOUT_COMPATIBILITY
   - TRACKCHANGE
3. FACE_NAME / PARA_SHAPE payload 차이
4. GenShape / Picture tuple payload 차이
```

다만 이번 단계는 한컴 에디터가 싫어할 가능성이 높은 structural defect 하나를 제거했다.

## 7. 검증

실행:

```text
cargo fmt --check
cargo test document_core::converters::hwpx_to_hwp::tests::header_footer_nested_tables_are_materialized
cargo build --bin rhwp
```

결과:

```text
success
```

테스트 실행 중 기존 경고는 출력되었지만 실패는 없다.

## 8. 판정 요청

한컴 에디터 확인 대상:

```text
output/poc/hwpx2hwp/task1099/stage2_header_footer_table_contract/exam-kor-1p-stage2.hwp
```

| file | 한컴 판정 유형 | 이미지 출력 | 바탕쪽 출력 | 지문 박스 출력 | 마지막 페이지 출력 | 비고 |
|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1099/stage2_header_footer_table_contract/exam-kor-1p-stage2.hwp` | 파일손상 |  |  |  |  | Header/Footer 내부 Table contract 후보 |

## 9. 다음 판단

```text
1. 이 파일이 한컴에서 정상 로딩되면:
   - 1페이지 파일손상 원인은 Header/Footer 내부 Table contract 누락으로 확정한다.
   - 2/3/4페이지 축소 샘플로 확장한다.

2. 이 파일도 파일손상이면:
   - 다음 후보는 SectionDef subtree graph 또는 DocInfo compatibility 계열로 넘어간다.
   - 단, 이번 수정은 4바이트 Table header 결함 제거로 유지한다.
```

## 10. 판정 후 결론

작업지시자 판정:

```text
파일손상
```

해석:

```text
1. Header/Footer 내부 Table tuple materialization은 필요한 수정이지만 파일손상의 최종 원인은 아니다.
2. Stage 1에서 확인된 4바이트 CTRL_HEADER(Table) / 22바이트 TABLE 결함은 제거되었으므로 유지한다.
3. 다음 단계는 SectionDef subtree graph 또는 DocInfo compatibility record 누락을 대상으로 한다.
```
