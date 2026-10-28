# PR #2620: url-validator.js hasHwpExtension()에 .hml 추가

## 이슈
- **Issue**: #2619 — hasHwpExtension()이 .hml을 누락하여 HML URL이 차단됨

## 변경
`rhwp-shared/security/url-validator.js`:
- `hasHwpExtension()`에 `.hml` 확장자 검사 추가
- 관련 주석 갱신

## 결과
- `.hml` 파일 URL이 확장 URL 검증을 통과
- Closes #2619
