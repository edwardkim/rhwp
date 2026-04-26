# 브라우저 확장 빌드 및 배포 매뉴얼 (Chrome/Edge/Firefox/Safari)

> 2026-04-23 확장: 원래 Chrome/Edge 전용이었으나 v0.2.1 사이클에서 Firefox (PR #169) 와 기존 Safari 가 합류하여 통합 매뉴얼로 재구성. 각 브라우저별 빌드·배포 경로는 공유 부분(WASM/폰트/Vite)과 고유 부분으로 나뉜다.

## 1. 사전 준비

### 필수 환경

| 항목 | 요구사항 |
|------|---------|
| Node.js | v20 이상 |
| npm | v10 이상 |
| WASM 빌드 | `pkg/` 폴더에 WASM 빌드 완료 상태 |
| 웹폰트 | `web/fonts/` 폴더에 woff2 파일 존재 |

### WASM 빌드 (아직 안 되어 있는 경우)

```bash
docker compose --env-file .env.docker run --rm wasm
```

---

## 2. 확장 프로그램 빌드

### 2.1 의존성 설치

```bash
cd rhwp-chrome
npm install
```

### 2.2 빌드 실행

```bash
npm run build
```

빌드 스크립트(`build.mjs`)가 다음을 수행한다:

1. **Vite 빌드** — rhwp-studio를 확장용으로 빌드 → `dist/`
2. **확장 파일 복사** — manifest.json, background.js, content-script, sw/, _locales/, 아이콘
3. **DevTools 주입** — viewer.html에 dev-tools-inject.js 스크립트 태그 삽입
4. **WASM 복사** — `pkg/` → `dist/wasm/`
5. **폰트 복사** — `web/fonts/`에서 필수 폰트 14개 → `dist/fonts/`

### 2.3 빌드 결과

```
rhwp-chrome/dist/          ← 이 폴더가 확장 프로그램 패키지
├── manifest.json
├── background.js
├── sw/                    ← Service Worker 모듈
├── viewer.html            ← rhwp-studio (편집기)
├── assets/                ← JS/CSS/WASM (Vite 빌드)
├── content-script.js/css  ← 웹페이지 HWP 링크 감지
├── dev-tools-inject.js    ← 개발자 도구
├── options.html           ← 사용자 설정
├── _locales/              ← i18n (한국어/영어)
├── icons/                 ← 브랜드 아이콘
├── fonts/                 ← 필수 웹폰트 (14개)
├── wasm/                  ← WASM 바이너리
├── images/                ← 툴바 아이콘 SVG
└── favicon.ico
```

### 2.4 빌드 크기 (2026-04-23 실측)

| 항목 | 크기 |
|------|------|
| WASM 바이너리 (`wasm/rhwp_bg.wasm`) | ~3.9MB (EMF/OLE/Chart 네이티브 렌더링 포함 · PR #221) |
| 폰트 (`fonts/`) | ~9MB (14개 woff2) |
| JS/CSS/HTML 번들 (`assets/`) | ~8.7MB |
| 전체 (`rhwp-chrome/dist/` · `rhwp-firefox/dist/`) | **~23MB** |

> 값은 `du -sh rhwp-chrome/dist` 실측. WASM 크기는 PR 머지에 따라 변동 가능 — 실측 갱신은 `cd rhwp-chrome/dist && du -sh`.

---

## 3. 로컬 테스트 (개발 모드)

### 3.1 Chrome에서 테스트

1. Chrome 주소창에 `chrome://extensions` 입력
2. 우측 상단 **개발자 모드** 토글 활성화
3. **압축 해제된 확장 프로그램을 로드합니다** 클릭
4. `rhwp-chrome/dist/` 폴더 선택
5. 확장 아이콘이 툴바에 표시되면 설치 완료

### 3.2 Edge에서 테스트

1. Edge 주소창에 `edge://extensions` 입력
2. 좌측 하단 **개발자 모드** 토글 활성화
3. **압축 풀린 항목 로드** 클릭
4. `rhwp-chrome/dist/` 폴더 선택

### 3.3 코드 수정 후 리로드

코드 수정 → 빌드 → 확장 리로드 순서:

```bash
# 1. 빌드
npm run build

# 2. 브라우저에서 리로드
#    chrome://extensions → 확장 카드의 새로고침(↻) 버튼 클릭
```

> **주의**: `dist/` 폴더는 WSL 안에 있으므로, Windows 호스트의 Chrome/Edge에서 테스트하려면 `dist/` 폴더를 Windows 쪽으로 복사해야 한다.

### 3.4 테스트 페이지

`rhwp-chrome/test/` 폴더에 5개 테스트 페이지가 있다. Live Server(5500 포트)로 실행:

```
http://localhost:5500/rhwp-chrome/test/index.html
```

| 테스트 | 검증 항목 |
|--------|----------|
| 01-auto-detect.html | 확장자 기반 HWP 링크 자동 감지 |
| 02-data-hwp-protocol.html | data-hwp-* 프로토콜 메타데이터 인식 |
| 03-dynamic-content.html | AJAX 동적 콘텐츠 MutationObserver |
| 04-devtools.html | rhwpDev 개발자 도구 |
| 05-gov-site-sim.html | 공공기관 게시판 시뮬레이션 |
| 06-security.html | CSP · XSS · 메시지 검증 등 보안 회귀 점검 |

> rhwp-firefox 는 동일한 6개 + `test/index.html` 허브가 있어 한 화면에서 모두 열람 가능.
| 06-security.html | CSP · XSS · 메시지 검증 등 보안 회귀 점검 |

> rhwp-firefox 는 동일한 6개 + `test/index.html` 허브가 있어 한 화면에서 모두 열람 가능.

### 3.5 디버깅

- **Service Worker 디버깅**: `chrome://extensions` → 확장 카드의 "서비스 워커" 링크 클릭
- **Content Script 디버깅**: 웹페이지에서 F12 → Console (content-script.js 로그 확인)
- **뷰어 디버깅**: 뷰어 탭에서 F12 → Console
- **개발자 도구**: 콘솔에서 `rhwpDev.inspect()` 실행

---

## 4. 스토어 배포

### 4.1 배포 에셋 준비

| 에셋 | 크기 | 위치 |
|------|------|------|
| 아이콘 | 128x128 px | `rhwp-chrome/icons/icon-128.png` |
| 로고 | 300x300 px | `assets/logo/logo-300.png` |
| 스크린샷 | 1280x800 px (최소 1장) | `assets/chrome/` 또는 `assets/edge/` |
| Small promotional tile | 440x280 px | `assets/chrome/promo-small-440x280.png` |
| Large promotional tile | 1400x560 px | `assets/chrome/promo-large-1400x560.png` |

#### 스크린샷 캡처 방법

1. F12 (개발자 도구) → Ctrl+Shift+M (디바이스 툴바)
2. 상단 크기 입력란에 **1280 x 800** 입력
3. 우측 ⋮ → **Capture screenshot**

### 4.2 Chrome Web Store 배포

#### 사전 요구사항

- Google 개발자 계정 등록 ($5 일회성)
- https://chrome.google.com/webstore/devconsole

#### 패키지 생성

```bash
cd rhwp-chrome/dist
zip -r ../rhwp-chrome.zip .
```

#### 제출 절차

1. [Chrome Developer Dashboard](https://chrome.google.com/webstore/devconsole) 접속
2. **새 항목** → `rhwp-chrome.zip` 업로드
3. 스토어 등록 정보 입력:
   - 이름: `rhwp - HWP Document Viewer & Editor` (영어) / `rhwp - HWP 문서 뷰어 & 에디터` (한국어)
   - 설명: 상세 기능 설명 (영어/한국어)
   - 카테고리: **Productivity**
   - 언어: Korean + English
4. 개인정보 보호:
   - 전용 목적: "HWP/HWPX 한글 문서 파일을 웹브라우저에서 열람하고 편집할 수 있도록 하는 문서 뷰어 및 에디터입니다."
   - 각 권한 사용 근거 입력 (activeTab, downloads, contextMenus, clipboardWrite, storage, host_permissions)
   - 원격 코드: 아니오
   - 개인정보처리방침 URL: `https://github.com/edwardkim/rhwp/blob/main/rhwp-chrome/PRIVACY.md`
5. 스크린샷/프로모션 이미지 업로드
6. **검토를 위해 제출**

#### 심사 소요 시간

- 일반적으로 1~3 영업일
- 거부 시 사유 확인 후 수정하여 재제출

### 4.3 Microsoft Edge Add-ons 배포

#### 사전 요구사항

- Microsoft 파트너 센터 계정
- https://partner.microsoft.com/dashboard/microsoftedge/

#### 제출 절차

1. [Edge 파트너 센터](https://partner.microsoft.com/dashboard/microsoftedge/) 접속
2. **확장 만들기** → `rhwp-chrome.zip` 업로드 (Chrome과 동일 패키지)
3. 등록 정보 입력:
   - 이름/설명 (영어/한국어)
   - 카테고리: **Productivity**
   - Markets: 241개 전체 (기본값)
   - 검색어: `HWP 뷰어`, `HWP viewer`, `HWPX viewer`, `한글 뷰어`, `HWP editor`, `한글 문서 열기`, `HWP 파일`
4. 개인정보처리방침 URL 입력
5. Notes for certification 입력 (테스트 방법 안내)
6. **검토를 위해 제출**

#### 심사 소요 시간

- 일반적으로 1~2 영업일

### 4.4 Firefox — rhwp-firefox 빌드 + AMO 제출

#### 빌드

```bash
cd rhwp-firefox
npm install
npm run build
```

- 빌드 산출물은 `rhwp-firefox/dist/`
- Chrome 과 동일한 Vite 번들 + WASM + 폰트 + 심볼릭 링크 dereference (`rhwp-shared/sw/download-interceptor-common.js` 실체화)
- Firefox MV3 요구사항:
  - `manifest.json` 의 `browser_specific_settings.gecko.id` 필수
  - `background.scripts` + `type: "module"` (Chrome 의 `service_worker` 방식과 다름)

#### 개발 모드 로드

1. Firefox 주소창 `about:debugging#/runtime/this-firefox`
2. **"임시 부가 기능 로드..."** → `rhwp-firefox/dist/manifest.json` 선택
3. 확장 콘솔: 해당 확장 → **"검사"** 버튼
4. 임시 부가 기능은 Firefox 재시작 시 자동 제거됨 → 재테스트 시 재로드

#### AMO (addons.mozilla.org) 제출

```bash
cd rhwp-firefox/dist
zip -r ../rhwp-firefox.zip .
```

- [AMO Developer Hub](https://addons.mozilla.org/developers/) 접속
- **Submit a New Add-on** → Self-distribution 또는 Listed 선택
- `rhwp-firefox.zip` 업로드
- AMO 자동 검증 (`web-ext lint`) 통과 필요
- 심사: 일반적으로 1~5 영업일 (수동 검증 대기열 상황 따라 변동)

> 자동 검증 실패 항목 디버깅: `npx web-ext lint` 를 `rhwp-firefox/dist/` 에서 로컬 실행 가능.

#### AMO 제출 4대 함정 (2026-04-23 rhwp-firefox v0.2.1 실측)

신규 제출 시 `web-ext lint` 는 통과해도 AMO 서버 단에서 연속 거부될 수 있다. 다음 4가지를 사전 점검하면 한 번에 통과 가능:

| # | 에러 | 원인 | 해결책 |
|---|------|------|--------|
| 1 | `data_collection_permissions property is required` | Firefox 140 에서 시행된 신규 필수 키 | `manifest.json` 의 `browser_specific_settings.gecko.data_collection_permissions.required = ["none"]` 선언 (실제 수집 없음 표명). lint 는 경고 내지만 AMO 요구가 우선 |
| 2 | `Unknown strict_min_version 999.0 for Firefox Android` | placeholder 버전 숫자 거부 — AMO 는 실존 Gecko 버전 (https://addons.mozilla.org/api/v5/applications/firefox/) 만 인정 | Android 옵트아웃은 `gecko_android` 키를 **완전 생략**. MDN 공식: "`gecko_android` 없으면 desktop only" |
| 3 | 중복된 부가 기능 ID가 발견됨 | 같은 gecko id 가 AMO 에 이미 등록됨 (타인 선점 또는 본인 이전 draft) | id 에 **플랫폼 suffix 포함** — 예: `rhwp@...` → `rhwp-firefox@...`. 첫 제출 전 AMO 공개 API 로 id 중복 확인: `curl https://addons.mozilla.org/api/v5/addons/addon/{id}/` |
| 4 | Source code submission required | minified/번들 JS 또는 WASM 바이너리 포함 시 AMO 가 소스 제출 요구. **GitHub URL 불가** — 파일 업로드 필수 | `git archive` 로 samples 제외 zip 생성: `git archive --format=zip --prefix=rhwp-source/ HEAD ':(exclude)samples' -o rhwp-source-{sha}.zip` (91MB → 37MB) |

#### Reviewer Notes (권한 정당화 + WASM 안전성)

AMO 심사자 대상 설명은 **영문** 권장 (검토자 국적 분산). 필수 포함 항목:

1. **확장이 하는 일** — 한 단락
2. **테스트 방법** — 심사자가 실제로 눌러볼 수 있게 샘플 HWP 링크 제공 (저장소 samples 경로)
3. **권한 사용 근거** — `<all_urls>` · `downloads` · `contextMenus` · `storage` · `clipboardWrite` · `activeTab` 각각
4. **WASM 설명** — "샌드박스 내 실행", "네트워크 요청 없음", "Rust 오픈소스 빌드"
5. **심볼릭 링크 해명** — `rhwp-shared/sw/` 는 소스에만 있고 dist 에는 실체 파일. `cpSync({ dereference: true })` 로 빌드 시 해제.
6. **제한 사항 명시** — HWPX 직접 저장 비활성 · Firefox Android 미지원 (근거: `downloads` API v79 제거, `contextMenus` 미지원)
7. **데이터 수집** — "없음" 선언 확인
8. **오픈소스** — 저장소 URL + 라이선스 + 제출 commit SHA

템플릿: `mydocs/release/amo_submission_v0.2.1.md` §3 참조.

#### 빌드 지침 (소스 zip 과 함께 제출 시)

AMO 는 소스 업로드와 별개로 **빌드 지침 텍스트** 를 요구한다. 심사자가 `diff` 로 업로드 zip 과 재빌드 결과를 비교하기 위함.

필수 포함:
- OS · Node.js · npm · Docker 버전 요구사항
- 소스 디렉토리 구조 설명 (rhwp-firefox / rhwp-shared / rhwp-studio / pkg / web/fonts / src / Cargo.toml)
- 재현 절차 (npm install → npm run build → dist/ 확인)
- 검증 방법 (`diff -r` 또는 zip 재생성 후 바이트 비교)
- 주의 사항 (Vite asset 해시 미세 차이, 심링크 → 실체 파일 변환)

템플릿: `mydocs/release/amo_submission_v0.2.1.md` 및 `mydocs/manual/memory/feedback_amo_submission_gotchas.md`.


### 4.5 Safari — rhwp-safari (macOS 전용)

Safari 확장은 macOS 환경에서만 빌드 가능하다 (`xcodebuild` + `safari-web-extension-converter` 의존).

```bash
cd rhwp-safari
./build.sh
```

내부적으로:
1. rhwp-chrome 빌드 호출 → `dist/` 재사용
2. Safari 전용 소스 교체 (background / content-script / manifest / options 4종)
3. Chrome 전용 파일 제거 (`sw/`, `dev-tools-inject.js`)
4. `xcrun safari-web-extension-converter` 로 Xcode 프로젝트 생성 (최초 1회)
5. `xcodebuild` 로 macOS 빌드

#### App Store 제출

- App Store Connect 계정 필요
- TestFlight 내부 테스트 → 심사 제출
- 상세 매뉴얼: 별도 (맥 환경 보유 시 작성 예정)

---

## 5. 버전 업데이트

### 5.1 버전 번호 변경

다음 파일의 버전을 일괄 변경한다:

| 파일 | 필드 |
|------|------|
| `rhwp-chrome/manifest.json` | `"version"` |
| `rhwp-chrome/package.json` | `"version"` |
| `rhwp-chrome/dev-tools-inject.js` | `VERSION` 상수 |
| `rhwp-chrome/content-script.js` | `data-hwp-extension-version` |

### 5.2 업데이트 빌드 및 제출

```bash
cd rhwp-chrome
npm run build
cd dist
zip -r ../rhwp-chrome.zip .
```

- **Chrome**: Developer Dashboard → 기존 항목 → **패키지** 탭 → 새 버전 업로드
- **Edge**: 파트너 센터 → 기존 확장 → **패키지 업데이트** → 새 zip 업로드

---

## 6. 트러블슈팅

### CSP 오류

확장 페이지에서 인라인 스크립트(`onclick` 등)를 사용하면 CSP 위반 오류가 발생한다.

```
Refused to execute inline event handler because it violates the following
Content Security Policy directive: "script-src 'self' 'wasm-unsafe-eval'"
```

**해결**: 인라인 핸들러 대신 `element.addEventListener()` 사용.

### WASM 로딩 실패

```
WebAssembly.instantiate(): Refused to compile or instantiate WebAssembly module
```

**해결**: `manifest.json`의 CSP에 `wasm-unsafe-eval`이 포함되어 있는지 확인.

```json
"content_security_policy": {
  "extension_pages": "script-src 'self' 'wasm-unsafe-eval'; object-src 'self'"
}
```

### Content Script가 동작하지 않음

- `chrome://extensions`에서 확장이 활성화되어 있는지 확인
- 확장 페이지(`chrome-extension://`)에서는 Content Script가 주입되지 않음 (Chrome 정책)
- `file://` URL에서 동작하려면 확장 설정에서 "파일 URL에 대한 액세스 허용" 활성화 필요

### 빌드 시 Vite 버전 충돌

프로젝트 루트에서 `npx vite`를 실행하면 글로벌 최신 버전을 설치하려 할 수 있다. 반드시 `rhwp-studio/node_modules`의 vite를 사용해야 한다. `build.mjs`가 이를 자동 처리한다.

### Service Worker 디버깅

MV3 Service Worker는 비영속적이므로:
- 전역 변수에 상태 저장 불가 → `chrome.storage` 사용
- 30초 이벤트 타임아웃 → 긴 작업은 뷰어 탭에 위임
- `chrome://extensions` → "서비스 워커" 링크로 DevTools 접근
