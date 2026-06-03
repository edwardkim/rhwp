# Task #1100 Stage 5 — 머리말 글상자 PARA 기준 음수 오프셋 반영

## 1. 배경

Stage 4에서 HWPX 바탕쪽의 `PAGE autoNum` 오치환은 제거했지만, 머리말 내부 글상자의 세로
위치가 HWP 정답지와 다르게 출력되는 문제가 남았다.

문제 샘플:

```text
samples/hwpx/exam_social.hwpx
samples/exam_social.hwp
```

작업지시자 관찰:

```text
머리말의 글상자 위치가 세로 문단 위 기준 -13.00이 적용되지 않는다.
새쪽번호 컨트롤에 정의된 쪽번호 위치를 잘못 해석하는 것으로 보인다.
```

## 2. 원인

HWPX의 머리말 2쪽 영역에는 표 셀 안에 비-TAC 글상자가 들어 있고, 해당 글상자 위치는 다음
계약으로 정의되어 있다.

```xml
vertRelTo="PARA"
vertAlign="TOP"
vertOffset="4294963611"
```

`4294963611`은 signed i32로 해석하면 `-3685` HWP unit이며, 이는 `-13.00mm`에 해당한다.

기존 table cell layout은 셀 내부 비-TAC 도형의 세로 기준을 항상 셀 inner area로 잡았다. 그래서
`vertRelTo=PARA`와 문단 기준 음수 오프셋이 반영되지 않았다.

또한 table layout 호출부는 비-TAC 도형을 배치할 때 문단 시작 y가 아니라 문단 조판 후 y를 넘겼다.
이 때문에 `PARA` 기준으로 배치해야 하는 글상자가 문단 위 기준을 잃었다.

## 3. 수정

수정 파일:

```text
src/renderer/layout/table_cell_content.rs
src/renderer/layout/table_layout.rs
src/renderer/layout/table_partial.rs
```

구현:

```text
1. 셀 내부 비-TAC 도형의 vertRelTo가 PARA이면 세로 기준 y를 para_y로 잡는다.
2. TOP/CENTER/BOTTOM 계산은 선택된 기준 영역(ref_y/ref_h)을 기준으로 수행한다.
3. table layout과 partial table layout에서 vertRelTo=PARA인 비-TAC 도형에는
   para_y_before_compose를 anchor y로 전달한다.
4. COLUMN/PAGE/PAPER 등 기존 기준은 기존처럼 셀 inner area 기준을 유지한다.
```

## 4. 생성 파일

```text
output/poc/hwpx/task1100/stage6_header_textbox_para_offset/exam_social_002.svg
output/poc/hwpx/task1100/stage6_header_textbox_para_offset_hwp/exam_social_002.svg
```

대표 확인:

```text
HWPX 머리말 글상자 텍스트 y = 73.29333333333334
HWP  머리말 글상자 텍스트 y = 73.29333333333334
```

폰트 계열 차이는 남아 있지만, 이번 stage의 대상인 머리말 글상자의 세로 위치는 정답지와 일치한다.

## 5. 검증

실행한 검증:

```text
cargo check
cargo fmt --check
target/debug/rhwp export-svg samples/hwpx/exam_social.hwpx \
  -o output/poc/hwpx/task1100/stage6_header_textbox_para_offset -p 1
target/debug/rhwp export-svg samples/exam_social.hwp \
  -o output/poc/hwpx/task1100/stage6_header_textbox_para_offset_hwp -p 1
```

결과:

```text
성공
```
