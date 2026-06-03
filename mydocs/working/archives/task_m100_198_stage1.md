---
타스크: #198 rhwp-chrome 다운로드 인터셉터 부작용 수정
단계: Stage 1 — 인터셉터 수정 + 단위 테스트
브랜치: local/task198
작성일: 2026-04-19
선행: 구현계획서 task_m100_198_impl.md (승인됨)
---

# Stage 1 단계별 완료 보고서

## 1. 목표

`shouldInterceptDownload(item)` 순수 함수 추출 + 리스너 분기 명확화 + 단위 테스트.

## 2. 산출물

### 2.1 수정 파일

[rhwp-chrome/sw/download-interceptor.js](rhwp-chrome/sw/download-interceptor.js):

- `HWP_EXTENSION_RE` 정규식 + `HWP_MIME_HINTS` 상수 신설
- `shouldInterceptDownload(item)` 순수 함수 export
- 리스너에서 `if (shouldInterceptDownload) { handleHwpDownload + suggest }` 분기 명확화
- HWP 가 아니면 `suggest` 호출 자체를 안 함 → Chrome 마지막 위치 기억 보존

### 2.2 신규 파일

[rhwp-chrome/sw/download-interceptor.test.js](rhwp-chrome/sw/download-interceptor.test.js): Node `--test` 러너 기반 20개 케이스.

## 3. 검증 결과

```
node --test rhwp-chrome/sw/download-interceptor.test.js

# tests 20
# pass 20
# fail 0
# duration_ms 81.4
```

### 3.1 테스트 분류

| 분류 | 개수 | 케이스 |
|---|---:|---|
| HWP 감지 (filename) | 3 | hwp/hwpx/대소문자 |
| HWP 감지 (URL/finalUrl) | 4 | URL/쿼리/redirect/finalUrl |
| HWP 감지 (mime) | 4 | haansoft/x-hwp/hwp+zip/대소문자 |
| 미감지 (false positive 방지) | 7 | 이미지/PDF/zip/오인 패턴/빈/null/빈mime |
| 다중 신호 조합 | 2 | filename+mime / filename+URL |

## 4. 정체성 셀프 체크

- [x] 최소 침습 — 단일 sw 모듈 + 신규 테스트 파일 1개
- [x] 분기 명확화 — `if HWP { suggest } else { skip }`
- [x] HWP 감지 다중화 — filename/URL/finalUrl/mime
- [x] 회귀 우선 — `handleHwpDownload` 시그니처 변경 없음, 기존 로직 보존
- [x] 작업지시자 결정 Q2 준수 — Chrome API mock 없이 순수 함수만 테스트

## 5. 다음 단계

Stage 2: dist 빌드.

`npm run build` 또는 `node build.mjs` 로 `rhwp-chrome/dist/` 갱신.

## 6. 승인 요청

본 단계 완료 보고서 승인 후 Stage 2 착수.
