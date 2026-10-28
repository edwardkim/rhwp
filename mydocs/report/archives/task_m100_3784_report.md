# task_m100_3784 처리결과 보고서 — macOS Option+G 찾아가기

- **Issue**: [#3784](https://github.com/edwardkim/rhwp/issues/3784)
- **브랜치**: `fix/3784-macos-option-g-goto`
- **상태**: PR 준비 완료, 생성 승인 대기

## 결과

macOS 영문 입력의 `Option+G`가 `©` 문자값으로 전달되더라도, 변하지 않는 물리 키 코드 `KeyG`로
`edit:goto`를 찾도록 보정했다. 한글 입력의 `Alt+ㅎ`와 일반 `Alt+G` 매핑은 유지된다.

## 검증

- shortcut-map 회귀 테스트 6건 통과
- Studio 전체 단위 테스트 721건 통과
- TypeScript `--noEmit` 통과
- headless Chromium에서 실제 활성 편집 입력기로 `©`/`KeyG` 이벤트를 전달해 찾아가기 대화상자와
  기본 동작 취소를 확인
- `git diff --check` 통과

## PR 준비 상태

PR 제목과 본문 초안은 `mydocs/pr_body_draft_3784.md`에 준비했다. 이슈와 구현·검증 기록은 같은
커밋으로 고정하며, 원격 push와 PR 생성은 별도 승인 뒤 수행한다.
