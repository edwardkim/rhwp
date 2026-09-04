# task-m100-3037: 필드 입력 대화상자 재사용 시 값 리셋 누락 수정

## 이슈

#3037 필드 입력 대화상자가 재사용 시 이전 값을 유지함 — 메모/이름/편집가능 체크박스 초기화 누락

## 원인

`rhwp-studio/src/command/commands/insert.ts`는 `FieldInsertDialog`를 모듈 전역
싱글턴(`fieldInsertDialog`)으로 한 번만 생성해 재사용한다. 그런데
`FieldInsertDialog.show()` 오버라이드는 안내문 입력(guideInput)에 포커스·선택만
수행할 뿐, memoInput·nameInput·editableCheckbox 값을 초기 상태로 되돌리지 않았다.
그 결과 사용자가 필드를 삽입한 뒤 대화상자를 다시 열면 이전에 입력했던 메모·이름과
체크박스 상태가 그대로 남아있었다.

## 수정

`field-insert-dialog.ts`의 `show()`에서 guideInput 기본값 설정과 함께
memoInput/nameInput을 빈 문자열로, editableCheckbox를 `true`로 리셋하도록 추가했다.

## 검증

- `rhwp-studio/tests/field-insert-dialog-reset.test.ts` 신규 테스트 추가
  (`table-create-dialog.test.ts`와 동일한 소스 문자열 검사 패턴)
- `npx tsx --test tests/field-insert-dialog-reset.test.ts` 통과 확인
