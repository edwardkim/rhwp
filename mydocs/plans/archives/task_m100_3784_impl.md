# 구현계획서 — task_m100_3784

- **이슈**: #3784
- **수행계획서**: `mydocs/plans/task_m100_3784.md`
- **브랜치**: `fix/3784-macos-option-g-goto`
- **절차 상태**: Stage 1·2 완료, PR 생성 승인 대기

## Stage 1 — 단축키 계약 보정

1. `defaultShortcuts`의 `edit:goto` 영문 정의에 `code: 'KeyG'`를 추가한다.
2. 기존 한글 `Alt+ㅎ` 정의는 별도 대체 경로로 그대로 유지한다.
3. `shortcut-map` 단위 테스트에 macOS 영문 `Option+G`의 문자 값 `©`와 물리 키 `KeyG` 조합을 추가한다.

## Stage 2 — focused 검증과 PR 준비

1. Studio 단위 테스트와 TypeScript 검사를 순차 실행한다.
2. 계획 대비 변경·검증 결과를 단계 기록에 남긴다.
3. 코드·테스트·문서를 하나의 일반 커밋으로 고정하고 PR 제목·본문 초안을 준비한다.

## 회귀 경계

- `altKey` 외 `ctrlKey`/`metaKey`가 없는 조합만 찾아가기로 인식한다.
- 문자 값이 `g`인 비-macOS 및 기존 한글 `ㅎ` 경로는 유지한다.
- `KeyH` 등 다른 물리 키는 찾아가기로 매칭하지 않는다.
