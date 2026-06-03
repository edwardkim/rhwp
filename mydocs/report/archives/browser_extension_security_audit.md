# 브라우저 확장 프로그램 보안 감사 보고서

**작성일**: 2026-04-09
**대상**: rhwp-chrome (Chrome/Edge), rhwp-safari (Safari)
**1차 감사**: 보안 전문가 에이전트
**2차 검토**: 보안 전문 컨설턴트 (10년 경력, OWASP/CWE/KISA 전문)

## 위협 모델

이 확장은 한국 공공사이트(정부24, 국민신문고 등)에서 HWP 문서를 열 때 사용된다. 사용자는 개인 신청서, 증명서 같은 **민감한 문서**를 다루며, 해커가 이 점을 집중 공략할 수 있다.

---

## 취약점 종합 (1차 감사 + 2차 검토)

| ID | 취약점 | 심각도 | 영향 범위 | 발견 |
|----|--------|--------|----------|------|
| C-01 | fetch-file 오픈 프록시 | **Critical** | 공통 | 1차 |
| C-02 | open-hwp URL 검증 없음 | **Critical** | 공통 | 1차 |
| H-01 | escapeHtml `"` 미이스케이프 (확정적 XSS) | **High** | Safari (Chrome은 동일 패턴 시 해당) | 1차 |
| H-02 | sender 검증 없음 | **High** | 공통 | 1차 |
| H-03 | `<all_urls>` 과도한 권한 | **High** | 공통 | 1차 |
| **N-01** | **web_accessible_resources 과도한 노출** | **High** | 공통 | **2차** |
| **N-02** | **dev-tools 스크립트 전 페이지 주입** | **High** | Chrome/Edge | **2차** |
| **N-03** | **CustomEvent 확장 fingerprinting** | **Medium** | 공통 | **2차** |
| **N-04** | **Array.from 메모리 폭발 DoS** | **Medium** | 공통 | **2차** |
| **N-05** | **다운로드 인터셉터 URL 미검증** | **Medium** | Chrome/Edge | **2차** |
| M-01 | HTTP 허용 (MITM) | Medium | 공통 | 1차 |
| M-02 | CSP 미비 | Medium | 공통 | 1차 |
| L-01 | WASM 파서 입력 미검증 | Low→Medium | 공통 | 1차(2차 승격) |

---

## 1차 감사 취약점 상세

### C-01: fetch-file 오픈 프록시 (Critical)

**위치**: background.js / sw/message-router.js — `case 'fetch-file'`

**현황**: content script가 임의 URL을 background에 전달하면 확장 권한으로 fetch, CORS 우회.

**공격 시나리오**:
- 내부 네트워크 스캔 (`192.168.*`, `localhost`)
- 클라우드 메타데이터 탈취 (`169.254.169.254`)
- CORS 우회 프록시

**2차 검토에서 추가 지적**:
- **DNS Rebinding**: IP 차단만으로 부족. 공격자가 DNS를 `127.0.0.1`로 리바인딩하면 도메인 화이트리스트 통과 후 내부 접근 가능
- **리다이렉트 체이닝**: `fetch()`가 302 리다이렉트를 따라가면 화이트리스트 URL이 내부 IP로 전환. `redirect: 'manual'` 필수
- **응답 바이트 매직 넘버 검증**: Content-Type 헤더는 서버가 위조 가능. HWP(`D0 CF 11 E0`), HWPX(`50 4B 03 04`) 시그니처 검증 필요
- **스트리밍 크기 제한**: `Content-Length` 헤더만으로 판단 시 chunked transfer encoding에서 우회

---

### C-02: open-hwp URL 검증 없음 (Critical)

**위치**: background.js / sw/viewer-launcher.js — `openViewer()`

**2차 검토 추가 지적**:
- **확장자 우회**: `https://evil.com/malware.exe?file=test.hwp` → pathname만 추출 후 확장자 검증 필수
- **Open Redirect**: 정부 사이트의 redirect 취약점 경유 시 도메인 화이트리스트 통과. 최종 fetch URL 재검증 필요
- **filename 새니타이즈 범위**: path traversal(`../../`), null byte(`%00`), 유니코드 정규화 공격(homoglyph), `@` 포함 도메인(`https://safe.go.kr@evil.com`)까지 방어

---

### H-01: escapeHtml `"` 미이스케이프 — 확정적 XSS (High)

**증명**:
```javascript
const div = document.createElement('div');
div.textContent = 'x" onerror="alert(1)';
console.log(div.innerHTML);
// 출력: x" onerror="alert(1)   ← 따옴표 미이스케이프!
```

**2차 검토 추가**: textContent 전환 시 **최대 길이 제한** 필요 (title: 200자, description: 500자) — UI 깨짐 + DoS 방지

---

### H-02: sender 검증 없음 (High)

**2차 검토 구체화**:
```javascript
// fetch-file: 확장 내부 페이지만
if (sender.origin !== `chrome-extension://${chrome.runtime.id}`) return;
if (!sender.url?.includes('viewer.html')) return;

// open-hwp: content script만
if (!sender.tab) return;
```

`externally_connectable` 미설정 시 외부 접근 차단되나, 이 전제조건을 **명시적으로 문서화** 필요.

---

### H-03: `<all_urls>` 과도한 권한 (High)

**2차 검토 현실적 권고**:
```json
{
  "host_permissions": [
    "*://*.go.kr/*",
    "*://*.or.kr/*",
    "*://*.ac.kr/*",
    "*://*.mil.kr/*"
  ],
  "optional_host_permissions": ["<all_urls>"]
}
```

**주의**: `content_scripts.matches`도 동일 조정 필수. 미조정 시 host_permissions 축소가 무의미. 미등록 사이트는 `chrome.scripting.registerContentScripts()`로 동적 등록.

---

## 2차 검토에서 신규 발견된 취약점

### N-01: web_accessible_resources 과도한 노출 (High)

**위치**: manifest.json — `web_accessible_resources`
```json
"web_accessible_resources": [{
  "resources": ["wasm/*", "fonts/*", "icons/*", "dev-tools-inject.js"],
  "matches": ["<all_urls>"]
}]
```

**문제**: 모든 웹페이지에서 확장의 WASM 바이너리, 폰트, dev-tools 스크립트에 접근 가능. 확장 ID로 직접 접근하여 WASM 취약점 분석에 활용 가능. `dev-tools-inject.js` 존재로 확장 설치 **fingerprinting** 가능.

**권고**: `matches`를 확장 내부 페이지로 제한, `dev-tools-inject.js`를 프로덕션 빌드에서 제거

---

### N-02: dev-tools 스크립트 전 페이지 주입 (High)

**위치**: content-script.js:32-34
```javascript
const devScript = document.createElement('script');
devScript.src = chrome.runtime.getURL('dev-tools-inject.js');
(document.head || document.documentElement).appendChild(devScript);
```

**문제**: 모든 페이지에 `dev-tools-inject.js` 주입. 악의적 페이지가 `window.rhwpDev`를 선점하여 프로토타입 오염 가능. 확장 설치 탐지 벡터.

**권고**: 프로덕션 빌드에서 **완전 제거**, 개발자 옵션 토글로만 활성화

---

### N-03: CustomEvent 확장 fingerprinting (Medium)

**위치**: content-script.js:26-29

**문제**: `data-hwp-extension` 속성과 `hwp-extension-ready` 이벤트로 확장 설치 여부와 **정확한 버전** 탐지 가능. 알려진 취약점이 있는 특정 버전 타겟팅 공격 가능.

**권고**: 공공사이트에서만 노출 (도메인 체크), 최소한 버전 정보 제거

---

### N-04: Array.from 메모리 폭발 DoS (Medium)

**위치**: sw/message-router.js:47
```javascript
return { data: Array.from(new Uint8Array(buffer)) };
```

**문제**: 100MB `Uint8Array` → `Array.from()` → 각 바이트가 JS Number(8바이트)로 확장 → **약 800MB 메모리**. Service Worker 메모리 제한(256MB) 초과 → 크래시.

**권고**: `ArrayBuffer`를 structured clone으로 직접 전달, 또는 크기 제한을 실용적 HWP 크기(20MB)로 하향

---

### N-05: 다운로드 인터셉터 URL 미검증 (Medium)

**위치**: sw/download-interceptor.js:35-37

**문제**: `item.url`에 대한 검증 없이 `openViewer()` 직접 호출. C-02 수정이 메시지 핸들러에만 적용되면 **우회 경로**.

**권고**: URL 검증을 `openViewer()` 함수 자체에 내장

---

## 한국 공공기관 특수성 (2차 검토)

| 항목 | 내용 |
|------|------|
| 전자정부 표준프레임워크 | 다운로드 URL에 확장자 없음 (`/cmm/fms/FileDown.do?atchFileId=...`). Content-Disposition 헤더 검사 필요 |
| 국가정보원 보안적합성 | CC 인증 시 WASM 메모리 안전성 검증 필수. `unsafe` 블록 현황 문서화 |
| 개인정보보호법 제29조 | 문서 내용 외부 미전송 기술적 증명 필요 (네트워크 감사 로그, CSP connect-src 제한) |
| 장애인차별금지법 | 공공 배포 시 WCAG 2.1 AA 접근성 준수 의무 |
| HTTP 레거시 | 일부 지자체가 HTTP로 HWP 서빙. HTTPS 강제 시 다운로드 실패 가능 |

---

## 수정 우선순위 (2차 검토 반영)

| 순위 | 항목 | 이유 |
|------|------|------|
| **1** | C-01 + DNS rebinding/리다이렉트 방어 | 즉시 악용 가능한 오픈 프록시 |
| **2** | H-01 innerHTML 제거 (DOM API 전환) | 확정적 XSS |
| **3** | N-02 dev-tools 주입 제거 (프로덕션) | 모든 페이지에 스크립트 주입 + fingerprinting |
| **4** | N-01 web_accessible_resources 축소 | 확장 내부 리소스 노출 |
| **5** | H-02 sender 검증 | fetch-file 남용 2차 방어선 |
| **6** | C-02 + N-05 URL 검증 (openViewer 내장) | 모든 진입점 통합 검증 |
| **7** | N-04 메모리 폭발 방지 | DoS 벡터 |
| **8** | H-03 + content_scripts.matches 연동 | 권한 축소 실효성 |
| **9** | N-03 fingerprinting 완화 | 버전 타겟팅 방지 |
| **10** | M-01 HTTPS + M-02 CSP 강화 | 방어 심화 |
| **11** | L-01 WASM 파서 강화 (타임아웃, 메모리 제한) | 장기 과제 |

---

## CSP 최종 권고 (2차 검토)

```
script-src 'self' 'wasm-unsafe-eval';
style-src 'self' 'unsafe-inline';
object-src 'none';
base-uri 'none';
frame-src 'none';
img-src 'self' https: data:;
connect-src 'self' https:;
```

`connect-src https:`가 넓으나, optional_host_permissions와 동적 CSP 연동이 현실적으로 어려워 차선책. 이 트레이드오프를 문서화.

---

## 보안 테스트 경계 케이스 (2차 검토 추가)

URL 검증 함수의 단위 테스트에 반드시 포함할 케이스:
- 유니코드 문자: `https://evil.com/ﬁle.hwp` (fi ligature)
- Double encoding: `%2527` → `%27` → `'`
- URL fragment: `https://safe.go.kr/file.hwp#javascript:alert(1)`
- IPv6: `http://[::1]/file.hwp`
- 도메인 `@` 포함: `https://safe.go.kr@evil.com/file.hwp`
- Open redirect: `https://gov.kr/redirect?to=https://evil.com/payload`
- 파일명 path traversal: `../../etc/passwd.hwp`
- Null byte: `file.hwp%00.exe`
