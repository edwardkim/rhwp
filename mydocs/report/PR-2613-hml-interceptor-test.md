# PR #2613: 다운로드 인터셉터 .hml 테스트 추가

## 이슈
- **Issue**: #2612 — .hml 파일명/URL 감지 테스트 누락

## 변경
`rhwp-shared/sw/download-interceptor-common.test.js`:
- hml 파일명 감지 테스트 추가
- hml URL 감지 테스트 추가 (쿼리 문자열 포함)
- 대소문자 무관 감지에 .HML 추가
- 파일명 일부 hml 미감지(false positive 방지) 추가

## 결과
- `node --test rhwp-shared/sw/download-interceptor-common.test.js` 통과
- Closes #2612
