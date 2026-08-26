# 구현계획 — Task M100 #6118 서식 도구 모음 1·2행 압축형 정리

- **상위 수행계획**: [task_m100_6118.md](task_m100_6118.md)
- **이슈**: [#6118](https://github.com/edwardkim/rhwp/issues/6118)
- **작성일**: 2026-08-26 KST
- **작업 브랜치**: `codex/issue-6118-responsive-style-bar`
- **통합 기준**: `upstream/devel@6b5c4f871972380c0866e2a8d27ac2bc67d257e6`
- **구현 상태**: Stage 3·최종 보고 완료, #6138 통합 구현 대기

## 1. 구현 불변식

- viewport가 아니라 실제 콘텐츠 측정으로 정한 두 경계만 사용한다.
- 시각 구조는 전체 1행과 최대 2행 두 개뿐이다.
- `#style-bar`는 항상 화면 전체 폭의 배경·경계를 소유한다.
- 내부 field/command track만 고유 폭으로 제한한다.
- control ID, command 순서, label, title, `aria-*`, active/disabled state를 유지한다.
- 더보기에서도 기존 paragraph button DOM과 command listener를 재사용한다.
- `src/command`, WASM, user settings와 theme 초기화는 변경하지 않는다.
- `#icon-toolbar`, `Ctrl+F1`과 #6138의 한 줄 그룹 스크롤은 변경하지 않는다.

## 2. DOM 구조안

현재 field group과 character/color/paragraph group을 재사용하되 command 행과 paragraph overflow host를
명시한다.

```text
#style-bar
├─ .sb-field-ribbon-group
│  └─ .sb-field-grid
└─ .sb-command-track
   ├─ .sb-character-band
   │  ├─ .sb-character-ribbon-group
   │  └─ .sb-color-ribbon-group
   └─ .sb-overflow-host
      ├─ #btn-style-overflow
      └─ #style-overflow-panel
         └─ .sb-paragraph-band
```

일반 1·2행 구간에서는 `.sb-overflow-host`와 panel을 `display:contents` 계열로 풀어 paragraph band가 기존
순서대로 inline에 참여하고 더보기 버튼은 숨긴다. command 최소 폭 아래에서는 host를 실제 dropdown으로
전환해 paragraph band를 panel 안에 표시한다. DOM을 재복제하거나 runtime reparent하지 않으므로 ID,
listener와 active state의 authority가 하나로 유지된다.

## 3. 파일별 변경안

| 파일 | 예정 변경 | 보존 계약 |
| --- | --- | --- |
| [`index.html`](../../rhwp-studio/index.html) | command track, overflow host/button/panel wrapper 추가 | 기존 control ID·순서·command wiring |
| [`style-bar.css`](../../rhwp-studio/src/styles/style-bar.css) | 1·2행 공통 밀도, track 폭, overflow panel 상태 | field/button/icon/dropdown 기본 형태와 token |
| [`responsive.css`](../../rhwp-studio/src/styles/responsive.css) | 기존 1024/768/767 강제 행을 콘텐츠 경계 규칙으로 교체 | menu/editor/status의 기존 반응형 정책 |
| `src/ui/style-toolbar-overflow.ts` | 더보기 열기/닫기, focus, Escape, 외부 click, active 상태 | command 실행·format state authority |
| [`main.ts`](../../rhwp-studio/src/main.ts) | overflow controller 초기화와 정리만 연결 | editor·toolbar 초기화 순서 |
| [`style-toolbar-grouped-ribbon.test.ts`](../../rhwp-studio/tests/style-toolbar-grouped-ribbon.test.ts) | DOM·더보기·명령 재사용 계약 | icon/dropdown affordance 계약 |
| [`responsive-toolbar-layout.test.ts`](../../rhwp-studio/tests/responsive-toolbar-layout.test.ts) | 1행·2행·최대 2행·intrinsic width 계약 | icon toolbar 기존 계약은 #6138까지 유지 |
| [`responsive.test.mjs`](../../rhwp-studio/e2e/responsive.test.mjs) | 실제 행 수·높이·overflow·더보기·field 조작 | 기존 canvas/menu/status smoke |
| [`e2e/MANIFEST.md`](../../rhwp-studio/e2e/MANIFEST.md) | 기존 항목 설명만 실제 범위와 동기화 | 중복 E2E 파일 추가 없음 |

## 4. CSS 상세 설계

### 4.1 압축 2행을 기본 구조로 사용

초기·제한 폭 구조는 다음 계약을 가진다.

- `#style-bar`: 1열 grid, `align-content:start`, 최대 두 track
- `.sb-field-ribbon-group`: 첫째 행
- `.sb-command-track`: 둘째 행, `display:flex`, `flex-wrap:nowrap`
- `.sb-field-grid`, `.sb-command-track`: `width:max-content`, `max-width:100%`, 왼쪽 정렬
- ribbon caption은 감추고 field label은 2행 구조에서 표시
- 기존 68px `min-height`와 stretch를 해제하고 실제 두 행 높이만 사용

`#style-bar` 자체에 `max-width`를 주지 않는다. 내부 track만 intrinsic width로 제한해 화면 오른쪽의 빈
공간을 유지하되 chrome 배경과 border는 끊기지 않게 한다.

### 4.2 전체 압축 1행

Stage 1에서 고정한 `FULL_ROW_MIN=976px` 이상에 하나의 media/container 경계를 둔다.

- `#style-bar`: flex row, nowrap, 중앙 정렬, 높이 36px 이하
- field group과 command track을 같은 행에 배치
- field label과 ribbon caption을 시각 감춤
- 기존 27px field와 button 밀도를 우선 재사용
- 1행 경계에서는 `scrollWidth <= clientWidth`와 모든 group top 좌표 동일을 검사

`FULL_ROW_MIN`은 1280/1024 같은 device 이름으로 선택하지 않았다. field/character/color/paragraph의
실측 콘텐츠 949.84px, bar padding과 group border 16px, 안전 여백 10.16px을 합한 976px이며 975/976px
E2E를 둔다. 상세 근거는 [Stage 1 보고서](../working/task_m100_6118_stage1.md)를 따른다.

### 4.3 좁은 field 행

지원 최소 375px에서 다음 순서로 폭을 줄인다.

1. 글꼴 field를 `minmax(81px, 1fr)`의 유연 열로 사용
2. style/language track 68/54px과 4px gap은 유지하고 child control을 track 폭에 맞춤
3. size/line-spacing의 현재 cohesive control shell은 유지
4. label은 ellipsis를 허용하되 input의 접근성 이름은 유지

375px의 내부 363px은 `68 + 54 + 81 + 72 + 72 + 16px gaps`로 정확히 맞춘다. 이때
`#style-bar.scrollWidth <= clientWidth`를 통과해야 하며 page 전체에 수평 scrollbar를 만들지 않는다.

### 4.4 command 더보기

Stage 1에서 정한 `COMMAND_INLINE_MIN=460px` 아래에서만 다음을 적용한다.

- character/color group과 더보기 버튼은 둘째 행에 유지
- paragraph group은 `#style-overflow-panel` 안에서 표시
- panel은 기존 paragraph icon button을 한 행 또는 명확한 grid로 표시
- `#btn-style-overflow`는 `aria-expanded`, `aria-controls`, 명확한 title/label을 제공
- panel 내부 active paragraph command는 더보기 버튼의 현재 정렬 아이콘과 접근성 설명에만 반영
- 더보기 버튼 표면은 panel이 열린 동안에만 활성 상태로 표시
- 외부 click·Escape·command 실행 후 닫고 trigger로 focus 복귀
- viewport가 넓어져 paragraph가 inline으로 돌아가면 열린 panel을 닫고 `aria-expanded=false`로 동기화

지원 최소 375px에서 character/color 251px, gap 5px, 더보기 38px, bar padding 12px의 합은 306px이다.
따라서 paragraph group 하나만 overflow 후보로 두는 것으로 충분하며 color group은 inline에 유지한다.

## 5. 접근성·상태 계약

- 숨긴 field label을 `aria-hidden` 처리하지 않고 연결된 form control의 이름을 유지한다.
- 더보기 button은 keyboard로 열 수 있고 panel 첫 command로 focus 이동할 수 있다.
- Tab 순서가 DOM의 기존 command 순서를 따른다.
- panel이 닫힌 동안 내부 command가 Tab 순서에 남지 않는다.
- paragraph active/disabled 상태는 inline과 panel에서 같은 실제 element에 반영된다.
- dropdown 중첩이 없는 paragraph group만 우선 overflow해 기존 color/effect popup과 충돌하지 않게 한다.

## 6. 테스트 설계

### 6.1 정적·단위 계약

- 전체 1행 경계가 1280 같은 기기 이름이 아니라 설명 가능한 content constant임을 확인
- 2행 구조의 field/command track과 `flex-wrap:nowrap` 확인
- paragraph group이 한 번만 존재하고 overflow panel이 같은 DOM을 사용하는지 확인
- 375px field grid의 합산 최소 폭과 `max-width:100%` 확인
- 더보기 open/close, Escape, outside click, focus return, active marker 단위 테스트
- 기존 command ID, dropdown arrow와 alignment mask icon 계약 유지

### 6.2 실제 브라우저 E2E

| viewport | 핵심 판정 |
| --- | --- |
| 1920×1080 / 1280×900 / 1024×768 | 전체 압축 1행, 모든 command inline |
| 975px / 976px | 정확히 2행↔1행 전환, overflow 없음 |
| 883×900 / 768×1024 / 460×900 | field+command 2행, 모든 command inline, track 비확장 |
| 459px / 460px | paragraph 더보기↔inline 전환 |
| 459×900 / 412×915 / 390×844 / 375×812 | 최대 2행, field 무 overflow, 더보기 command 실행 |

각 viewport에서 style bar height, group top 좌표 수, `scrollWidth/clientWidth`, field/command 가시성,
더보기 `aria-expanded`와 paragraph command 실행을 기록한다. default skin은 전 viewport, 1행·2행·더보기
대표 경계에서는 flat/oldschool과 light/dark도 확인한다.

## 7. 검증 명령

구현 뒤 같은 checkout에서 순차 실행한다.

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

E2E가 Vite를 요구하면 [개발 환경 안내](../manual/dev_environment_guide.md)의 표준 WASM·Vite 절차를
사용하고 URL, browser, viewport, DPR을 stage 보고서에 기록한다. Studio chrome 변경이므로 renderer
PDF/SVG visual sweep과 Rust 전체 회귀는 기본 게이트가 아니다.

## 8. 위험과 중단 조건

| 위험 | 완화·중단 조건 |
| --- | --- |
| 전체 1행 경계가 skin/font마다 흔들림 | 가장 넓은 실측과 안전 여백을 사용하고 세 skin 경계 E2E를 둔다. |
| `display:contents`가 focus/popup을 깨뜨림 | wrapper를 유지한 grid/flex 방식으로 전환하고 실제 button DOM은 복제하지 않는다. |
| 더보기에서 현재 정렬 상태가 보이지 않음 | trigger 아이콘과 accessible label에 현재 정렬을 표시하되 닫힌 버튼을 선택 상태로 오인시키지 않는다. |
| 375px field가 맞지 않음 | touch height를 줄이지 않고 style/language gap과 font min-width만 재조정한다. |
| paragraph 외 group까지 숨겨야 함 | 측정 근거를 stage 보고서에 남기고 group 단위 2차 후보만 추가한다. |
| #6115·#6138 충돌 | `#icon-toolbar`와 toolbox visibility 파일을 수정하지 않고 별도 통합 검토로 남긴다. |

## 9. 커밋·보고 단위

1. 재작성 계획 커밋: 이 수행·구현 계획과 오늘 할 일
2. Stage 1 커밋: 기준선·콘텐츠 경계 보고서
3. Stage 2 커밋: DOM/CSS/controller/test/E2E와 구현 보고서
4. Stage 3 커밋: 검증·대표 증적·최종 보고서
5. #6138 후속 커밋: 별도 계획·구현·검증을 유지하되 최종 PR은 #6118과 통합

각 stage를 local commit으로 고정한 뒤 다음 단계로 간다. #6118과 #6138의 이슈·문서·커밋·검증 범위는
분리하지만, 두 작업 완료 뒤 통합 반응형 검증을 한 번 더 수행해 PR 한 건으로 제출한다. remote push와
PR 생성은 그때 사용자에게 별도로 승인받는다.

## 10. 승인 게이트

Stage 3에서 12개 viewport와 세 스킨의 light/dark 전수 검증, 실제 control 조작, 대표 화면과 최종
보고서를 완료했다. #6118 구현은 로컬 완료다. 다음 단계는 #6138을 같은 브랜치의 별도 작업 단위로
진행하는 것이며, 완료 전에는 원격 push와 PR을 수행하지 않는다.
