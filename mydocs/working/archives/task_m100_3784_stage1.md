# task_m100_3784 Stage 1 결과 — macOS Option+G 찾아가기

- **Issue**: [#3784](https://github.com/edwardkim/rhwp/issues/3784)
- **브랜치**: `fix/3784-macos-option-g-goto`
- **기준**: `upstream/devel` `cc3829116`
- **기록 시각**: 2026-08-02 KST
- **상태**: 구현·focused 검증 완료, PR 생성 승인 대기

## 1. 기준선과 원인

macOS 영문 입력에서 `Option+G`는 `KeyboardEvent.key = '©'`와
`KeyboardEvent.code = 'KeyG'`로 전달된다. 기존 `edit:goto` 매핑은 문자 값 `g`만 검사해
찾아가기를 열지 못했다. 한글 입력의 `ㅎ` 경로는 별도 문자 매핑으로 동작했다.

`matchShortcut()`은 modifier 검증 후 `key`와 `code`를 모두 비교할 수 있었고, 이미
`Option+C`와 장평·자간 단축키가 물리 키 보정을 사용한다. 따라서 입력 처리 순서나 IME 경계를
바꾸지 않고 찾아가기 정의에만 `code: 'KeyG'`를 추가했다.

## 2. 구현

- `rhwp-studio/src/command/shortcut-map.ts`
  - 영문 `Alt+G` 정의를 `key: 'g', code: 'KeyG'`로 보정했다.
  - 한글 `Alt+ㅎ` 정의는 그대로 유지했다.
- `rhwp-studio/tests/shortcut-map.test.ts`
  - macOS 영문 `key: '©', code: 'KeyG', altKey: true`를 `edit:goto`로 고정했다.
  - 기존 영문 `g`, 한글 `ㅎ` 및 다른 물리 키 `KeyH`의 비매칭도 함께 확인했다.

## 3. 검증

| 검증 | 결과 |
| --- | --- |
| `node --test tests/shortcut-map.test.ts` | 6 passed |
| `./node_modules/.bin/tsc --noEmit` | 통과 |
| `npm test` | 721 passed / 0 failed |
| headless Chromium 실제 이벤트 경로 | 통과 |
| `git diff --check` | 통과 |

브라우저 검증에서는 문서를 만든 뒤 편집 영역을 클릭해 입력기를 활성화하고, 활성 입력기에
`key: '©'`, `code: 'KeyG'`, `altKey: true`인 취소 가능한 `keydown` 이벤트를 전달했다.
이벤트는 `preventDefault()` 되었고 찾아가기 대화상자의 제목이 `찾아가기`로 표시됐다.

## 4. 비범위와 다음 단계

- 다른 macOS `Option` 조합의 문자값 보정은 다루지 않았다.
- 실제 macOS 하드웨어의 키보드 레이아웃 검증은 별도 사용자 확인 대상이다. 본 단계는 브라우저가
  전달하는 해당 이벤트 계약과 rhwp-studio의 실제 입력 처리 경로를 검증했다.
- 코드·테스트·문서 커밋 후 PR 제목과 본문을 준비하고, PR 생성은 작업지시자 승인 뒤 수행한다.
