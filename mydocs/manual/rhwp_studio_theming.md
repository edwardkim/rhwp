---
kind: reference
status: active
canonical: mydocs/manual/rhwp_studio_theming.md
last_verified: 2026-08-13
---

# rhwp-studio 테마 토큰과 스킨 제작

rhwp-studio의 색·형태 값은 전부 `src/styles/base.css`의 CSS 변수(토큰)로 구동된다.
이 문서는 토큰 계층, 테마 적용 파이프라인, 그리고 새 스킨을 추가할 때 지켜야 할
규칙을 정의한다.

## 테마 축은 두 개다

| 축 | 설정 키 | root 속성 | 값 |
| --- | --- | --- | --- |
| 모드 (밝기) | `theme.mode` | `data-theme-effective` | `light` \| `dark` (`system`은 OS 설정으로 해석) |
| 스킨 (룩) | `theme.skin` | `data-theme-skin` | 기본 스킨은 속성 없음, 옵트인 스킨은 스킨명 (예: `flat`) |

두 축은 독립이다. 스킨은 밝기 축을 침범하지 않아야 하며(아래 다크 가드 규칙),
다크 팔레트는 `:root[data-theme-effective="dark"]`가 항상 소유한다.

## 토큰 계층

`base.css`의 `:root` 블록이 유일한 토큰 정의 지점이다 (약 170여 개).

1. **원시 토큰** — 실제 값이 정의되는 곳.
   - 표면·경계·텍스트·상호작용: `--ui-*` (예: `--ui-bg`, `--ui-border-subtle`, `--ui-hover`)
   - 액센트: `--accent-*` (`--accent-primary` 등)
   - 문서 작업 영역: `--doc-*` (`--doc-workspace`, `--doc-paper`, `--doc-shadow`)
   - 눈금자: `--ruler-*`
   - 형태: `--radius-*`, `--shadow-*`, `--space-*`, `--font-size-*`, `--control-height-*`
   - UI 크롬 글꼴: `--font-family-ui` — 메뉴·툴바 등 크롬 전용이며 **문서 본문 렌더링
     글꼴과 무관하다**. 스킨이 재정의할 수 있는 토큰이다 (플랫 스킨은 시스템에 설치된
     Pretendard 를 우선 사용하고, 글꼴 파일은 번들하지 않는다).
2. **시맨틱 별칭** — 원시 토큰을 참조하는 이름 (예: `--color-primary: var(--accent-primary)`,
   `--color-surface: var(--ui-surface)`). 컴포넌트 CSS가 역할 이름으로 소비할 수 있게 한다.

소비 규칙: 컴포넌트/영역 CSS는 **토큰만 참조**하고 색을 하드코딩하지 않는다.
(canvas에 그리는 코드는 `cssVar()` 헬퍼로 토큰을 읽는다 — 예: `src/view/ruler.ts`.)

## 적용 파이프라인

1. `public/theme-init.js` — 페이지 렌더 전에 `rhwp-settings`를 읽어
   `data-theme-mode` / `data-theme-effective` / `data-theme-skin`을 세팅한다 (FOUC 방지,
   확장 CSP 때문에 외부 파일 + 동기 로드 유지).
2. `src/core/theme.ts` — 런타임 전환. `applyTheme()`가 위 dataset을 갱신하고
   `syncThemeMenu()`가 메뉴 체크 상태(`data-theme-mode-choice` / `data-theme-skin-choice`)를
   동기화한다.
3. `src/core/user-settings.ts` — `theme.mode` / `theme.skin` 저장과 정규화.
4. CSS cascade — 기본 팔레트(`:root`) → 다크(`:root[data-theme-effective="dark"]`) →
   스킨 파일(마지막 import). 동일 특이도에서는 후행 로드가 이기므로 스킨의 색 변수는
   반드시 다크 가드를 건다(아래).

## 새 스킨 추가 체크리스트

`styles/theme-flat.css`(플랫 스킨)를 참조 구현으로 삼는다.

1. `src/styles/theme-<name>.css`를 만들고 `src/style.css` 마지막에 import 한다.
2. **모든 규칙을 `[data-theme-skin="<name>"]` 아래에 스코프한다.** 속성이 없으면
   파일 전체가 비활성이어야 한다 (기본 스킨 = 무속성).
3. **색 변수 재정의는 라이트 가드를 건다**:
   `:root[data-theme-skin="<name>"]:not([data-theme-effective="dark"]) { … }`.
   가드 없는 `:root[data-theme-skin]` 블록에는 형태 토큰(`--radius-*`, `--shadow-*`)만
   허용한다. 다크까지 재정의하는 스킨이라면
   `:root[data-theme-skin="<name>"][data-theme-effective="dark"]` 블록을 별도로 둔다.
4. 표면 규칙(배경·보더)은 **토큰 참조만** 사용한다 — 다크에서 색이 자동으로 따라온다.
5. 배선: `user-settings.ts`의 `ThemeSkin` 유니온·정규화, `theme.ts`, 보기 > 테마 메뉴에
   `menuitemradio` + `data-theme-skin-choice` 항목, `theme-init.js`의 스킨 판독을 갱신한다.
6. UI 명칭·접두어는 [rhwp-studio UI 명칭과 CSS 접두어](rhwp_studio_ui_conventions.md)를
   따르고, 새 DOM을 추가하지 않는 것을 기본으로 한다.
7. `tests/theme-skin.test.ts`의 정적 검사(스코프·다크 가드)와
   `tests/user-settings.test.ts`의 설정 왕복 테스트를 통과해야 한다.

## 관련 파일

| 역할 | 파일 |
| --- | --- |
| 토큰 정의 | `rhwp-studio/src/styles/base.css` |
| 스킨 참조 구현 | `rhwp-studio/src/styles/theme-flat.css` |
| FOUC 방지 초기화 | `rhwp-studio/public/theme-init.js` |
| 런타임 전환 | `rhwp-studio/src/core/theme.ts` |
| 설정 저장 | `rhwp-studio/src/core/user-settings.ts` |
| 스킨 규칙 테스트 | `rhwp-studio/tests/theme-skin.test.ts` |
