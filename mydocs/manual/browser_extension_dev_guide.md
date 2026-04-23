# 브라우저 확장 프로그램 개발 가이드 (Safari/Chrome/Edge/Firefox)

**작성일**: 2026-04-09 · **최종 갱신**: 2026-04-23
**대상**: rhwp 프로젝트 컨트리뷰터
**교훈 기반**: Task #83 Safari 확장 개발, Task #84 보안 수정, PR #169 Firefox 포팅, PR #214 rhwp-shared 공통 모듈 도입

---

## 1. Manifest V3 필수 규칙

### 인라인 스크립트 금지

MV3의 CSP는 `extension_pages`에서 인라인 스크립트를 **완전 차단**한다.

```html
<!-- ❌ 동작하지 않음 -->
<script>
  console.log('인라인 스크립트');
</script>

<!-- ✅ 올바른 방법 -->
<script src="options.js"></script>
```

**popup.html, options.html, viewer.html** 모두 해당. `<style>` 인라인은 CSP에 `'unsafe-inline'`을 명시하면 허용.

### Service Worker vs Background Scripts

| 항목 | Chrome/Edge | Firefox | Safari |
|------|-----------|---------|--------|
| 형식 | `service_worker` + `type: "module"` | `background.scripts` + `type: "module"` (Event Page 방식) | `scripts` + `persistent: false` |
| ES module import | ✅ 지원 | ✅ 지원 | ❌ 미지원 |
| 라이프사이클 | 비영속적 (유휴 시 종료) | 비영속적 | 비영속적 |
| 라이브 재시작 | 자동 | 자동 | 수동 |

Safari는 ES module을 지원하지 않으므로, **단일 파일로 번들링**하거나 별도 소스를 관리해야 한다.

Firefox MV3는 **Event Page** 방식이라 Chrome 의 Service Worker 와 구조가 다르다. `background.scripts` 에 `type: "module"` 을 붙여야 import 가능.

### CSP 설정 주의사항

```json
"content_security_policy": {
  "extension_pages": "script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; object-src 'none'; base-uri 'none'; frame-src 'none'; img-src 'self' https: data:; connect-src 'self' https: http:;"
}
```

- `'wasm-unsafe-eval'`: WASM 실행에 필수. 일반 `eval()`은 허용하지 않음
- `connect-src`: `http:`를 포함해야 HTTP 사이트 fetch 가능. **보안 검증은 JS 코드 레벨에서 수행**
- `object-src 'none'`: Flash/Java 플러그인 차단
- `base-uri 'none'`: `<base>` 태그 주입 방지

---

## 2. Safari Web Extension 특수사항

### storage API

| API | Chrome | Safari |
|-----|--------|--------|
| `storage.sync` | ✅ 안정 | ❌ **불안정 — 값이 유지되지 않음** |
| `storage.local` | ✅ 안정 | ✅ 안정 |

**Safari에서는 반드시 `storage.local`을 사용한다.** 기기 간 동기화가 필요하면 별도 구현.

### downloads API

Safari는 `chrome.downloads` / `browser.downloads` API를 **지원하지 않는다**.

대안: content-script에서 HWP 링크 클릭을 가로채어 뷰어를 연다.
```javascript
anchor.addEventListener('click', () => {
  browser.runtime.sendMessage({ type: 'open-hwp', url: anchor.href });
  // preventDefault 하지 않으면 다운로드도 동시 진행
});
```

### 변환 도구 사용법

```bash
# Chrome 확장 빌드 → Safari Xcode 프로젝트 변환
xcrun safari-web-extension-converter rhwp-chrome/dist \
  --project-location rhwp-safari \
  --app-name "HWP Viewer" \
  --bundle-identifier com.edwardkim.rhwp-safari \
  --no-open --no-prompt
```

변환 후 반드시 수정할 항목:
1. `background` 형식 변경 (`service_worker` → `scripts`)
2. ES module import 제거 (단일 파일 번들링)
3. `downloads` 권한 제거
4. `storage.sync` → `storage.local` 전환

### 개발자 서명

Safari에서 개발 중인 확장을 로드하려면:
1. **Safari → 설정 → 고급 → "웹 개발자를 위한 기능 표시"** 체크
2. **Safari → 개발 → "서명되지 않은 확장 허용"** 체크
3. Safari를 재시작할 때마다 2번을 다시 체크해야 함

---

## 3. 보안 — 반드시 지켜야 할 규칙

### innerHTML 사용 금지

```javascript
// ❌ XSS 취약 — textContent→innerHTML은 " 를 이스케이프하지 않음
const div = document.createElement('div');
div.textContent = userInput;
card.innerHTML = `<img src="${div.innerHTML}">`;
// userInput이 'x" onerror="alert(1)' 이면 XSS 발생!

// ✅ DOM API 사용
const img = document.createElement('img');
img.src = validatedUrl;  // URL 검증 필수
img.alt = 'preview';
card.appendChild(img);
```

**모든 사용자 입력(data-* 속성, URL, 파일명)은 DOM API로 처리한다.**

### fetch-file은 오픈 프록시가 될 수 있다

background에서 `fetch(message.url)`을 무검증으로 실행하면:
- 내부 네트워크 스캔 (192.168.*, localhost)
- 클라우드 메타데이터 탈취 (169.254.169.254)
- CORS 우회 프록시

**필수 검증 항목:**
1. HTTPS 프로토콜 강제 (설정으로 HTTP 허용 가능)
2. 내부 IP 차단 (127.*, 10.*, 192.168.*, 169.254.*, ::1)
3. `redirect: 'manual'` — 리다이렉트 대상 URL 재검증
4. `credentials: 'omit'` — 쿠키 전송 차단
5. 응답 매직 넘버 검증 (HWP: `D0 CF 11 E0`, HWPX: `50 4B 03 04`)
6. 파일 크기 제한
7. sender 검증 (viewer.html만 허용)

### sender 검증 필수

```javascript
// ❌ 모든 발신원의 메시지를 무조건 수락
browser.runtime.onMessage.addListener((message, sender, sendResponse) => {
  fetch(message.url); // 위험!
});

// ✅ 발신자 검증
browser.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.type === 'fetch-file') {
    // 내부 페이지(viewer.html)만 허용
    if (!sender.url?.startsWith(browser.runtime.getURL(''))) {
      sendResponse({ error: 'Unauthorized' });
      return;
    }
  }
});
```

### URL 검증 시 주의할 경계 케이스

| 공격 | 예시 | 방어 |
|------|------|------|
| userinfo 주입 | `https://safe.go.kr@evil.com/file.hwp` | `URL.username` 체크 |
| query 확장자 | `https://evil.com/mal.exe?f=test.hwp` | pathname만 확인 |
| 유니코드 | `https://evil.com/ﬁle.hwp` (fi ligature) | NFC 정규화 |
| IPv6 로컬 | `http://[::1]/file.hwp` | 패턴 매칭 |
| DNS rebinding | 정상 도메인 → 127.0.0.1 | `redirect: 'manual'` |
| 정부 다운로드 | `https://gov.kr/FileDown.do?id=123` | 허용 도메인이면 통과, 매직 넘버 재검증 |

---

## 4. 한글 인코딩 문제

### background → content-script 메시지의 한글 깨짐

Safari에서 background script의 한글 문자열이 `sendResponse`를 통해 content-script로 전달될 때 **인코딩이 깨질 수 있다**.

```javascript
// ❌ background.js에서 한글 직접 전달 — 깨질 수 있음
sendResponse({ message: '로컬 서버 접근이 차단되었습니다.' });

// ✅ 코드(영문)만 전달, 한글은 수신측에서 생성
sendResponse({ ok: false, reason: 'private-ip', hostname: 'localhost' });

// content-script.js에서 한글 메시지 생성
function getBlockedMessage(reason, hostname) {
  switch (reason) {
    case 'private-ip':
      return { title: '로컬 서버(' + hostname + ') 접근이 차단되었습니다.' };
  }
}
```

한글을 Unicode escape로 인코딩하면 더 안전하다:
```javascript
'\uB85C\uCEEC \uC11C\uBC84'  // = '로컬 서버'
```

---

## 5. UX — 사용자 경험 원칙

### 차단 시 반드시 사용자에게 알린다

```
❌ 배지 클릭 → 아무 반응 없음 (사용자 혼란)
✅ 배지 클릭 → 토스트 메시지 "로컬 서버 접근이 차단되었습니다. 설정에서 개발자 도구를 켜주세요."
```

### 명시적 행위 vs 자동 동작

| 행위 | 도메인 제한 | 이유 |
|------|-----------|------|
| 배지 클릭 (명시적) | ❌ 적용 안 함 | 사용자가 "이 파일을 열겠다"는 의도 |
| 컨텍스트 메뉴 (명시적) | ❌ 적용 안 함 | 동일 |
| 링크 자동 클릭 가로채기 | ✅ 적용 | 사용자 의도 불확실 |

### 설정은 즉시 반영, 즉시 확인 가능

- 토글 변경 → 즉시 저장 → "저장되었습니다" 피드백
- 설정 페이지 재진입 시 값이 유지되어야 함 (Safari `storage.local` 사용)

---

## 6. 디자인 — 플랫폼별 가이드라인

| 플랫폼 | 디자인 시스템 | 핵심 색상 |
|--------|------------|----------|
| Safari (macOS/iOS) | Apple HIG | `#007AFF`(Blue), `#34C759`(Green), `#FF3B30`(Red), `#86868b`(Secondary) |
| Chrome/Edge | Material Design 3 | `#1b73e8`(Blue), `#34a853`(Green), `#ea4335`(Red) |

Safari 확장의 UI는 Apple Human Interface Guidelines를 따른다:
- 세그먼트 컨트롤 스타일 탭 바 (둥근 배경, 활성 탭 흰색 카드 + 그림자)
- Apple 스타일 토글 (31px 높이, `#34C759` 초록 체크)
- 12px 둥근 모서리 카드
- `@media (prefers-color-scheme: dark)` 다크 모드 자동 대응
- `-apple-system` 폰트 패밀리

---

## 7. 빌드 파이프라인 필수 항목

### JS 문법 검사

빌드 스크립트에 `node --check`를 포함하여 문법 오류 시 빌드를 중단한다.

```bash
for jsfile in src/background.js src/content-script.js src/options.js; do
  if ! node --check "$jsfile"; then
    echo "문법 오류: $jsfile"
    exit 1
  fi
done
```

### Safari 빌드 체크리스트

1. ✅ Chrome 확장 빌드 (`npm run build`)
2. ✅ Safari 전용 dist 생성 (Chrome dist 복사 + 소스 교체)
3. ✅ JS 문법 검사 (`node --check`)
4. ✅ Xcode 프로젝트 재생성 (`safari-web-extension-converter`)
5. ✅ macOS 빌드 (`xcodebuild`)
6. ✅ Safari에서 수동 테스트

---

## 8. 테스트 체크리스트

### 기능 테스트

- [ ] 배지 표시 (HWP 링크 감지)
- [ ] 배지 클릭 → 뷰어 열기
- [ ] 호버 카드 표시/위치/전환
- [ ] 링크 클릭 → 다운로드 + 뷰어 동시
- [ ] 컨텍스트 메뉴 → 뷰어 열기
- [ ] 설정 저장/로드/반영
- [ ] 보안 로그 기록/조회/초기화

### 보안 테스트

- [ ] XSS 테스트 페이지 (`test/06-security.html`) — alert 미실행
- [ ] 내부 IP fetch 차단 (devMode OFF)
- [ ] javascript:/data: URL 차단
- [ ] 비-HWP 파일 매직 넘버 차단
- [ ] sender 검증 (외부 페이지에서 fetch-file 차단)

### 플랫폼 테스트

- [ ] macOS Safari
- [ ] iOS Safari (Simulator)
- [ ] Chrome (비교 동작 확인)
- [ ] Firefox (`about:debugging` 임시 부가 기능 로드)
- [ ] 다크 모드 UI

---

## 9. 브라우저 API 네임스페이스 차이 (`chrome.*` vs `browser.*`)

**배경**: PR #169 (Firefox 포팅) 에서 확립된 규칙. 매뉴얼 신설 섹션.

### 네임스페이스

| 브라우저 | 네임스페이스 | 반환 형태 |
|----------|------------|----------|
| Chrome / Edge | `chrome.*` | 대부분 callback (일부 Promise 혼용) |
| Firefox | `browser.*` (네이티브) | **Promise** (async/await 친화) |
| Safari | `browser.*` (polyfill) 또는 `chrome.*` | 상황에 따라 다름 |

### 포팅 시 주의

rhwp-firefox 는 `browser.*` 네임스페이스를 네이티브 사용. Chrome 코드를 가져올 때:

1. **전역 치환**: `chrome.` → `browser.`
2. **Callback → await**: `chrome.storage.sync.get({...}, cb)` → `const d = await browser.storage.sync.get({...})`
3. **`chrome.runtime.lastError` 체크 제거**: Promise rejection 으로 자연스럽게 처리
4. **Fire-and-forget**: Chrome 은 무시해도 안전하나 Firefox 는 unhandled Promise warning → `.catch(() => {})` 명시 권장

### rhwp 프로젝트의 실제 적용

- `rhwp-chrome/` — `chrome.*` 만
- `rhwp-firefox/` — `browser.*` 만
- `rhwp-safari/src/` — Safari Web Extension Converter 가 자동 처리

동일 파일을 공유할 필요는 없음. 각 확장이 자기 네임스페이스를 가지고 공통 로직만 `rhwp-shared/` 에서 공유 (아래 10절 참조).

---

## 10. 공통 모듈 공유 (`rhwp-shared/` + 심볼릭 링크)

**배경**: PR #214 에서 `download-interceptor` 의 판정 로직을 Chrome/Firefox 간 공유하기 위해 도입된 패턴. 매뉴얼 신설 섹션.

### 문제

Chrome 과 Firefox 가 서로 다른 리스너(API) 를 사용하지만, **"이 다운로드가 HWP 인가?"** 같은 **순수 판정 함수** 는 두 플랫폼에서 동일하게 동작해야 한다. 파일을 두 번 유지하면 한쪽만 수정하는 드리프트 발생.

### 해결 — 공통 원본 + 심볼릭 링크

```
rhwp-shared/
└── sw/
    ├── download-interceptor-common.js        # 원본
    └── download-interceptor-common.test.js   # 원본의 Node 테스트

rhwp-chrome/sw/
└── download-interceptor-common.js            # → ../../rhwp-shared/sw/download-interceptor-common.js (심링크)

rhwp-firefox/sw/
└── download-interceptor-common.js            # → ../../rhwp-shared/sw/download-interceptor-common.js (심링크)
```

- 개발 중: 심링크를 통해 참조 (Vite dev 서버 는 심링크 자연 지원)
- 빌드 시: `cpSync({ dereference: true })` 로 심링크를 실체 파일로 복사

### 빌드 스크립트 패턴

`rhwp-chrome/build.mjs` · `rhwp-firefox/build.mjs` 에서:

```js
copy(resolve(__dirname, 'sw'), resolve(DIST, 'sw'), {
  filter: (src) => !EXCLUDE_FROM_DIST.test(src),
  dereference: true,     // ← 심링크 → 실체 파일
});
```

`dereference: true` 없이 dist 에 복사하면 **심링크 자체가 복사되어 스토어 심사에서 거부**된다 (Chrome Web Store 는 심링크 포함 zip 을 악성으로 판정 가능).

### 테스트 전략

공통 모듈의 순수 함수는 `rhwp-shared/sw/*.test.js` 에 Node `--test` 로 작성:

```bash
node --test rhwp-shared/sw/download-interceptor-common.test.js
```

- 의존성: Node 기본 `test` 러너만 사용 (서드파티 없음)
- 플랫폼별 리스너 (onCreated/onChanged/onDeterminingFilename) 는 테스트 대상 외 — 순수 판정 함수만

---

## 11. 다운로드 가로채기의 Chrome/Firefox 구조 차이

**배경**: PR #214 에서 확립. 매뉴얼 신설 섹션.

### API 차이

| 이벤트 | Chrome/Edge | Firefox |
|--------|-----------|---------|
| 다운로드 감지 시점 | `downloads.onDeterminingFilename` (**filename 결정 직전 한 번**) | `downloads.onCreated` + `downloads.onChanged` (2단계) |
| 파일명 변경 가능 | ✅ (콜백에서 `suggest({filename})`) | 제한적 (onCreated 에서만 가능) |
| MIME / 크기 확정 시점 | onDeterminingFilename 에서 완전히 결정됨 | onCreated 에는 **미정**, onChanged 에서 확정 |

### Firefox 의 2단계 감지 패턴

```js
browser.downloads.onCreated.addListener((item) => {
  // 1차: url 로 일단 HWP 의심 여부 판정
  if (isLikelyHwp(item)) {
    pendingIds.add(item.id);
  }
});

browser.downloads.onChanged.addListener((delta) => {
  // 2차: filename / mime 확정 후 재판정
  if (!pendingIds.has(delta.id)) return;
  browser.downloads.search({id: delta.id}).then(([item]) => {
    if (shouldInterceptDownload(item)) { /* 가로채기 */ }
  });
});
```

`isLikelyHwp` 는 저비용 url 휴리스틱, `shouldInterceptDownload` 는 `rhwp-shared/` 의 공통 판정 함수.

### Chrome 의 단일 단계 패턴

```js
chrome.downloads.onDeterminingFilename.addListener((item, suggest) => {
  if (shouldInterceptDownload(item)) {
    // HWP → 기본 다운로드 + 뷰어 탭 열기
    openInViewer(item);
    return true;  // 비동기 suggest 반환 예약
  }
  // HWP 가 아니면 기본 저장 위치 기억 동작 보존
  return false;
});
```

### 교차 교훈

- **판정 함수** 는 공통 (`rhwp-shared/sw/download-interceptor-common.js`) — `shouldInterceptDownload(item)`
- **리스너 구조** 는 플랫폼별 (Chrome 단일 / Firefox 2단계)
- 동일 판정을 두 리스너에서 호출하므로 로직 일관성 보장
