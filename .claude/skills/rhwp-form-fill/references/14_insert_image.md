# 14 — insert-image (직인·서명)

관공서 서식은 값만으로는 끝나지 않는다. 직인·서명이 쪽 좌표에 올라가야
제출이다.

`edit insert-image <파일> --image <그림> [--page N] [--x N --y N] [--width N --height N]`
(#3719 §6-5).

도장·서명용 **쪽 좌표(용지 기준 floating)** 축이다. 본문 흐름에 그림을
끼우는 `insert-picture` 와 다르다. 제출 직인은 insert-image.

```bash
rhwp edit insert-image output/작성본.hwp \
  --image 직인.png \
  --page 0 --x 28346 --y 28346 --width 8504 --height 8504 \
  -o output/날인본.hwp --json | jq '{output, overflow}'
```

## 단위

전부 **HWPUNIT = 1/7200 inch**. 픽셀도 mm 도 아니다.

- A4 세로 = 59528 × 84188
- 1mm ≈ 283.46 HWPUNIT
- 100mm, 30mm 도장 → `--x 28346 --y 28346 --width 8504 --height 8504`

30 을 그대로 주면 0.1mm 짜리 점이 찍히거나 안 보인다.

`--page` 는 0 부터. 한컴 표기(1부터)와 혼동하지 않는다. 범위 밖은
exit 2.

## 그림 형식

`png` · `jpg` · `jpeg` · `bmp` · `tif` · `tiff`. 확장자와 내용을 둘 다
검사. 그 밖은 문서를 읽기 전에 exit 2.

## overflow

쪽 밖으로 나가도 **자르지 않는다**. `overflow` 배열로만 알린다.
에이전트는 렌더를 보지 않으므로 신호가 없으면 잘린 도장을 완성본으로
오판한다.

삽입은 막지 않는다. 판단은 호출자. `--width/--height` 축소 또는
`--x/--y` 조정.

`--dry-run` 에서도 overflow 를 볼 수 있다.

## 파이프라인 위치

```
fill-fields --verify  →  insert-image  →  sanitize
```

sanitize 가 그림을 지우지 않는다(본문·BinData 가 아니라 메타).
순서를 뒤집어도 그림은 남지만, 제출본은 sanitize 가 마지막.

## 하지 말 것

- 누름틀 `인`/`서명` 에 글자 "인" 을 넣어 도장을 대체
- mm 숫자를 HWPUNIT 없이 전달
- `--page 1` 을 첫 쪽으로
- 직인 전용 하위명령을 새로 만들기
