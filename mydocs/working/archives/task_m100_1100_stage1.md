# Task M100-1100 Stage 1 작업 기록

## 1. 목적

`samples/hwpx/exam_social.hwpx`가 `samples/exam_social.hwp` 정답지와 다르게 렌더링되는 원인을
문서 정보, IR 차이, 페이지 덤프, SVG 산출물로 좁힌다.

## 2. 대상

```text
source HWPX: samples/hwpx/exam_social.hwpx
oracle HWP:  samples/exam_social.hwp
```

## 3. 산출물

```text
output/poc/hwpx/task1100/stage1_exam_social_render_diff/
```

생성된 SVG:

```text
hwp_svg/exam_social_001.svg .. exam_social_004.svg
hwpx_svg/exam_social_001.svg .. exam_social_007.svg
hwp_overlay/exam_social_001.svg
hwpx_overlay/exam_social_001.svg
```

정리 문서:

```text
output/poc/hwpx/task1100/stage1_exam_social_render_diff/hwp_info.md
output/poc/hwpx/task1100/stage1_exam_social_render_diff/hwpx_info.md
output/poc/hwpx/task1100/stage1_exam_social_render_diff/ir_diff_summary.md
output/poc/hwpx/task1100/stage1_exam_social_render_diff/render_diff_candidates.md
output/poc/hwpx/task1100/stage1_exam_social_render_diff/generated_files.md
```

## 4. 핵심 관찰

HWP 정답지는 4페이지, HWPX 렌더링은 7페이지로 조판된다.

하지만 IR 차이는 매우 작다.

```text
ir-diff summary:
  2건 pos
  1건 cc
  1건 char_offsets[0]
```

즉, 문서 구조나 텍스트가 크게 다른 것이 아니라 동일한 표/문단을 렌더러가 다르게 해석하는 문제에
가깝다.

## 5. 차이가 시작되는 지점

첫 페이지에서 HWP와 HWPX의 조판이 갈라진다.

HWP 정답지:

```text
section0 page1 column0:
  Table pi=6
  ...
  Table pi=12   # 5x2 선택지 표가 한 단 안에 통째로 배치됨

section0 page1 column1:
  FullParagraph pi=14
```

HWPX 렌더링:

```text
section0 page1 column0:
  Table pi=6
  PartialParagraph pi=6 lines=0..1   # TAC 표 호스트 문단이 추가 배치됨
  ...
  PartialTable pi=12 rows=0..3

section0 page1 column1:
  PartialTable pi=12 rows=3..5
```

결과적으로 HWPX는 `pi=12` 선택지 표를 두 단으로 분할하고, HWP에서 첫 페이지 오른쪽 단에 들어가야 할
내용이 다음 페이지로 밀린다.

## 6. `pi=6` TAC 표 비교

`section 0 para 6`은 HWP/HWPX 모두 같은 형태의 호스트 문단이다.

```text
text="  "
line_seg:
  vpos=52477, lh=8970, th=8970, bl=7625, ls=460, cs=1100, sw=30588
table:
  row=3, col=3
  size=30557x8970
  treat_as_char=true
  text_wrap=TopAndBottom
```

차이:

```text
HWP  table.attr = 0x04000004
HWPX table.attr = 0x00000000
```

HWPX XML에는 `hp:pos treatAsChar="1"`가 존재한다. 따라서 HWPX 파서는
`table.common.treat_as_char=true`로 의미를 보존하지만, HWP5 raw `table.attr` bit 0은 없다.

## 7. `pi=12` 선택지 표 비교

`section 0 para 12`도 HWP/HWPX의 의미 정보는 거의 같다.

```text
table:
  row=5, col=2
  page_break=RowBreak
  repeat_header=true
  treat_as_char=true
  size=30613x8580
  outer_margin=141
```

차이:

```text
HWP  table.attr = 0x0400000e
HWPX table.attr = 0x00000000
```

이 표는 HWP 정답지에서는 분할되지 않고 한 단에 배치되지만, HWPX 렌더링에서는
`PartialTable`로 분할된다.

## 8. 코드상 원인 후보

HWPX 파싱 결과는 `table.common.treat_as_char`에 TAC 의미를 가진다.

그런데 HWP 파서와 HWPX 파서가 만드는 `Table` IR이 아직 동일한 의미 계약을 만족하지 않는다.

HWP 파서:

```text
CTRL_HEADER ctrl_data -> CommonObjAttr 파싱
CommonObjAttr.attr -> table.attr 동기화
HWPTAG_TABLE attr -> raw_table_record_attr 보존
raw_ctrl_data 보존
```

HWPX 파서:

```text
hp:pos treatAsChar -> table.common.treat_as_char 파싱
hp:tbl pageBreak/repeatHeader/noAdjust -> 일부 필드 파싱
하지만 CommonObjAttr에 해당하는 packed attr/table.attr/raw_table_record_attr는 구성하지 않음
```

즉, 렌더러가 HWP5 raw attr bit를 직접 보는 것도 문제지만, 더 근본적으로는 HWP에서 이미 구현된
`Table` 의미 필드 materialization이 HWPX 파서 경로에는 빠져 있다.

확인된 코드 지점:

```text
src/renderer/typeset.rs
  format_table:
    let is_tac = table.attr & 0x01 != 0;

  typeset_tac_table:
    tac_count 계산에서 table.attr & 0x01 사용

  place_table_with_text:
    table.attr & 0x01 기준으로 post_table_start / trailing line_spacing 계산

src/renderer/height_measurer.rs
  is_tac_table_inline:
    table.common.treat_as_char 사용
```

즉, HWPX 쪽에서는 TAC 표가 `common.treat_as_char=true`인데도 일부 조판 분기에서는 TAC로 취급되지
않는다. 그 결과 표가 배치된 뒤 호스트 문단의 공백 줄이 `PartialParagraph`로 중복 배치되고,
후속 표가 분할되면서 페이지 수가 4페이지에서 7페이지로 늘어난다.

## 9. Stage 2 제안

다음 단계는 렌더러 우회보다 먼저 HWP 파서가 제공하던 `Table` 의미 계약을 HWPX 파서/IR 경계에서
맞추는 것이다.

수정 방향:

```text
1. HWPX `hp:tbl` + `hp:pos`를 파싱한 뒤 CommonObjAttr packed attr를 구성한다.
   - pack_common_attr_bits(table.common)
   - table.attr 동기화

2. HWPX 표 record 성격의 attr도 HWP 정답지 관례에 맞게 구성 가능한지 확인한다.
   - pageBreak/repeatHeader/noAdjust
   - table padding이 있으면 bit 26 보강
   - 필요 시 `raw_table_record_attr` 구성

3. 그래도 렌더러 내부에 raw attr 직접 참조가 남아 있으면, 그 지점을 IR 의미 helper로 정리한다.

4. HWP 정답지 경로는 기존과 같은 결과를 유지해야 한다.
   - HWP는 attr bit와 common이 둘 다 true인 케이스가 많으므로 동작 변화가 없어야 한다.

5. HWPX `exam_social.hwpx`는 page count와 page item 배치가 HWP 정답지에 가까워지는지 검증한다.
```

검증:

```text
cargo fmt --check
cargo check
target/debug/rhwp dump-pages samples/exam_social.hwp
target/debug/rhwp dump-pages samples/hwpx/exam_social.hwpx
target/debug/rhwp export-svg ...
```

## 10. 승인 요청

Stage 2에서 HWP 파서가 구성하던 `Table` 의미 필드를 HWPX 파서/IR 경계에서도 materialize하고,
그 후에도 남는 렌더러 raw attr 의존 지점을 최소 범위로 정리하겠다.
