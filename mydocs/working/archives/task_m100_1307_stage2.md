# Task m100 #1307 Stage 2 보고서

## 목적

Stage 1 보안 정책이 공공기관 다운로드 URL처럼 pathname에 `.hwp/.hwpx`가 직접 드러나지 않는 합법적인 문서 링크를 과도하게 차단할 수 있어 호환성 보정을 진행했다.

## 판단

링크 유형을 두 단계로 나누는 것이 적절하다.

1. 자동 확장자 감지 링크
   - 예: `https://example.go.kr/files/report.hwp`
   - 기존처럼 pathname 기준 `.hwp/.hwpx`를 요구한다.

2. 명시적 문서 링크
   - 예: `<a href="https://example.go.kr/FileDown.do?id=123" data-hwp="true">`
   - 사이트가 `data-hwp="true"`로 rhwp 통합 의도를 명시한 경우, 확장자 없는 공개 다운로드 URL을 허용한다.
   - 단, 내부망/localhost/private IP 차단, redirect 재검증, `credentials: "omit"`은 유지한다.

## 구현 내용

### content script

수정 파일:

- `rhwp-chrome/content-script.js`
- `rhwp-firefox/content-script.js`

변경:

- `isExplicitHwpLink(anchor)` 추가
- `extract-thumbnail` 메시지에 `allowDownloadUrl` 전달
- prefetch queue에서 같은 URL이 중복될 경우 `allowDownloadUrl`을 OR 병합

### service worker

수정 파일:

- `rhwp-chrome/sw/message-router.js`
- `rhwp-firefox/sw/message-router.js`
- `rhwp-chrome/sw/thumbnail-extractor.js`
- `rhwp-firefox/sw/thumbnail-extractor.js`

변경:

- `message.allowDownloadUrl === true`일 때만 thumbnail fetch의 `requireDocumentPath`를 해제
- 자동 감지 링크는 기존처럼 `requireDocumentPath: true`
- 내부망/redirect/credentials 정책은 그대로 적용

### 테스트

수정 파일:

- `rhwp-chrome/sw/fetch-security.test.mjs`

추가 케이스:

- `https://example.go.kr/FileDown.do?id=123` 공개 다운로드 URL 허용
- `http://127.0.0.1/FileDown.do?id=123` 내부망 다운로드 URL 차단

## 검증

실행:

```bash
node rhwp-chrome/sw/fetch-security.test.mjs
node --check rhwp-chrome/content-script.js
node --check rhwp-firefox/content-script.js
node --check rhwp-chrome/sw/message-router.js
node --check rhwp-firefox/sw/message-router.js
node --check rhwp-chrome/sw/thumbnail-extractor.js
node --check rhwp-firefox/sw/thumbnail-extractor.js
cd rhwp-chrome && npm run build
cd rhwp-firefox && npm run build
```

결과:

- 정책 테스트 통과
- Chrome/Firefox content script 문법 체크 통과
- Chrome/Firefox service worker 문법 체크 통과
- Chrome extension build 통과
- Firefox extension build 통과

## 남은 실제 브라우저 검증

- 제보 PoC의 localhost/private URL 자동 thumbnail fetch 차단 확인
- `data-hwp="true"`가 없는 일반 자동 감지 링크는 `.hwp/.hwpx` pathname만 thumbnail 허용되는지 확인
- `data-hwp="true"`가 있는 공공기관형 다운로드 URL에서 공개 URL thumbnail/열기 흐름이 유지되는지 확인
- 사용자가 직접 드래그&드롭한 로컬 파일 열기 흐름이 영향받지 않는지 확인
