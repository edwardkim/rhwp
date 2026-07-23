# task-m100-3144: 책갈피 대화상자 재사용 시 정렬 상태 불일치 수정

## 이슈

#3144 책갈피 대화상자 재사용 시 정렬 상태 불일치 — 라디오는 '위치'로 초기화되지만
sortMode는 이전 값 유지

## 원인

`rhwp-studio/src/command/commands/insert.ts`는 `BookmarkDialog`를 모듈 전역
싱글턴(`bookmarkDialog`)으로 한 번만 생성해 재사용한다. `show()`는 `build()`로
DOM을 매번 새로 만들어 정렬 라디오가 항상 '위치(P)' 체크 상태로 생성되지만,
인스턴스 필드 `sortMode`는 리셋되지 않아 이전 세션의 `'name'`이 남는다. 이어지는
`refreshList()`가 stale `sortMode`로 정렬해 라디오 표시와 목록 정렬이 어긋난다.

#3037(필드 입력 대화상자 재사용 시 이전 값 유지)과 같은 결함 클래스다.

## 수정

`bookmark-dialog.ts`의 `show()`에서 `refreshList()` 호출 전에
`this.sortMode = 'position'`으로 리셋해 새로 만들어지는 라디오 초기 상태와
일치시켰다.

## 검증

- `rhwp-studio/tests/bookmark-dialog-sort-reset.test.ts` 신규 테스트 추가
  (`field-insert-dialog-reset.test.ts`와 동일한 소스 문자열 검사 패턴)
- 수정 전 red 확인 → 수정 후 `node --test tests/bookmark-dialog-sort-reset.test.ts` 통과
