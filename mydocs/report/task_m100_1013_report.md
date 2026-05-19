# M100 #1013 최종 결과 보고서

## 1. 작업 개요

- 이슈: #1013
- 브랜치: `local/task1013`
- 대상 샘플:
  - `samples/hwpx/hy-002.hwpx`
  - `samples/hwpx/hancom-hwp/hy-002.hwp`
  - `pdf-large/hwpx/hy-002.pdf`
- 회귀 guard:
  - `samples/hwpx/hy-001.hwpx`
  - `samples/hwpx/hancom-hwp/hy-001.hwp`
  - `pdf-large/hwpx/hy-001.pdf`
- 목표: `hy-002` 2페이지 글상자 내부 non-TAC 그림이 한컴 에디터/PDF보다 작게 렌더링되는 문제를 수정한다.

## 2. 원인

문제 그림은 단독 그림 shape가 아니라 글상자 내부 문단의 non-TAC 그림이다.

```text
hp:rect
  hp:drawText
    hp:subList
      hp:p
        hp:run
          hp:pic treatAsChar="0"
```

현재 rhwp는 글상자 `vertAlign=CENTER` 처리 시 line segment 높이만으로 content height를 계산했다.
그 결과 실제 콘텐츠인 non-TAC 그림 높이가 빠지고, 가운데 정렬 offset이 발생했다.

이후 그림 배치 컨테이너 높이를 `inline_y` 이후 남은 높이로 잡으면서 그림이 과도하게 축소되었다.

```text
기대 크기: 약 642.8 x 57.6 px
기존 rhwp 출력: 약 395.4 x 35.5 px
```

즉, 축소는 한컴 호환 동작이 아니라 rhwp의 글상자 내부 조판 계산 오류였다.

## 3. 처리 내용

수정 파일:

```text
src/renderer/layout/shape_layout.rs
src/wasm_api/tests.rs
```

수정 내용:

```text
1. 글상자 세로 정렬 content height 계산에 non-TAC Picture 높이를 포함했다.
2. 같은 축에서 non-TAC Shape 높이도 포함했다.
3. hy-002 HWP/HWPX 입력 모두에서 글상자 내부 image2가 선언 크기대로 렌더링되는 테스트를 추가했다.
4. hy-001 테스트를 함께 실행해 #974 글상자 내부 그림 처리 회귀를 확인했다.
```

## 4. 결과

수정 후 `hy-002` 문제 그림 bbox:

```text
HWPX input: 642.38 x 57.73 px
HWP input:  642.38 x 57.73 px
```

한컴/PDF 기준:

```text
약 642.8 x 57.6 px
```

따라서 rhwp의 과도한 축소 문제는 해결되었다.

시각 판정 대상:

```text
output/poc/hwpx2hwp/task_m100_1013/stage2_textbox_non_tac_picture_fix/svg_hwpx/hy-002_002.svg
output/poc/hwpx2hwp/task_m100_1013/stage2_textbox_non_tac_picture_fix/svg_hwp/hy-002_002.svg
```

작업지시자 시각 판정:

```text
SVG 시각 판정: 통과
웹 캔버스 시각 판정: 통과
```

## 5. 검증

수행한 검증:

```text
cargo fmt --check
cargo check
cargo test --lib test_hy002_textbox_non_tac_picture_keeps_declared_size
cargo test --lib test_hy001_textbox_inline_pictures_render_for_hwp_and_hwpx
cargo run --quiet --bin rhwp -- export-svg samples/hwpx/hy-002.hwpx -p 1 -o output/poc/hwpx2hwp/task_m100_1013/stage2_textbox_non_tac_picture_fix/svg_hwpx
cargo run --quiet --bin rhwp -- export-svg samples/hwpx/hancom-hwp/hy-002.hwp -p 1 -o output/poc/hwpx2hwp/task_m100_1013/stage2_textbox_non_tac_picture_fix/svg_hwp
docker compose --env-file .env.docker run --rm wasm
```

결과:

```text
fmt check: 성공
cargo check: 성공
hy-002 테스트: 성공
hy-001 guard 테스트: 성공
SVG export: 성공
wasm 빌드: 성공
웹 캔버스 판정용 wasm 동기화: 성공
SVG 시각 판정: 통과
웹 캔버스 시각 판정: 통과
```

참고: 테스트 실행 중 기존 경고가 출력되었지만, 이번 변경과 무관한 기존 경고이며 대상 테스트는 통과했다.

wasm 산출물:

```text
pkg/rhwp_bg.wasm = 4,825,755 bytes
rhwp-studio/public/rhwp_bg.wasm = 4,825,755 bytes
```

## 6. 산출물

계획서:

```text
mydocs/plans/task_m100_1013.md
```

단계별 보고서:

```text
mydocs/working/task_m100_1013_stage1.md
mydocs/working/task_m100_1013_stage2.md
```

분석/검증 산출물:

```text
output/poc/hwpx2hwp/task_m100_1013/stage1_hy002_textbox_picture_size/
output/poc/hwpx2hwp/task_m100_1013/stage2_textbox_non_tac_picture_fix/
```

## 7. 판정

#1013의 목표인 `hy-002` 글상자 내부 non-TAC 그림 크기 보정은 완료로 판정한다.
SVG와 웹 캔버스 양쪽 시각 판정을 모두 통과했다.

## 8. 승인 요청

작업지시자가 최종 결과 보고서를 승인했다.
