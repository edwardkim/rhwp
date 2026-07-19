---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# 확장 viewer CSP 인라인 스크립트 차단 + public 자산 누락 (Task #1444)

## 증상

크롬/엣지 확장 viewer 에서 (웹앱 rhwp-studio 는 정상):

```
viewer.html:8 Executing inline script violates the following Content Security Policy
directive 'script-src 'self' 'wasm-unsafe-eval''. ... The action has been blocked.
icon_small_ko_dark.svg:1  Failed to load resource: net::ERR_FILE_NOT_FOUND
```

## 근본 원인 — 확장 환경 특수성 2가지 (자기 검증 ≠ 확장 환경)

### ① 인라인 스크립트는 확장 CSP 가 금지

`rhwp-studio/index.html` `<head>` 의 다크테마 FOUC 방지 인라인 `<script>`(테마 즉시 적용)
가 빌드 시 viewer.html 에 인라인된 채 남는다. 확장 manifest CSP
`extension_pages: "script-src 'self' 'wasm-unsafe-eval'"` 는 `unsafe-inline` 이 없어 인라인
실행을 차단한다. 웹앱은 CSP 가 느슨해 통과 → **확장에서만 깨짐**.

→ 해결: 인라인 IIFE 를 `rhwp-studio/public/theme-init.js` 로 분리하고 index.html `<head>`
최상단에서 `<script src="/theme-init.js"></script>`(동기, module/defer 금지 — FOUC 방지
유지)로 로드. CSP `'self'` 충족.

### ② 확장 vite 는 `publicDir: false` — public/ 자산이 자동 복사 안 됨 (핵심 함정)

확장 빌드 `rhwp-chrome/vite.config.ts`·`rhwp-firefox/vite.config.ts` 는 `publicDir: false`
(samples/images 등 대용량 public 제외). 그래서:
- `public/theme-init.js` 가 dist 로 **복사되지 않음** → viewer.html 의 `/theme-init.js`
  src 가 404 (분리만으로 해결 안 됨).
- `public/images/icon_small_ko_dark.svg` 도 자동 복사 안 됨 → base.css 다크 모드 404.

웹앱 vite(rhwp-studio)는 publicDir 기본값이라 public/ 을 복사 → 웹앱은 정상. **확장만**
`build.mjs` 가 favicon·아이콘처럼 **필요한 public 자산을 개별 copy 한다.**

→ 해결: `rhwp-chrome/build.mjs`·`rhwp-firefox/build.mjs` 에 `theme-init.js` 와
`icon_small_ko_dark.svg` 개별 복사 라인 추가.

## 재발 방지 체크리스트

- [ ] rhwp-studio 에 인라인 `<script>` 추가 금지 — 확장 CSP 가 막는다. 외부 파일 + src.
- [ ] 확장 viewer 가 참조하는 새 public/ 자산(이미지·스크립트)은 **`build.mjs` 에 개별
      copy 추가 필수** (확장 vite 는 `publicDir: false` — 자동 복사 안 됨).
- [ ] chrome·firefox build.mjs 둘 다 정정 (별도 파일, 같은 누락 반복).
- [ ] 새 확장 기능은 chrome://extensions unpacked 로드로 viewer 콘솔 CSP/404 확인
      (웹앱 정상 = 확장 정상 아님).

## 관련

- Task #1444, PR #1420(다크테마) 회귀.
- `rhwp-studio/public/theme-init.js`(신규), `rhwp-studio/index.html`,
  `rhwp-chrome/build.mjs`·`rhwp-firefox/build.mjs`, manifest CSP.
- [[project_branch_policy]]
