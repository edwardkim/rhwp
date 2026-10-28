# Task M100 #1476 stage1 완료 보고서

- 이슈: #1476 rhwp-studio 단축키 표시를 플랫폼별로 분기
- 브랜치: `task_m100_1476`
- 작성일: 2026-06-22
- 상태: 구현, focused 검증, IAB 검증, Pages 배포, 작업지시자 시각 검증 완료.

## 1. 구현 요약

상단 메뉴 단축키 표시를 `CommandRegistry`의 `CommandDef.shortcutLabel`과 `formatShortcutLabel()` 경로로
동기화했다. macOS에서는 `Ctrl`/`Alt`/`Shift` 기반 표시가 `⌘`/`⌥`/`⇧` 기호로 바뀌고,
Windows/Linux 계열에서는 기존 `Ctrl`/`Alt`/`Shift` 표시가 유지된다.

## 2. 변경 내용

### 2.1 메뉴 단축키 표시 동기화

- `rhwp-studio/src/engine/navigation-keymap.ts`
  - macOS 단축키 표시를 `Command`/`Option` 텍스트가 아니라 Apple 기호 `⌘`/`⌥`/`⇧`로 변환하도록 보정했다.
  - 예: `Ctrl+S` → `⌘S`, `Alt+N` → `⌥N`, `Ctrl+Shift+S` → `⌘⇧S`
- `rhwp-studio/src/ui/menu-shortcut-labels.ts` 추가
  - `.md-item[data-cmd]`를 순회한다.
  - `CommandRegistry`에서 같은 command id의 `shortcutLabel`을 조회한다.
  - `.md-shortcut`이 있으면 표시 문자열을 갱신하고, 없으면 새로 만든다.
  - 표시 문자열은 `formatShortcutLabel()`을 통과시킨다.
- `rhwp-studio/src/ui/menu-bar.ts`
  - `CommandRegistry`를 생성자에서 받는다.
  - 초기화 시 `syncMenuShortcutLabels()`를 호출한다.
- `rhwp-studio/src/main.ts`
  - `MenuBar` 생성 시 `registry`를 함께 전달한다.
- `rhwp-studio/src/command/commands/view.ts`, `rhwp-studio/src/command/commands/format.ts`
  - 상단 메뉴에는 하드코딩 단축키가 있었지만 `CommandDef.shortcutLabel`이 빠져 있던 보기/서식 항목을 보강했다.
  - `쪽 맞춤`, `폭 맞춤`, `100%`, `문단 부호`, `투명 선`, 글자 크기, 정렬, 줄 간격 계열이 같은 registry 동기화 경로를 타도록 했다.

### 2.2 회귀 테스트

- `rhwp-studio/tests/menu-shortcut-labels.test.ts` 추가
  - macOS override에서 `Ctrl+S`가 `⌘S`로 표시되는지 검증
  - macOS override에서 `Alt+N`이 `⌥N`으로 표시되는지 검증
  - 기타 플랫폼 override에서 `Ctrl+S`와 `Alt+N`이 유지되는지 검증
  - `index.html` 하드코딩 값보다 `CommandRegistry.shortcutLabel`이 우선되는지 검증
  - `.md-shortcut`이 없는 항목도 registry 값으로 생성되는지 검증
  - 상단 메뉴에 하드코딩 단축키가 있는 보기/서식 항목의 `CommandDef.shortcutLabel` 누락을 소스 기반으로 고정

### 2.3 PR workflow 문서 보강

- `mydocs/manual/pr_review_workflow.md`
  - merge 후 `upstream/task_m100_1470` 같은 원격 PR head 작업 브랜치도 삭제하도록 절차를 보강했다.
  - 로컬/원격 추적 브랜치 잔존 확인 명령을 추가했다.

## 3. IAB 검증

build-web-apps frontend-testing-debugging 절차와 Browser/IAB를 사용해 `http://localhost:7700/`에서 확인했다.

- Page identity
  - URL: `http://localhost:7700/`
  - title: `rhwp-studio`
- Blank page check
  - `#menu-bar`와 `#scroll-container` 존재 확인
- Framework overlay
  - Vite/Next 계열 오류 overlay 없음
- Console health
  - error/warn 로그 없음
- Interaction proof
  - 파일 메뉴를 클릭해 dropdown open 상태 확인
  - 파일 메뉴 표시 결과:
    - `새로 만들기`: `⌥N`
    - `저장`: `⌘S`
    - `다른 이름으로 저장(A)...`: `⌘⇧S`
    - `편집 용지`: `F7`
    - `인쇄`: `⌘P`
  - 상단 메뉴 전체 audit 결과:
    - 메뉴 수: 8
    - 단축키 표시 항목: 63
    - `Ctrl`, `Alt`, `Command`, `Option` 텍스트 잔여: 0
    - 대표 확인값:
      - 편집: `모양 복사=⌥C`, `문서 비교=⌥⇧V`, `찾아가기(G)=⌥G`
      - 보기: `쪽 맞춤=⌘G,P`, `폭 맞춤=⌘G,W`, `투명 선=⌥V,T`
      - 입력: `수식=⌘M,M`, `문자표=⌥F10`
      - 서식: `글자 크기 크게=⌥⇧E`, `왼쪽 정렬=⌘⇧L`, `줄 간격 늘림=⌥⇧Z`
      - 쪽: `쪽 나누기=⌘Enter`, `다단 설정=⌘⌥Enter`
      - 표: `왼쪽에 칸 추가하기=⌥Insert`, `블록 합계=⌘⇧S`

## 4. 검증 결과

```bash
cd rhwp-studio && node --test tests/menu-shortcut-labels.test.ts
cd rhwp-studio && npx tsc --noEmit
cd rhwp-studio && npm test
git diff --check
```

결과:

- `node --test tests/menu-shortcut-labels.test.ts`
  - 통과: 6 passed
- `npx tsc --noEmit`
  - 통과
- `npm test`
  - 통과: 116 passed
- `git diff --check`
  - 통과

## 5. 남은 작업

- PR 본문 확인 후 PR 생성
- 필요 시 전체 PR 검증 명령을 별도 승인 후 수행
