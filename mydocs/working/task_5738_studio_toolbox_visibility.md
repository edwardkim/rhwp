# #5738 rhwp-studio 도구 상자(기본/서식) 표시 상태 저장·복원

## 목표

`보기 > 도구 상자 > 기본/서식` 의 보이기/숨기기를 사용자 설정으로 저장해 다음 실행에서
복원하고, 메뉴에서 현재 켜짐/꺼짐을 알 수 있게 한다. 숨김 상태로 재시작할 때 도구 모음이
잠깐 보였다 사라지는 깜빡임이 없어야 한다.

## 문제

- 두 커맨드가 인라인 `style.display` 와 커맨드 모듈의 지역 변수(`visible`)에만 상태를 두어
  창을 닫으면 사라졌다. 재시작하면 항상 둘 다 보임이었다.
- `index.html` 의 두 항목이 `md-item disabled` 로 굳어 있어 비활성으로 보였고, 현재 상태를
  나타내는 표시가 없었다.

## 구현

1. `user-settings.ts` 의 `view` 절에 `toolbarBasic` / `toolbarFormat` 을 추가한다. 기본값은
   둘 다 `true`(보임)이고, 저장값이 없거나 boolean 이 아니면 기존 `normalizeBoolean` 경로로
   기본값을 채운다. 저장은 기존 단일 키 `rhwp-settings` 를 그대로 쓴다.
2. `src/view/toolbox-visibility.ts` 를 새로 두고 "설정값 → 루트 표시 상태 + 메뉴 체크 표시"
   한 방향만 담당하게 한다. `document` 를 인자로 받아 전역 없이 검증할 수 있다.
3. 숨김을 인라인 `style.display` 가 아니라 루트 data 속성
   (`data-toolbox-basic` / `data-toolbox-format` = `shown` | `hidden`)으로 표현하고,
   `src/style.css` 에 **숨김만** 하는 규칙을 둔다. 보임일 때 인라인 값을 남기지 않으므로
   스킨·뷰어 모드 CSS(`.rhwp-chrome-no-toolbar` 등)와 충돌하지 않는다.
4. 같은 속성을 `public/theme-init.js`(번들보다 먼저 동기 실행되는 기존 테마 FOUC 방지
   스크립트)가 저장값을 읽어 첫 페인트 전에 찍는다. 이것이 깜빡임을 없애는 자리다.
5. `view:toolbox-basic` / `view:toolbox-format` 은 설정을 토글하고 `syncToolboxMenu()` 로
   같은 경로를 태운다. `main.ts` 는 시작 시 같은 함수로 복원한다.
6. 메뉴 항목의 `disabled` 를 걷어내고 `role="menuitemcheckbox"` 로 두어, 켜짐이면
   `active` 클래스 + `aria-checked="true"` 로 상태를 표시한다(앞쪽 체크 글리프는 두지 않는다).

## 검증

| 검증 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | 통과 (Rust 변경 없음) |
| `npx tsc --noEmit -p tsconfig.json` (rhwp-studio) | 통과 |
| `npm test` (rhwp-studio) | 1039개 중 1038 통과 · 0 실패 · 1 skip |
| `npm run e2e:toolbox-visibility` (rhwp-studio) | 10개 단언 전부 PASS |

헤드리스 Chrome(dev 서버 `:7700`) e2e 실측:

- 첫 방문(저장값 없음): `data-toolbox-basic="shown"`, `#icon-toolbar` / `#style-bar`
  `display: flex` — 기본값 보임.
- 숨기기로 저장 후 reload: `DOMContentLoaded` 부터 60프레임을 샘플링해
  **도구 모음이 보인 프레임 0**. 첫 샘플부터 `display: none`.
- 메뉴 상태: 숨김이면 `active=false` · `aria-checked="false"`, `disabled` 아님.
- 다시 보이기로 토글: `localStorage.rhwp-settings.view` 가
  `{ toolbarBasic: true, toolbarFormat: false }` 로 저장되고 메뉴 체크 상태가 따라간다.

e2e 는 `rhwp-studio/e2e/toolbox-visibility.test.mjs`(TC1 기본값 · TC2 메뉴 체크 표시 ·
TC3 토글 저장 · TC4 리로드 복원과 깜빡임 0프레임)다. 단위 계약 테스트는
`rhwp-studio/tests/toolbox-visibility.test.ts` 3건(설정 → 루트 상태·메뉴 체크
동기화, 메뉴 항목이 체크형 활성 항목, 숨김 규칙과 첫 페인트 초기화가 같은 data 속성 사용)과
`rhwp-studio/tests/user-settings.test.ts` 2건(저장·기본값, 새 모듈 인스턴스에서 복원)이다.
