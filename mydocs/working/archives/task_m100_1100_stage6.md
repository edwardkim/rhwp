# Task #1100 Stage 6 — 머리말 한정 음수 문단내 위 보정

## 1. 배경

Stage 5에서는 HWPX와 HWP의 rhwp SVG 출력이 같아졌지만, 작업지시자가 한컴 편집기 기준으로 다시
확인한 결과 둘 다 같은 방향으로 틀린 상태였다.

문제 지점:

```text
samples/hwpx/exam_social.hwpx
page 2 머리말의 `2(사회·문화)` 글상자
```

한컴 편집기 관찰:

```text
머리말 안 글상자의 `문단내 위` 값이 -13.00mm인 경우,
한컴은 위로 올리지 않고 0mm와 동일하게 배치한다.
```

중요한 제한:

```text
이 보정은 머리말 렌더링에만 적용한다.
본문, 바탕쪽, 꼬리말, 일반 표/글상자에는 적용하지 않는다.
```

## 2. 원인

HWPX의 짝수쪽 머리말에는 표 셀 안에 비-TAC 글상자가 들어 있다.

원본 XML 계약:

```xml
vertRelTo="PARA"
vertAlign="TOP"
vertOffset="4294963611"
```

`4294963611`은 signed i32로 해석하면 `-3685` HWP unit이며, `-13.00mm`이다.

기존 rhwp 렌더러는 이 음수 값을 그대로 적용하여 글상자를 머리말 영역 위로 올렸다.
하지만 한컴 편집기는 머리말 문맥에서 이 음수 값을 0처럼 처리한다.

## 3. 수정

수정 파일:

```text
src/renderer/layout.rs
src/renderer/layout/table_cell_content.rs
src/renderer/layout/table_layout.rs
src/renderer/layout/table_partial.rs
src/renderer/layout/shape_layout.rs
```

구현:

```text
1. layout_header_footer_paragraphs()에서 머리말 여부를 table layout까지 전달한다.
2. table layout/partial table layout에서 해당 플래그를 셀 내부 도형 배치까지 전달한다.
3. 셀 내부 비-TAC 도형이 다음 조건을 만족할 때만 vertical_offset을 0으로 clamp한다.
   - 머리말 문맥
   - vertRelTo=PARA
   - vertAlign=TOP 또는 INSIDE
   - vertical_offset < 0
4. 머리말 바깥의 동일 속성 도형은 기존처럼 음수 offset을 보존한다.
```

## 4. 생성 파일

```text
output/poc/hwpx/task1100/stage8_header_textbox_negative_offset_header_only/exam_social_002.svg
```

대표 좌표:

```text
수정 전:
  rect y = 35.89333333333334
  text y = 73.29333333333334

수정 후:
  rect y = 85.02666666666667
  text y = 122.42666666666668
```

`rect y=85.0266px`는 용지 위쪽 여백 22.0mm 아래의 머리말 영역 내 배치와 일치한다.

## 5. 검증

실행한 검증:

```text
cargo fmt --check
cargo check
cargo run --bin rhwp -- export-svg samples/hwpx/exam_social.hwpx \
  -o output/poc/hwpx/task1100/stage8_header_textbox_negative_offset_header_only -p 1
```

결과:

```text
성공
```
