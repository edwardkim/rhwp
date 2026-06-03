# 브라우저 확장 프로그램 보안 취약점 수정 계획 (v2)

**작성일**: 2026-04-09
**기반 문서**: `mydocs/report/browser_extension_security_audit.md` (1차 감사 + 2차 검토)

## 원칙

- **기존 기능 100% 유지**
- **Chrome/Edge/Safari 동시 적용** — 공통 보안 모듈 공유
- **방어 심층화(Defense in Depth)** — 단일 검증 실패 시에도 다른 계층에서 차단

---

## 공통 보안 모듈 구조

```
rhwp-shared/
  security/
    url-validator.js        — URL 화이트리스트, 스킴/IP/확장자/리다이렉트 검증
    url-validator.test.js   — 경계 케이스 단위 테스트
    filename-sanitizer.js   — 파일명 새니타이즈
    sender-validator.js     — 메시지 발신자 검증
    file-signature.js       — HWP/HWPX 매직 넘버 검증
    constants.js            — 화이트리스트 도메인, 허용 확장자, 크기 제한
```

빌드 시 `rhwp-chrome/dist/`, `rhwp-safari/dist/`에 복사. `web_accessible_resources`에 포함하지 않음.

---

## 수정 항목 (우선순위 순)

### 1순위: C-01 fetch-file 오픈 프록시 (Critical)

**기능 유지**: viewer.html에서 HWP 파일 fetch → 정상 동작

| 검증 항목 | 규칙 |
|----------|------|
| 프로토콜 | `https:`만 허용 |
| 내부 IP 차단 | 127.*, 10.*, 192.168.*, 169.254.*, ::1, *.local, IPv6 로컬 |
| DNS Rebinding 방어 | `redirect: 'manual'` 설정 → 리다이렉트 대상 URL 재검증 |
| 쿠키 | `credentials: 'omit'` |
| 응답 Content-Type | `text/html`, `application/json` 차단 |
| **매직 넘버 검증 (주 검증)** | HWP: `D0 CF 11 E0`, HWPX: `50 4B 03 04`. **URL에 .hwp 확장자가 없어도 매직 넘버로 HWP 파일 확인** |
| **Content-Disposition 검증** | `attachment; filename="*.hwp"` 헤더로 HWP 파일 확인. 정부사이트 `*.do?id=...` 패턴 대응 |
| 파일 크기 | 스트리밍 누적 체크, 20MB 상한 (실용적 HWP 크기) |
| 발신자 | viewer.html만 허용 (`sender.url` 검증) |

**정부사이트 다운로드 패턴 대응**:
```
https://www.gov.kr/cmm/fms/FileDown.do?atchFileId=1234
https://www.moel.go.kr/common/downloadFile.do?file_seq=20230600739
```
이런 URL은 pathname에 `.hwp` 확장자가 없다. 검증 전략:
1. URL 자체의 확장자는 **보조 수단** (있으면 빠른 판별)
2. **응답 헤더 Content-Disposition**의 filename이 `.hwp/.hwpx`인지 확인 (주 검증)
3. **응답 바이트 매직 넘버**로 최종 확인 (방어 심층화)
4. 호스트가 허용 도메인(`*.go.kr` 등)이면 확장자 없는 URL도 fetch 허용

---

### 2순위: H-01 XSS — innerHTML 제거 (High)

**기능 유지**: 호버 카드 모든 필드 동일 표시

| 현재 | 수정 후 |
|------|--------|
| `escapeHtml()` + `innerHTML` | **DOM API** (`createElement`, `textContent`, `setAttribute`) |

추가 방어:
- thumbnail URL: `https:`, `http:`만 허용, `javascript:`, `data:` 차단
- 텍스트 길이 제한: title 200자, description 500자, author 100자
- `img.referrerPolicy = 'no-referrer'`

---

### 3순위: N-02 dev-tools 주입 제거 (High)

**기능 유지**: 개발자 도구는 옵션 토글로 활성화 시에만 동작

| 현재 | 수정 후 |
|------|--------|
| 모든 페이지에 `dev-tools-inject.js` 주입 | 프로덕션 빌드에서 제거, storage 설정 `devMode: true` 시에만 주입 |

```javascript
// content-script.js
if (settings.devMode) {
  const devScript = document.createElement('script');
  devScript.src = browser.runtime.getURL('dev-tools-inject.js');
  document.head.appendChild(devScript);
}
```

---

### 4순위: N-01 web_accessible_resources 축소 (High)

**기능 유지**: viewer.html에서 WASM/폰트 정상 로드

| 현재 | 수정 후 |
|------|--------|
| `matches: ["<all_urls>"]` | `matches: ["<all_urls>"]` 유지하되 리소스 목록 축소 |

```json
"web_accessible_resources": [{
  "resources": ["wasm/*", "fonts/*", "icons/*"],
  "matches": ["<all_urls>"]
}]
```

- `dev-tools-inject.js` 제거 (3순위와 연동)
- viewer.html이 확장 내부 페이지이므로 WASM/폰트 접근은 항상 가능
- 단, content-script에서 아이콘 참조가 필요하면 `icons/*`만 유지

---

### 5순위: H-02 sender 검증 (High)

**기능 유지**: 메시지 정상 동작

| 메시지 | 허용 발신자 | 검증 방법 |
|--------|-----------|----------|
| `fetch-file` | 내부 페이지 (viewer.html) | `sender.url?.startsWith(browser.runtime.getURL(''))` |
| `open-hwp` | content script (탭) | `sender.tab && sender.tab.id != null` |
| `get-settings` | 모두 허용 | `externally_connectable` 미설정으로 외부 차단 전제 |

---

### 6순위: C-02 + N-05 URL 검증 통합 (Critical + Medium)

**기능 유지**: 뷰어 열기, 다운로드 인터셉터 정상 동작

URL 검증을 `openViewer()` 함수 자체에 내장하여 모든 진입점(메시지 핸들러, 다운로드 인터셉터, 컨텍스트 메뉴)에서 일관 적용.

| 검증 항목 | 규칙 |
|----------|------|
| 프로토콜 | `https:`, `http:`만 허용. `javascript:`, `data:`, `blob:` 차단 |
| URL 판별 (3단계) | ① pathname에 `.hwp/.hwpx` → 즉시 허용 |
| | ② 허용 도메인(`*.go.kr` 등) + 다운로드 패턴(`*.do?*`, `download*`) → 허용 (viewer에서 재검증) |
| | ③ 그 외 → 차단 |
| filename | 영숫자, 한글, `.`, `-`, `_`만 허용, 255자 제한 |
| filename 공격 방어 | path traversal(`../../`), null byte(`%00`), `@` 도메인, 유니코드 정규화 |

**정부사이트 다운로드 패턴**: URL에 `.hwp`가 없는 경우(`FileDown.do?id=...`), `openViewer()`는 허용하되 viewer 내부에서 fetch 후 Content-Disposition + 매직 넘버로 HWP 파일 여부를 재검증한다. HWP가 아니면 뷰어에 오류 표시.

---

### 7순위: N-04 메모리 폭발 방지 (Medium)

**기능 유지**: 파일 데이터 전달 정상 동작

| 현재 | 수정 후 |
|------|--------|
| `Array.from(new Uint8Array(buffer))` (100MB → 800MB 메모리) | `ArrayBuffer` structured clone 직접 전달 |

```javascript
// 수정 전
sendResponse({ data: Array.from(new Uint8Array(buf)) });

// 수정 후
sendResponse({ data: buf });  // ArrayBuffer structured clone
```

---

### 8순위: H-03 권한 최소화 + content_scripts 연동 (High)

**기능 유지**: 공공사이트 즉시 동작, 기타 사이트는 사용자 선택

```json
{
  "host_permissions": [
    "*://*.go.kr/*",
    "*://*.or.kr/*",
    "*://*.ac.kr/*",
    "*://*.mil.kr/*"
  ],
  "optional_host_permissions": ["<all_urls>"],
  "content_scripts": [{
    "matches": [
      "*://*.go.kr/*",
      "*://*.or.kr/*",
      "*://*.ac.kr/*",
      "*://*.mil.kr/*"
    ],
    "js": ["content-script.js"],
    "css": ["content-script.css"]
  }]
}
```

옵션 페이지에서 "모든 사이트에서 활성화" 토글 → `chrome.permissions.request({ origins: ["<all_urls>"] })` + `chrome.scripting.registerContentScripts()` 동적 등록.

---

### 9순위: N-03 fingerprinting 완화 (Medium)

**기능 유지**: 공공사이트에서 확장 감지 가능

| 현재 | 수정 후 |
|------|--------|
| 모든 페이지에 `data-hwp-extension` + 버전 노출 | 허용 도메인에서만 노출, 버전 정보 제거 |

```javascript
// 허용 도메인에서만 확장 존재 알림
const allowedDomains = ['.go.kr', '.or.kr', '.ac.kr', '.mil.kr'];
const isAllowed = allowedDomains.some(d => location.hostname.endsWith(d));
if (isAllowed) {
  document.documentElement.setAttribute('data-hwp-extension', 'rhwp');
  // 버전 정보 제거
  window.dispatchEvent(new CustomEvent('hwp-extension-ready', {
    detail: { name: 'rhwp', capabilities: ['preview'] }
  }));
}
```

---

### 10순위: M-01 HTTPS + M-02 CSP 강화 (Medium)

**M-01 HTTP 처리**:
- `https:` 우선 시도
- 실패 시 `http:` 허용하되 뷰어에 "안전하지 않은 연결" 경고 표시
- 옵션 페이지에서 "HTTP 완전 차단" 토글 제공

**M-02 CSP 최종**:
```
script-src 'self' 'wasm-unsafe-eval';
style-src 'self' 'unsafe-inline';
object-src 'none';
base-uri 'none';
frame-src 'none';
img-src 'self' https: data:;
connect-src 'self' https:;
```

---

### 11순위: L-01 WASM 파서 강화 (Medium)

| 방어 | 구현 |
|------|------|
| 파일 크기 제한 | 20MB (fetch-file에서 이미 제한) |
| 파싱 타임아웃 | Web Worker + 5초 타임아웃 |
| 메모리 제한 | `WebAssembly.Memory` maximum 파라미터 |
| 패닉 핸들링 | try-catch → 사용자 오류 UI |
| unsafe 블록 현황 | 문서화 (CC 인증 대비) |

---

## 사용자 설정 페이지 (options.html)

보안 정책과 기능 설정을 사용자가 직접 조절할 수 있도록 옵션 페이지를 구성한다. `chrome.storage.sync`로 저장하여 기기 간 동기화.

### 설정 항목

| 카테고리 | 설정 | 키 | 기본값 | 설명 |
|----------|------|-----|--------|------|
| **사이트 권한** | 허용 사이트 목록 | `allowedDomains` | `[".go.kr", ".or.kr", ".ac.kr", ".mil.kr"]` | 사용자가 도메인 추가/삭제 |
| | 모든 사이트 활성화 | `allSitesEnabled` | `false` | 토글 시 `permissions.request()` 호출 |
| **기능** | HWP 자동 열기 | `autoOpen` | `true` | 링크 클릭 시 뷰어 자동 열기 |
| | 배지 표시 | `showBadges` | `true` | HWP 링크 옆 H 배지 |
| | 호버 카드 | `hoverPreview` | `true` | 마우스 오버 시 미리보기 카드 |
| **보안** | HTTP 허용 | `allowHttp` | `true` | `false` 시 HTTPS만 허용 |
| | HTTP 경고 표시 | `httpWarning` | `true` | HTTP 파일 열 때 경고 표시 |
| | 최대 파일 크기 (MB) | `maxFileSize` | `20` | fetch-file 크기 제한 |
| **개발** | 개발자 도구 | `devMode` | `false` | rhwpDev 주입 활성화 |
| | 보안 로그 | `securityLog` | `false` | 차단된 요청 기록 |

### UI 구성

```
┌─────────────────────────────────────────────┐
│  rhwp 설정                                    │
├─────────────────────────────────────────────┤
│                                             │
│  📌 사이트 권한                               │
│  ┌─────────────────────────────────────┐   │
│  │ .go.kr    [✕]                        │   │
│  │ .or.kr    [✕]                        │   │
│  │ .ac.kr    [✕]                        │   │
│  │ .mil.kr   [✕]                        │   │
│  │ [도메인 추가...          ] [추가]      │   │
│  └─────────────────────────────────────┘   │
│  ☐ 모든 사이트에서 활성화 (보안 주의)         │
│                                             │
│  📄 기능                                     │
│  ☑ HWP 링크 자동 열기                        │
│  ☑ 링크 옆 배지(H) 표시                      │
│  ☑ 호버 시 미리보기 카드                      │
│                                             │
│  🔒 보안                                     │
│  ☑ HTTP 연결 허용 (비권장)                    │
│  ☑ HTTP 사용 시 경고 표시                     │
│  최대 파일 크기: [20] MB                      │
│                                             │
│  🛠 개발                                     │
│  ☐ 개발자 도구 (rhwpDev)                     │
│  ☐ 보안 로그 기록                            │
│  [보안 로그 보기]                             │
│                                             │
└─────────────────────────────────────────────┘
```

### 설정과 보안 연동

| 설정 | 연동 대상 |
|------|----------|
| `allowedDomains` | content-script 도메인 체크, fetch-file URL 검증, fingerprinting 노출 범위 |
| `allSitesEnabled` | `chrome.permissions.request/remove` + `chrome.scripting.registerContentScripts` |
| `autoOpen` | content-script `interceptHwpClick()` |
| `allowHttp` | url-validator의 프로토콜 검증 |
| `maxFileSize` | fetch-file 스트리밍 크기 체크 |
| `devMode` | content-script dev-tools 주입 여부 |
| `securityLog` | background에서 차단 이벤트 기록 → 옵션 페이지에서 열람 |

### 보안 로그 구조

```javascript
// chrome.storage.local에 저장 (최근 100건 FIFO)
{
  securityEvents: [
    {
      time: "2026-04-09T12:34:56Z",
      type: "fetch-blocked",       // fetch-blocked | url-blocked | sender-blocked
      url: "http://192.168.0.1/",
      reason: "내부 IP 차단",
      sender: "content-script"
    },
    ...
  ]
}
```

---

## 보안 테스트 경계 케이스

URL 검증 단위 테스트에 반드시 포함:

| 케이스 | 입력 | 기대 결과 |
|--------|------|----------|
| 유니코드 | `https://evil.com/ﬁle.hwp` (fi ligature) | 차단 |
| Double encoding | `%2527` → `%27` → `'` | 차단 |
| URL fragment | `https://safe.go.kr/file.hwp#javascript:alert(1)` | 허용 (fragment 무시) |
| IPv6 로컬 | `http://[::1]/file.hwp` | 차단 |
| `@` 도메인 | `https://safe.go.kr@evil.com/file.hwp` | 차단 |
| Open redirect | `https://gov.kr/redirect?to=https://evil.com` | 리다이렉트 대상 재검증 |
| Path traversal | `../../etc/passwd.hwp` | 차단 |
| Null byte | `file.hwp%00.exe` | 차단 |
| Query 확장자 | `https://evil.com/mal.exe?f=test.hwp` | 차단 (pathname 기준) |
| 정상 공공 | `https://www.gov.kr/file.hwp` | 허용 |
| 정부 다운로드 (do 패턴) | `https://www.gov.kr/cmm/fms/FileDown.do?id=123` | 허용 (viewer에서 재검증) |
| 정부 다운로드 (npaid 패턴) | `https://www.epeople.go.kr/nep/pttn/gnrlPttn/pttnSmpFileDown.npaid?...` | 허용 (viewer에서 재검증) |
| 민간 다운로드 (미허용 도메인) | `https://unknown.com/download.do?id=123` | 차단 (확장자 없음 + 미허용 도메인) |
| 허용 도메인 비-HWP | `https://www.gov.kr/page.html` | fetch-file에서 매직 넘버 불일치 시 차단 |

---

## 적용 순서 및 일정

| 순서 | 수정 | Chrome | Safari | 비고 |
|------|------|--------|--------|------|
| 1 | 공통 보안 모듈 | `rhwp-shared/security/` | 동일 | url-validator, sender-validator, file-signature, constants |
| 2 | C-01 fetch-file 검증 | `sw/message-router.js` | `src/background.js` | 공통 모듈 참조 |
| 3 | H-01 innerHTML → DOM API | `content-script.js` | `src/content-script.js` | 각각 별도 수정 |
| 4 | N-02 dev-tools 조건부 주입 | `content-script.js` | 해당 없음 | settings.devMode 연동 |
| 5 | N-01 WAR 축소 | `manifest.json` | `src/manifest.json` | 양쪽 동시 |
| 6 | H-02 sender 검증 | `sw/message-router.js` | `src/background.js` | 공통 모듈 참조 |
| 7 | C-02+N-05 openViewer 검증 | `sw/viewer-launcher.js` | `src/background.js` | 공통 모듈 참조 |
| 8 | N-04 메모리 | `sw/message-router.js` | `src/background.js` | 양쪽 동시 |
| 9 | **옵션 페이지 구현** | `options.html` + `options.js` | `options.html` + `options.js` | 설정 UI + storage 연동 |
| 10 | H-03 권한 축소 | `manifest.json` | `src/manifest.json` | 옵션 페이지 연동 |
| 11 | N-03 fingerprinting | `content-script.js` | `src/content-script.js` | allowedDomains 연동 |
| 12 | M-01+M-02 | manifest + background | manifest + background | allowHttp 연동 |
| 13 | L-01 WASM | viewer.html (공통) | viewer.html (공통) | 장기 |

---

## 한국 공공기관 규정 대응

| 규정 | 대응 |
|------|------|
| 개인정보보호법 제29조 | CSP `connect-src` 제한으로 문서 내용 외부 미전송 기술 증명 |
| 국가정보원 보안적합성 | `unsafe` 블록 현황 문서화, WASM 메모리 안전성 검증 |
| 장애인차별금지법 | WCAG 2.1 AA 접근성 준수 (뷰어 UI) |
| 전자정부 표준프레임워크 | Content-Disposition 헤더 기반 HWP 파일 감지 |
