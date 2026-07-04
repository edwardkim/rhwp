## 요약

Closes #1476

- rhwp-studio 상단 메뉴 단축키 표시를 `CommandRegistry.shortcutLabel` 기준으로 동기화했습니다.
- macOS에서는 긴 `Command`/`Option` 텍스트 대신 Apple 기호 `⌘`, `⌥`, `⇧`로 표시하고, Windows/Linux 계열에서는 기존 `Ctrl`/`Alt` 표기를 유지합니다.
- 보기/서식 메뉴에 하드코딩 단축키는 있었지만 registry 단축키가 빠져 있던 항목을 보강하고, 누락이 다시 생기지 않도록 회귀 테스트를 추가했습니다.
- PR 리뷰 workflow 문서에 merge 후 `upstream/task_m100_1470` 같은 원격 PR head 브랜치 삭제 절차를 추가했습니다.

## 주요 변경

- 메뉴 단축키 표시 동기화
  - `MenuBar`가 `CommandRegistry`를 받아 초기화 시 `.md-item[data-cmd]`의 `.md-shortcut`을 갱신합니다.
  - `index.html`에 남아 있는 하드코딩 단축키보다 `CommandDef.shortcutLabel`을 우선합니다.
  - `.md-shortcut`이 없는 항목도 registry 값이 있으면 표시 요소를 생성합니다.
- macOS 표시 정책
  - `Ctrl+S` -> `⌘S`
  - `Alt+N` -> `⌥N`
  - `Ctrl+Shift+S` -> `⌘⇧S`
  - 기타 플랫폼에서는 `Ctrl+S`, `Alt+N` 같은 기존 표시를 유지합니다.
- 누락 shortcutLabel 보강
  - 보기 메뉴: `쪽 맞춤`, `폭 맞춤`, `100%`, `문단 부호`, `투명 선`
  - 서식 메뉴: 글자 크기, 정렬, 줄 간격 계열
- 회귀 테스트
  - macOS/기타 플랫폼 표시 변환을 focused 테스트로 고정했습니다.
  - 상단 메뉴 하드코딩 단축키와 registry `shortcutLabel` 누락 항목을 소스 기반 테스트로 고정했습니다.

## 수동 검증 절차

1. GitHub Pages 배포본 접속
   - `https://jangster77.github.io/rhwp/`
   - 브라우저 캐시가 있으면 강제 새로고침한다.

2. macOS 표시 검증
   - 파일 메뉴를 연다.
   - 기대값:
     - `새로 만들기`: `⌥N`
     - `저장`: `⌘S`
     - `다른 이름으로 저장(A)...`: `⌘⇧S`
     - `인쇄`: `⌘P`
   - 보기/서식/쪽/표 메뉴도 연다.
   - 기대값:
     - `Ctrl`, `Alt`, `Command`, `Option` 긴 텍스트가 남지 않고 `⌘`, `⌥`, `⇧` 기호로 표시된다.

3. Windows 표시 검증
   - Windows 브라우저에서 같은 URL을 연다.
   - 파일 메뉴를 연다.
   - 기대값:
     - `저장`: `Ctrl+S`
     - `다른 이름으로 저장(A)...`: `Ctrl+Shift+S`
     - `새로 만들기`: `Alt+N`
   - 실제 단축키 동작은 기존 shortcut-map 경로를 유지한다.

## 검증

- `cd rhwp-studio && node --test tests/menu-shortcut-labels.test.ts` (6 passed)
- `cd rhwp-studio && npx tsc --noEmit`
- `cd rhwp-studio && npm test` (116 passed)
- `git diff --check`
- Browser/IAB 검증
  - URL: `http://localhost:7700/`
  - title: `rhwp-studio`
  - console error/warn 없음
  - 상단 메뉴 수: 8
  - 단축키 표시 항목: 63
  - `Ctrl`, `Alt`, `Command`, `Option` 텍스트 잔여: 0
  - 파일 메뉴 클릭 후 `⌥N`, `⌘S`, `⌘⇧S`, `⌘P` 표시 확인
- GitHub Pages 배포 검증
  - run: `https://github.com/jangster77/rhwp/actions/runs/27946414455`
  - Build/Deploy 성공
  - `https://jangster77.github.io/rhwp/` HTTP 200 확인
- 작업지시자 시각 검증 완료
