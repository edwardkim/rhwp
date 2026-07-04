# Task M100 #1520 최종 보고서

## 요약

GitHub의 실제 문서 파일 URL이 아닌 `edit`, `commits`, `blame`, `tree` 라우팅 URL에 rhwp 배지와 hover 미리보기가 붙는 오탐을 정정했다.

기존 content script는 `anchor.href` 전체가 `.hwp`/`.hwpx`로 끝나는지만 검사했다. GitHub의 `https://github.com/{owner}/{repo}/edit/{branch}/path/file.hwp` 같은 URL은 파일 다운로드 URL이 아니라 HTML 페이지인데도 후보로 잡혔고, 클릭 후 viewer에서야 "실제 HWP/HWPX 파일이 아닙니다" 오류가 발생했다.

## 변경 내용

- `rhwp-shared/sw/document-url-resolver.js`
  - GitHub 문서 URL 분류 함수 추가
  - `blob`/raw 문서 URL은 `openable`
  - `edit`, `commits`, `blame`, `tree`는 `not-document`
  - 일반 직접 `.hwp/.hwpx` pathname은 `openable`
  - query 문자열에만 `.hwp/.hwpx`가 있는 경우는 후보에서 제외

- `rhwp-shared/sw/document-url-resolver.test.js`
  - GitHub `blob`, raw, `edit`, `commits`, `blame`, `tree`, query 위장 케이스 테스트 추가

- `rhwp-chrome/content-script.js`
- `rhwp-firefox/content-script.js`
- `rhwp-safari/src/content-script.js`
  - 배지/hover/prefetch/autoOpen 후보가 같은 `isHwpLink()` 판정을 사용하도록 URL pathname 기반 분류 적용
  - `data-hwp="true"` 명시 링크는 기존 동작 유지

## 해결되는 문제

- GitHub `edit/main/.../*.hwp`에서 배지와 hover 미리보기가 생성되지 않는다.
- GitHub `commits/main/.../*.hwp`에서 배지와 hover 미리보기가 생성되지 않는다.
- 같은 계열의 `blame`, `tree` URL도 문서 후보에서 제외된다.
- HTML 오류 페이지를 HWP 파일로 열려고 해서 viewer 오류 모달이 뜨는 흐름을 사전에 줄인다.
- 실제 GitHub `blob` 문서 URL과 raw 문서 URL은 계속 rhwp로 열 수 있다.

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

참고:

- 초기 작업은 오래된 `local/devel` 기준에서 시작됐으나, 최종 변경은 최신 `upstream/devel` 기준 `local/task1520-upstream` 브랜치로 포팅해 검증했다.
- Safari 전체 `build.sh`/Xcode 빌드는 이번 범위에서 제외했다.
- Chrome `rhwp-chrome/dist`와 Firefox `rhwp-firefox/dist`를 실제 로드해 GitHub #1520 재현 페이지에서 수동 확인했다.
- 수동 확인 결과 `edit`/`commits` 링크에는 배지가 없고, `blob`/raw 링크에는 배지가 표시됐다.

## 남은 후속

- hover 카드 지연 표시 타이머 취소 누락 문제는 별도 이슈 #1521에서 처리한다.
