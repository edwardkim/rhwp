# Task M100-1013 Stage 2 구현 보고서

## 1. 목적

`hy-002` 2페이지 글상자 내부 non-TAC 그림이 한컴/PDF보다 작게 렌더링되는 문제를 수정한다.

## 2. 원인

Stage 1에서 확인한 직접 원인은 다음이다.

```text
글상자 vertical_align=CENTER 처리 시 line_seg 높이만 content height로 계산했다.
이 문단의 실제 콘텐츠는 non-TAC 그림인데, 그림 높이가 content height에서 빠졌다.
그 결과 CENTER offset이 발생하고, 그림 container height가 offset 이후 남은 높이로 줄었다.
layout_picture_full이 그 줄어든 높이에 맞춰 그림을 비율 축소했다.
```

현재 출력이 약 `395.37 x 35.53 px`였던 이유도 이 계산과 일치한다.

## 3. 수정

수정 파일:

```text
src/renderer/layout/shape_layout.rs
src/wasm_api/tests.rs
```

수정 내용:

```text
1. 글상자 세로 정렬 content height 계산에 non-TAC Picture 높이를 포함한다.
2. 동일 축에서 non-TAC Shape 높이도 포함한다.
3. hy-002 HWP/HWPX 입력 모두에서 글상자 내부 image2가 600px x 50px 이상으로
   렌더링되는 회귀 테스트를 추가한다.
4. hy-001 기존 guard 테스트를 함께 실행해 #974 회귀를 확인한다.
```

## 4. 결과

Stage 2 SVG export 결과:

```text
output/poc/hwpx2hwp/task_m100_1013/stage2_textbox_non_tac_picture_fix/svg_hwpx/hy-002_002.svg
output/poc/hwpx2hwp/task_m100_1013/stage2_textbox_non_tac_picture_fix/svg_hwp/hy-002_002.svg
```

문제 그림 bbox:

```text
HWPX input: width=642.38px, height=57.73px
HWP  input: width=642.38px, height=57.73px
```

Stage 1 기준값:

```text
HWPX pic curSz/sz ~= 642.5 x 57.7 px
PDF image display ~= 642.8 x 57.6 px
```

따라서 축소 문제는 renderer 기준에서 해결되었다.

## 5. 검증

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
fmt check: success
cargo check: success
hy-002 test: success
hy-001 guard: success
wasm build: success
```

## 6. 다음 확인

작업지시자 시각 판정 대상:

```text
output/poc/hwpx2hwp/task_m100_1013/stage2_textbox_non_tac_picture_fix/svg_hwpx/hy-002_002.svg
output/poc/hwpx2hwp/task_m100_1013/stage2_textbox_non_tac_picture_fix/svg_hwp/hy-002_002.svg
```

작업지시자 시각 판정:

```text
통과
```

웹 캔버스 판정 준비:

```text
pkg/rhwp_bg.wasm
rhwp-studio/public/rhwp_bg.wasm
```

`pkg/rhwp_bg.wasm`과 `rhwp-studio/public/rhwp_bg.wasm`은 같은 파일로 동기화했다.

시각 판정 통과로 Stage 2 구현은 완료 처리한다.
