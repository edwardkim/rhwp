# PR #????: @rhwp/editor README에 exportHml·getHmlSaveState 문서 추가

## 이슈
- **Issue**: #2691 — README에 exportHml·getHmlSaveState API 문서 누락

## 분석

`npm/editor/index.js`에는 `exportHml()`과 `getHmlSaveState()`가 구현되어 있지만 `npm/editor/README.md`의 API 문서에는 누락되어 있었다. SDK 사용자가 해당 API의 존재를 인지할 수 없었다.

## 변경

`README.md`의 API 섹션에 `exportHwpVerify()` 다음에 두 메서드 추가:

1. `exportHml()` — HML(XML) 내보내기 예제 코드 포함
2. `getHmlSaveState()` — HML 저장 가능 여부 반환

## 결과
- **Branch**: `pr/fix-issue-2691-editor-readme-hml`
- **PR**: https://github.com/edwardkim/rhwp/pull/???? (생성 후 업데이트)
- **Closes**: #2691
