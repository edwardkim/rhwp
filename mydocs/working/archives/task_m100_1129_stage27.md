# Task 1129 Stage 27 - 짝수 배율 격자 점 소실 보정

## 사용자 지적

- 짝수 배율에서는 격자 점이 보이지 않고, 홀수 배율에서만 보인다.

## 원인 가설

- radial-gradient 점은 원 중심과 반경이 배율/배경 위치의 소수점 좌표에 걸릴 때 브라우저 샘플링으로 사라질 수 있다.
- Stage 26의 `zoom * 0.5px` 반경은 확대 배율에서 점 크기까지 커져 한컴오피스의 1px 점과 달라진다.

## 수정 계획

- 점 격자를 radial-gradient 대신 반복 SVG tile의 1px 사각 점으로 렌더링한다.
- 점 배경 위치는 CSS 픽셀에 스냅하여 짝수/홀수 배율 모두 중심 픽셀이 살아나게 한다.
- 격자 간격은 `settings.horizontalMm`, `settings.verticalMm`, `zoom` 계산을 유지한다.

## 수정 내용

- `dots` 패턴의 `background-image`를 radial-gradient에서 `data:image/svg+xml` tile로 변경했다.
- tile 내부에는 1px 고정 점만 두어 확대 배율에서도 점 자체가 2px 이상으로 커지지 않게 했다.
- 점 패턴의 `background-position`은 `Math.round()`로 CSS 픽셀 스냅한다.

## 검증 계획

- `npm run build`
- `cargo fmt --all -- --check`
- `git diff --check`

## 추가 피드백

- 이번 변경은 `rhwp-studio`의 TypeScript/Vite 레이어만 수정하므로 `wasm-pack build`는 수행하지 않는다.
