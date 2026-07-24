# 완료 보고서 — Task M100-2958

- 이슈: #2958
- 제목: HWP3 글자 음영색(shade_color) 변환 누락 — convert_char_shape 에서 IR로 복사되지 않음
- 작성일: 2026-07-22
- 브랜치: `task/m100-2958-hwp3-shade-color`

## 1. 완료 내용

`src/parser/hwp3/mod.rs`의 `convert_char_shape()`는 `Hwp3CharShape.text_color`
(글자색, offset 24)는 `hwp3_color_index_to_color_ref()`로 매핑하지만, 바로 옆
필드인 `shade_color`(글자 음영색, offset 23)는 매핑하지 않고 방치하고 있었다.

그 결과 IR `CharShape.shade_color`는 `CharShape::default()`의 값인 `0`(검정,
`ColorRef = u32` 기본값)으로 항상 남는다. 그런데 렌더러(`html.rs`, `svg.rs`,
`skia/text_replay.rs`, `canvaskit_policy.rs` 등)는 "형광펜 음영 없음"을
`0x00FFFFFF`(흰색) sentinel로 판정하므로 (`shade_color & 0x00FFFFFF !=
0x00FFFFFF`), 값이 `0`으로 남으면 음영 없는 문단도 잠재적으로 "음영 있음"으로
오판될 소지가 있는 상태였다.

이 필드는 `text_color`와 동일한 8색 팔레트(검정~흰색 인덱스 0~7)를 쓰므로
기존 `hwp3_color_index_to_color_ref()` 헬퍼를 그대로 재사용해 매핑했다.
색상 인덱스 7(흰색)은 헬퍼를 거치면 정확히 `0x00FFFFFF`로 변환되어, 음영이
없는 HWP3 문서(글자 음영색 인덱스가 흰색인 경우)는 그대로 "음영 없음"
sentinel과 일치하게 되어 회귀가 없다.

같은 함수의 `text_color` 미변환 문제는 이미 #1692에서 지적되어 수정되었으나,
당시 분석 근거(`records.rs:193`)에 `shade_color`, `text_color` 필드가 함께
언급되었음에도 수정은 `text_color`에만 적용되고 `shade_color`는 그대로
누락되어 있었다.

## 2. 주요 변경

- `src/parser/hwp3/mod.rs`
  - `convert_char_shape()`에 `cs.shade_color =
    hwp3_color_index_to_color_ref(hwp3_cs.shade_color);` 추가 (5줄, 주석 포함)
  - 단위 테스트 `task2958_convert_char_shape_preserves_shade_color` 추가
    (red: 기존 코드에서는 `cs.shade_color`가 항상 `0` → 테스트가 기대하는
    `0x00FF0000`과 불일치해 실패 / green: 수정 후 통과)

## 3. 검증 결과

통과:

- `cargo check --lib`
- `cargo test --lib task2958` → `task2958_convert_char_shape_preserves_shade_color ... ok`
- `rustfmt --edition 2021 src/parser/hwp3/mod.rs`

## 4. 범위 밖 (Out of scope)

- HWP3 문단 모양의 `word_spacing`(낱말 간격)은 파싱만 되고 IR `ParaShape`에
  대응 필드가 없어 변환되지 않는다. 이는 IR 확장이 필요한 별도 스코프이므로
  이번 tiny fix에는 포함하지 않았다.
