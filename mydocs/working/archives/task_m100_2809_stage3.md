# Task #2809 Stage 3 — 실제 rhwp 문자 위치 재생 정정

- 브랜치: `task/2809-distribute-align`
- 선행: [`task_m100_2809_stage2.md`](task_m100_2809_stage2.md)
- 상태: 완료

## 문제 정의

native SVG, WASM SVG와 페이지 레이어 트리에는 위쪽 `다 같 이`와 아래쪽
`다 같 이`의 문자 span 차이가 존재하지만, 실제 rhwp 편집기 래스터 화면에서는 두
문단이 같게 보인다. 브라우저 캐시와 WASM 갱신 문제가 아님은 강제 새로고침 후에도
동일한 사용자 화면으로 확인했다.

## 분석 범위

1. 실제 편집기가 선택한 CanvasKit/Canvas2D 백엔드를 진단한다.
2. `textRun.positions`와 `displayPositions`가 glyph 매핑·fallback 과정에서 어떻게
   최종 draw 좌표로 변환되는지 추적한다.
3. 원문 자간과 공백 분배가 중복 적용되거나 소실되는 지점을 정정한다.

## 완료 조건

1. 실제 rhwp 화면에서 위쪽과 아래쪽 문단이 정상 PDF와 같은 방향으로 다르게 보인다.
2. 레이어 트리 좌표 검사뿐 아니라 최종 화면 픽셀 기반 회귀 검사가 실패/성공을
   구분한다.
3. WASM 재빌드, 실제 편집기 캡처, visual sweep, 전체 회귀 검증과 증적 재생성을
   완료한다.

## 원인

실제 편집기는 기본 `canvas2d` 백엔드를 사용했다. `WebCanvasRenderer::draw_text`의
폰트 폭 보정은 브라우저 실측 glyph 폭이 레이아웃의 cluster advance보다 크면 glyph를
advance 폭에 맞춰 가로로 축소한다. 위쪽 문단의 음수 자간은 한글 한 글자의 advance를
약 절반으로 줄이므로 이 조건이 발동했고, 문자 위치뿐 아니라 glyph 외형까지 눌렸다.

PDF와 native SVG는 음수 자간을 다음 글자의 시작 위치에 반영하고 glyph 폭은
유지한다. 따라서 Canvas는 `letter_spacing < 0`이면 폰트 폭 보정을 생략했다.
동시에 `Split` 분배 계산에서 마지막 glyph의 음수 자간만 실제 잉크 여유로 예약해
정상 폭 glyph가 셀 우측 clip을 넘지 않게 했다. 두 보정 중 하나만 적용하면 각각
glyph 눌림 또는 우측 잘림이 남으므로 결합 적용이 완료 조건이다.

## 검증 결과

- 실제 백엔드: `canvas2d`.
- 문자 위치 span: 위 `77.973px`, 아래 `76.693px`.
- 최종 Canvas 첫 `다` glyph 잉크 폭: 위 `28px`, 아래 `28px`.
- 위쪽 마지막 `이` glyph 잉크 폭: `22px`, 첫 glyph 대비 `78.6%`로 무잘림.
- 수정 전 위쪽 glyph 잉크 폭은 약 `15px`, 아래쪽은 `27px`였다.
- E2E assertion: `7/7` 통과, 실제 편집기 페이지 보기 100% 캡처 갱신.
- `wasm-pack build --target web --out-dir pkg`: 통과.
- rhwp Studio production build: 통과.
- 전체 lib: `2512 passed; 0 failed; 7 ignored`.
- SVG snapshot: `8 passed; 0 failed`.
- clippy `-D warnings`, fmt: 통과.
- 144dpi visual sweep: `flagged=0/1`.
