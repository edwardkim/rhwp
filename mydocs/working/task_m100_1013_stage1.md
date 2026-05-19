# Task M100-1013 Stage 1 분석 보고서

## 1. 목적

`hy-002` 샘플에서 2페이지 글상자 내부 이미지가 한컴 에디터/PDF보다 작게 렌더링되는 원인을
구현 전에 분리한다.

대상 파일:

```text
samples/hwpx/hy-002.hwpx
samples/hwpx/hancom-hwp/hy-002.hwp
pdf-large/hwpx/hy-002.pdf
```

## 2. 분석 산출물

```text
output/poc/hwpx2hwp/task_m100_1013/stage1_hy002_textbox_picture_size/
```

주요 파일:

```text
hy002_hwpx_textbox_picture_trace.md
hy002_ir_layout_metrics.md
stage1_findings.md
svg_hwpx/hy-002_002.svg
svg_hwp/hy-002_002.svg
```

## 3. 문제 구조

문제 그림은 단독 그림 shape가 아니라 글상자 내부 문단의 non-TAC 그림이다.

```text
hp:rect
  hp:drawText
    hp:subList
      hp:p
        hp:run
          hp:pic treatAsChar="0"
```

이 구조는 편집자가 그림을 페이지에 직접 올린 것이 아니라, 글상자라는 컨테이너 안에서 문단 내용처럼
배치하려 한 것으로 보인다. 한컴은 이 경우 글상자 내부 조판 계약을 적용한다.

## 4. 기준 크기

HWPX의 내부 그림 표시 크기:

```text
pic curSz = 48190 x 4331 HU
96dpi 기준 ~= 642.5 x 57.7 px
```

정답 PDF의 page 2 이미지:

```text
bitmap = 837 x 75 px
ppi = 125 x 125
96dpi 환산 ~= 642.8 x 57.6 px
```

따라서 한컴/PDF 기준은 HWPX의 `pic curSz/sz`와 일치한다.

## 5. 현재 rhwp 출력

현재 rhwp SVG export에서 같은 그림은 다음 크기로 출력된다.

```text
width  = 395.37 px
height = 35.53 px
```

즉, 기대 크기의 약 `61.5%`로 축소된다.

## 6. 원인 분석

HWPX 입력과 한컴 HWP 입력을 dump하면 대상 글상자와 그림은 실질적으로 같은 IR로 들어온다.

```text
HWPX -> IR: 동일 구조
HWP  -> IR: 동일 구조
```

따라서 원인은 parser 차이가 아니라 renderer의 글상자 내부 배치 계산이다.

현재 `layout_textbox_content`는 글상자 `vertAlign=CENTER`를 처리할 때 line segment 높이만으로
content height를 계산한다.

```text
글상자 inner height ~= 57.7 px
line segment height ~= 13.3 px
center offset ~= (57.7 - 13.3) / 2 ~= 22.2 px
```

그 뒤 non-TAC 그림을 배치할 때 `inline_y` 이후 남은 높이만 picture container height로 사용한다.

```text
remaining height ~= 57.7 - 22.2 ~= 35.5 px
```

`layout_picture_full`은 그림 높이 `57.7px`을 컨테이너 높이 `35.5px`에 맞추기 위해 비율 축소한다.
이 값이 현재 SVG 출력 크기와 일치한다.

## 7. 결론

Stage 1의 결론:

```text
글상자 내부 non-TAC 그림을 실제 콘텐츠 높이로 보지 않고,
line segment 높이만 기준으로 vertical alignment offset을 계산한 것이 직접 원인이다.
```

한컴/PDF는 이 그림을 글상자 내부 조판 콘텐츠로 취급하여 HWPX `pic curSz/sz`에 가까운 크기로
출력한다.

## 8. Stage 2 후보

Stage 2 구현 후보:

```text
1. 글상자 내부 non-TAC 그림을 content height 계산에 포함한다.
2. non-TAC 그림의 scale clamp 기준 높이를 `inline_y 이후 남은 높이`가 아니라
   글상자 inner_area.height로 분리한다.
3. y 위치 보정과 scale clamp를 같은 값으로 처리하지 않는다.
4. `hy-001`을 guard로 함께 확인한다.
```

우선 구현 후보는 다음 한 줄로 요약된다.

```text
text box 내부 non-TAC picture는 vertical alignment offset 때문에 줄어든 잔여 높이로 scale clamp하지 않는다.
```

## 9. 검증

실행한 확인:

```text
cargo run --quiet --bin rhwp -- dump samples/hwpx/hy-002.hwpx --section 0 --para 30
cargo run --quiet --bin rhwp -- dump samples/hwpx/hancom-hwp/hy-002.hwp --section 0 --para 30
cargo run --quiet --bin rhwp -- dump-pages samples/hwpx/hy-002.hwpx -p 1
cargo run --quiet --bin rhwp -- dump-pages samples/hwpx/hancom-hwp/hy-002.hwp -p 1
cargo run --quiet --bin rhwp -- export-svg samples/hwpx/hy-002.hwpx -p 1 -o output/poc/hwpx2hwp/task_m100_1013/stage1_hy002_textbox_picture_size/svg_hwpx
cargo run --quiet --bin rhwp -- export-svg samples/hwpx/hancom-hwp/hy-002.hwp -p 1 -o output/poc/hwpx2hwp/task_m100_1013/stage1_hy002_textbox_picture_size/svg_hwp
pdfinfo pdf-large/hwpx/hy-002.pdf
pdfimages -list pdf-large/hwpx/hy-002.pdf
```

Stage 1에서는 구현 변경을 하지 않았다.

## 10. 다음 단계

Stage 2에서는 `src/renderer/layout/shape_layout.rs`의 글상자 내부 non-TAC 그림 배치 계산을
수정하고, `hy-002`와 `hy-001`을 함께 guard로 검증한다.
