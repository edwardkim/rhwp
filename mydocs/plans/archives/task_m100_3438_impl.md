# 구현 계획서 — Task M100-3438: 스타일 저장 반환 계약 정리

- 이슈: https://github.com/edwardkim/rhwp/issues/3438
- 수행 계획서: `mydocs/plans/task_m100_3438.md`
- 작성일: 2026-08-13
- 브랜치: `local/task_m100_3438`

## 1. 설계

`apply` 콜백의 반환값을 `boolean`으로 바꾼다.

- 새 스타일 생성은 ID를 곧바로 `updateStyleShapes()`에 전달하고 성공을 반환한다.
- 기존 스타일 수정에서 `updateStyle()`가 `false`이면 `false`를 반환한다.
- 입력 핸들러가 있는 경로는 `false`를 `null`로 바꿔 `SnapshotCommand`의 no-op 계약을 사용한다.
- 모양 수정이 요청됐는데 `updateStyleShapes()`가 `false`이면 throw한다. 이는 동일 ID로 앞선
  수정이 완료된 뒤의 계약 위반이므로 조용한 무변경으로 숨기지 않는다.

## 2. 수정 파일

- `rhwp-studio/src/ui/style-edit-dialog.ts`
- `rhwp-studio/tests/style-undo-routing.test.ts`

필요하면 공용 no-op 계약을 이미 검증하는
`rhwp-studio/tests/undo-noop-skip.test.ts`에 style API 계약 가드를 추가한다.

## 3. 단계

1. `apply`의 반환형과 snapshot operation을 실제 bool 계약에 맞춘다.
2. 생성 음수 ID 가드를 제거하고 업데이트 bool을 검사한다.
3. 테스트를 갱신해 존재하지 않는 실패 신호를 다시 도입하지 못하게 한다.
4. TypeScript·focused test·전체 Studio test와 브라우저 스모크를 수행한다.

## 4. 완료 기준

- 불가능한 `newId >= 0` 가드가 없다.
- `updateStyle()` 실패는 undo 스택에 snapshot을 남기지 않는다.
- `updateStyleShapes()`의 실패를 무시하지 않는다.
- Studio 기본 검증과 실제 브라우저 스타일 저장 동작이 통과한다.

## 5. 승인 요청

위 설계대로 구현을 시작한다. 승인 전에는 소스 코드를 수정하지 않는다.
