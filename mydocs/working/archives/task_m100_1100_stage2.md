# Task M100-1100 Stage 2 작업 기록

## 1. 목적

Stage 1에서 확인한 HWP/HWPX 표 IR 차이를 해결한다.

핵심 문제:

```text
HWP 파서:
  CTRL_HEADER CommonObjAttr -> table.common.attr -> table.attr 동기화
  HWPTAG_TABLE attr -> table.raw_table_record_attr 보존

HWPX 파서:
  hp:pos/hp:tbl 의미 필드는 파싱하지만 table.common.attr/table.attr/raw_table_record_attr를 구성하지 않음
```

이 차이 때문에 렌더러가 HWPX의 TAC 표를 HWP와 다르게 조판했다.

## 2. 수정한 소스

```text
src/parser/hwpx/section.rs
src/document_core/converters/hwpx_to_hwp.rs
src/serializer/hwpx/table.rs
```

## 3. 구현 내용

### 3.1 HWPX Table CommonObjAttr materialization

`hp:tbl` 파싱이 끝난 뒤 다음 값을 구성한다.

```text
table.common.attr = HWP5 CommonObjAttr bit packing + 0x08000000
table.attr = table.common.attr
```

반영된 HWPX 속성:

```text
hp:pos@treatAsChar
hp:pos@flowWithText
hp:pos@allowOverlap
hp:pos@holdAnchorAndSO
hp:pos@vertRelTo / horzRelTo
hp:pos@vertAlign / horzAlign
hp:pos@vertOffset / horzOffset
hp:tbl@textWrap
hp:sz@widthRelTo / heightRelTo
```

### 3.2 HWPX Table record attr materialization

`hp:tbl`의 HWP5 TABLE record 성격도 구성한다.

```text
pageBreak      -> low bits
repeatHeader   -> bit 2
noAdjust       -> bit 3
inMargin 있음 -> bit 26
```

### 3.3 noAdjust 보존

`table.attr`를 CommonObjAttr와 동기화하면 기존처럼 `table.attr & 0x08`만으로는
HWPX `noAdjust`를 판단할 수 없다. 따라서 다음 경로에서 `raw_table_record_attr`도 함께 참조하도록
수정했다.

```text
src/document_core/converters/hwpx_to_hwp.rs
src/serializer/hwpx/table.rs
```

## 4. 검증 결과

### 4.1 페이지 수

수정 전:

```text
samples/hwpx/exam_social.hwpx -> 7 pages
```

수정 후:

```text
samples/hwpx/exam_social.hwpx -> 4 pages
samples/exam_social.hwp       -> 4 pages
```

### 4.2 최초 발산 지점 해소

수정 전 HWPX page 1:

```text
Table pi=6
PartialParagraph pi=6 lines=0..1
PartialTable pi=12 rows=0..3
PartialTable pi=12 rows=3..5
```

수정 후 HWPX page 1:

```text
Table pi=6
Table pi=12
```

`pi=6`의 중복 `PartialParagraph`가 사라지고, `pi=12` 선택지 표가 HWP 정답지처럼 한 단에 통째로
배치된다.

### 4.3 대표 표 attr

`section0 para12`:

```text
HWPX after fix:
  raw_table_record_attr = 0x0400000e
  common.treat_as_char = true
  common.wrap = TopAndBottom

HWP oracle:
  raw_table_record_attr = 0x0400000e
```

## 5. 생성한 시각 판정 파일

```text
output/poc/hwpx/task1100/stage2_table_attr_materialization/hwp_svg/exam_social_001.svg
output/poc/hwpx/task1100/stage2_table_attr_materialization/hwp_svg/exam_social_002.svg
output/poc/hwpx/task1100/stage2_table_attr_materialization/hwp_svg/exam_social_003.svg
output/poc/hwpx/task1100/stage2_table_attr_materialization/hwp_svg/exam_social_004.svg

output/poc/hwpx/task1100/stage2_table_attr_materialization/hwpx_svg/exam_social_001.svg
output/poc/hwpx/task1100/stage2_table_attr_materialization/hwpx_svg/exam_social_002.svg
output/poc/hwpx/task1100/stage2_table_attr_materialization/hwpx_svg/exam_social_003.svg
output/poc/hwpx/task1100/stage2_table_attr_materialization/hwpx_svg/exam_social_004.svg

output/poc/hwpx/task1100/stage2_table_attr_materialization/hwpx_overlay/exam_social_001.svg
```

## 6. 실행한 검증

```text
cargo fmt --check
cargo test parser::hwpx::section::tests::test_parse_hwpx_table_materializes_hwp_common_attrs
cargo check
cargo build --bin rhwp
target/debug/rhwp dump-pages samples/hwpx/exam_social.hwpx
target/debug/rhwp info samples/hwpx/exam_social.hwpx
target/debug/rhwp export-svg samples/hwpx/exam_social.hwpx ...
target/debug/rhwp export-svg samples/exam_social.hwp ...
```

결과:

```text
성공
```

`export-svg` 중 page 3에서 기존 HWP 정답지와 동일한 `LAYOUT_OVERFLOW` 경고가 1회 출력된다.

## 7. 추가 회귀 기준

작업지시자 추가 확인 사항:

```text
exam_social.hwpx:
  - 바탕쪽은 1개로 구성된다.
  - 해당 바탕쪽은 양쪽 페이지에 적용된다.
  - 머리말 영역과 꼬리말 영역이 구성되어 있다.
```

따라서 Stage 2 수정은 표 배치만 맞추는 것으로 끝나면 안 된다. 다음 항목도 함께 유지되어야 한다.

```text
1. HWPX 렌더링 page count가 HWP 정답지와 동일한 4페이지를 유지한다.
2. 바탕쪽 적용 범위가 HWP 정답지와 다르게 분기되지 않는다.
3. 머리말/꼬리말 영역이 본문 조판을 밀거나 당기지 않는다.
4. Stage 1의 최초 발산 지점인 TAC 표 뒤 PartialParagraph 및 선택지 표 분할이 재발하지 않는다.
```

이번 구현은 renderer 우회가 아니라 HWP 파서에서 이미 구성하던 Table IR 계약을 HWPX 파서에서도
구성하도록 맞춘 것이다. 그러므로 바탕쪽/머리말/꼬리말 자체를 변경하지 않고, 본문 TAC 표의
CommonObjAttr 및 TABLE record attr 누락만 보정한다.

## 8. 승인 상태와 다음 단계

작업지시자 시각 판정을 요청한다.

판정 대상:

```text
output/poc/hwpx/task1100/stage2_table_attr_materialization/hwpx_svg/
```

Stage 2 방향은 작업지시자 승인을 받았다. 시각 판정에서 HWP 정답지와 HWPX SVG가 일치하면 최종
보고서 작성 단계로 넘어간다.
