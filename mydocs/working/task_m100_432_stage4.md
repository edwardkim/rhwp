---
타스크: #432 브라우저 확장 실제 HWP/HWPX 응답 검증 및 파일 URL 해석 계층 도입
단계: Stage 4 — 빌드/검증 + 보고서
브랜치: local/task432
작성일: 2026-04-29
---

# Stage 4 완료 보고서

## 1. 작업 요약

Stage 1~3에서 구현한 URL 해석 계층과 원격 응답 바이트 검증이 Chrome/Firefox 확장 빌드 산출물에 포함되는지 확인했다.

이번 단계에서는 추가 소스 구현 없이 자동 테스트, 확장 빌드, dist 구조 확인, 빌드 산출물 문자열 확인을 수행했다.

## 2. 검증 결과

### 순수 함수 테스트

```bash
node --test rhwp-shared/sw/document-url-resolver.test.js
```

- 결과: 통과
- 테스트: 12개 통과
- 확인 범위:
  - GitHub blob HWP/HWPX URL raw 변환
  - raw URL/일반 URL 기존 동작 유지
  - query 문자열에만 `.hwp`가 있는 위장 URL 미변환
  - malformed URL 안전 처리

```bash
node --test rhwp-shared/sw/download-interceptor-common.test.js
```

- 결과: 통과
- 테스트: 26개 통과
- 확인 범위:
  - 기존 다운로드 감지 규칙 회귀 없음
  - HWP/HWPX 파일명, URL, MIME 감지 유지
  - DEXT5 차단 규칙 유지

### 확장 빌드

```bash
cd rhwp-firefox
npm run build
```

- 결과: 통과
- 출력: `rhwp-firefox/dist`

```bash
cd rhwp-chrome
npm run build
```

- 결과: 통과
- 출력: `rhwp-chrome/dist`

Vite 빌드 중 다음 경고가 표시되었으나 빌드는 완료되었다.

- `/images/icon_small_ko.svg`는 빌드 시점에 해석되지 않고 런타임 경로로 유지됨
- viewer chunk가 500KB를 초과한다는 기존 크기 경고

## 3. dist 산출물 확인

### resolver 복사 상태

다음 파일이 각 확장 dist에 일반 파일로 복사되었다.

- `rhwp-firefox/dist/sw/document-url-resolver.js`
- `rhwp-chrome/dist/sw/document-url-resolver.js`

`test -L` 확인 결과 둘 다 symlink가 아니며, `build.mjs`의 `dereference: true` 복사가 의도대로 동작했다.

### viewer/thumbnail 연결 상태

각 dist의 service worker 모듈에서 다음 연결을 확인했다.

- `viewer-launcher.js`가 `resolveDocumentUrl()`을 import한다.
- viewer URL의 `url` 파라미터에 `resolveDocumentUrl(options.url)` 결과를 넣는다.
- `thumbnail-extractor.js`가 fetch 전에 `resolveDocumentUrl(url)`을 적용한다.

### GitHub blob URL 변환 확인

각 dist의 resolver를 Node ESM으로 import해 동일한 입력을 확인했다.

입력:

```text
https://github.com/edwardkim/rhwp/blob/devel/saved/pr360-edward.hwp
```

출력:

```text
https://raw.githubusercontent.com/edwardkim/rhwp/devel/saved/pr360-edward.hwp
```

Firefox dist와 Chrome dist 모두 같은 결과를 반환했다.

### viewer 응답 검증 포함 확인

빌드된 viewer asset에서 Stage 3 오류 문구가 포함됨을 확인했다.

```text
실제 HWP/HWPX 파일이 아닙니다. 파일 미리보기/오류 페이지가 반환되었습니다.
```

따라서 HTML 미리보기/오류 응답은 dist viewer에서도 파서 호출 전에 차단된다.

## 4. 수동 GUI 검증 상태

현재 Codex 세션에서는 Firefox 확장을 실제 브라우저에 로드한 뒤 GitHub wiki 카드 클릭까지 진행하는 GUI 수동 검증은 수행하지 않았다.

대신 이번 단계에서 확인한 범위는 다음과 같다.

- 확장 dist 빌드 성공
- dist service worker에 URL resolver 포함
- dist viewer/thumbnail 경로가 resolver를 호출
- dist viewer asset에 원격 응답 바이트 검증 포함
- GitHub blob URL이 dist resolver에서 raw URL로 변환됨

## 5. 완료 기준 대비

| 기준 | 결과 |
|---|---|
| URL resolver 테스트 통과 | 완료 |
| 기존 download interceptor 테스트 통과 | 완료 |
| Firefox 확장 빌드 통과 | 완료 |
| Chrome 확장 빌드 통과 | 완료 |
| dist에 resolver 실체 파일 포함 | 완료 |
| dist viewer에 응답 검증 포함 | 완료 |
| 실제 Firefox GUI 카드 클릭 검증 | 미수행 |

## 6. 결론

#432의 코드 관점 변경은 특정 GitHub wiki 링크만을 위한 rule-based patch가 아니라, provider별 URL 해석 계층과 원격 응답 바이트 검증 계층으로 일반화되어 적용되었다.

작업지시자 승인 후 최종 보고서 기준으로 타스크 완료 처리를 진행할 수 있다.
