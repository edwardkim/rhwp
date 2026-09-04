# Task M100 #3953 Stage 1 - 대형 문서 찾아가기 복구

- 이슈: [#3953](https://github.com/edwardkim/rhwp/issues/3953)
- 브랜치: `fix/issue-page-goto-large-document`
- 기준: `upstream/devel` `cf5d462dc`
- 기록일: 2026-08-04 KST
- 상태: 구현 및 재현 검증 완료

## 재현

`samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`를
rhwp-studio에서 연다. macOS 영문 입력 상태에서 `Option+G`로 찾아가기를 열고 100쪽 이상을
입력하면 대상 쪽으로 이동하지 않는다. 이후 같은 단축키로 대화상자를 다시 열 수 없다고 보고됐다.

## 원인 가설

`getPositionOfPage()`는 쪽 첫 `PageItem`의 상위 문단 위치를 반환한다. 첫 항목이 표이면
`InputHandler.moveCursorTo()`가 해당 위치의 커서 사각형을 찾지 못해 false를 반환할 수 있다.
현재 페이지 찾아가기는 반환값을 확인하지 않고 닫지만, 책갈피 찾아가기는 인접 문단 fallback을
이미 사용한다.

## 구현 계획

1. 페이지 찾아가기에도 상위 문단 직접 이동 뒤 인접 문단 탐색 fallback을 적용한다.
2. 직접 이동과 fallback이 모두 실패하면 오류를 표시하고 모달을 유지한다.
3. 상태 표시줄 `현재 쪽 / 전체 쪽`을 버튼으로 만들어 `edit:goto` 커맨드를 dispatch한다.
4. 실제 219쪽 HWP에서 158쪽 이동, 실패 재입력, 상태 표시줄 클릭 진입을 headless E2E로
   검증한다.

## 수용 기준

- 158쪽 대상에서 커서와 스크롤 위치가 대상 쪽으로 이동한다.
- 이동 실패는 모달을 닫거나 키보드 capture handler를 남기지 않는다.
- 상태 표시줄 클릭과 `Option+G`가 같은 찾아가기 대화상자를 연다.

## 구현

- `CanvasView.gotoPage()`를 추가해 커서 배치와 무관하게 전역 쪽의 화면 offset으로 이동한다.
- `GotoDialog`는 화면 이동 뒤 첫 문단 직접 이동과 ±5 인접 문단 fallback을 시도한다. 화면 이동 자체가
  불가능하거나 입력기가 없으면 오류를 표시하고 모달을 유지한다.
- 모달 종료 후 `InputHandler.focus()`를 호출해 숨겨진 textarea로 포커스를 복구한다. 이로써 Escape나
  실패 뒤에도 다음 `Option+G`가 단축키 핸들러에 도달한다.
- 상태 표시줄의 쪽 표시는 키보드 접근 가능한 버튼이며 `edit:goto`를 dispatch한다.

## 검증 결과

| 검증 | 결과 |
| --- | --- |
| `git diff --check` | 통과 |
| `npx tsc --noEmit` | 통과 |
| `npm test` | 759/759 통과 |
| `issue-3953-large-document-goto.test.mjs` | 실제 HWP 219쪽 로드, 158쪽 이동·상태표시줄 클릭·잘못된 입력 뒤 `Option+G` 재호출 통과 |
| 158쪽 직접 probe | `158 / 219 쪽`, scrollTop `178015`, cursor `(section=0, paragraph=1724)`, `getPageOfPosition=157`(0-base) |
