# Stage 1 완료보고서: URL resolver + 순수 함수 테스트

- **타스크**: [#432](https://github.com/edwardkim/rhwp/issues/432)
- **마일스톤**: M100 (v1.0.0)
- **브랜치**: `local/task432`
- **작성일**: 2026-04-29
- **단계**: Stage 1 — URL resolver + 순수 함수 테스트

## 1. 완료 내용

브라우저 확장에서 "링크 URL"과 "실제 파일 바이트 URL"이 다른 경우를 호출부에 흩어진 rule로 처리하지 않도록 공통 resolver를 추가했다.

### 1.1 공통 URL resolver 추가

신규 파일:

- `rhwp-shared/sw/document-url-resolver.js`

주요 함수:

- `isDocumentPath(pathname)`
  - pathname 기준 `.hwp`/`.hwpx` 문서 경로 판정
  - query/hash가 섞인 입력은 문서 경로로 보지 않음
- `resolveGithubBlobUrl(parsed)`
  - GitHub `blob` URL을 raw URL로 변환
  - `.hwp`/`.hwpx` 경로만 변환
- `resolveDocumentUrl(url)`
  - provider adapter를 통과해 실제 fetch 대상 URL 반환
  - adapter 미매칭 또는 URL 파싱 실패 시 원본 URL 반환

### 1.2 Chrome/Firefox symlink 추가

신규 symlink:

- `rhwp-chrome/sw/document-url-resolver.js`
- `rhwp-firefox/sw/document-url-resolver.js`

기존 `download-interceptor-common.js`와 동일하게 `rhwp-shared/sw/` 공통 파일을 참조하도록 구성했다.

### 1.3 순수 함수 테스트 추가

신규 파일:

- `rhwp-shared/sw/document-url-resolver.test.js`

검증 케이스:

- GitHub blob HWP URL -> raw URL 변환
- GitHub blob HWPX URL -> raw URL 변환
- 인코딩된 경로 보존
- raw URL/일반 URL 미변환
- query 문자열에만 `.hwp`가 있는 URL 미변환
- malformed URL 원본 반환

## 2. 중간 발견 및 수정

첫 테스트 실행에서 `isDocumentPath('/download?file=sample.hwp')`가 true로 판정되는 오탐을 발견했다.

원인:

- helper가 "pathname" 입력을 전제로 했지만, 방어적으로 query/hash 포함 문자열도 받을 수 있음

수정:

- `?` 또는 `#`가 포함된 입력은 문서 경로로 보지 않도록 보강

## 3. 검증 결과

실행 명령:

```bash
node --test rhwp-shared/sw/document-url-resolver.test.js
node --test rhwp-shared/sw/download-interceptor-common.test.js
```

결과:

- `document-url-resolver.test.js`: 12개 통과
- `download-interceptor-common.test.js`: 26개 통과

Node가 `MODULE_TYPELESS_PACKAGE_JSON` 경고를 출력했지만, 기존 공통 테스트와 동일한 ESM 감지 경고이며 테스트 실패는 아니다.

## 4. 변경 파일

| 파일 | 구분 | 내용 |
|---|---|---|
| `rhwp-shared/sw/document-url-resolver.js` | 신규 | 공통 URL resolver |
| `rhwp-shared/sw/document-url-resolver.test.js` | 신규 | 순수 함수 테스트 |
| `rhwp-chrome/sw/document-url-resolver.js` | 신규 | shared resolver symlink |
| `rhwp-firefox/sw/document-url-resolver.js` | 신규 | shared resolver symlink |

## 5. 다음 단계

Stage 2에서 viewer/thumbnail 경로에 `resolveDocumentUrl()`을 적용한다.

예상 변경:

- `rhwp-chrome/sw/viewer-launcher.js`
- `rhwp-firefox/sw/viewer-launcher.js`
- `rhwp-chrome/sw/thumbnail-extractor.js`
- `rhwp-firefox/sw/thumbnail-extractor.js`

## 6. 승인 요청

Stage 1 완료를 확인해주면 Stage 2에 착수한다.
