# Stage 2 완료보고서: viewer/thumbnail URL 해석 적용

- **타스크**: [#432](https://github.com/edwardkim/rhwp/issues/432)
- **마일스톤**: M100 (v1.0.0)
- **브랜치**: `local/task432`
- **작성일**: 2026-04-29
- **단계**: Stage 2 — viewer/thumbnail 경로 통합

## 1. 완료 내용

Stage 1에서 추가한 `resolveDocumentUrl()`을 Chrome/Firefox 확장의 viewer 실행 경로와 썸네일 추출 경로에 연결했다.

### 1.1 viewer URL 파라미터 정규화

변경 파일:

- `rhwp-chrome/sw/viewer-launcher.js`
- `rhwp-firefox/sw/viewer-launcher.js`

변경 내용:

- `./document-url-resolver.js`에서 `resolveDocumentUrl()` import
- `buildViewerUrl(viewerBase, options)` helper 추가
- `openViewer()`와 `openViewerOrReuse()`가 URL 파라미터를 만들기 전에 `resolveDocumentUrl(options.url)`을 적용

효과:

- GitHub wiki의 `https://github.com/.../blob/.../file.hwp` 링크를 카드에서 열면 viewer에는 raw URL이 전달된다.
- GitHub raw URL과 일반 HWP URL은 기존 그대로 전달된다.
- 옵션이 비어 있을 때는 불필요한 `viewer.html?` 형태 대신 `viewer.html`을 반환한다.

### 1.2 썸네일 fetch 대상 정규화

변경 파일:

- `rhwp-chrome/sw/thumbnail-extractor.js`
- `rhwp-firefox/sw/thumbnail-extractor.js`

변경 내용:

- `./document-url-resolver.js`에서 `resolveDocumentUrl()` import
- `extractThumbnailFromUrl(url)` 내부 fetch 대상을 `resolveDocumentUrl(url)`로 변경
- 썸네일 캐시 key는 기존처럼 원본 URL을 유지

효과:

- content-script의 카드 상태와 캐시는 원본 링크 기준으로 유지된다.
- 실제 네트워크 요청만 raw URL로 이동하므로 GitHub blob 링크 카드에서도 `PrvImage` 추출 가능성이 열린다.

## 2. 검증 결과

실행 명령:

```bash
node --test rhwp-shared/sw/document-url-resolver.test.js
node --test rhwp-shared/sw/download-interceptor-common.test.js
node --input-type=module -e "await import('./rhwp-chrome/sw/viewer-launcher.js'); await import('./rhwp-firefox/sw/viewer-launcher.js'); await import('./rhwp-chrome/sw/thumbnail-extractor.js'); await import('./rhwp-firefox/sw/thumbnail-extractor.js');"
```

결과:

- `document-url-resolver.test.js`: 12개 통과
- `download-interceptor-common.test.js`: 26개 통과
- Chrome/Firefox `viewer-launcher.js`, `thumbnail-extractor.js` ESM import 검사 통과

Node가 `MODULE_TYPELESS_PACKAGE_JSON` 경고를 출력했지만, 기존 공통 테스트와 동일한 ESM 감지 경고이며 테스트 실패는 아니다.

## 3. 변경 파일

| 파일 | 내용 |
|---|---|
| `rhwp-chrome/sw/viewer-launcher.js` | viewer URL 파라미터에 resolver 적용 |
| `rhwp-firefox/sw/viewer-launcher.js` | viewer URL 파라미터에 resolver 적용 |
| `rhwp-chrome/sw/thumbnail-extractor.js` | 썸네일 fetch 대상에 resolver 적용 |
| `rhwp-firefox/sw/thumbnail-extractor.js` | 썸네일 fetch 대상에 resolver 적용 |

## 4. 남은 작업

Stage 2는 URL 해석 적용까지 완료했다. 아직 viewer 내부의 원격 응답 바이트 검증은 적용하지 않았다.

다음 Stage 3에서 수행할 작업:

- `rhwp-studio/src/main.ts`의 `loadFromUrlParam()`에서 `wasm.loadDocument()` 전 바이트 검증
- HTML/오류 페이지 응답을 HWP 파서에 넘기지 않도록 차단
- 사용자 오류 메시지 개선

## 5. 승인 요청

Stage 2 완료를 확인해주면 Stage 3에 착수한다.
