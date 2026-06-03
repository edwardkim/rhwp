# Task M100-1071 Stage 5 완료 보고서

## 1. 목표

Stage 4 시각 판정에서 남은 문제인 TAC 다각형 커서 이동 실패를 정정한다.

Stage 4에서 이미 해결된 항목:

```text
shape-001.hwpx 의 다각형과 다각형 사이 공백 출력
```

남은 항목:

```text
HWP/HWPX 모두 다각형을 한 글자로 인식해서 커서 이동하기 실패
```

## 2. 원인 분리

브라우저에서 `ArrowRight` 이벤트와 `cursor.moveHorizontal()` offset 이동을 확인한 결과,
문서 내부 offset 은 이미 정상 이동했다.

```text
0 -> 1 -> 2 -> 3 -> 4
```

하지만 `get_cursor_rect_native()` 가 반환하는 caret x 좌표는 정상 이동하지 않았다.

문제는 offset 이동 로직이 아니라 cursor rect 계산 로직이었다.

## 3. 핵심 원인

`shape-001` 첫 문단의 실제 텍스트는 공백 2개다.

```text
para.text = "  "
```

그러나 렌더 트리의 TextRun에는 TAC 도형 자리표시자 `U+FFFC` 가 섞여 있었다.

```text
TextRun 예:
  "￼￼"
  "￼  "
  "￼"
```

기존 cursor rect 계산은 이 자리표시자 TextRun을 실제 편집 문자처럼 따라가며 x 좌표를 잡았다.
그 결과 offset 은 움직이지만 caret x 는 도형과 공백의 논리 흐름을 반영하지 못했다.

## 4. 구현

수정 파일:

```text
src/document_core/queries/cursor_rect.rs
```

구현 내용:

```text
1. show_control_codes=false 인 일반 편집 모드에서 TAC inline control 전용 caret stop 을 먼저 계산한다.
2. RenderTree에서 해당 문단의 Shape/Picture/Table/Equation bbox 를 수집한다.
3. TextRun에서는 U+FFFC 자리표시자를 건너뛰고 실제 para.text 문자만 caret stop 으로 사용한다.
4. logical control position 기준으로:
   - control position     -> 도형 왼쪽
   - control position + 1 -> 도형 오른쪽
5. 실제 텍스트 문자와 TAC control bbox를 합쳐 논리 caret x 좌표를 구성한다.
```

이 방식은 TextRun `char_start` 자체를 전역적으로 바꾸지 않는다.
커서 좌표 계산 단계에서만 TAC 도형과 실제 텍스트의 논리 흐름을 재구성한다.

## 5. 테스트 보강

수정/추가 파일:

```text
tests/issue_1071_tac_cursor_nav.rs
tests/hwpx_to_hwp_adapter.rs
```

검증 내용:

```text
1. shape-001.hwpx 에서 좌우 이동 offset:
   0 -> 1 -> 2 -> 3 -> 4
   4 -> 3 -> 2 -> 1 -> 0

2. shape-001.hwp 와 shape-001.hwpx 모두에서 offset 0..4 의 cursor rect x 가 매 logical unit 마다 전진한다.

3. HWPX secPr control stream 보존 이후 baseline page count 기대값을 현 상태에 맞게 조정한다.
```

## 6. 검증

실행한 명령:

```text
cargo fmt
cargo check
cargo test --test issue_1071_tac_cursor_nav
cargo test --test issue_1067_shape_rotation
cargo test --test issue_598_footnote_marker_nav
cargo test --lib logical_positions_ignore_section_and_column_controls
cargo test --test hwpx_to_hwp_adapter
docker compose --env-file .env.docker run --rm wasm
```

결과:

```text
cargo check: success
issue_1071_tac_cursor_nav: 3 passed
issue_1067_shape_rotation: 7 passed
issue_598_footnote_marker_nav: 4 passed
logical_positions_ignore_section_and_column_controls: passed
hwpx_to_hwp_adapter: 49 passed, 0 failed, 11 ignored
wasm build: success
```

WASM 산출물:

```text
pkg/rhwp.js
pkg/rhwp_bg.wasm
rhwp-studio/public/rhwp.js
rhwp-studio/public/rhwp_bg.wasm
```

`pkg` 와 `rhwp-studio/public` 의 WASM/JS 산출물은 바이트 단위로 동일하다.

## 7. 결론

Stage 5는 #1071의 두 축을 모두 정정했다.

```text
1. HWPX TAC 다각형 사이 공백 배치 성공
2. HWP/HWPX TAC 다각형을 한 글자처럼 통과하는 cursor rect 성공
```

작업지시자는 갱신된 WASM 산출물로 rhwp-studio에서 최종 상호작용 판정을 수행한다.

## 8. 작업지시자 판정

작업지시자 판정 결과:

```text
동작 테스트 성공
```

성공 항목:

```text
1. shape-001.hwpx 에서 다각형과 다각형 사이 공백 출력 성공
2. HWP/HWPX 모두 TAC 다각형을 한 글자 단위로 통과하는 커서 이동 성공
```

## 9. 후속 개선 후보

기능 판정은 통과했지만, 다음 UX 개선 후보가 함께 제기되었다.

```text
현상:
  다각형 다음에 연속된 두 개의 space가 있고 그 다음 다각형이 있는 경우,
  첫 번째 space 이동은 짧게 느껴지고 다음 이동은 다음 다각형 앞까지 전진하는 것처럼 느껴진다.

판단:
  현재 동작은 논리 offset 기준으로는 맞다.
  다만 TAC 도형 폭과 space 폭이 크게 다르기 때문에 사용자가 체감하는 caret 이동 간격이 고르지 않다.

처리:
  같은 이슈에서 함께 처리한다.
  앞 TAC 도형 오른쪽과 뒤 TAC 도형 왼쪽 사이의 시각 간격을 연속 space 개수로 균등 분배한다.
  논리 offset 수는 유지하되, 연속 space 구간의 caret 이동 체감을 균등하게 만든다.
```

추가 구현:

```text
src/document_core/queries/cursor_rect.rs
```

추가 테스트:

```text
tests/issue_1071_tac_cursor_nav.rs
  issue_1071_spaces_between_tac_shapes_have_balanced_caret_steps
```

검증:

```text
cargo fmt
cargo check
cargo test --test issue_1071_tac_cursor_nav
cargo test --test issue_1067_shape_rotation
cargo test --test issue_598_footnote_marker_nav
cargo test --test hwpx_to_hwp_adapter
docker compose --env-file .env.docker run --rm wasm
```
