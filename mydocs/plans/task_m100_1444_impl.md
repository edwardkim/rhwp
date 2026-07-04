# Task M100 #1444 구현계획서 — 인라인 테마 스크립트 외부 분리 + 다크 아이콘 복사

- 이슈: #1444, 마일스톤 M100, 브랜치 `local/task1444`
- 작성일: 2026-06-20
- 수행계획서: `mydocs/plans/task_m100_1444.md`

## 0. 빌드 흐름 (확인 완료)

- `rhwp-chrome/build.mjs`: vite 로 rhwp-studio 빌드 → `dist/index.html` → `viewer.html`
  리네임. **vite 가 `public/` 을 dist 루트로 복사** → `public/theme-init.js` → `dist/theme-init.js`.
- build.mjs 가 이미 `</head>` 앞에 `<script src="/dev-tools-inject.js"></script>` 주입 →
  **외부 src 스크립트가 확장 CSP `'self'` 로 정상 동작함이 검증된 패턴**.
- firefox CSP 도 chrome 과 동일(`script-src 'self' 'wasm-unsafe-eval'`) → 같은 수정으로 해결.

## 1. 수정 ① — 인라인 테마 스크립트 외부 분리 (CSP)

### 1.1 신규 `rhwp-studio/public/theme-init.js`

`index.html` 인라인 IIFE 를 그대로 옮긴다 (다크테마 FOUC 방지: localStorage
`rhwp-settings` → `data-theme-mode`/`data-theme-effective`/`colorScheme`/meta 갱신).

### 1.2 `rhwp-studio/index.html`

`<head>` 최상단의 인라인 `<script>...</script>` 블록을
`<script src="/theme-init.js"></script>` 로 교체. **위치 유지**(meta 직후, style/번들 전) +
**동기 로드**(module/defer 금지) → FOUC 방지 보장. vite 가 public/theme-init.js 를
dist 로 복사하므로 웹앱·확장 양쪽 `/theme-init.js` 참조 유효.

## 2. 수정 ② — 다크 아이콘 복사 (build.mjs)

`rhwp-chrome/build.mjs:94` 라이트 아이콘 복사 직후에 추가:

```js
copy(resolve(ROOT, 'rhwp-studio', 'public', 'images', 'icon_small_ko_dark.svg'),
     resolve(DIST, 'images', 'icon_small_ko_dark.svg'));
```

(웹앱은 vite 가 public/images 를 복사하므로 무관 — 확장 build.mjs 만 누락이었음.)

## 3. 단계별 구현

### 1단계 — 분리 + 참조 + 복사
- `public/theme-init.js` 신규(인라인 IIFE 이전).
- `index.html` 인라인 → `<script src="/theme-init.js">`.
- `build.mjs` 다크 아이콘 복사 추가.
- `cd rhwp-studio && npx tsc --noEmit` (영향 없음 확인).

### 2단계 — 확장 재빌드 + 검증
- `cd rhwp-chrome && npm run build` → dist/viewer.html 인라인 스크립트 부재 +
  dist/theme-init.js + dist/images/icon_small_ko_dark.svg 존재 확인.
- viewer 로드(호스트 Chrome) CSP 위반·404 로그 0 확인.
- `cd rhwp-studio && npm run build` 웹앱 빌드 성공 + `e2e/theme-mode.test.mjs` 통과(회귀 0).
- firefox 빌드 동형 확인.

### 3단계 — zip 재생성 + 문서
- chrome/edge/firefox zip + AMO source zip 0.2.5 재생성(이미 0.2.5 버전, 코드만 정정).
- 트러블슈팅 `extension_csp_inline_theme_script.md` + 단계별/최종 보고서.

## 4. 검증

- 확장 viewer: CSP 인라인 위반·다크 아이콘 404 미발생.
- 다크/라이트 전환 + FOUC 방지 (웹앱·확장).
- `e2e/theme-mode.test.mjs` 통과, `npm run build`(studio/chrome/firefox) 성공.

## 5. 위험과 대응

| 위험 | 대응 |
|------|------|
| 외부 스크립트 FOUC 재발 | `<head>` 최상단 동기 `<script src>`(module/defer 금지) — dev-tools 주입 선례와 동형 |
| theme-init.js 가 dist 누락 | vite public/ 복사 동작 — 2단계에서 dist 존재 확인 |
| firefox 미반영 | firefox 도 같은 index.html·public/ 사용, build 동형 확인 |
