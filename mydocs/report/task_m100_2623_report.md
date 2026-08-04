# PR #2624: recovery-format.ts baseNameWithoutKnownExtension()에 .hml 추가

## 이슈
- **Issue**: #2623 — baseNameWithoutKnownExtension()이 .hml 확장자 미인식

## 변경
`rhwp-studio/src/recovery/recovery-format.ts:9`: `.hml` 조건 추가

## 결과
- HML 문서 복구 파일명에서 확장자가 올바르게 제거됨
- Closes #2623
