# Stage 1 - #2190 CanvasKit 글머리 기호 폰트 서브셋 보정

## 재현 확인

- `web/fonts/NotoSansKR-Regular.woff2`의 CanvasKit glyph ID는 `U+25A0`(`■`)와
  `U+25AA`(`▪`)에서 모두 `0`이다.
- 같은 파일은 `U+00B7`(`·`)와 `U+AC00`(`가`)에는 유효 glyph ID를 반환한다.
- #2191의 `ParagraphBuilder` 보완은 shaping 경로를 고친 것이며, 누락된 glyph를 폰트에
  추가하지 않으므로 #2190의 원인을 해결하지 않는다.

## Stage 1 범위

1. Google Fonts Noto Sans KR variable source를 weight 400 정적 폰트로 만든다.
2. 기존 Regular cmap을 보존하고 `U+2500-257F`, `U+25A0-25FF`를 추가한 TTF/WOFF2를 생성한다.
3. CanvasKit 실번들에서 글머리/박스 기호가 glyph ID `0`이 아님을 검증하는 회귀 테스트를 추가한다.
4. Render Diff CI가 폰트 자산 변경에도 해당 회귀 테스트를 실행하도록 연결한다.

## 완료 조건

- `■`, `▪`, `□`, `○`, `─`가 `web/fonts/NotoSansKR-Regular.woff2`에서 유효 glyph ID를 반환한다.
- 기존 한글/기본 라틴 coverage가 유지된다.
- 서브셋 생성 입력과 범위가 `web/fonts/FONTS.md`에 재현 가능하게 기록된다.

## 결과

- Google Fonts `NotoSansKR[wght].ttf` 입력 SHA-256
  `194018e6b2b293a7964f037b25c0249ce1418bc9ab3c971060a03aa57861e252`에서 weight 400
  정적 subset을 생성했다.
- `■`/`▪`/`□`/`○`/`─`/`가`는 새 TTF와 WOFF2 cmap에서 모두 유효 glyph를 가진다.
- 생성 도구는 timestamp 재계산을 끄므로 동일 입력으로 TTF와 WOFF2를 바이트 단위 재현한다.
- headless Chrome CanvasKit default backend에서 `hwpx_sample2.hwpx` 29쪽을 로드해 p1의 `■`와
  `▪` 글머리가 tofu 없이 표시되는 것을 확인했다. 대표 캡처는
  `mydocs/report/assets/task_m100_2190/canvaskit_hwpx_sample2_p1.png`에 남겼다.
