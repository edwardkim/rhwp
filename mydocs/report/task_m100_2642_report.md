# PR #2644: codeql.yml isReviewReferencePath에 .hml 확장자 추가

## 이슈
- **Issue**: #2642 — codeql.yml의 리뷰 전용 경로 판정에서 .hml 누락

## 분석

codeql.yml의 preflight 단계에서 isReviewReferencePath() 함수가
HWP/HWPX 샘플 파일만 리뷰 전용으로 인식한다. HML 샘플 파일이
추가된 PR은 불필요하게 전체 CodeQL 분석을 트리거한다.

## 변경

`.github/workflows/codeql.yml:97`에 `filename.endsWith('.hml')` 조건 추가

## 결과
- HML 샘플 파일만 추가된 PR의 CI 효율화
- 기존 HWP/HWPX/PDF/PNG 경로에 영향 없음
- Closes #2642
