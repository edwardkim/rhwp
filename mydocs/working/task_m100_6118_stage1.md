# Stage 1 처리 결과 — #6118 서식 도구 모음 콘텐츠 경계 고정

- **이슈**: [#6118](https://github.com/edwardkim/rhwp/issues/6118)
- **기준**: `upstream/devel@6b5c4f871972380c0866e2a8d27ac2bc67d257e6`
- **작업 브랜치**: `codex/issue-6118-responsive-style-bar`
- **계측일**: 2026-08-26 KST
- **결론**: `FULL_ROW_MIN=976px`, `COMMAND_INLINE_MIN=460px`, `FIELD_MIN=375px`

## 1. 범위와 재현 조건

Stage 1은 제품 source를 수정하지 않고 현행 `#style-bar`의 행 수, 높이, 실제 콘텐츠 폭과 overflow만
계측했다. 로컬 Vite URL `http://127.0.0.1:7717/`, DPR 2 환경에서 viewport를 1920px부터 375px까지
변경하며 `getBoundingClientRect()`, `clientWidth`, `scrollWidth`를 기록했다.

실행 중인 checkout은 #6115 검증용이지만 현재 브랜치와 `style-bar.css`, `responsive.css`가 byte-identical이고
`index.html` 차이는 `#style-bar` 밖의 `#toolbox-basic-toggle` 한 줄뿐이다. 따라서 서식 바 DOM과 geometry는
현재 #6118 기준선과 동일하다.

## 2. 현행 반응형 기준선

| viewport | 행 수 | `#style-bar` 높이 | 실제 배치 | overflow |
| ---: | ---: | ---: | --- | --- |
| 1920, 1400, 1280, 1100, 1040, 1024 | 1 | 69px | field 508.84 + character 289 + paragraph 212px | 없음 |
| 1023, 1000, 900, 883, 768 | 2 | 86px | field 500px / command 500px, 나머지 폭은 비움 | 없음 |
| 767, 600, 500, 460, 459, 412, 390 | 3 | 123px | field / character+color / paragraph 강제 분리 | 없음 |
| 375 | 3 | 123px | field grid 378px이 내부 가용 폭 363px보다 큼 | root 9px 넘침 |

관찰된 두 점이 회귀의 원인을 확정한다.

- 1024→1023px 전환은 콘텐츠가 아니라 `@media (max-width:1023px)` 때문에 발생한다. 1023px에서 실제
  두 행 콘텐츠는 오른쪽 506px까지만 사용한다.
- 768→767px 전환도 콘텐츠가 아니라 `flex-direction:column`과 paragraph `width:100%` 때문에 발생한다.
  command 전체를 한 행에 놓는 데 필요한 폭은 안전 여백을 포함해 460px뿐이다.

## 3. 고정한 콘텐츠 경계

### 3.1 전체 압축 1행: `FULL_ROW_MIN=976px`

현행 데스크톱에서 label/caption을 제외한 실제 콘텐츠 union은 다음과 같다.

| 구역 | 실측 폭 |
| --- | ---: |
| field grid | 493.84px |
| character controls | 174px |
| color controls | 85px |
| paragraph controls | 197px |
| **콘텐츠 합계** | **949.84px** |

전체 바 좌우 padding 12px과 그룹 경계 4px을 더하면 965.84px이다. subpixel 반올림과 skin 경계의
안전 여백 10.16px을 더해 전환점을 976px로 고정한다. flat/oldschool skin은 `#style-bar`의 색과 border
표현만 바꾸고 control box geometry를 재정의하지 않으므로 같은 경계를 사용한다.

Stage 2의 전체 1행 모드는 ribbon의 불필요한 수평 padding을 제거하고 이 976px 예산 안에서 모든 field와
command를 한 줄에 유지해야 한다. 검증점은 975px(2행)과 976px(1행)이다.

### 3.2 command 한 행: `COMMAND_INLINE_MIN=460px`

모바일 밀도에서 character+color의 실제 span은 251px, paragraph 여섯 버튼은 184px이다. track gap 5px과
바 좌우 padding 12px을 합하면 452px이며 8px 안전 여백을 포함한 460px부터 paragraph를 inline에 둔다.

459px 이하에서는 paragraph group만 더보기로 이동한다. character+color 251px, gap 5px, 현재
arrow-button과 같은 38px 더보기, 바 padding 12px의 합은 306px이다. 지원 최소 375px보다 69px 작으므로
color group을 추가로 숨기지 않아도 최대 2행 계약을 지킬 수 있다.

### 3.3 field 지원 최소: `FIELD_MIN=375px`

현행 375px viewport에서 바 내부 가용 폭은 363px이지만 field grid의 고정 최소는 378px이다.

```text
현행: 68 + 54 + 96 + 72 + 72 + (4 × 4 gap) = 378px
목표: 68 + 54 + 81 + 72 + 72 + (4 × 4 gap) = 363px
```

따라서 Stage 2에서는 글꼴 열만 `minmax(81px, 1fr)`로 낮추고 각 field control을 track 폭에 맞춘다.
size/line-spacing shell과 27px 입력 높이는 유지한다. 이 계약은 viewport 375px에서 bar와 root 모두
`scrollWidth <= clientWidth`여야 한다.

## 4. 보존할 DOM·동작 계약

더보기는 paragraph 요소를 복제하지 않고 같은 DOM authority를 inline 또는 panel로 보여준다.

| 구역 | 그대로 보존할 항목 |
| --- | --- |
| field | `style-name`, `font-lang`, `font-name`, `font-size`, `btn-size-up`, `btn-size-down`, `linespacing-select`, `btn-ls-up`, `btn-ls-down` |
| character | `btn-bold`, `btn-italic`, `btn-underline`, `btn-strike`, `btn-charfx`; `emboss`, `engrave`, `outline`, `superscript`, `subscript` data-format |
| color | `btn-text-color`, `text-color-picker`, `btn-highlight`, 기존 palette·indicator DOM |
| paragraph | `btn-align-left`, `btn-align-center`, `btn-align-right`, `btn-align-justify`, `btn-align-distribute`, `btn-align-split` |
| 접근성·상태 | 연결된 label, title, `aria-*`, DOM tab 순서, focus, active/disabled class, 기존 event listener |

새로 추가할 더보기 trigger/panel만 새 ID를 가지며, 기존 paragraph button ID와 command binding은 바꾸지 않는다.

## 5. Stage 1 종료 판정

- [x] 현행 1920~375px 행 수·높이·overflow를 동일 방식으로 계측했다.
- [x] 전체 1행 전환점을 976px로 고정했다.
- [x] paragraph 더보기 전환점을 460px로 고정했다.
- [x] 375px field grid의 15px 축소 지점을 글꼴 열로 한정했다.
- [x] 기존 field/command/accessibility/state 보존 목록을 확정했다.
- [x] `rhwp-studio` 제품 source·test·E2E 변경은 0건이다.

Stage 1은 완료했다. 다음 단계는 사용자 승인 뒤 DOM wrapper, 1·2행 CSS, paragraph 더보기 controller와
계약 테스트를 함께 구현하는 Stage 2다.
