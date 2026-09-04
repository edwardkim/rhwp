# Task M100 #1444 최종 보고서 — 확장 viewer CSP 인라인 + 다크 아이콘 404

- 이슈: #1444 "확장 viewer CSP 인라인 스크립트 차단 + 다크 아이콘 404 (다크테마 확장 회귀)"
- 마일스톤: M100 (v1.0.0)
- 브랜치: `local/task1444`
- 작성일: 2026-06-20

## 1. 개요

확장 0.2.5 배포 테스트에서 viewer 의 CSP 인라인 스크립트 차단·다크 아이콘 404 를 해소했다.
둘 다 PR #1420(다크테마)이 확장 빌드·CSP 경로를 고려하지 않은 회귀(웹앱 정상, 확장만 깨짐).

## 2. 근본 원인

- **① CSP**: index.html 다크테마 FOUC 방지 인라인 `<script>` 가 확장 CSP
  (`script-src 'self' 'wasm-unsafe-eval'`, unsafe-inline 없음) 위반 → 테마 초기화 미실행.
- **② 아이콘 404**: base.css:197 이 참조하는 `icon_small_ko_dark.svg` 가 확장 dist 에 없음.
- **핵심 함정**: 확장 vite 는 `publicDir: false` 라 `public/` 자산을 자동 복사하지 않는다.
  build.mjs 가 필요한 public 자산을 개별 copy 한다 (라이트 아이콘·favicon). 다크 아이콘과
  (분리한)theme-init.js 의 copy 라인이 누락이었다.

## 3. 해소

- `rhwp-studio/public/theme-init.js`(신규): index.html 인라인 IIFE 이전.
- `index.html`: 인라인 `<script>` → `<script src="/theme-init.js">`(동기, FOUC 유지).
- `rhwp-chrome/build.mjs`·`rhwp-firefox/build.mjs`: `theme-init.js` + `icon_small_ko_dark.svg`
  개별 복사 추가 (chrome·firefox 둘 다 누락이라 공통 정정).

## 4. 검증

- 확장 재빌드: dist 에 theme-init.js(원본 동일)·다크 아이콘 존재, viewer 인라인 부재.
- 웹앱 빌드 성공 + `e2e/theme-mode.test.mjs` 전부 PASS (테마 전환·FOUC 방지 회귀 0).
- **작업지시자 로컬 개발자모드(unpacked) 판정 통과** — viewer CSP 위반·다크 아이콘 404
  실제 미발생, 다크 테마 정상.

## 5. 후속 — zip 재생성·배포

스토어가 동일 버전(0.2.5) 재업로드를 허용하지 않아, 본 정정은 추가 기능과 함께 **다음
확장 버전(0.2.6+)으로 묶어 배포**한다(작업지시자 결정). 따라서 본 타스크에서 0.2.5 zip
재생성·재제출은 수행하지 않는다. 코드 정정은 devel/main 에 반영되어 다음 빌드에 포함된다.

## 6. 산출물

- 수행계획서: `mydocs/plans/task_m100_1444.md`
- 구현계획서: `mydocs/plans/task_m100_1444_impl.md`
- 단계별 보고서: `mydocs/working/task_m100_1444_stage{1,2}.md`
- 최종 보고서: 본 문서
- 트러블슈팅: `mydocs/troubleshootings/extension_csp_inline_theme_script.md`
- 코드: `rhwp-studio/public/theme-init.js`, `rhwp-studio/index.html`,
  `rhwp-chrome/build.mjs`, `rhwp-firefox/build.mjs`
