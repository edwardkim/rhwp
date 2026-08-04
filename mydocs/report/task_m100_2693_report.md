# PR #????: hwpctl Open() 커서 위치 초기화

## 이슈
- **Issue**: #2693 — Open()이 커서 위치를 초기화하지 않음

## 분석
`Clear()`는 `createBlankDocument()` 후 커서를 (0,0,0)으로 초기화하지만, `Open()`은 `wasmDoc` 교체만 하고 cursor 필드를 초기화하지 않아 이전 문서의 커서 위치가 잔류하는 불일치가 있었다.

## 변경
`Open()` 성공 경로에 `cursorSection = cursorPara = cursorPos = 0` 추가 (Clear와 동일).

## 결과
- **Branch**: `pr/fix-issue-2693-hwpctl-open-cursor`
- **PR**: https://github.com/edwardkim/rhwp/pull/2694
- **Closes**: #2693
