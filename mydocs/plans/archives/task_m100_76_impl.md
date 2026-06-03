# Task #76: Chrome/Edge 확장 프로그램 — 구현 계획서

## 1단계: 확장 프로그램 기본 구조 + rhwp-studio 탑재

### 목표
Manifest V3 기반 확장 프로그램 프로젝트를 생성하고, rhwp-studio를 빌드하여 확장 내에서 HWP 파일을 열어볼 수 있는 최소 동작을 확인한다.

### 작업 내용

1. `rhwp-chrome/` 디렉터리 생성
2. `manifest.json` 작성 (Manifest V3)
   - `wasm-unsafe-eval` CSP 설정 (WASM 실행 보장)
   - `web_accessible_resources` — WASM, 폰트, 아이콘
   - `permissions` — `activeTab`, `downloads`, `clipboardWrite`
   - `action` — 팝업 또는 뷰어 페이지 열기
3. rhwp-studio Vite 빌드 설정 수정
   - `vite.config.ts`에 Chrome 확장용 빌드 모드 추가
   - 빌드 산출물을 `rhwp-chrome/dist/`로 출력
   - WASM + 폰트를 확장 번들에 포함
4. `viewer.html` — rhwp-studio의 `index.html`을 확장 환경에 맞게 구성
5. 로컬 HWP 파일 열기 동작 확인
   - 확장 아이콘 클릭 → 뷰어 탭 오픈
   - 파일 선택 → WASM 파싱 → Canvas 렌더링
6. 폰트 번들링 전략
   - 필수 폰트만 선별 (Pretendard, NotoSerifKR, GowunBatang, NanumGothic 등)
   - WASM 3.3MB + 필수 폰트 ~3MB = 확장 전체 ~7MB 목표

### 산출물
- `rhwp-chrome/` 프로젝트 구조
- 확장 설치 → 파일 열기 → Canvas 렌더링 + 편집 동작 확인

---

## 2단계: 파일 연결 + 다운로드 가로채기

### 목표
웹에서 HWP/HWPX 파일 다운로드 시 자동으로 뷰어 탭에서 열고, 로컬 파일 드래그&드롭, 컨텍스트 메뉴를 지원한다.

### 작업 내용

1. `background.js` (Service Worker) 구현
   - `chrome.downloads.onDeterminingFilename` — `.hwp`/`.hwpx` 다운로드 감지
   - 다운로드 가로채기 → viewer.html에서 자동 열기
   - 사용자 설정: 자동 열기 on/off (기본값: on)
2. 드래그 & 드롭 지원
   - viewer.html에 drag-over/drop 이벤트 핸들러
   - 빈 뷰어 탭에 파일 드래그 시 바로 열기
3. `file://` 프로토콜 지원
   - 로컬 `.hwp` 파일을 브라우저에서 열면 뷰어로 표시
   - `manifest.json`에 `file_handlers` 설정
4. 컨텍스트 메뉴 추가
   - HWP 링크 우클릭 → "rhwp로 열기" 메뉴
   - `chrome.contextMenus.create()` 구현
5. 인쇄 기능
   - Canvas → 인쇄용 레이아웃 생성
   - `window.print()` 연동
6. HWP 저장 (편집 후)
   - `exportHwp()` → `chrome.downloads.download()`로 파일 저장
   - 저장 경로 지정 가능

### 산출물
- .hwp/.hwpx 다운로드 시 자동 뷰어 오픈
- 드래그&드롭, 컨텍스트 메뉴, 인쇄, 저장 동작

---

## 3단계: Content Script + HWP 웹 통합 프로토콜

### 목표
공공 웹사이트 방문 시 페이지 내 HWP/HWPX 링크를 자동 감지하고, `data-hwp-*` 프로토콜을 지원하여 최적의 사용자 경험을 제공한다.

### 작업 내용

1. `content-script.js` 구현 — 자동 감지 모드
   - 페이지 로드 시 DOM 스캔: `a[href$=".hwp"], a[href$=".hwpx"]` 감지
   - 감지된 링크 옆에 rhwp 아이콘 배지 삽입
   - 아이콘 클릭 시 뷰어 탭에서 열기
   - MutationObserver로 동적 콘텐츠(게시판 AJAX 로딩) 대응
2. `data-hwp-*` 프로토콜 지원
   - `data-hwp="true"` 마커 인식 (1순위)
   - `data-hwp-title`, `data-hwp-pages`, `data-hwp-size` → 정보 카드 표시
   - `data-hwp-thumbnail` → 인라인 미리보기 이미지
   - `data-hwp-category` → 카테고리별 아이콘 분류
3. 호버 미리보기
   - 링크 호버 300ms 후 팝업 표시
   - `data-hwp-thumbnail` 있으면 이미지 표시
   - 없으면 메타데이터 카드 (제목, 페이지 수, 크기)
4. 페이지 수준 `<meta>` 태그 지원
   - `hwp:integration`, `hwp:hover-preview`, `hwp:batch-enabled`
5. 확장 존재 알림
   - `document.documentElement`에 `data-hwp-extension="rhwp"` 속성 주입
   - `hwp-extension-ready` CustomEvent 발행
6. CORS 우회 파일 로딩
   - `host_permissions`로 공공사이트 HWP 파일 직접 fetch
   - Background Service Worker를 통한 프록시 패턴

### 산출물
- 공공 웹사이트에서 HWP 링크 자동 감지 + 아이콘 표시
- `data-hwp-*` 프로토콜 동작
- 호버 미리보기

---

## 4단계: 마무리 + 스토어 배포

### 목표
확장 프로그램을 마무리하고, Chrome Web Store와 Microsoft Edge Add-ons에 배포한다. 공공 웹사이트 개발자 가이드를 제공한다.

### 작업 내용

1. 확장 아이콘/에셋
   - 아이콘 제작 (16, 32, 48, 128px)
   - Chrome Web Store 스크린샷 (1280x800, 최소 1장)
   - Edge Add-ons 스크린샷
   - 프로모션 타일 이미지
2. 오류 처리 강화
   - 손상된 HWP 파일 → 친절한 에러 메시지
   - WASM 로딩 실패 → 폴백 안내
   - 대용량 파일 경고 (50MB 초과)
3. 성능 최적화
   - WASM lazy loading (첫 파일 오픈 시 로드)
   - 가시 페이지만 렌더링 (가상 스크롤)
   - 폰트 lazy loading (사용 시 로드)
4. 사용자 설정 페이지 (`options.html`)
   - 자동 열기 on/off
   - 호버 미리보기 on/off
   - 기본 동작 선택 (뷰어/다운로드)
5. 개발자 가이드 문서 작성
   - `data-hwp-*` 프로토콜 사양서
   - 적용 수준별 예제 (최소/표준/고급)
   - CMS별 적용 가이드 (그누보드, XpressEngine 등)
6. 개인정보 처리방침 작성
   - 파일이 서버로 전송되지 않음 명시
   - 수집하는 데이터 없음
7. 스토어 배포
   - Chrome Web Store 개발자 등록 + 심사 제출
   - Microsoft Edge Add-ons 제출 (동일 패키지)
   - README.md 작성

### 산출물
- Chrome Web Store + Edge Add-ons 배포 완료
- 공공 웹사이트 개발자 가이드
- 개인정보 처리방침
