# Task #1100 최종 보고서 — exam_social.hwpx HWP 정답지 렌더링 정합

- 이슈: [#1100](https://github.com/edwardkim/rhwp/issues/1100)
- 브랜치: `local/task_m100_1100`
- 마일스톤: M100
- 작성일: 2026-05-24

## 1. 작업 결과

`samples/hwpx/exam_social.hwpx`가 `samples/exam_social.hwp` 정답지와 다르게 렌더링되던 문제를
해결했다.

정정:

```text
Stage 2 시점의 완료 판단은 바탕쪽 웹 렌더링 검증이 빠진 상태였다.
작업지시자 확인 결과 HWPX 웹 캔바스에서 바탕쪽이 보이지 않았고,
Stage 3에서 shape common attr와 line start/end point 누락을 추가로 수정했다.
Stage 4에서 HWPX 바탕쪽 `pageNumber=0` / `hasNumRef=0` 조건을 반영해
바탕쪽 내부 `PAGE autoNum`을 실제 쪽번호로 오인 치환하던 문제를 정정했다.
Stage 5에서 머리말 표 셀 안 비-TAC 글상자의 `vertRelTo=PARA` 및 `vertOffset=-13.00mm`
계약을 확인했다.
Stage 6에서 한컴 편집기 관찰 기준을 반영해, 머리말 문맥에서만 음수 `문단내 위` 값을 0처럼
배치하도록 정정했다.
Stage 7에서 짝수쪽 머리말의 `AutoNumber(Page) + fwSpace + 제목` 구조에서 공백 run 전체를
쪽번호로 바꾸던 문제를 정정했다.
```

핵심 결과:

```text
수정 전 HWPX 렌더링: 7 pages
수정 후 HWPX 렌더링: 4 pages
HWP 정답지 렌더링:    4 pages
```

작업지시자 승인 기준:

```text
- 바탕쪽 1개, 양쪽 적용 유지
- 머리말/꼬리말 영역 유지
- 바탕쪽 `pageNumber=0`인 경우 내부 PAGE autoNum 오출력 없음
- 짝수쪽 머리말 `AutoNumber(Page)`는 한 번만 출력
- page count 4 유지
- TAC 표 뒤 PartialParagraph 재발 없음
- 선택지 표 분할 재발 없음
```

현재 상태:

```text
native SVG 기준:
  - page count 4 유지
  - HWPX 바탕쪽 세로선 좌표가 HWP 정답지와 일치
  - HWPX 바탕쪽 pageNumber=0에서 PAGE autoNum 오치환 제거
  - HWPX 머리말 글상자 음수 `문단내 위` 값은 머리말 문맥에서만 0으로 clamp
  - HWPX 짝수쪽 머리말의 PAGE autoNum placeholder 한 글자만 쪽번호로 치환

웹 캔바스:
  - Stage 7 wasm 빌드 완료
  - pkg 산출물을 web/rhwp.js, web/rhwp_bg.wasm, web/*.d.ts에 반영
  - 작업지시자 웹 판정 대기
```

## 2. 원인

HWP 파서는 표를 읽을 때 다음 두 계약을 IR에 구성하고 있었다.

```text
CTRL_HEADER CommonObjAttr -> table.common.attr/table.attr
HWPTAG_TABLE attr         -> table.raw_table_record_attr
```

반면 HWPX 파서는 `hp:pos`, `hp:tbl`, `hp:sz`의 의미 값은 읽었지만 HWP 경로와 같은 packed
table attr를 구성하지 않았다.

그 결과 HWPX 렌더링에서 TAC 표의 조판 계약이 HWP 정답지와 달라졌고, page 1에서 다음 발산이 발생했다.

```text
수정 전:
  Table pi=6
  PartialParagraph pi=6
  PartialTable pi=12 rows=0..3
  PartialTable pi=12 rows=3..5

수정 후:
  Table pi=6
  Table pi=12
```

이번 문제는 렌더러 일반 규칙 문제가 아니라, HWPX parser가 HWP parser와 같은 Table IR 계약을
구성하지 않은 문제로 확정했다.

## 3. 변경 내용

수정 파일:

```text
src/parser/hwpx/section.rs
src/document_core/converters/hwpx_to_hwp.rs
src/serializer/hwpx/table.rs
src/model/header_footer.rs
src/parser/body_text.rs
src/renderer/layout.rs
src/renderer/layout/shape_layout.rs
src/renderer/layout/table_cell_content.rs
src/renderer/layout/table_layout.rs
src/renderer/layout/table_partial.rs
tests/issue_1100_exam_social_hwpx_header.rs
```

구현 내용:

```text
1. HWPX table parsing 후 CommonObjAttr bitfield를 materialize한다.
2. hp:tbl pageBreak/repeatHeader/noAdjust/inMargin을 TABLE record attr로 materialize한다.
3. table.attr를 CommonObjAttr로 쓰더라도 noAdjust가 사라지지 않도록 raw_table_record_attr를 함께 참조한다.
4. HWPX serializer의 noAdjust 판단도 raw_table_record_attr를 함께 참조하도록 정합한다.
5. HWPX shape parsing 후 CommonObjAttr bitfield를 materialize한다.
6. HWPX hp:line의 hc:startPt / hc:endPt를 LineShape.start/end로 파싱한다.
7. HWPX masterPage@pageNumber를 보존하고, pageNumber=0 + hasNumRef=0인 바탕쪽에서는
   내부 PAGE autoNum 치환을 억제한다.
8. 셀 내부 비-TAC 글상자의 vertRelTo=PARA를 문단 시작 y 기준으로 배치한다.
9. 머리말 문맥에서만 비-TAC 글상자의 `vertRelTo=PARA`, `vertAlign=TOP/INSIDE`,
   `vertical_offset < 0` 조건을 0 offset 배치로 clamp한다.
10. AutoNumber(Page)는 공백 run 전체가 아니라 placeholder 문자만 현재 쪽번호로 치환한다.
```

대표 검증 값:

```text
section0 para12:
  HWPX after fix raw_table_record_attr = 0x0400000e
  HWP oracle    raw_table_record_attr = 0x0400000e

masterpage line:
  HWPX after fix common.attr = 0x044a4700
  HWPX after fix start/end   = (0,0) -> (100,100)
  HWPX SVG line              = (514.006,132.16) -> (514.02,1364.28)
  HWP oracle SVG line        = (514.006,132.16) -> (514.02,1364.28)

page2 header text box:
  HWPX vertRelTo=PARA, vertOffset=-13.00mm
  Stage 5 rhwp SVG text y = 73.29333333333334
  Stage 6 rhwp SVG text y = 122.42666666666668
  Stage 6 rhwp SVG rect y = 85.02666666666667

page2 even header page auto number:
  first placeholder = "2"
  following fwSpace = "\u{2007}"
```

Stage 5의 HWP/HWPX SVG 일치는 rhwp HWP 렌더러가 같은 위치 해석을 공유한 결과였으므로 최종
판정 근거로 사용하지 않는다. Stage 6에서는 작업지시자가 확인한 한컴 편집기 동작을 기준으로
머리말 문맥에 한정해 보정했다.

## 4. 산출물

단계 기록:

```text
mydocs/working/task_m100_1100_stage1.md
mydocs/working/task_m100_1100_stage2.md
mydocs/working/task_m100_1100_stage3.md
mydocs/working/task_m100_1100_stage4.md
mydocs/working/task_m100_1100_stage5.md
mydocs/working/task_m100_1100_stage6.md
mydocs/working/task_m100_1100_stage7.md
mydocs/working/task_m100_1100_stage8.md
```

시각 판정 파일:

```text
output/poc/hwpx/task1100/stage2_table_attr_materialization/hwp_svg/
output/poc/hwpx/task1100/stage2_table_attr_materialization/hwpx_svg/
output/poc/hwpx/task1100/stage2_table_attr_materialization/hwpx_overlay/
output/poc/hwpx/task1100/stage4_hwpx_line_points_svg/
output/poc/hwpx/task1100/stage5_masterpage_page_number_guard/
output/poc/hwpx/task1100/stage6_header_textbox_para_offset/
output/poc/hwpx/task1100/stage6_header_textbox_para_offset_hwp/
output/poc/hwpx/task1100/stage8_header_textbox_negative_offset_header_only/
output/poc/hwpx/task1100/stage9_even_header_page_auto_once/
output/poc/hwpx/task1100/stage10_footer_page_number_restore/
```

## 5. 검증

실행한 검증:

```text
cargo fmt --check
cargo test parser::hwpx::section::tests::test_parse_hwpx_table_materializes_hwp_common_attrs
cargo test parser::hwpx::section::tests::test_parse_hwpx_masterpage_line_materializes_shape_common_attr
cargo test parser::hwpx::section::tests
cargo check
cargo test issue_1100_hwpx_header_negative_para_offset_clamped_to_header_origin
cargo test --test issue_1100_exam_social_hwpx_header
cargo build --bin rhwp
target/debug/rhwp dump-pages samples/hwpx/exam_social.hwpx
target/debug/rhwp info samples/hwpx/exam_social.hwpx
target/debug/rhwp export-svg samples/hwpx/exam_social.hwpx
target/debug/rhwp export-svg samples/hwpx/exam_social.hwpx -o output/poc/hwpx/task1100/stage5_masterpage_page_number_guard -p 0
target/debug/rhwp export-svg samples/hwpx/exam_social.hwpx -o output/poc/hwpx/task1100/stage6_header_textbox_para_offset -p 1
target/debug/rhwp export-svg samples/exam_social.hwp -o output/poc/hwpx/task1100/stage6_header_textbox_para_offset_hwp -p 1
target/debug/rhwp export-svg samples/hwpx/exam_social.hwpx -o output/poc/hwpx/task1100/stage8_header_textbox_negative_offset_header_only -p 1
target/debug/rhwp export-svg samples/hwpx/exam_social.hwpx -o output/poc/hwpx/task1100/stage9_even_header_page_auto_once -p 1
target/debug/rhwp export-svg samples/hwpx/exam_social.hwpx -o output/poc/hwpx/task1100/stage10_footer_page_number_restore -p 1
target/debug/rhwp export-svg samples/exam_social.hwp
docker compose --env-file .env.docker run --rm wasm
```

최종 확인:

```text
cargo fmt --check: 통과
관련 회귀 테스트: 통과
Issue #1100 머리말 음수 offset 회귀 테스트: 통과
Issue #1100 짝수쪽 머리말 AutoNumber(Page) 단일 치환 회귀 테스트: 통과
Issue #1100 바탕쪽 하단 쪽번호 보존 회귀 테스트: 통과
cargo check: 통과
Stage 4 wasm build: 통과
Stage 5 wasm build: 통과
Stage 6 wasm build: 통과
Stage 7 wasm build: 통과
Stage 8 wasm build: 통과
Stage 3 작업지시자 웹 시각 판정: 통과
Stage 4 native SVG 확인: 통과
Stage 5 native SVG 확인: 통과
Stage 6 native SVG 확인: 통과
Stage 8 native SVG 확인: 통과
Stage 8 웹 캔바스 시각 판정: 통과
```

비고:

```text
export-svg 중 page 3에서 HWP 정답지와 동일한 LAYOUT_OVERFLOW 경고가 1회 출력된다.
이번 수정으로 새로 생긴 경고가 아니라 정답지 경로에서도 관찰되는 기존 조판 경계 경고다.
```

## 6. 완료 판단

Issue #1100의 범위인 `exam_social.hwpx`의 HWP 정답지 대비 HWPX 렌더링 차이는 native SVG 및
웹 캔바스 기준으로 해결되었다.

이번 수정은 HWPX를 HWP로 저장하는 기능이 아니라, HWPX parser가 HWP parser와 동일한 Table IR
계약 및 Shape IR 계약을 생성하도록 맞춘 렌더링 정합 작업이다.

Stage 8 수정까지 포함한 wasm 재빌드와 작업지시자 웹 캔바스 시각 판정까지 완료했다.
