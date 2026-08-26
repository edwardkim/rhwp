# 구현계획 — Task M100 #6118 데스크톱 서식 도구 모음 압축형 복원

- **상위 수행계획**: [task_m100_6118.md](task_m100_6118.md)
- **이슈**: [#6118](https://github.com/edwardkim/rhwp/issues/6118)
- **작성일**: 2026-08-26 KST
- **작업 브랜치**: `codex/issue-6118-desktop-style-bar-compact`
- **통합 기준**: `upstream/devel@6b5c4f871972380c0866e2a8d27ac2bc67d257e6`
- **구현 상태**: 승인 대기, 제품 코드 변경 0건

## 1. 구현 원칙

이번 수정의 단일 진입점은 CSS media query다. 새 상태 변수나 사용자 설정 없이 viewport가
`1280px` 이상이면 압축형, 그보다 좁으면 현행 반응형 그룹형을 사용한다.

```text
index.html의 현재 그룹 DOM
       │
       ├─ >= 1280px ─ responsive.css 데스크톱 압축 규칙
       │                한 줄 / label 시각 감춤 / <= 36px
       │
       └─ <= 1279px ─ 기존 tablet·mobile 규칙 유지
```

구조·기능을 CSS와 분리해 다음 불변식을 지킨다.

- `index.html`의 컨트롤 ID, 순서, label 연결, title, `aria-*`를 바꾸지 않는다.
- `src/commands`, `src/core`, `src/main.ts`, `userSettings`, theme 초기화 코드는 바꾸지 않는다.
- 아이콘 mask/SVG, 드롭다운 DOM과 클릭·키보드 이벤트를 재사용한다.
- 색을 직접 하드코딩하지 않고 기존 `--ui-*`, `--color-*` 토큰을 사용한다.
- 좁은 화면 규칙의 selector 특이도를 불필요하게 높이지 않는다.

## 2. 파일별 변경안

| 파일 | 예정 변경 | 변경하지 않을 계약 |
| --- | --- | --- |
| [`responsive.css`](../../rhwp-studio/src/styles/responsive.css) | `@media (min-width: 1280px)` 압축형 규칙 추가 | 1279px 이하 breakpoint와 mobile landscape 숨김 정책 |
| [`style-bar.css`](../../rhwp-studio/src/styles/style-bar.css) | 공통 토큰 또는 중복 제거가 꼭 필요할 때만 최소 보정 | 입력·버튼·아이콘·드롭다운 기본 형태 |
| [`style-toolbar-grouped-ribbon.test.ts`](../../rhwp-studio/tests/style-toolbar-grouped-ribbon.test.ts) | DOM 자산 보존과 데스크톱 label 시각 감춤 계약 | 명령 affordance와 아이콘 계약 |
| [`responsive-toolbar-layout.test.ts`](../../rhwp-studio/tests/responsive-toolbar-layout.test.ts) | 1280 이상 압축형, 1279 이하 그룹형 정적 계약 | toolbar·tablet·mobile 기존 회귀 계약 |
| [`responsive.test.mjs`](../../rhwp-studio/e2e/responsive.test.mjs) | 경계 viewport와 실제 geometry·overflow·가시성 검증 | 기존 canvas·menu·toolbar·status smoke |
| [`e2e/MANIFEST.md`](../../rhwp-studio/e2e/MANIFEST.md) | 기존 항목 설명을 실제 검증 범위와 맞출 때만 갱신 | 새 중복 E2E 파일 추가 없음 |

`index.html`, 스킨 CSS와 TypeScript 제품 코드는 원칙적으로 수정하지 않는다. 구현 중 현재 markup만으로
접근성 이름을 보존할 수 없다는 근거가 생길 때에만 최소 markup 보정을 별도 stage 기록에 제안하고
승인 범위를 다시 확인한다.

## 3. CSS 상세 설계

### 3.1 넓은 데스크톱 media query

`responsive.css`의 1279px 이하 규칙과 맞닿는 위치에 다음 역할의 규칙을 둔다.

- `#style-bar`
  - `flex-direction: row`
  - `flex-wrap: nowrap`
  - `align-items: center`
  - `align-content: normal`
  - 목표 높이 최대 36px, 세로 overflow 없음
- `.sb-ribbon-group`
  - `min-height: 0`
  - `flex-direction: row`
  - 중앙 정렬, 압축 padding
  - 기존 `var(--ui-border)`로 그룹 경계 유지
- `.sb-ribbon-label`, `.sb-field-label`
  - 데스크톱에서만 시각 감춤
  - 연결된 form label·접근성 이름이 유지되는 방식을 사용
- `.sb-field-grid`, `.sb-command-band`, `.sb-command-group`
  - 줄바꿈 금지
  - `min-width: 0`과 필요한 `flex` 축소를 제한적으로 적용
- 콤보 폭
  - 우선 현재 폭을 유지한다.
  - 1280px 실측에서 overflow가 있을 때만 과거 압축형 폭을 참고해 글꼴 계열 필드부터 최소 조정한다.

기본 `style-bar.css`의 68px 그룹 규칙은 좁은 화면의 기반으로 남긴다. 데스크톱 override를 명시적으로
추가하면 현재 태블릿·모바일 selector와 markup을 대규모로 되돌릴 필요가 없다.

### 3.2 접근성과 명령 보존

- 숨긴 텍스트가 form control의 접근성 이름을 제공한다면 DOM에서 제거하거나 `aria-hidden`으로 만들지 않는다.
- `display: none`이 접근성 이름을 소실하는 조합이면 공용 visually-hidden 패턴을 사용한다.
- Tab 순서, Enter/Space activation, select 열기, 색·글자 효과 dropdown 진입을 변경하지 않는다.
- `#btn-*`, `#style-name`, `#font-lang`, `#font-name`, `#font-size`, `#linespacing-select`의 존재와 순서를
  정적 테스트에서 고정한다.

### 3.3 스킨 호환

[플랫 스킨](../../rhwp-studio/src/styles/theme-flat.css)과
[올드스쿨 스킨](../../rhwp-studio/src/styles/theme-oldschool.css)은 `#style-bar` 표면과 경계만 override하므로
geometry는 데스크톱 media query가 소유한다. 스킨별 geometry override를 새로 만들지 않고 다음만 확인한다.

- default/flat/oldschool × light/dark에서 높이와 줄 수가 동일하다.
- oldschool의 top/bottom bevel을 포함해 계산 높이가 36px를 넘지 않는다.
- focus, hover, selected와 그룹 경계의 명암 대비가 유지된다.

## 4. 테스트 설계

### 4.1 정적 계약 테스트

`style-toolbar-grouped-ribbon.test.ts`는 현재 DOM 자산과 명령 affordance가 유지됨을 검증하고,
`responsive-toolbar-layout.test.ts`는 다음 CSS 계약을 검증한다.

| 구간 | 계약 |
| --- | --- |
| `min-width: 1280px` | nowrap, row, 그룹 최소 높이 해제, 제목 시각 감춤 |
| `1024px..1279px` | 현재 제목 포함 그룹형과 separator 정책 유지 |
| `768px..1023px` | 현재 tablet grid와 compact track 유지 |
| `<=767px` | 현재 mobile column/grid, touch control, landscape 정책 유지 |

정적 테스트가 CSS 문자열 하나에 과도하게 결합되지 않도록 media query 구간을 먼저 잘라 selector별 핵심
속성만 검사한다.

### 4.2 실제 브라우저 E2E

새 파일 대신 기존 `responsive.test.mjs`를 확장한다. 각 viewport에서 `getBoundingClientRect()`와
computed style로 다음을 수집한다.

- `styleBarHeight`, `scrollWidth`, `clientWidth`
- 각 `.sb-ribbon-group`의 `top`, `bottom`, `height`
- 서로 다른 group top 좌표의 개수(한 줄 여부)
- 필수 컨트롤의 visible/clickable 여부
- `.sb-ribbon-label`, `.sb-field-label`의 시각 표시 상태
- canvas/editor/status 기존 smoke 결과

대표 viewport는 다음과 같다.

| viewport | 목적 |
| --- | --- |
| 1920×1080 | 일반 넓은 데스크톱 압축형 |
| 1280×900 | 새 breakpoint 포함 경계 |
| 1279×900 | 그룹형으로 돌아가는 바로 아래 경계 |
| 1024×768 | 중간 데스크톱 그룹형 |
| 768×1024 | tablet grid |
| 412×915 | mobile touch layout |

기본 스킨은 모든 viewport에서 실행한다. 1280×900에서는 flat/oldschool과 light/dark 조합을 추가 계측하고
대표 스크린샷을 남긴다. 문서 renderer 변경이 아니므로 PDF/SVG visual sweep은 실행하지 않는다.

## 5. 검증 명령

구현 완료 뒤 같은 checkout에서 순차 실행한다.

```bash
(cd rhwp-studio && npx tsc --noEmit)
npm --prefix rhwp-studio test
npm --prefix rhwp-studio run build
node rhwp-studio/e2e/responsive.test.mjs --mode=headless
cargo fmt --all
cargo fmt --all -- --check
python3 scripts/check_markdown_links.py
git diff --check
```

E2E가 실행 중인 Vite를 요구하면 [개발 환경 안내](../manual/dev_environment_guide.md)의 표준 WASM·Vite
절차를 사용하고 실제 URL, browser, viewport, DPR을 stage 보고서에 기록한다. source가 `rhwp-studio`에만
한정되므로 Rust 전체 회귀와 renderer 시각 sweep은 기본 게이트가 아니다.

## 6. 위험과 중단 조건

| 위험 | 완화 또는 중단 조건 |
| --- | --- |
| 1280px에서 전체 컨트롤이 맞지 않음 | 필드 폭을 실측 기반으로 최소 조정하고 명령 숨김은 금지한다. 맞지 않으면 breakpoint 재논의로 중단한다. |
| 제목 감춤으로 접근성 이름 소실 | accessibility tree/label 연결을 확인하고 visually-hidden 방식으로 전환한다. |
| oldschool bevel로 높이 초과 | box sizing과 경계 포함 높이를 계측하되 스킨 전용 geometry 분기는 만들지 않는다. |
| narrow layout selector 회귀 | 1279/1024/768/412px E2E가 실패하면 데스크톱 override 범위를 좁힌다. |
| #6115와 충돌 | `#icon-toolbar`, visibility state, `Ctrl+F1` 관련 파일을 수정하지 않는다. 충돌이 필요하면 별도 조정 승인을 받는다. |
| 현재 markup만으로 한컴형 밀도 불가 | 과거 DOM 전체 복원으로 확대하지 않고 측정 근거와 최소 markup 변경안을 먼저 보고한다. |

## 7. 커밋·보고 단위

1. 계획 승인 커밋: 이 수행·구현 계획과 오늘 할 일만 포함
2. Stage 1 커밋: 기준선·회귀 계약 보고서
3. Stage 2 커밋: CSS, 정적 테스트, E2E와 구현 보고서
4. Stage 3 커밋: 검증 결과, 최종 보고서, PR용 대표 증적

각 단계가 끝날 때 현재 단계의 변경만 커밋하고 다음 단계로 넘어간다. remote push와 PR 생성은 모든
로컬 검증 완료 뒤 사용자에게 별도로 승인을 받는다.

## 8. 승인 게이트

현재 구현 상태는 **승인 대기**다. 이 문서가 승인되기 전에는 위 파일의 제품·테스트 변경을 시작하지 않는다.
