# PR #2622: compare-dialog .hml 파일 선택 지원

## 이슈
- **Issue**: #2621 — 파일 선택 시 .hml 파일이 제외됨

## 변경
`rhwp-studio/src/ui/compare-dialog.ts`:
- 파일 선택 `accept` 속성에 `.hml` 추가
- 확장자 검증에 `.hml` 추가
- 에러 메시지에 HML 명시

## 결과
- 스튜디오 IR 비교 기능에서 .hml 파일 선택 가능
- Closes #2621
