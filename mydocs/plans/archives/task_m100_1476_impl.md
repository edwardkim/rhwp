# Task M100 #1476 구현 계획서

- 이슈: #1476
- 작업 브랜치: `task_m100_1476`
- 작성일: 2026-06-22
- 상태: Stage 1 구현/검증 완료. 커밋 전 작업지시자 확인 대기

## 1. 구현 목표

상단 메뉴 단축키 표시를 `CommandRegistry`의 `CommandDef.shortcutLabel`과 `formatShortcutLabel()` 경로로
통일한다. macOS에서는 `Ctrl+S` 계열 표시가 `⌘S`로, `Alt+N` 계열 표시가 `⌥N`으로 보이고,
Windows/Linux 계열에서는 기존 `Ctrl+S`/`Alt+N` 표시가 유지되어야 한다.

## 2. 변경 파일

### 2.1 `rhwp-studio/src/ui/menu-bar.ts`

- `CommandRegistry` 타입을 import한다.
- 생성자에 `registry: CommandRegistry`를 추가한다.
- 생성자 초기화 단계에서 `syncShortcutLabels()`를 호출한다.
- `syncShortcutLabels()`는 다음을 수행한다.
  - `.md-item[data-cmd]`를 순회한다.
  - `registry.get(cmdId)?.shortcutLabel`을 조회한다.
  - `shortcutLabel`이 있으면 `.md-shortcut` 요소를 찾거나 생성한다.
  - `.md-shortcut.textContent`에 `formatShortcutLabel(shortcutLabel)`을 넣는다.
  - 기존 하드코딩 label과 registry label이 다르면 registry 값을 우선한다.

### 2.2 `rhwp-studio/src/main.ts`

- `new MenuBar(..., dispatcher)` 호출을 `new MenuBar(..., dispatcher, registry)` 형태로 바꾼다.

### 2.3 `rhwp-studio/tests/menu-shortcut-labels.test.ts`

- `CommandRegistry`와 `syncMenuShortcutLabels()`를 조합한 최소 DOM 테스트를 추가한다.
- 테스트용 메뉴 항목/컨테이너 stub을 만든다.
- macOS override:
  - `globalThis.__rhwpTestPlatformKind = 'mac'`
  - `file:save` registry shortcutLabel은 `Ctrl+S`
  - DOM 초기 값이 `Ctrl+S`여도 생성 후 `⌘S`가 되는지 확인한다.
  - `file:new-doc` registry shortcutLabel은 `Alt+N`
  - DOM 초기 값이 `Alt+N`이면 생성 후 `⌥N`이 되는지 확인한다.
- 기타 플랫폼 override:
  - `globalThis.__rhwpTestPlatformKind = 'other'`
  - 생성 후 `Ctrl+S`/`Alt+N`이 유지되는지 확인한다.
- registry 우선 검증:
  - DOM 초기 값이 `Ctrl+OLD`이고 registry 값이 `Ctrl+Shift+S`이면 macOS에서 `⌘⇧S`가 되는지 확인한다.

### 2.4 `mydocs/manual/pr_review_workflow.md`

- 7.6 후속 처리 절차에 원격 PR head 브랜치 삭제를 명시한다.
- `upstream/task_m100_1470` 같은 브랜치를 merge 후 삭제하는 예시를 포함한다.

## 3. 테스트 명령

구현 직후:

```bash
cd rhwp-studio && npx tsc --noEmit
cd rhwp-studio && npm test
git diff --check
```

필요 시 focused 테스트:

```bash
cd rhwp-studio && node --test tests/menu-shortcut-labels.test.ts
```

브라우저 검증:

```bash
cd rhwp-studio && ./node_modules/.bin/vite --host 0.0.0.0 --port 7700 --force
```

- Browser/IAB에서 `http://localhost:7700/` 접속
- 파일 메뉴를 열어 `새로 만들기 ⌥N`, `저장 ⌘S`, `다른 이름으로 저장 ⌘⇧S`, `인쇄 ⌘P` 표시 확인

## 4. 커밋 계획

Stage 1:

- 상단 메뉴 단축키 표시 동기화 구현
- focused 테스트 추가
- `pr_review_workflow.md` 원격 브랜치 삭제 절차 포함
- 검증 후 `mydocs/working/task_m100_1476_stage1.md` 작성
- 작업지시자 확인 후 커밋

커밋 제목 후보:

```text
task 1476: 플랫폼별 메뉴 단축키 표시 보정
```

## 5. 리스크와 대응

- `MenuBar` 생성자 인자 변경으로 다른 생성 경로가 깨질 수 있다.
  - `rg "new MenuBar"`로 호출부를 확인하고 `tsc`로 고정한다.
- 하드코딩 단축키와 registry 단축키가 다른 항목이 있을 수 있다.
  - registry 값을 원천으로 삼아 커맨드 팔레트/컨텍스트 메뉴와 상단 메뉴 표시를 맞춘다.
- macOS 표시만 바꾸고 실제 키 입력 동작은 바꾸지 않는다.
  - 기존 `shortcut-map.ts`와 keyboard handler 테스트를 유지한다.
