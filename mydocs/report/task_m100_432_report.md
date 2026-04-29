---
타스크: #432 브라우저 확장 실제 HWP/HWPX 응답 검증 및 파일 URL 해석 계층 도입
브랜치: local/task432
작성일: 2026-04-29
이슈: https://github.com/edwardkim/rhwp/issues/432
---

# 최종 결과 보고서

## 1. 문제 정의

rhwp-firefox 테스트 중 GitHub wiki 페이지의 `saved/pr360-edward.hwp` 링크 카드에서 뷰어를 열면 다음 오류가 발생했다.

```text
파일 로드 실패: 유효하지 않은 파일: CFB 오류
```

같은 파일을 GitHub blob 원본 페이지로 이동한 뒤 raw 경로 기반으로 열면 정상 로드되었다.

분석 결과 문제는 특정 파일 자체가 아니라 다음 일반 문제였다.

- 링크 URL은 `.hwp`로 끝나지만 실제 fetch 응답은 HWP 바이트가 아닐 수 있다.
- GitHub blob URL은 브라우저 표시/미리보기 페이지이며 실제 파일 바이트 URL은 raw URL이다.
- viewer가 응답 바이트를 검증하지 않고 곧바로 HWP 파서에 넘기면 HTML 응답이 CFB 오류로 보인다.

## 2. 해결 방향

이번 타스크는 특정 GitHub wiki 링크 하나만 예외 처리하지 않고 두 계층으로 문제를 분리했다.

| 계층 | 역할 |
|---|---|
| URL 해석 계층 | provider별 파일 상세/미리보기 URL을 실제 파일 바이트 URL로 변환 |
| 응답 검증 계층 | 파서 호출 전 실제 HWP/HWPX 바이트인지 확인 |

GitHub는 첫 provider adapter로 구현했으며, 이후 다른 서비스의 파일 상세 페이지도 같은 resolver 구조에 추가할 수 있다.

## 3. 구현 요약

### Stage 1

- `rhwp-shared/sw/document-url-resolver.js` 추가
- GitHub blob HWP/HWPX URL을 raw URL로 변환
- Chrome/Firefox 확장에서 공통 resolver를 symlink로 참조
- 순수 함수 테스트 추가

### Stage 2

- Chrome/Firefox `viewer-launcher.js`에 resolver 적용
- Chrome/Firefox `thumbnail-extractor.js`에 resolver 적용
- 썸네일 cache key는 원본 URL로 유지하고 fetch 대상만 resolved URL로 변경

### Stage 3

- `rhwp-studio/src/main.ts`의 `loadFromUrlParam()`에 원격 응답 바이트 검증 추가
- 직접 fetch와 service worker `fetch-file` fallback 모두 검증
- HTML/오류 페이지 응답은 파서 호출 전에 차단
- 사용자에게 CFB 내부 오류 대신 원인 중심 메시지 표시

### Stage 4

- resolver 테스트, 기존 다운로드 감지 테스트 통과 확인
- Firefox/Chrome 확장 빌드 통과 확인
- dist 산출물에 resolver와 viewer 응답 검증이 포함됨을 확인

## 4. 변경 파일

| 파일 | 주요 변경 |
|---|---|
| `rhwp-shared/sw/document-url-resolver.js` | 공통 문서 URL resolver 추가 |
| `rhwp-shared/sw/document-url-resolver.test.js` | resolver 순수 함수 테스트 추가 |
| `rhwp-chrome/sw/document-url-resolver.js` | shared resolver symlink |
| `rhwp-firefox/sw/document-url-resolver.js` | shared resolver symlink |
| `rhwp-chrome/sw/viewer-launcher.js` | viewer URL 파라미터에 resolver 적용 |
| `rhwp-firefox/sw/viewer-launcher.js` | viewer URL 파라미터에 resolver 적용 |
| `rhwp-chrome/sw/thumbnail-extractor.js` | 썸네일 fetch 전에 resolver 적용 |
| `rhwp-firefox/sw/thumbnail-extractor.js` | 썸네일 fetch 전에 resolver 적용 |
| `rhwp-studio/src/main.ts` | 원격 응답 바이트 검증과 오류 메시지 개선 |
| `mydocs/working/task_m100_432_stage*.md` | 단계별 완료 보고서 |
| `mydocs/orders/20260429.md` | 오늘 할일 상태 갱신 |

## 5. 검증 결과

```bash
node --test rhwp-shared/sw/document-url-resolver.test.js
```

- 통과: 12개

```bash
node --test rhwp-shared/sw/download-interceptor-common.test.js
```

- 통과: 26개

```bash
cd rhwp-firefox
npm run build
```

- 통과

```bash
cd rhwp-chrome
npm run build
```

- 통과

dist 산출물에서 다음도 확인했다.

- `document-url-resolver.js`가 symlink가 아닌 일반 파일로 복사됨
- viewer/thumbnail 모듈이 resolver를 import함
- GitHub blob URL이 raw URL로 변환됨
- viewer asset에 원격 응답 검증 오류 문구가 포함됨

## 6. 남은 확인 사항

실제 Firefox GUI에서 확장을 로드한 뒤 GitHub wiki 카드 hover/click까지 진행하는 수동 검증은 현재 Codex 세션에서 수행하지 않았다.

작업지시자가 로컬 Firefox에서 확인할 시나리오는 다음과 같다.

1. `rhwp-firefox/dist`를 임시 확장으로 로드한다.
2. GitHub wiki의 `saved/pr360-edward.hwp` 카드에서 썸네일이 표시되는지 확인한다.
3. 카드의 `rhwp로 열기`를 클릭한다.
4. viewer가 raw URL을 받아 문서를 정상 로드하는지 확인한다.
5. 의도적으로 HTML 응답 URL을 viewer `?url=`에 넣었을 때 CFB 오류 대신 원인 중심 오류가 표시되는지 확인한다.

## 7. 결론

#432는 현재 보고된 GitHub wiki 링크 하나만을 위한 분기 처리가 아니라, 파일 URL 해석과 응답 바이트 검증이라는 일반 계층으로 해결했다.

Chrome/Firefox 확장 빌드는 통과했고, dist 산출물에도 변경이 반영되었다. 작업지시자 승인 후 타스크 완료 처리와 `local/devel` 병합 절차로 이동할 수 있다.
