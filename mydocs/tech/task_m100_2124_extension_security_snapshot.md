# Task M100 #2124 — browser extension security snapshot

- 이슈: #2124
- 단계: Stage 4 — extension security snapshot
- 작성일: 2026-07-10
- 기준 브랜치: `upstream/devel`
- 기준 커밋: `3077f96d1f9931c50d6d62be77b389d4f66470a9`
- 관련 문서:
  - `mydocs/manual/browser_extension_dev_guide.md`
  - `mydocs/report/archives/browser_extension_security_audit.md`

## 1. 목적

이 문서는 Chrome, Firefox, Safari extension 표면의 보안 기준선을 고정한다. #2124에서는 extension 보안
정책을 수정하지 않고, Phase A/B 리팩터링 전에 유지해야 할 검증 지점과 잔여 위험을 분리한다.

`CLAUDE.md`의 필수 참조 문서에는 `mydocs/report/browser_extension_security_audit.md`가 적혀 있지만,
현재 저장소의 실제 파일은 `mydocs/report/archives/browser_extension_security_audit.md`이다. Stage 4에서는
실제 존재하는 archive 경로를 기준으로 참조했다.

## 2. 가이드 기준

`mydocs/manual/browser_extension_dev_guide.md`의 extension 리팩터링 기준은 다음으로 요약된다.

| 항목 | 기준 |
|------|------|
| MV3 script policy | inline script 금지, background/content/viewer 분리 |
| WASM CSP | Chrome/Firefox는 `wasm-unsafe-eval` 필요 |
| remote fetch | scheme, private/internal IP, redirect, credential, magic number, size를 검증 |
| local file/drop | 사용자 확인 또는 명시적 opt-in 필요 |
| sender validation | internal viewer page와 web page content script sender를 구분 |
| URL edge case | userinfo, javascript/data URL, localhost/private IP, redirect를 차단 |
| test checklist | XSS, internal IP, magic number, sender spoofing, CSP 회귀를 포함 |

## 3. manifest snapshot

| 표면 | manifest | MV | 주요 권한 | host permission | CSP | WAR |
|------|----------|----|-----------|-----------------|-----|-----|
| Chrome | `rhwp-chrome/manifest.json` | 3 | `activeTab`, `downloads`, `contextMenus`, `clipboardWrite`, `storage` | `<all_urls>` | `script-src 'self' 'wasm-unsafe-eval'; object-src 'self'` | `wasm/*`, `fonts/*`, `icons/*`, `dev-tools-inject.js` |
| Firefox | `rhwp-firefox/manifest.json` | 3 | `activeTab`, `downloads`, `contextMenus`, `clipboardWrite`, `storage` | `<all_urls>` | `script-src 'self' 'wasm-unsafe-eval'; object-src 'self'` | `wasm/*`, `fonts/*`, `icons/*`, `dev-tools-inject.js` |
| Safari | `rhwp-safari/src/manifest.json` | 3 | `activeTab`, `contextMenus`, `storage` | `<all_urls>` | `script-src 'self' 'unsafe-eval' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; object-src 'none'; base-uri 'none'; frame-src 'none'; img-src 'self' https: data:; connect-src 'self' https: http:` | `wasm/*`, `fonts/*`, `icons/*` |

주의:

- Chrome/Firefox는 content script와 host permission이 모두 `<all_urls>`다.
- Chrome/Firefox는 `dev-tools-inject.js`가 WAR에 포함된다.
- Safari는 dev-tools injection WAR이 없고 CSP가 더 세분화되어 있지만, `unsafe-eval`과 `unsafe-inline`
  허용이 있다.
- #2124에서는 manifest 권한 축소를 수행하지 않는다.

## 4. message contract와 sender validation

| message type | Chrome/Firefox 현재 상태 | Safari 현재 상태 | 기준선 판단 |
|--------------|--------------------------|------------------|-------------|
| `fetch-file` | `isTrustedExtensionPageSender(sender, runtimeApi)`로 viewer page sender 확인. `fetchDocumentWithPolicy()`가 URL/redirect/credential policy를 담당. | `isInternalPage(sender)` 확인. URL, devMode/private host, HTTP 설정, redirect, content-type, size, magic number 확인. | 핵심 보안 경계. 임의 완화 금지. |
| `open-hwp` | background handler에서 별도 sender check 없이 `openViewer(message.url, message.filename)` 호출. viewer launch path에서 URL validation이 충분히 박혀 있지는 않다. | `isContentScript(sender)` 확인 후 URL 검증과 filename sanitize. | Chrome/Firefox는 audit C-02/N-05와 연결되는 잔여 위험. |
| `extract-thumbnail` | `isWebPageSender(sender)` 확인 후 `extractThumbnailFromUrl()` 사용. 내부에서 fetch policy 사용. | content script sender 확인. thumbnail 추출 경로는 Safari 내부 구현. | web page sender와 internal page sender를 섞지 않아야 한다. |
| `get-settings` | storage read. sender restriction 없음. | settings 응답. | 민감 데이터 확장 시 sender policy 재검토 필요. |

## 5. fetch and file validation

| 항목 | Chrome/Firefox | Safari |
|------|----------------|--------|
| scheme | `http`, `https` 허용. default `allowHttp: true` | `http`, `https` 허용. `allowHttp` 설정과 warning policy 존재 |
| userinfo | 차단 | 차단 |
| localhost/private IP | 차단 | 기본 차단, devMode에서 일부 허용 가능 |
| single-label host | 차단 | host validation 보유 |
| redirect | `redirect: manual`, 최대 5회 재검증 | `redirect: manual`, redirect location 재검증 |
| credentials | `omit` | `omit` |
| size limit | service worker fetch path에는 별도 size limit 없음 | 기본 20 MB limit |
| magic number | service worker 내부에는 없음. `rhwp-studio` 수신 후 `assertRemoteDocumentBytes()`에서 HWP/HWP3/HWPX 검증 | background에서 HWP/HWPX signature 확인 |
| byte return shape | number array | `ArrayBuffer` |

Chrome/Firefox의 `fetch-file` byte는 `Array.from(new Uint8Array(buffer))`로 반환된다. `rhwp-studio`는
`new Uint8Array(result.data)`로 수신하고 `assertRemoteDocumentBytes(data)`를 호출한다. 현재 동작은
호환되지만, memory expansion과 service worker level size limit 부재는 잔여 위험으로 남긴다.

## 6. 공유 보안 모듈 상태

`rhwp-shared/security/*`에는 URL validator, file signature, sender validator, filename sanitizer,
security log 유틸리티가 있다. 다만 현재 Chrome/Firefox의 active fetch path는
`rhwp-chrome/sw/fetch-security.js`와 `rhwp-firefox/sw/fetch-security.js`이며, Safari는
`rhwp-safari/src/background.js`에 inline equivalent를 가진다.

공유 모듈 존재만으로 active path가 통합되어 있다고 판단하면 안 된다. Phase A/B에서 중복 제거를 시도할 때는
실제 import path와 build output을 기준으로 검증해야 한다.

## 7. content script와 노출 표면

| 표면 | 현재 상태 | 관련 audit 항목 |
|------|-----------|----------------|
| Chrome content script | `data-hwp-extension`, version `0.2.8`, `hwp-extension-ready` event, `dev-tools-inject.js` injection | N-02, N-03 |
| Firefox content script | manifest version 기반 version 노출, `hwp-extension-ready` event, `dev-tools-inject.js` injection | N-02, N-03 |
| Safari content script | allowed domain 중심 announce, dev-tools injection 없음 | N-03 완화 방향 |
| Chrome/Firefox download observer | shared `download-interceptor-common.js`, `download-observer-state.js`, `document-url-resolver.js` symlink | N-05 |

Chrome/Firefox의 extension presence 노출과 dev-tools injection은 기존 감사 문서에 기록된 알려진 위험이다.
#2124에서는 제거하지 않고, 리팩터링 전 기준선으로만 기록한다.

## 8. 잔여 위험 목록

| 위험 | 현재 기준선 | 후속 단계 |
|------|-------------|-----------|
| Chrome/Firefox `open-hwp` sender validation 약함 | background message handler에 explicit sender guard가 없다. | Phase A/B 보안 리팩터링 후보 |
| Chrome/Firefox `fetch-file` service worker size limit 부재 | studio 수신 후 signature 검증은 있으나 SW 단계 size cap은 없다. | smoke/security gate 후보 |
| Chrome/Firefox byte array 반환 | number array로 memory expansion 가능 | typed binary transport 검토 |
| `<all_urls>` host permission | 현재 기능 범위를 위해 유지 | 권한 축소는 별도 UX/기능 검토 후 |
| WAR의 `dev-tools-inject.js` | Chrome/Firefox에서 web accessible | dev tool 기능 유지 필요성 재검토 |
| Safari CSP의 `unsafe-eval`, `unsafe-inline` | 현재 Safari build/runtime 요구 가능성 | Safari build 검증 후 축소 후보 |

## 9. Stage 4 확인

실행한 정적 확인:

```bash
node --check rhwp-safari/src/background.js
node --check rhwp-safari/src/content-script.js
node --check rhwp-chrome/sw/fetch-security.js
node --check rhwp-firefox/sw/fetch-security.js
node --check rhwp-chrome/sw/message-router.js
node --check rhwp-firefox/sw/message-router.js
node --check rhwp-chrome/content-script.js
node --check rhwp-firefox/content-script.js
```

실행한 shared module test:

```bash
node --test rhwp-shared/sw/download-interceptor-common.test.js rhwp-shared/sw/document-url-resolver.test.js rhwp-shared/sw/download-observer-state.test.js
```

실행한 package build와 dist 계약 검사:

```bash
npm --prefix rhwp-chrome ci
npm --prefix rhwp-chrome run build
npm --prefix rhwp-firefox ci
npm --prefix rhwp-firefox run build
node --test scripts/frontend-extension-dist.test.mjs
```

결과:

- 63개 테스트 통과.
- Chrome/Firefox package build 통과.
- dist 계약 3개 테스트 통과: source/dist manifest 일치, CSP/WAR, inline script 부재,
  36개 WOFF2와 WASM copy, Safari의 dev-tools WAR 제외를 확인했다.
- syntax check 실패 없음.

미실행 항목:

- Chrome/Firefox/Safari packaged extension install smoke.
- 실제 remote redirect/private IP 차단 브라우저 E2E.
- permission prompt UX 확인.

미실행 항목은 smoke manifest의 release/PR gate 후보로 분리한다.
