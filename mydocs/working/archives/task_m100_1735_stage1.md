# Stage 1 완료보고 — 렌더 치환 (M100 #1735)

- **이슈**: [edwardkim/rhwp#1735](https://github.com/edwardkim/rhwp/issues/1735)
- **브랜치**: `local/task1735`

## 목표
방점(U+302E/U+302F) 결합문자 dotted-circle(◌) 아티팩트 제거 — 렌더 경로에서 spacing 가운데 점으로 치환.

## 변경
- `src/renderer/composer.rs`
  - `tone_mark_display(ch) -> Option<char>` 신설: `U+302E → U+00B7`(· MIDDLE DOT), `U+302F → U+205A`(⁚ TWO DOT PUNCTUATION).
  - `expand_pua_display_text` 루프에서 PUA 분기 이전에 적용. 렌더 전용 choke point이므로 svg/html/canvas/web_canvas/skia + paint/json(스튜디오) 전 표면 동시 정합. IR 텍스트 불변.
- `src/renderer/composer/tests.rs`
  - `test_expand_tone_marks_to_spacing_dot` 추가(TDD): 원본 방점·U+25CC 미포함, `〮 각` → `· 각` 치환 검증.

## TDD
- RED: `expand_pua_render_text("〮 각")` 가 원본 방점을 그대로 반환 → 실패 확인.
- GREEN: 치환 구현 후 통과.

## 검증
- `export-svg` 결과: 방점 자리에 `<circle cx=134.84 cy=138.93 r=1.07>` 렌더(Task #257 · 벡터 도형 경로 활용). SVG에 U+302E/U+25CC 미출현.
- `export-pdf`: `· 각 항목에 명시되어 있는` — 한컴 정답지 `·각 항목에 명시되어 있는`의 가운데 점과 시각 일치. dotted-circle 아티팩트 소멸.

## 발견
- rhwp는 U+00B7(·)을 폰트 비의존 `<circle>` 벡터로 렌더(Task #257)하므로, 방점을 ·로 치환하면 자동으로 CJK x-height 중앙에 깔끔한 점으로 표시됨 — 별도 위치 보정 불필요.
