# Task M100 #1444 — 2단계 완료 보고서 (재빌드 + 검증)

- 브랜치: `local/task1444`
- 작성일: 2026-06-20

## 1. 1단계 가정 정정 — 확장은 `publicDir: false`

1단계는 "vite 가 public/theme-init.js 를 dist 로 복사"한다고 가정했으나, **확장
`vite.config.ts` 는 `publicDir: false`** (samples/images 등 대용량 public 제외)라 복사되지
않았다. 1차 재빌드에서 `dist/theme-init.js` 누락 확인.

정정: `rhwp-chrome/build.mjs`·`rhwp-firefox/build.mjs` 에 `theme-init.js` **개별 복사** 추가
(다크 아이콘·favicon 과 동일 방식). 웹앱은 publicDir 기본값이라 자동 포함되어 무관.

## 2. 검증 (재빌드 후)

### 확장 (rhwp-chrome 재빌드)
- `dist/theme-init.js` 존재 + 원본과 **동일**(diff 0).
- `dist/images/icon_small_ko_dark.svg` 존재.
- `dist/viewer.html`: 인라인 테마 스크립트 흔적 0, `<script src="/theme-init.js">` 유지.
  절대경로 `/theme-init.js` 는 viewer 탭(`chrome-extension://<id>/`)에서 dist 루트
  theme-init.js 로 해석 → 동작.
- vite 경고 `<script src="/theme-init.js"> ... can't be bundled without type="module"` 는
  **정보성**(번들 대신 src 보존 = 의도된 동작). 에러 아님. dev-tools-inject.js src 주입과 동형.

### 웹앱 (rhwp-studio)
- `npm run build` 성공(✓ built). `dist/theme-init.js` 포함(웹앱 vite publicDir 기본).
- `e2e/theme-mode.test.mjs` (호스트 CDP) **전부 PASS** — system/dark/light 전환, effective
  theme, color-scheme/meta 갱신, FOUC 방지 동작. **테마 회귀 0**.

## 3. 잔여 — 확장 실로드 판정

확장 viewer 의 CSP 위반·다크 아이콘 404 **실제 미발생**은 chrome://extensions unpacked
로드가 필요(자동화 곤란). 빌드 산출물(인라인 부재·theme-init.js·다크 아이콘 존재)로
정확성은 확인됐다. 실로드 판정은 작업지시자 환경 요청.

## 4. 다음 단계

- 3단계: 0.2.5 zip 재생성(chrome/edge/firefox + AMO source) + 트러블슈팅·최종 보고서.
