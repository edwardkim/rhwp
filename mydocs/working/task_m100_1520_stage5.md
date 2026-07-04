# Task M100 #1520 Stage 5 완료 기록

## 배경

초기 구현 브랜치 `local/task1520`은 오래된 `local/devel` 기준이었다. 최신 `upstream/devel`에는 #1307 계열의 `fetch-security` 모듈과 Chrome/Firefox content script 보안 보강이 포함되어 있어, 최신 기준으로 포팅해 재검증했다.

## 수행

- `upstream/devel`을 `42d7f6bc`로 갱신했다.
- 새 브랜치 `local/task1520-upstream`을 최신 `upstream/devel`에서 생성했다.
- 기존 작업 변경을 stash로 보관한 뒤 새 브랜치에 적용했다.
- Chrome/Firefox content script의 upstream 변경(Shadow DOM hover card, `isTrusted`, `allowDownloadUrl`)을 유지한 상태로 URL 후보 판정을 병합했다.
- 최신 upstream의 `mydocs/orders/20260625.md`에 #1520 행을 병합했다.

## 검증

통과:

- `node --test rhwp-shared/sw/document-url-resolver.test.js`
- `node rhwp-chrome/sw/fetch-security.test.mjs`
- `node --test rhwp-shared/sw/download-interceptor-common.test.js`
- `node --check rhwp-chrome/content-script.js`
- `node --check rhwp-firefox/content-script.js`
- `node --check rhwp-safari/src/content-script.js`
- `npm run build` (`rhwp-chrome`)
- `npm run build` (`rhwp-firefox`)

수동 확인:

- Chrome `rhwp-chrome/dist` 로드 후 GitHub #1520 재현 링크에서 확인했다.
- Firefox `rhwp-firefox/dist` 로드 후 GitHub #1520 재현 링크에서 확인했다.
- `edit/main/samples/2010-01-06.hwp`와 `commits/main/samples/2010-01-06.hwp`에는 배지가 생성되지 않았다.
- `blob/main/samples/2010-01-06.hwp`와 raw URL에는 배지가 생성됐다.

## 비고

- `pdf-large/hwpx/2026_oss_rst.pdf`는 최신 upstream checkout 시 LFS 필터와 저장 blob 불일치로 modified로 표시된다. 이번 작업 범위에 포함하지 않는다.
