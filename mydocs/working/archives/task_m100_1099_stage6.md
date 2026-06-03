# Task M100-1099 Stage 6 작업 기록

## 1. 목적

`samples/hwpx/exam_kor.hwpx` 전체 파일은 rhwp-studio에서는 정상 렌더링되지만, HWP 저장 후
한컴 에디터에서 파일손상 판정을 받았다.

1~4페이지 축소 샘플은 한컴에서 정상 로딩되므로, 전체 파일에서만 등장하는 HWP5 저장 계약을
정답지 `samples/exam_kor.hwp`와 비교해 분리한다.

## 2. 입력

```text
source HWPX: samples/hwpx/exam_kor.hwpx
oracle HWP:  samples/exam_kor.hwp
generated:   saved/111exam_kor.hwp
```

## 3. Stage 7: 3x2 RowBreak 표 CTRL_DATA

정답 HWP와 저장본을 비교한 결과, 저장본에는 정답지의 table 직후 `CTRL_DATA` 9건이 누락되어
있었다.

```text
output/poc/hwpx2hwp/task1099/stage6_full_exam_contract_trace/missing_hints.md
output/poc/hwpx2hwp/task1099/stage6_full_exam_contract_trace/table_ctrl_data_trace.md
```

누락된 record는 모두 같은 104바이트 payload다.

```text
ParameterSet ps_id=0x021b count=1
  item id=0x0242 type=ParameterSet
    ParameterSet ps_id=0x0242 count=11
      item ids: 0x4000..0x400a
      values: 3826, 1048, 28346, 8475, 708, 0, 2, 9, 0, 59528, 84188
```

전체 파일에는 다음 조건의 표가 9개 존재하고, 정답지에는 이 표들 뒤에 동일한 `CTRL_DATA`가 붙는다.

```text
row_count = 3
col_count = 2
page_break = RowBreak
repeat_header = true
```

구현 후보:

```text
src/document_core/converters/hwpx_to_hwp.rs
```

검증 파일:

```text
output/poc/hwpx2hwp/task1099/stage7_full_exam_table_ctrl_data_candidate/exam-kor-1p-stage7.hwp
output/poc/hwpx2hwp/task1099/stage7_full_exam_table_ctrl_data_candidate/exam-kor-4p-stage7.hwp
output/poc/hwpx2hwp/task1099/stage7_full_exam_table_ctrl_data_candidate/exam_kor-stage7.hwp
```

작업지시자 판정:

| file | 한컴 판정 유형 | 바탕쪽 출력 | 지문 박스 출력 | 표/셀 배치 | 마지막 페이지 출력 | 비고 |
|---|---|---|---|---|---|---|
| `exam-kor-1p-stage7.hwp` | 성공 | 성공 | 성공 | 성공 | 성공 | 1p guard |
| `exam-kor-4p-stage7.hwp` | 성공 | 성공 | 성공 | 성공 | 성공 | 4p guard |
| `exam_kor-stage7.hwp` | 파일손상 |  |  |  |  | full target |

Stage 7 결론:

```text
1. table CTRL_DATA 누락은 실제 차이다.
2. 1p/4p guard는 성공한다.
3. full target은 여전히 파일손상이라 full-only 계약이 추가로 남아 있다.
```

## 4. Stage 9: DOC_DATA + FORBIDDEN_CHAR

Stage 7 이후 full target에만 남은 DocInfo 차이를 재확인했다. 정답지 full HWP에는 `DOC_DATA`와
그 하위 `FORBIDDEN_CHAR`가 존재하지만, 1p/4p 축소 샘플에는 없다.

HWPX full header에는 다음 옵션이 있다.

```xml
<hh:docOption>
  <hh:linkinfo pageInherit="1" footnoteInherit="0"/>
</hh:docOption>
```

따라서 `pageInherit=1` 또는 `footnoteInherit=1`일 때만 다음 DocInfo record 묶음을 materialize했다.

```text
DOC_DATA level=0 size=80
FORBIDDEN_CHAR level=1 size=16
```

정답지와 Stage 9 full 산출물 비교:

| record | oracle | generated |
|---|---|---|
| `DOC_DATA` | size 80, hash match | size 80, hash match |
| `FORBIDDEN_CHAR` | size 16, hash match | size 16, hash match |

작업지시자 판정:

```text
exam_kor-stage9.hwp:
  - 한컴 파일손상
  - 단, 이번에는 16페이지까지 렌더링됨
```

Stage 9 결론:

```text
1. Stage 7보다 파일손상 지점이 뒤로 이동했다.
2. rhwp dump-pages 기준 16페이지는 section=1, 17페이지는 section=2다.
3. 남은 손상 지점은 section 2 진입부 계약으로 좁혀졌다.
```

## 5. Stage 10: 섹션 전환 DocInfo bundle

Stage 10은 Stage 9의 `DOC_DATA + FORBIDDEN_CHAR`에 더해, 정답지에만 남은 다음 DocInfo record를
full HWPX의 `docOption/linkinfo` 축에서 materialize했다.

```text
COMPATIBLE_DOCUMENT level=0 size=4
LAYOUT_COMPATIBILITY level=1 size=20
TRACKCHANGE level=1 size=1032
```

정답지와 Stage 10 full 산출물의 payload/hash는 모두 일치했다.

작업지시자 판정:

```text
exam_kor-stage10.hwp:
  - Stage 9와 동일한 지점에서 렌더링이 끝남
  - 한컴 파일손상
```

Stage 10 결론:

```text
COMPATIBLE_DOCUMENT + LAYOUT_COMPATIBILITY + TRACKCHANGE bundle은 이번 파일손상 지점의 직접 원인이 아니다.
```

## 6. Stage 11: 후속 구역 첫 문단 break_type

정답지와 Stage 10 생성본의 section 1/2 첫 문단을 비교했다.

핵심 차이:

```text
oracle section 1 para 0: SectionDef control 존재, ColumnDef control 존재, PARA_HEADER break_type = 쪽나누기(0x04)
generated section 1 para 0: SectionDef control 존재, ColumnDef control 존재, PARA_HEADER break_type = 구역나누기(0x03)

oracle section 2 para 0: SectionDef control 존재, ColumnDef control 존재, PARA_HEADER break_type = 쪽나누기(0x04)
generated section 2 para 0: SectionDef control 존재, ColumnDef control 존재, PARA_HEADER break_type = 구역나누기(0x03)
```

Stage 11 후보는 다음 조건에만 적용한다.

```text
section_idx > 0
first paragraph contains Control::SectionDef
first paragraph raw_break_type != 0x04
```

적용 내용:

```text
first paragraph raw_break_type = 0x04
```

추가 테스트:

```text
following_section_first_paragraph_break_type_materializes_as_page_break
```

작업지시자 판정:

```text
exam_kor-stage11.hwp:
  - 한컴 파일손상
  - Stage 10과 동일한 지점까지 렌더링됨
```

Stage 11 결론:

```text
후속 구역 첫 문단의 break_type 차이는 정답지와 맞춰야 할 계약이지만, 이번 파일손상 지점의 직접 원인은 아니다.
```

## 7. Stage 12: OPTIONAL_PAGE 바탕쪽 extension 직렬화

section 2의 바탕쪽 구조를 정답지와 비교했다.

정답지 section 2:

```text
바탕쪽 [0] Both, is_ext=false, ext_flags=0x0000
바탕쪽 [1] Odd,  is_ext=false, ext_flags=0x0000
바탕쪽 [2] Both, is_ext=true,  overlap=true, ext_flags=0x0007
```

Stage 11 생성본 section 2:

```text
바탕쪽 [0] Both, is_ext=false, ext_flags=0x0000
바탕쪽 [1] Odd,  is_ext=false, ext_flags=0x0000
바탕쪽 [2] Even, is_ext=false, ext_flags=0x0000
```

HWPX `Contents/masterpage8.xml`은 다음 속성을 갖는다.

```xml
<masterPage id="masterpage8" type="OPTIONAL_PAGE" pageNumber="4" pageDuplicate="0" pageFront="0">
```

기존 파서는 `OPTIONAL_PAGE`를 별도로 해석하지 않아 일반 바탕쪽으로 저장했다. HWP5에서는 일반
바탕쪽이 `Both/Odd/Even` 순서로 해석되므로, 세 번째 일반 바탕쪽이 `Even`으로 재해석되었다.

Stage 12 후보는 HWPX `masterPage@type="OPTIONAL_PAGE"`를 다음과 같이 해석한다.

```text
apply_to = Both
is_extension = true
overlap = true
ext_flags = 0x0007
```

수정 파일:

```text
src/parser/hwpx/section.rs
```

추가 테스트:

```text
test_parse_master_page_optional_page_extension
test_parse_master_page_last_page_extension
```

생성 파일:

```text
output/poc/hwpx2hwp/task1099/stage12_optional_masterpage_candidate/exam-kor-1p-stage12.hwp
output/poc/hwpx2hwp/task1099/stage12_optional_masterpage_candidate/exam-kor-4p-stage12.hwp
output/poc/hwpx2hwp/task1099/stage12_optional_masterpage_candidate/exam_kor-stage12.hwp
```

rhwp reload:

```text
exam-kor-1p-stage12.hwp: ok
exam-kor-4p-stage12.hwp: ok
exam_kor-stage12.hwp:    ok, pages=20
```

Stage 12 full target의 section 2 바탕쪽 dump:

```text
바탕쪽 [0] Both, is_ext=false, overlap=false, ext_flags=0x0000
바탕쪽 [1] Odd,  is_ext=false, overlap=false, ext_flags=0x0000
바탕쪽 [2] Both, is_ext=true,  overlap=true,  ext_flags=0x0007
```

작업지시자 판정:

| file | 한컴 판정 유형 | 바탕쪽 출력 | 지문 박스 출력 | 표/셀 배치 | 마지막 페이지 출력 | 비고 |
|---|---|---|---|---|---|---|
| `exam-kor-1p-stage12.hwp` | 성공 | 성공 | 성공 | 성공 | 성공 | 1p guard |
| `exam-kor-4p-stage12.hwp` | 성공 | 성공 | 성공 | 성공 | 성공 | 4p guard |
| `exam_kor-stage12.hwp` | 성공 | 성공 | 성공 | 성공 | 성공 | full target |

Stage 12 결론:

```text
HWPX masterPage type="OPTIONAL_PAGE"
  -> HWP5 일반 Even 바탕쪽이 아니다.
  -> HWP5 extension 바탕쪽으로 저장해야 한다.
  -> apply_to=Both, is_extension=true, overlap=true, ext_flags=0x0007
```

최종 파일손상 해소의 직접 원인은 Stage 12의 `OPTIONAL_PAGE` extension 직렬화다.

## 8. 실행한 검증

```text
cargo fmt --check
cargo test --quiet table_layout_ctrl_data
cargo test --quiet following_section_first_paragraph_break_type_materializes_as_page_break
cargo test --quiet test_parse_master_page_optional_page_extension
cargo test --quiet test_parse_master_page_last_page_extension
```

결과:

```text
success
```
