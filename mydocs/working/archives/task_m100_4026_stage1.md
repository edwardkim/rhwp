# Task M100 #4026 Stage 1 - 각주 전역 단축키 복구

- 이슈: [#4026](https://github.com/edwardkim/rhwp/issues/4026)
- 브랜치: `fix/issue-4026-footnote-global-shortcuts`
- 기준: `upstream/devel` `4473112d9`
- 기록일: 2026-08-04 KST
- 상태: 구현 및 검증 완료

## 재현

각주가 있는 문서에서 각주 표식 또는 각주 영역을 선택해 각주 편집 모드에 진입한다. macOS에서
`Option+G` 또는 `Cmd+Z`를 누르면 각각 찾아가기와 되돌리기가 실행되지 않는다.

## 원인

`InputHandler.onKeyDown()`은 각주 전용 문자 입력·방향키·Enter·삭제를 처리한 뒤 무조건 반환한다.
따라서 그 뒤에 있는 공통 `Cmd/Ctrl` 및 `Option` 단축키 라우팅까지 도달하지 못한다.

## 구현 계획

1. 서브모드에서 안전한 전역 명령(`undo`, `redo`, `goto`)만 별도 게이트로 dispatcher에 전달한다.
2. 각주·머리말/꼬리말의 문자 입력과 전용 편집 키는 기존 분기에 그대로 둔다.
3. 실제 `footnote-01.hwp`에서 각주 텍스트 입력 뒤 `Cmd+Z` 복원과 `Option+G` 대화상자 표시를 검증한다.

## 수용 기준

- 각주 편집 중 `Cmd+Z`가 방금 입력한 텍스트를 되돌리고 각주 모드를 유지한다.
- 각주 편집 중 `Option+G`가 찾아가기 대화상자를 연다.
- TypeScript 검사와 Studio 단위 테스트가 통과한다.

## 구현

- `input-handler-keyboard.ts`에 서브모드 공통 단축키 게이트를 추가했다. 이 게이트는
  `edit:undo`, `edit:redo`, `edit:goto`만 dispatcher에 전달한다.
- 머리말/꼬리말과 각주 전용 분기 시작점에서 이 게이트를 호출한다. 따라서 기존 문자 입력,
  방향키, Enter, 삭제와 Escape 처리는 기존 순서와 분기를 유지한다.
- `issue-4026-footnote-global-shortcuts.test.mjs`는 실제 `footnote-01.hwp`에서 각주 편집 모드로
  진입해 영문자 입력, `Cmd+Z` 복원, `Option+G` 대화상자 표시를 확인한다.
- E2E 매니페스트에는 새 #4026 테스트와 최신 `devel`에 누락돼 있던 #3953 찾아가기 테스트를 함께
  등록해 파일 목록과 단일 권위 표를 다시 일치시켰다.

## 검증 결과

| 검증 | 결과 |
| --- | --- |
| `git diff --check` | 통과 |
| `cd rhwp-studio && npx tsc --noEmit` | 통과 |
| `issue-4026-footnote-global-shortcuts.test.mjs --mode=headless` | `Cmd+Z` 커서 `1 -> 0`, 각주 모드 유지, `Option+G` 찾아가기 대화상자 표시 통과 |
| `npm run e2e:manifest-check` | `83개 파일 / 83개 행`, 이상 없음 |
| `cd rhwp-studio && npm test` | 763/763 통과 |
