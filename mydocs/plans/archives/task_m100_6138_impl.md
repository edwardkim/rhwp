# 구현계획 — Task M100 #6138 기본 도구 상자 한 줄 그룹 스크롤

- **상위 수행계획**: [task_m100_6138.md](task_m100_6138.md)
- **이슈**: [#6138](https://github.com/edwardkim/rhwp/issues/6138)
- **작성일**: 2026-08-26 KST
- **작업 브랜치**: `codex/issue-6118-responsive-style-bar`
- **통합 기준**: `upstream/devel@1011a8947`
- **구현 상태**: Stage 3·최종 보고 완료, #6118 통합 PR 승인 대기

## 1. 구현 불변식

- 외부 `#icon-toolbar` ID와 visibility/theme/focus 예외 계약을 유지한다.
- 기존 `.tb-group`, `.tb-sep`, button·menu DOM의 순서와 identity를 유지한다.
- 명령을 clone하거나 breakpoint별 대체 command를 만들지 않는다.
- 모든 너비에서 label 포함 44px desktop button과 56px toolbar 높이를 유지한다.
- overflow 여부는 고정 viewport 이름이 아니라 `scrollWidth > clientWidth`로 판정한다.
- 시각 scrollbar를 숨겨도 native horizontal scroll과 keyboard focus 도달성을 유지한다.
- `#style-bar`, command, WASM, user setting schema는 변경하지 않는다.

## 2. 파일별 변경안

| 파일 | 예정 변경 | 보존 계약 |
| --- | --- | --- |
| [`index.html`](../../../rhwp-studio/index.html) | 이전/다음 버튼, viewport·track wrapper | 기존 group·sep·button 순서와 ID |
| [`toolbar.css`](../../../rhwp-studio/src/styles/toolbar.css) | 한 줄 viewport, 숨긴 scrollbar, nav 상태 | 기존 button/icon/theme token |
| [`responsive.css`](../../../rhwp-studio/src/styles/responsive.css) | wrap·label 숨김·button 축소 제거 | menu/editor/status 반응형 |
| `src/ui/icon-toolbar-scroller.ts` | overflow 측정, group 이동, resize/mode/focus 동기화 | native scroll과 기존 command state |
| [`main.ts`](../../../rhwp-studio/src/main.ts) | controller 초기화 | 문서·toolbar 초기화 순서 |
| [`responsive-toolbar-layout.test.ts`](../../../rhwp-studio/tests/responsive-toolbar-layout.test.ts) | 한 줄·밀도·wrapper 계약 | #6118 style bar 계약 |
| `tests/icon-toolbar-scroller.test.ts` | controller·접근성·group 이동 source 계약 | 명령 동작 비개입 |
| [`responsive.test.mjs`](../../../rhwp-studio/e2e/responsive.test.mjs) | 한 줄 높이, nav/scroll/mode/visibility 조작 | 기존 #6118·canvas smoke |
| theme CSS | nav hover/focus/disabled skin 표현 | 기존 skin token과 geometry |

## 3. controller 설계

`IconToolbarScroller`는 root, viewport, track, 이전·다음 버튼만 소유한다.

- `ResizeObserver`: root·viewport·track의 폭 변화 뒤 overflow를 rAF 한 번으로 재계산한다.
- `MutationObserver`: track 직계 group/sep의 `style`·`hidden` 변경만 감지한다. button active 변경에는 반응하지
  않아 사용 중 scroll 위치가 불필요하게 초기화되지 않게 한다.
- `scroll`: 현재 `scrollLeft`와 최대값으로 이전·다음 disabled·edge hidden을 갱신한다.
- `focusin`: offscreen command에 keyboard focus가 가면 native `scrollIntoView(inline:nearest)`로 보인다.
- mode group 변경: `scrollLeft=0`으로 복귀한 뒤 overflow·disabled를 다시 계산한다.

이동 목표는 현재 scrollLeft보다 큰/작은 첫 가시 `.tb-sep`의 track 기준 경계다. 바깥 toolbar 기준
`offsetLeft`를 직접 쓰지 않고 divider·track 좌표의 차와 nav 4px 간격으로 0점을 정규화한다. 목표는
`0..maxScroll`로 clamp하고 reduced-motion 환경에서는 즉시, 그 외에는 240ms ease-out으로 이동한다.
목적지가 끝점이면 같은 frame에 해당 nav의 opacity·3px 퇴장을 시작한다.

## 4. DOM·CSS 계약

- root: `display:flex`, `flex-wrap:nowrap`, `height/min-height:56px`, `overflow:hidden`
- nav: overflow가 있을 때만 `hidden=false`, track 위 absolute 24px 표면과 theme token hover/focus/disabled를
  사용한다. toolbar와 같은 배경은 항상 유지하고 테두리·강조 표면은 hover/focus에서만 표시한다.
- viewport: `flex:1`, `min-width:0`, `overflow-x:auto`, `overflow-y:hidden`, `touch-action:pan-x`
- track: `display:flex`, `width:max-content`, `min-width:100%`, `height:100%`, `flex-wrap:nowrap`
- group: `flex:0 0 auto`
- scrollbar: Firefox·WebKit에서 시각적으로만 감추고 scroll 기능은 유지

nav가 숨겨진 상태의 전체 폭으로 먼저 overflow를 판정한다. nav는 absolute overlay라 시작·중간·끝 전환이
viewport 폭을 바꾸지 않는다. `ResizeObserver`가 viewport 변화와 최대 scroll 값을 다시 측정하고 track 기준
divider anchor를 사용하므로 목표 command는 잘리지 않는다.

## 5. 접근성 계약

- 이전/다음 button에 `aria-label`, `title`, native `disabled`를 제공한다.
- overflow가 없을 때 nav는 `hidden`이라 접근성 트리와 Tab 순서에서 제외한다.
- viewport는 `aria-label="기본 도구 상자 명령"`을 제공한다.
- 기존 button Tab 순서와 accessible name은 DOM 순서를 그대로 따른다.
- 시작·끝 disabled·`aria-hidden`·시각 숨김은 화면 위치와 항상 일치하고, absolute nav는 root의 8px
  padding과 track 정렬을 바꾸지 않는다.
- #6115로 root가 숨겨지면 nav를 포함한 전체 껍데기가 함께 사라진다.

## 6. 테스트 설계

### 6.1 정적·단위 계약

- outer `#icon-toolbar`, viewport, track, nav button과 기존 group 단일 identity
- toolbar `nowrap`, 56px, desktop label·44px button 유지
- 1023/767 breakpoint가 label·button·toolbar 높이를 바꾸지 않음
- ResizeObserver·MutationObserver, scroll/focus listeners와 dispose
- track 기준 가시 divider 경계 산출, clamp, 시작·끝 disabled·edge hidden nav

### 6.2 브라우저 E2E

| viewport | 판정 |
| --- | --- |
| 1920, 1280px | 56px 한 줄, nav 숨김, 전체 group 표시 |
| 1024, 962, 961, 883, 808, 807, 768px | 56px 한 줄, nav 표시, label 유지, divider 버튼 이동 |
| 412, 375px | 56px 한 줄, 첫→끝→첫 group 도달, page overflow 없음 |

overflow viewport에서 다음/이전 클릭, track 기준 `scrollLeft`, 시작·중간·끝 버튼 표시, horizontal wheel,
마지막 command focus를 검증한다. mode group의 inline style을 실제 mode 이벤트와 같은 방식으로 전환한 뒤
시작 위치·nav 상태를 다시 확인한다. #6115 visibility는 `data-toolbox-basic=hidden/shown`에서 root와 nav가
함께 숨고 복귀하는지 본다.

## 7. 검증 명령

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

Studio chrome 변경이므로 renderer PDF/SVG visual sweep과 Rust 전체 회귀는 기본 게이트가 아니다. 다만
default/flat/oldschool × light/dark 대표 화면과 #6118 통합 viewport 근거를 남긴다.

## 8. 위험과 중단 조건

| 위험 | 완화·중단 조건 |
| --- | --- |
| nav 표시가 viewport 폭을 줄여 측정 loop 발생 | hidden→overflow 두 상태가 rAF/ResizeObserver로 수렴하는지 E2E로 고정한다. |
| smooth scroll 중 disabled 판정이 흔들림 | scroll event마다 갱신하고 E2E는 정지 조건까지 기다린다. |
| button active mutation이 scroll을 초기화 | 직계 group/sep의 display 관련 mutation만 mode 변화로 취급한다. |
| focus가 화면 밖 명령에 남음 | focusin에서 inline nearest로 viewport만 이동한다. |
| #6115 숨김 뒤 측정값 0 | ResizeObserver 재표시 callback에서 다시 측정한다. |
| #6118과 세로 위치 혼동 | 각 toolbar 높이·행 수를 별도 필드로 기록하고 통합 screenshot을 남긴다. |

## 9. 커밋·보고 단위

1. 계획·Stage 1: 수행/구현 계획, 오늘 할 일, 기준선 보고
2. Stage 2: DOM/CSS/controller/test/E2E와 구현 보고
3. Stage 3: 테마·통합 검증, 대표 증적과 최종 보고

각 단계는 별도 local commit으로 고정한다. #6118과 #6138의 이슈·문서·검증은 분리하지만 원격 push와 PR은
두 작업을 포함한 한 건으로만 수행하며 사용자 승인을 다시 받는다.
