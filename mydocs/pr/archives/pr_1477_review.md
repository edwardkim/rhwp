# PR #1477 리뷰 기록 - 플랫폼별 메뉴 단축키 표시 보정

- PR: https://github.com/edwardkim/rhwp/pull/1477
- 작성일: 2026-06-22
- 작성자: collaborator self-merge 후보 경로
- 문서 작성 시점 참고 head: `0031c92f09afb0b04e20dc62b25526d2e6a50258`
- base: `devel`
- head: `jangster77:task_m100_1476`

## 1. PR 메타

| 항목 | 확인 내용 |
|------|-----------|
| 작성자 | `jangster77` |
| PR 상태 | open, draft 아님 |
| merge 상태 | 문서 작성 시점 `MERGEABLE` |
| 관련 이슈 | `Closes #1476` |
| 규모 | 문서 작성 시점 13 files, +650 / -16 |
| 커밋 수 | 2개 + 본 self-merge review 문서 커밋 예정 |

`draft`, `mergeable`, `head SHA`, `CI 상태`는 변하는 값이므로 이 문서는 작성 시점 값을 참고로만 기록한다.
최종 merge 판단은 merge 직전 최신 PR head 기준으로 다시 확인한다.

## 2. 변경 범위

### 2.1 상단 메뉴 단축키 표시 동기화

- `MenuBar`가 `CommandRegistry`를 주입받도록 변경했다.
- 초기화 시 `.md-item[data-cmd]`를 순회해 같은 command id의 `CommandDef.shortcutLabel`을 조회한다.
- `.md-shortcut`이 있으면 registry 값을 기준으로 갱신하고, 없으면 새로 생성한다.
- `index.html` 하드코딩 단축키보다 command registry 값을 우선하도록 했다.

### 2.2 macOS Apple 기호 표시 정책

- `formatShortcutLabel()`의 macOS 표시를 긴 `Command`/`Option` 텍스트 대신 Apple 기호로 바꿨다.
- 대표 변환:
  - `Ctrl+S` -> `⌘S`
  - `Alt+N` -> `⌥N`
  - `Ctrl+Shift+S` -> `⌘⇧S`
- Windows/Linux 계열에서는 기존 `Ctrl`/`Alt` 표기를 유지한다.

### 2.3 보기/서식 메뉴 shortcutLabel 보강

- 상단 메뉴에는 단축키가 하드코딩되어 있었지만 registry `shortcutLabel`이 빠져 있던 항목을 보강했다.
- 보기 메뉴: `쪽 맞춤`, `폭 맞춤`, `100%`, `문단 부호`, `투명 선`
- 서식 메뉴: 글자 크기, 정렬, 줄 간격 계열

### 2.4 PR workflow 문서 보강

- `mydocs/manual/pr_review_workflow.md`에 merge 후 원격 PR head 브랜치 삭제 절차를 추가했다.
- `upstream/task_m100_1470` 같은 collaborator self-merge 작업 브랜치를 merge 후 정리하는 명령과 확인 절차를 기록했다.

## 3. 리스크

| 리스크 | 판단 |
|--------|------|
| 상단 메뉴와 command registry 표시 불일치 | 신규 `menu-shortcut-labels.test.ts`에서 registry 우선 동기화와 `.md-shortcut` 생성 경로를 고정했다. |
| macOS 표시가 다시 긴 텍스트로 회귀 | `navigation-keymap.test.ts`와 focused 메뉴 테스트에서 `⌘`, `⌥`, `⇧` 표시를 고정했다. |
| 보기/서식 메뉴 일부 항목의 단축키 누락 | 소스 기반 테스트로 하드코딩 메뉴와 registry `shortcutLabel` 누락 항목을 고정했다. |
| 실제 UI 표시와 테스트 간 괴리 | IAB에서 전체 상단 메뉴를 열어 8개 메뉴, 63개 단축키 항목, `Ctrl`/`Alt`/`Command`/`Option` 텍스트 잔여 0개를 확인했다. |
| Windows 표시 회귀 | 테스트 override에서 기타 플랫폼은 `Ctrl+S`, `Alt+N` 표시를 유지하는지 확인했다. 작업지시자가 Windows 시각 검증을 완료했다. |

## 4. 검증

프론트 전용 로컬 검증:

```bash
cd rhwp-studio && node --test tests/menu-shortcut-labels.test.ts
cd rhwp-studio && npx tsc --noEmit
cd rhwp-studio && npm test
git diff --check
```

검증 결과:

- `node --test tests/menu-shortcut-labels.test.ts`: 6 passed
- `npx tsc --noEmit`: pass
- `npm test`: 116 passed
- `git diff --check`: pass

이번 PR의 소스 변경은 `rhwp-studio` 프론트와 문서에 한정되므로, 작업지시자 지시에 따라 cargo/clippy 전체 회귀는 수행하지 않았다.

IAB 검증:

- URL: `http://localhost:7700/`
- title: `rhwp-studio`
- console error/warn 없음
- 상단 메뉴 수: 8
- 단축키 표시 항목: 63
- `Ctrl`, `Alt`, `Command`, `Option` 텍스트 잔여: 0
- 파일 메뉴 클릭 후 `새로 만들기=⌥N`, `저장=⌘S`, `다른 이름으로 저장(A)...=⌘⇧S`, `인쇄=⌘P` 표시 확인

GitHub Pages / 작업지시자 시각 검증:

- Pages URL: `https://jangster77.github.io/rhwp/`
- Pages run: `https://github.com/jangster77/rhwp/actions/runs/27946414455`
- Build/Deploy 성공
- 작업지시자가 Windows 환경에서 시각 검증 완료

GitHub Actions 작성 시점 참고값:

- Build & Test: in progress
- Analyze (javascript-typescript): pass
- Canvas visual diff: pass
- Analyze (python): pass
- Analyze (rust): in progress
- WASM Build: skipped
- CodeQL: neutral

본 review 문서 커밋 push 후 GitHub Actions가 다시 실행되므로, merge 전 최신 head 기준으로 위 상태를 재확인한다.

## 5. 판단

작성 시점 기준으로 #1476의 요구 사항인 플랫폼별 상단 메뉴 단축키 표시 분기와 macOS 기호 표시 정책이 PR 범위에 포함되어 있다.

최종 조건:

1. 본 review 문서 2건과 오늘할일 문서가 PR head에 포함된다.
2. push 후 최신 PR head 기준 GitHub Actions가 통과한다.
3. 작업지시자 승인 상태가 유지된다.

위 조건 충족 시 collaborator self-merge 후보로 merge 수용한다.
