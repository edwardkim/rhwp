# Task M100 #1444 — 1단계 완료 보고서 (theme-init 분리 + 다크 아이콘 복사)

- 브랜치: `local/task1444`
- 작성일: 2026-06-20
- 추가/수정: `rhwp-studio/public/theme-init.js`(신규), `rhwp-studio/index.html`,
  `rhwp-chrome/build.mjs`, `rhwp-firefox/build.mjs`

## 1. 수정 ① — 인라인 테마 스크립트 외부 분리 (CSP)

- 신규 `rhwp-studio/public/theme-init.js`: index.html 인라인 IIFE 를 그대로 이전
  (다크테마 FOUC 방지: localStorage `rhwp-settings` → `data-theme-*`/`colorScheme`/meta).
- `index.html`: `<head>` 최상단 인라인 `<script>` 블록 → `<script src="/theme-init.js"></script>`
  (동기, module/defer 금지 — 번들 전 즉시 실행으로 FOUC 방지 유지). 위치 보존.
- vite 가 public/theme-init.js 를 dist 로 복사 → 웹앱·확장 양쪽 `/theme-init.js` 유효.
  확장 CSP `'self'` 충족 (dev-tools-inject.js src 주입과 동형 패턴).

## 2. 수정 ② — 다크 아이콘 복사 (chrome + firefox build)

- `rhwp-chrome/build.mjs`·`rhwp-firefox/build.mjs` 둘 다 라이트 아이콘만 복사했음.
  각각 `icon_small_ko_dark.svg` 복사 라인 추가 (base.css:197 다크 모드 참조 → 404 해소).
  (firefox build 도 동일 누락이라 공통 정정.)

## 3. 검증

- `cd rhwp-studio && npx tsc --noEmit`: **exit 0** (분리 영향 없음).
- `index.html`: 인라인 테마 코드 흔적 0, `<script src="/theme-init.js">` 참조 확인.

## 4. 다음 단계

- 2단계: 확장 재빌드 → dist 인라인 부재 + theme-init.js + 다크 아이콘 존재 +
  viewer CSP/404 미발생(호스트 Chrome) + 웹앱 빌드/e2e theme-mode 회귀 0.
- 3단계: 0.2.5 zip 재생성 + 문서·보고서.
