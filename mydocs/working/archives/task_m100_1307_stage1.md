# Task m100 #1307 Stage 1 보고서

## 범위

브라우저 확장 프로그램의 service worker fetch 경로 보안 강화를 1차 구현했다.

대상:

- Chrome 확장
- Firefox 확장
- content script synthetic click 방어
- service worker `fetch-file`, `extract-thumbnail` 방어
- 자동 썸네일 fetch URL 정책

## 확인한 원인

기존 구조에서는 다음 경로가 위험했다.

1. content script가 웹페이지의 HWP/HWPX 링크를 감지한다.
2. 자동 썸네일 프리페치 또는 hover 미리보기가 service worker에 `extract-thumbnail` 메시지를 보낸다.
3. service worker가 `message.url`을 검증 없이 fetch한다.
4. PrvImage가 있으면 `dataUri` 형태의 결과가 content script로 돌아간다.
5. content script가 page DOM에 이미지를 삽입한다.

이 구조는 신뢰할 수 없는 페이지가 localhost/private network URL을 문서 링크처럼 삽입했을 때 확장 권한을 통한 요청 유발로 이어질 수 있다.

## 구현 내용

### 1. 공통 fetch 보안 정책 추가

추가 파일:

- `rhwp-chrome/sw/fetch-security.js`
- `rhwp-firefox/sw/fetch-security.js`

정책:

- `http:`/`https:` 외 scheme 차단
- `username:password@host` userinfo URL 차단
- localhost, loopback, link-local, private IP 대역 차단
- `.local`, `.internal`, `.intranet`, `.lan`, `.home`, `.corp` 등 내부 호스트 suffix 차단
- single-label intranet host 차단
- thumbnail fetch는 pathname 기준 HWP/HWPX 문서 경로만 허용
- `credentials: "omit"`
- `redirect: "manual"` 후 `Location` 재검증
- redirect 대상이 내부망이면 차단

### 2. message sender 검증

수정 파일:

- `rhwp-chrome/sw/message-router.js`
- `rhwp-firefox/sw/message-router.js`

적용:

- `fetch-file`: extension viewer(`viewer.html`) sender만 허용
- `extract-thumbnail`: 실제 웹페이지 content script sender만 허용
- URL fetch는 `fetchDocumentWithPolicy()` 경유

### 3. 자동 썸네일 fetch 보호

수정 파일:

- `rhwp-chrome/sw/thumbnail-extractor.js`
- `rhwp-firefox/sw/thumbnail-extractor.js`

적용:

- URL 검증 후 정규화된 safe URL만 캐시 키로 사용
- private/internal URL은 fetch 전에 차단
- GitHub blob URL은 기존 resolver를 통해 raw URL로 정규화 후 검증

### 4. synthetic click 방어

수정 파일:

- `rhwp-chrome/content-script.js`
- `rhwp-firefox/content-script.js`

적용:

- badge click과 hover card click에서 `event.isTrusted`가 아닌 이벤트를 무시
- 페이지 스크립트가 `element.click()` 또는 synthetic `MouseEvent`로 viewer 로드를 유발하는 경로 축소

## 검증

실행:

```bash
node rhwp-chrome/sw/fetch-security.test.mjs
node --check rhwp-chrome/sw/message-router.js
node --check rhwp-firefox/sw/message-router.js
node --check rhwp-chrome/content-script.js
node --check rhwp-firefox/content-script.js
cd rhwp-chrome && npm run build
cd rhwp-firefox && npm run build
```

결과:

- fetch security policy test 통과
- Chrome/Firefox JS syntax check 통과
- Chrome extension build 통과
- Firefox extension build 통과

## 남은 확인

- 실제 브라우저에서 제보 PoC와 동일한 로컬 시나리오 재현 차단 확인
- public HWP/HWPX 링크 hover preview 정상 동작 확인
- viewer에서 public remote HWP/HWPX 열기 정상 동작 확인

## 참고

- 내부 제보 원문: `mydocs/feedback/ext-security-20260606.md`
- 공개 이슈: #1307
