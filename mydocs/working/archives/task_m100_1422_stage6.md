# Stage 6 완료 보고서 — Task M100-1422

- 이슈: https://github.com/edwardkim/rhwp/issues/1422
- 브랜치: `local/task1422`
- 단계: Stage 6 — Chrome Auto Dark Mode 최종 점검
- 완료 시각: 2026-06-17 15:25

## 1. 작업 요약

Chrome `Auto Dark Mode for Web Contents`가 켜진 환경에서 rhwp-studio의 명시적 테마 선택을 보존하도록
브라우저 색상 스킴 힌트를 보강했다.

핵심 변경은 다음과 같다.

- 초기 HTML에 `<meta name="color-scheme" content="light dark">`를 추가했다.
- 테마 적용 시 root inline `color-scheme`과 meta `color-scheme`을 `only light` / `only dark`로 동기화했다.
- `밝게` 테마는 Chrome 강제 다크 변환을 억제하고, `어둡게` 테마는 Chrome 자동 변환이 아니라 앱 dark token으로 표시되도록 했다.
- e2e helper에 `CHROME_EXTRA_ARGS`를 추가해 Chrome feature flag 기반 회귀 테스트를 실행할 수 있게 했다.
- Auto Dark Mode 전용 픽셀 회귀 테스트를 추가했다.

## 2. 수정 파일

- `rhwp-studio/src/core/theme.ts`
- `rhwp-studio/index.html`
- `rhwp-studio/e2e/helpers.mjs`
- `rhwp-studio/e2e/theme-mode.test.mjs`
- `rhwp-studio/e2e/dialog-theme.test.mjs`
- `rhwp-studio/e2e/theme-auto-dark.test.mjs`
- `mydocs/orders/20260617.md`

## 3. 검증 결과

```bash
cd rhwp-studio && npm run build
```

- 통과

```bash
cd rhwp-studio && VITE_URL=http://127.0.0.1:7702 CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' node e2e/theme-mode.test.mjs --mode=headless
```

- 통과
- `dark only`, `light only`, meta `only dark`, meta `only light` 기대값 확인

```bash
cd rhwp-studio && VITE_URL=http://127.0.0.1:7702 CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' node e2e/dialog-theme.test.mjs --mode=headless
```

- 통과
- Stage 5에서 고정한 다이얼로그 dark token/문서 preview 정책 유지 확인

```bash
cd rhwp-studio && VITE_URL=http://127.0.0.1:7702 CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' CHROME_EXTRA_ARGS='--enable-features=WebContentsForceDark' node e2e/theme-auto-dark.test.mjs --mode=headless
```

- 통과
- Auto Dark Mode 활성 감지 확인
- `밝게` 테마 menu-bar 픽셀: `(245,245,245)`로 밝게 유지
- `어둡게` 테마 menu-bar 픽셀: `(43,48,55)`로 앱 dark token 유지

## 4. 브라우저 확인

- dev server는 `http://127.0.0.1:7702/`에서 실행 중이다.
- 인앱 브라우저에서 해당 URL 로드와 rhwp-studio 타이틀 표시를 확인했다.

## 5. 잔여 작업

- Stage 1~6 변경분 기준 최종 보고서를 작성하고 전체 작업 종료 승인을 요청한다.
- 이슈 close는 작업지시자 승인 후에만 수행한다.
