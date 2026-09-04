# PR #2653: render-diff.yml isReviewReferencePath에 .hml 확장자 추가

## 이슈
- **Issue**: #2652 — render-diff.yml 리뷰 전용 경로 판정에서 .hml 누락

## 분석

#2644(codeql.yml), #2646(ci.yml)에서 isReviewReferencePath()에 .hml을
추가했지만 render-diff.yml은 누락되었다. 3개 워크플로우(ci, codeql, render-diff)가
동일한 preflight 로직을 공유하므로 모두 일관되어야 한다.

## 변경
`.github/workflows/render-diff.yml:107`에 `filename.endsWith('.hml')` 조건 추가

## 결과
- 3개 워크플로우 모두 일관성 확보
- HML 샘플 파일만 추가된 PR에서 Render Diff 불필요한 실행 방지
- Closes #2652
