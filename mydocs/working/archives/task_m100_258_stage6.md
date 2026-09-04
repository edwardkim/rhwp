# Task M100-258 Stage 6 — 누름틀 대화상자 바깥 클릭 닫힘 방지

- 이슈: https://github.com/edwardkim/rhwp/issues/258
- 브랜치: `local/task_m100_258`
- 작성일: 2026-06-15
- 선행 커밋: `20b4918b` (`task 258: 누름틀 저장 라운드트립 검증`)

## 1. 문제

누름틀 `필드 입력` 대화상자에서 모달 박스 바깥의 회색 overlay를 클릭하면 대화상자가
바로 닫힌다. 한컴 데스크톱 대화상자처럼 명시적인 닫기/취소/확인 버튼으로만 닫히는
동작이 필요하다.

## 2. 수정 범위

- `ModalDialog`에 overlay 클릭 닫기 여부를 선택할 수 있는 옵션을 추가한다.
- `FieldInsertDialog`와 `FieldEditDialog`는 overlay 클릭으로 닫히지 않게 한다.
- 기존 다른 대화상자의 동작은 유지한다.

## 3. 검증 계획

- `npm run build`
- `git diff --check`

## 4. 진행 기록

- `ModalDialog` 생성자에 `closeOnOverlayClick` 옵션을 추가했다.
- 누름틀 `FieldInsertDialog`, `FieldEditDialog`는 overlay 클릭으로 닫히지 않게 설정했다.
- 다른 대화상자는 기존 기본값을 유지한다.

## 5. 검증 결과

- `npm run build` 통과
- `git diff --check` 통과
