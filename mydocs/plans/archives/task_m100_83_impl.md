# Task #83: Safari 확장 프로그램 HWP 뷰어 — 구현 계획서 (v2 갱신)

## 현재 아키텍처

### Chrome 확장 (`rhwp-chrome/`)
- Manifest V3, background.js (service_worker + ES module) + content-script.js
- WASM Canvas 렌더링 (rhwp_bg.wasm)
- 오픈소스 폰트 14종 번들 (woff2)
- Vite 기반 빌드 (`build.mjs`)

### Safari 확장 (`rhwp-safari/`)
- Chrome dist 기반, Safari 전용 소스로 교체 (`src/`)
- `browser.*` API (WebExtension 표준)
- `storage.local` 사용 (Safari `storage.sync` 불안정)
- Apple HIG 디자인 옵션 페이지

### 공통 보안 모듈 (`rhwp-shared/security/`)
- url-validator.js, filename-sanitizer.js, file-signature.js
- sender-validator.js, security-log.js, constants.js

---

## 구현 단계 (6단계, 실제 진행 반영)

### 1단계: Chrome dist → Safari 전용 dist 분리 ✅

**작업 완료:**
- `rhwp-chrome/` `npm run build` → `dist/` 생성
- `rhwp-safari/dist/`에 복사 후 Safari 전용 소스 교체
- `build.sh` 빌드 스크립트 작성

---

### 2단계: Safari 전용 background.js 재작성 ✅

**작업 완료:**
- ES module 제거, 단일 파일
- `browser.*` API 전용
- `storage.local` 사용
- URL 3단계 검증 (프로토콜 → 내부 IP → 확장자/도메인)
- fetch-file: `redirect: 'manual'`, `credentials: 'omit'`, 매직 넘버 검증
- sender 검증: fetch-file은 내부 페이지만, open-hwp은 content script만
- openViewer: `explicit` 플래그로 명시적/자동 구분
- 보안 이벤트 로깅 (250건 FIFO)

---

### 3단계: Safari 전용 content-script.js 재작성 ✅

**작업 완료:**
- `browser.*` API 전용
- innerHTML 완전 제거 → DOM API (`createElement`, `textContent`)
- 썸네일 URL 스킴 검증, 텍스트 길이 제한
- fingerprinting 완화 (허용 도메인에서만 확장 노출, 버전 제거)
- 차단 시 토스트 알림 (reason 코드 → content-script에서 한글 메시지 생성)
- 호버 카드: 뷰포트 넘침 방지, 링크 전환 시 타이머 충돌 해결

---

### 4단계: 옵션 페이지 + 보안 로그 ✅

**작업 완료:**
- Apple HIG 디자인 (세그먼트 컨트롤 탭, Apple 토글, 다크 모드)
- 인라인 스크립트 분리 (`options.js`) — MV3 CSP 준수
- 5개 탭: 사이트 | 기능 | 보안 | 개발 | 로그
- 사이트 허용 목록 CRUD, 도메인 추가/삭제
- 보안 로그 조회/초기화 (테이블 UI, 유형별 색상 뱃지)
- `storage.local` 즉시 저장/반영

---

### 5단계: macOS + iOS 빌드 테스트 (예정)

**작업 내용:**
1. macOS Safari 전체 기능 통합 테스트
   - 배지 표시, 호버 카드, 배지 클릭 → 뷰어
   - 링크 클릭 → 다운로드 + 뷰어 동시
   - 설정 저장/반영, 보안 로그
2. 보안 테스트 (`test/06-security.html`)
   - XSS 6종 공격 시도 → alert 미실행 확인
3. iOS Simulator 빌드 + 동작 확인

---

### 6단계: App Store 배포 준비 (예정)

**작업 내용:**
1. Apple Developer Program 계정
2. 앱 아이콘 (1024x1024 등 필수 사이즈)
3. 스크린샷 (macOS + iOS 각 사이즈)
4. App Store Connect 메타데이터
5. 개인정보 처리방침 URL
6. 심사 제출

---

## 빌드 파이프라인 (`rhwp-safari/build.sh`)

```
[1/6] Chrome 확장 빌드 (npm run build)
[2/6] Safari dist 생성 (Chrome dist 복사)
[3/6] JS 문법 검사 (node --check)
[4/6] Safari 전용 소스 적용 (src/ → dist/)
[5/6] Xcode 프로젝트 생성 (safari-web-extension-converter)
[6/6] macOS 빌드 (xcodebuild)
```

## 리스크 (계획 시 예측 vs 실제 발생)

| 예측 | 실제 |
|------|------|
| WASM CSP 제한 | CSP `connect-src`에서 `http:` 차단 → manifest 수정 |
| Downloads API 미지원 | content-script 클릭 가로채기로 대체 |
| (미예측) storage.sync 불안정 | `storage.local`로 전환 |
| (미예측) 인라인 스크립트 차단 | options.js 분리 |
| (미예측) 한글 인코딩 깨짐 | reason 코드 방식으로 해결 |
| (미예측) 보안 취약점 13건 | Task #84로 분기, 공통 보안 모듈 구축 |
