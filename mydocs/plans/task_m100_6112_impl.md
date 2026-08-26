# 구현 계획 — Task M100 #6112

- **이슈**: [#6112](https://github.com/edwardkim/rhwp/issues/6112)
- **수행 계획**: [`task_m100_6112.md`](task_m100_6112.md)
- **브랜치**: `codex/issue-6112-toolbox-collapse`
- **작성일**: 2026-08-26 KST
- **문서 성격**: 로컬 구현 초안 `d707d4cf2`의 사후 설계 재구성

## 설계 원칙

새 토글 상태나 별도 이벤트 경로를 만들지 않는다. 모든 사용자 입력은 기존 커맨드로 수렴하고,
#5738이 마련한 설정과 DOM 동기화 경로가 상태의 단일 원천이 된다.

```text
우측 버튼 / 보기 메뉴 / Ctrl+F1
              ↓
      view:toolbox-basic
              ↓
 userSettings.setToolbarBasic()
              ↓
      syncToolboxMenu()
              ↓
 applyToolboxVisibility()
      ├─ root data-toolbox-basic
      ├─ 메뉴 active + aria-checked
      └─ 버튼 active + aria-expanded + label/title
```

## 파일별 변경

### 단일 커맨드와 단축키

- `rhwp-studio/src/command/commands/view.ts`
  - 기존 `view:toolbox-basic`에 `shortcutLabel: 'Ctrl+F1'`만 추가한다.
- `rhwp-studio/src/command/shortcut-map.ts`
  - `{ key: 'f1', ctrl: true }`를 같은 커맨드에 연결한다.
  - 기존 `ctrlOrMeta` 판정으로 Windows/Linux Ctrl과 macOS Command를 함께 지원한다.
- `rhwp-studio/src/main.ts`
  - 편집 textarea 밖에서도 동작하는 전역 보기 단축키 집합에 도구 상자 커맨드를 포함한다.
  - textarea 안에서는 InputHandler가 기존 shortcut map을 소유하므로 이중 실행하지 않는다.

### 우측 버튼

- `rhwp-studio/index.html`
  - `#menu-bar` 말단에 `data-cmd="view:toolbox-basic"` 직접 버튼을 추가한다.
  - `aria-controls="icon-toolbar"`로 제어 대상을 명시한다.
- `rhwp-studio/src/ui/menu-bar.ts`
  - 이벤트 위임 대상을 드롭다운 `.md-item[data-cmd]`와 직접 `.menu-command[data-cmd]`로 제한한다.
  - 두 표면 모두 기존 `CommandDispatcher`와 동일한 파라미터 수집을 사용한다.
- `rhwp-studio/src/styles/menu-bar.css`
  - 메뉴바 오른쪽 정렬, hover/focus 상태는 기존 디자인 토큰을 사용한다.
  - 접기/펴기 표시는 CSS border 화살표로 그려 새 이미지 자산을 만들지 않는다.

### 기본값·복원·접근성

- `rhwp-studio/src/core/user-settings.ts`
  - `toolbarBasic` 신규·미설정 기본값을 `false`로 바꾼다.
  - `toolbarFormat` 기본값 `true`는 유지한다.
- `rhwp-studio/public/theme-init.js`
  - 번들 전 기본 상태도 `toolbarBasic=false`로 맞춘다.
  - 저장값이 명시적으로 `true`일 때만 펼쳐, 기존 사용자 선택을 보존한다.
- `rhwp-studio/src/view/toolbox-visibility.ts`
  - 메뉴 항목은 기존 `aria-checked` 계약을 유지한다.
  - `aria-controls`가 대상 도구 상자를 가리키는 직접 버튼만 `aria-expanded`, 상태별 이름과 툴팁을
    갱신한다.

## 실패 계약과 테스트

1. `Ctrl/Command+F1`은 `view:toolbox-basic`에 매핑되고 단독 F1은 매핑되지 않는다.
2. 신규 기본값은 기본 도구 상자 `false`, 서식 도구 상자 `true`다.
3. 메뉴와 버튼은 같은 표시 상태를 서로 다른 ARIA 계약으로 표현한다.
4. 정적 마크업, dispatcher 위임, 커맨드 단축키 표기와 전역 경로가 함께 존재한다.
5. FOUC 초기화 스크립트의 기본값과 앱 설정 기본값이 일치한다.
6. 실제 브라우저에서 물리 버튼 클릭과 실제 Ctrl+F1 입력, 저장·리로드를 순서대로 검증한다.

## 위험과 완화

| 위험 | 완화 |
| --- | --- |
| 기존 사용자의 펼침 선택이 초기화됨 | 명시적 `toolbarBasic: true`를 초기 스크립트와 설정 정규화 모두에서 보존 |
| 버튼과 보기 메뉴 상태 불일치 | `applyToolboxVisibility`가 같은 `data-cmd` 표면을 한 번에 갱신 |
| textarea/버튼 포커스에 따라 단축키 무동작 | InputHandler와 전역 보기 단축키가 같은 shortcut map을 사용 |
| 버튼에 잘못된 `aria-checked` 부여 | `aria-controls`로 직접 버튼을 구분해 `aria-expanded`만 적용 |
| 새 아이콘 자산의 테마 불일치 | CSS 화살표와 기존 색상 토큰만 사용 |

## 비구현 사항

한글 2024의 소형 상태와 rhwp의 `style-bar` 높이를 완전히 같게 만드는 재배치는 이 이슈에 포함하지
않는다. #6112는 큰 아이콘 기본 도구 상자에 대한 접기/펴기 진입점과 기본 정책만 담당한다.
