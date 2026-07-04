# Task M100-258 Stage 8 — 누름틀 입력 후 커서 이탈과 삭제 확인

- 이슈: https://github.com/edwardkim/rhwp/issues/258
- 브랜치: `local/task_m100_258`
- 작성일: 2026-06-15
- 선행 커밋: `bac4f266` (`task 258: 누름틀 삽입 직후 안내문 표시`)

## 1. 문제

한컴의 누름틀은 사용자가 안내문에 값을 입력한 뒤 커서를 누름틀 밖으로 이동할 수 있고,
누름틀 자체를 지우려 할 때 `[누름틀]을 지울까요?` 확인 대화상자를 띄운다.

현재 rhwp-studio에서는 누름틀 내부 입력 뒤 커서가 바깥으로 자연스럽게 빠지지 않아 이후
본문 입력이 막히고, 누름틀 삭제 확인 흐름도 한컴 동작과 다르다.

## 2. 조사 범위

- ClickHere 활성 필드와 커서 위치 갱신 로직
- Backspace/Delete가 필드 경계에서 어떻게 차단되는지
- 기존 `field:remove` 명령과 확인 대화상자 재사용 가능성
- 양식 모드 보호 정책과 일반 편집 모드의 누름틀 삭제 정책 분리

## 3. 수정 방향

- 누름틀 내부 끝 위치에서 오른쪽 방향 입력/이동이 본문 다음 위치로 이어지도록 한다.
- 누름틀 경계에서 Backspace/Delete로 누름틀 자체를 지우려 할 때 확인 대화상자를 표시한다.
- 확인 취소 시 문서는 변경하지 않는다.
- 한컴 동작 비교를 위해 삽입/입력/삭제 흐름을 Stage8에서 별도 검증한다.

## 4. 검증 계획

- `npm run build`
- `cargo test --test issue_258_clickhere_form_mode`
- `git diff --check`

## 5. 진행 기록

- `Backspace`/`Delete`가 ClickHere 경계에 걸릴 때 단순 차단하지 않고
  `지우기` 확인 대화상자를 띄우도록 했다.
- 컨텍스트 메뉴의 `누름틀 지우기`도 같은 확인 대화상자를 거치도록 통일했다.
- 누름틀 끝에서 오른쪽 방향키를 누르면 같은 `charOffset`이라도 필드 밖 경계 상태로
  취급하고 active field를 해제하도록 했다.
- Rust text insertion은 active field가 아닌 상태에서 비어 있지 않은 ClickHere 끝 위치에
  입력한 글자를 field range 안으로 흡수하지 않도록 보정했다.
- 빈 ClickHere 첫 입력은 기존처럼 field range 값으로 들어가게 유지했다.
- `issue_258_clickhere_form_mode`에 active/inactive field end 삽입 회귀 테스트를 추가했다.

## 6. 검증 결과

- `cargo fmt` 통과
- `cargo test --test issue_258_clickhere_form_mode` 통과
- `npm run build` 통과
- `git diff --check` 통과
