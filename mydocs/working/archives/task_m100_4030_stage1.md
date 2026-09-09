# Task M100 #4030 Stage 1 - 각주에서 대형 문서 찾아가기 전 본문 전환

- 이슈: [#4030](https://github.com/edwardkim/rhwp/issues/4030)
- 브랜치: `fix/issue-4030-footnote-goto-transition`
- 기준: `upstream/devel` `9aa0ec8b6`
- 기록일: 2026-08-04 KST
- 상태: 구현 및 상세 검증 완료

## 재현

`samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`는
219쪽이며, 본문 문단 216의 각주 1번을 편집한 뒤 `Option+G`로 200쪽을 입력하면 대화상자는 닫힌다.
그러나 각주 편집 모드가 남아 상태 표시줄은 `10 / 219 쪽`으로 표시되고, 대상 쪽의 본문 caret 상태가
완료되지 않는다.

## 조사 결과

대상 200쪽의 본문 위치는 문단 2191로 계산되고 `GotoDialog`는 해당 위치로 이동을 시도한다. 다만
`Cursor`가 각주 모드인 상태에서는 `getRect()`와 caret 갱신이 각주 전용 위치를 계속 사용한다. 따라서
본문 위치와 viewport는 바뀌어도 마지막 상태 표시와 caret 갱신이 원 각주가 있는 10쪽 결과로 덮인다.

같은 각주를 명시적으로 종료한 뒤 같은 200쪽 이동을 실행한 제어 실험에서는 `200 / 219 쪽`,
`scrollTop=225242`, 본문 모드가 확인됐다.

## 구현 계획

1. `InputHandler`에 문서 본문 탐색을 위한 각주 모드 종료 API를 추가하고 기존 `footnoteModeChanged`
   이벤트 계약을 유지한다.
2. `GotoDialog`가 화면 이동에 성공한 뒤 본문 cursor를 배치하기 전에 위 API를 호출한다. 화면 이동 자체가
   실패하면 기존 각주 편집 상태를 보존한다.
3. 실제 219쪽 HWP의 각주 1번에서 `Option+G → 200`을 수행하는 E2E를 추가해 모드 종료, 상태 표시줄,
   viewport, 본문 cursor, 다음 `Option+G` 재호출을 함께 검증한다.

## 수용 기준

- 각주 편집 중 `Option+G → 200`이 본문 모드로 전환되고 상태 표시줄에 `200 / 219 쪽`을 표시한다.
- 대상 쪽의 본문 cursor가 배치되고 viewport가 실제 대상 위치까지 이동한다.
- 이동 완료 후 `Option+G`를 다시 열고 닫을 수 있다.
- TypeScript, Studio 단위 테스트, 새 headless E2E, E2E 매니페스트 검사가 통과한다.

## 구현

- `InputHandler.exitFootnoteModeForBodyNavigation()`을 추가했다. 각주 모드일 때만 cursor의 저장된
  본문 위치를 복원하고 기존 `footnoteModeChanged=false` 이벤트를 발행한다. 문서 내용을 바꾸지 않으므로
  history나 dirty 상태는 변경하지 않는다.
- `GotoDialog`는 페이지 화면 이동이 성공한 뒤, 본문 cursor를 배치하기 전에 이 API를 호출한다. 따라서
  화면 이동 실패 경로는 각주 편집 상태를 그대로 보존한다.
- `issue-4030-footnote-goto-transition.test.mjs`와 E2E 매니페스트를 추가했다. 테스트는 실제 219쪽 HWP의
  각주 1번(문단 216)에서 시작하고, 임의로 만든 문서나 고정 canvas 좌표에 의존하지 않는다.

## 검증 결과

### 수정 전 및 제어 실험

| 경로 | 관찰 결과 |
| --- | --- |
| 수정 전 각주 1번에서 `Option+G → 200` | 본문 cursor는 문단 2191로 바뀌었지만 각주 모드가 남아 상태 표시줄이 `10 / 219 쪽`, `scrollTop=10000`으로 남음 |
| 같은 각주를 명시 종료한 제어 경로 | `200 / 219 쪽`, `scrollTop=225242`, 본문 cursor 문단 2191, 각주 모드 종료 |

### 수정 후 자동 검증

| 검증 | 결과 |
| --- | --- |
| `git diff --check` | 통과 |
| `cd rhwp-studio && npx tsc --noEmit` | 통과 |
| `cd rhwp-studio && npm run e2e:manifest-check` | `84개 파일 / 84개 행`, 이상 없음 |
| `issue-4030-footnote-goto-transition.test.mjs --mode=headless` | 실제 219쪽 HWP의 각주 1번에서 200쪽 이동, 각주 모드 종료, `200 / 219 쪽`, viewport `225242 / 225377.5`, 문단 2191 cursor, 재호출 통과 |
| `issue-3953-large-document-goto.test.mjs --mode=headless` | 158쪽 이동, 상태 표시줄 진입, 잘못된 입력 재입력 통과 |
| `issue-4026-footnote-global-shortcuts.test.mjs --mode=headless` | 각주 `Cmd+Z` 및 `Option+G` 대화상자 표시 회귀 없음 |
| `cd rhwp-studio && npm test` | 763/763 통과 |
