# 구현 계획서 — Task M100-1158: rhwp-studio 테마 서비스와 UI chrome 다크테마

- 이슈: https://github.com/edwardkim/rhwp/issues/1158
- 수행 계획서: `mydocs/plans/task_m100_1158.md`
- 작성일: 2026-06-16
- 브랜치: `local/task_m100_1158`

## 1. 설계 요약

테마 선택값과 실제 적용 테마를 분리한다.

- 저장값: `system | light | dark`
- 실제 적용값: `light | dark`
- DOM 반영:
  - `document.documentElement.dataset.themeMode`
  - `document.documentElement.dataset.themeEffective`

CSS는 `:root`의 light 기본값과 `:root[data-theme-effective="dark"]` override로 구성한다. 기존
`--color-*` 변수는 호환 alias로 남기고, 신규 semantic token을 주요 UI chrome에 우선 적용한다.

## 2. 수정 파일

예상 수정:

- `rhwp-studio/src/core/user-settings.ts`
- `rhwp-studio/src/core/theme.ts` 신규
- `rhwp-studio/src/command/types.ts`
- `rhwp-studio/src/command/commands/view.ts`
- `rhwp-studio/src/main.ts`
- `rhwp-studio/index.html`
- `rhwp-studio/src/styles/base.css`
- `rhwp-studio/src/styles/menu-bar.css`
- `rhwp-studio/src/styles/toolbar.css`
- `rhwp-studio/src/styles/style-bar.css`
- `rhwp-studio/src/styles/editor.css`
- `rhwp-studio/src/styles/status-bar.css`
- `rhwp-studio/src/styles/dialogs.css`
- `rhwp-studio/src/styles/command-palette.css`
- `rhwp-studio/e2e/theme-mode.test.mjs` 신규

## 3. Stage 1 — 테마 서비스

1. `ThemeMode`, `ThemeSettings` 타입을 추가한다.
2. `defaultSettings()`에 `theme: { mode: 'system' }`을 추가한다.
3. 기존 설정 로드 시 `theme` 기본값 병합을 보장한다.
4. `getThemeSettings()`, `setThemeMode()` 또는 `updateThemeSettings()`를 추가한다.
5. 신규 `theme.ts`에서 다음 API를 제공한다.
   - `getThemeMode()`
   - `setThemeMode(mode)`
   - `getEffectiveTheme(mode)`
   - `applyTheme(mode)`
   - `initThemeSync(onChange?)`
6. `matchMedia('(prefers-color-scheme: dark)')` 변경 시 system 모드의 effective theme만 갱신한다.

## 4. Stage 2 — 메뉴/명령 연동

1. `CommandServices`에 `getThemeMode`, `setThemeMode`를 추가한다.
2. `viewCommands`에 다음 명령을 추가한다.
   - `view:theme-system`
   - `view:theme-light`
   - `view:theme-dark`
3. 각 명령은 설정 저장 후 DOM theme dataset과 메뉴 active 상태를 갱신한다.
4. `index.html` 보기 메뉴에 `테마` 서브메뉴를 추가한다.
5. `main.ts`에서 앱 초기화 전후로 theme sync를 설치한다.

## 5. Stage 3 — CSS semantic token 적용

`base.css`에 신규 token을 추가한다.

```css
--ui-bg
--ui-surface
--ui-surface-raised
--ui-border
--ui-text
--ui-text-muted
--ui-hover
--ui-active
--ui-selected
--doc-workspace
--doc-paper
--doc-shadow
--focus-ring
--selection-fill
--caret-color
--resize-handle
```

전환 방침:

- body/root/chrome 배경은 `--ui-*` 사용
- 편집 작업영역은 `--doc-workspace` 사용
- canvas/page 배경은 `--doc-paper` 사용
- 기존 `--color-*`는 신규 token alias로 남겨 기존 CSS 회귀를 줄인다.

## 6. Stage 4 — e2e 스모크 테스트

신규 `rhwp-studio/e2e/theme-mode.test.mjs`:

1. Vite 페이지에 접속한다.
2. localStorage를 초기화하고 기본값 `system`을 확인한다.
3. `view:theme-dark` 명령 또는 메뉴 클릭으로 dark를 선택한다.
4. `documentElement.dataset.themeMode === 'dark'` 확인.
5. `#scroll-content canvas`의 computed background가 흰색 계열인지 확인한다.
6. 새로고침 후 dark 설정이 유지되는지 확인한다.
7. light로 되돌린 뒤 dataset과 주요 chrome 색상 변화 확인.

## 7. 검증 명령

```bash
cd rhwp-studio && npm run build
cd rhwp-studio && node e2e/theme-mode.test.mjs
```

Rust/WASM 코드를 수정하지 않으므로 `cargo` 전체 테스트는 기본 검증에서 제외한다. 단, Vite가
`@wasm/rhwp.js` 산출물을 요구하는 상태면 `wasm-pack build --target web --out-dir pkg`를 먼저 수행한다.

## 8. 완료 기준

- `system | light | dark` 선택과 저장이 동작한다.
- dark에서 앱 chrome이 일관되게 어두워진다.
- 편집 용지는 dark에서도 흰색이다.
- light 설정으로 되돌리면 기존 light UI와 같은 계열로 표시된다.
- `npm run build`와 신규 e2e 스모크가 통과한다.

## 9. 승인 요청

위 계획대로 Stage 1부터 구현한다. 승인 전에는 소스 코드를 수정하지 않는다.
