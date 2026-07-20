# PR #2645: CI 워크플로우 리뷰 전용 경로에 .hml 확장자 추가

## 이슈
- **Issue**: #2642의 연장 — ci.yml, render-diff.yml에도 동일 누락

## 분석

codeql.yml(#2644) 외에도 ci.yml과 render-diff.yml의 `isReviewReferencePath()`
함수에서 .hml이 누락되어 HML 샘플만 추가된 PR이 불필요하게 CI 전체를
트리거한다.

## 변경

- `.github/workflows/ci.yml`: .hml 조건 추가
- `.github/workflows/render-diff.yml`: .hml 조건 추가

## 결과
- 3개 워크플로우(ci, codeql, render-diff) 모두 일관성 확보
- Closes #2642
