# Task M100 #1487 Stage 4

- 이슈: #1487 확장 옵션에 외부 웹폰트 사용 안 함 설정 추가
- PR: #1490
- 브랜치: `task_m100_1487`
- 작성일: 2026-06-23
- 상태: 구현/검증 완료

## 배경

PR CodeQL 집계 체크에서 테스트 코드의 URL 부분 문자열 검사에 대해
`Incomplete URL substring sanitization` 경고가 발생했다.

문제 지점은 제품 코드가 아니라 `font-loader-offline-mode.test.ts`의 assertion이며,
`includes('https://cdn.jsdelivr.net/')`로 CDN URL 포함 여부를 확인한 것이 원인이다.

## 구현 방향

- 제품 코드는 변경하지 않는다.
- 테스트 assertion에서 URL 부분 문자열 검사를 제거한다.
- CSS/FontFace source의 `url(...)` 값을 추출한 뒤 `URL` 객체로 파싱한다.
- CDN 여부는 `protocol`과 `hostname`을 정확히 비교해 판단한다.

## 검증 계획

```bash
cd rhwp-studio && npm test
cd rhwp-studio && npm run build
git diff --check
```

## 구현 내용

- 테스트 코드의 `includes('https://cdn.jsdelivr.net/')` 기반 URL 부분 문자열 검사를 제거했다.
- CSS/FontFace source의 `url(...)` 값을 추출하는 helper를 추가했다.
- 추출한 URL은 `new URL(...)`로 파싱하고 `protocol`, `hostname`을 비교해 CDN/외부 URL 여부를 판단하도록 변경했다.
- 제품 코드와 런타임 동작은 변경하지 않았다.

## 검증 결과

- `cd rhwp-studio && npm test`: 통과 (130 passed)
- `cd rhwp-studio && npm run build`: 통과
  - 기존과 동일한 CanvasKit browser compatibility/chunk size 경고만 출력됨
- `git diff --check`: 통과
