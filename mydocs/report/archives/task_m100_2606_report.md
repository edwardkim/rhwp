# PR #2607: VSCode .hml 파일 컨텍스트 메뉴 누락 수정

## 이슈
- **Issue**: #2606 — .hml 파일 탐색기/에디터 컨텍스트 메뉴에 rhwp 명령이 표시되지 않음

## 분석
`rhwp-vscode/package.json`에서 `customEditors.selector`는 `*.hml`을 포함하지만,
`menus`의 `when` 조건에는 `.hwp`/`.hwpx`만 있고 `.hml`이 누락됨.

## 변경
모든 `menus[*].when`에 `resourceExtname == .hml` 조건 추가 (총 6곳)

## 결과
- `.hml` 파일에서도 탐색기/에디터 컨텍스트 메뉴 명령이 올바르게 표시됨
- **PR**: https://github.com/edwardkim/rhwp/pull/2607
- **Closes**: #2606
