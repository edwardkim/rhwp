# task-m100-3145: 수식 편집기 재사용 시 기호 검색 상태 리셋 누락 수정

## 이슈

#3145 수식 편집기 대화상자 재사용 시 기호 검색어·결과 드롭다운이 초기화되지
않고 남음

## 원인

`rhwp-studio/src/command/commands/insert.ts`는 `EquationEditorDialog`를 모듈
전역 싱글턴(`equationEditorDialog`)으로 생성해 재사용하고, `build()`는 `built`
플래그로 최초 1회만 실행되어 DOM이 인스턴스에 계속 남는다. `open()`은
scriptArea/fontSizeInput/colorInput/입력 모드는 문서 값으로 리셋하지만 기호
검색 입력(`searchInput`)과 결과 드롭다운(`searchResults`)은 건드리지 않고,
`hide()`도 autocomplete 드롭다운만 닫는다. 그 결과 닫았다 다시 열면 이전
검색어와 펼쳐진 결과 목록이 그대로 남았다.

#3037(필드 입력 대화상자 재사용 시 이전 값 유지)과 같은 결함 클래스다.

## 수정

`equation-editor-dialog.ts`의 `open()`에서 `searchInput.value`를 비우고
`searchResults`를 `display: none`으로 닫아 기호 검색 상태를 초기화했다.

## 검증

- `rhwp-studio/tests/equation-editor-search-reset.test.ts` 신규 테스트 추가
  (`field-insert-dialog-reset.test.ts`와 동일한 소스 문자열 검사 패턴)
- 수정 전 red 확인 → 수정 후 `node --test tests/equation-editor-search-reset.test.ts` 통과
