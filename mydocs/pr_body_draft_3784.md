# fix(rhwp-studio): macOS 영문 Option+G 찾아가기 보정 (#3784)

## 요약

macOS 영문 입력에서 `Option+G`가 문자값 `©`로 전달되어 찾아가기 명령이 실행되지 않던 문제를
수정합니다. 문자값 대신 변하지 않는 물리 키 코드 `KeyG`도 매칭합니다.

- 영문 `Alt+G` 정의에 `code: 'KeyG'` 추가
- `©`/`KeyG` macOS 이벤트 회귀 테스트 추가
- 기존 영문 `g`와 한글 `ㅎ` 매핑 유지

## 검증

- `node --test tests/shortcut-map.test.ts` — 6 passed
- `./node_modules/.bin/tsc --noEmit`
- `npm test` — 721 passed
- headless Chromium에서 활성 편집기에 `key='©'`, `code='KeyG'`, `altKey=true` 이벤트를 주입해
  기본 동작 취소와 찾아가기 대화상자 표시 확인
- `git diff --check`

Closes #3784
