# Task #76: Chrome/Edge 확장 프로그램 — 1단계 완료 보고서

## 완료 항목

### Manifest V3 확장 프로그램 기본 구조
- `rhwp-chrome/` 프로젝트 생성
- `manifest.json` — CSP `wasm-unsafe-eval`, 권한(activeTab, downloads, contextMenus, clipboardWrite, storage), Content Script, i18n
- 국제화 — `_locales/ko`, `_locales/en` 메시지 파일

### Service Worker 모듈 구조
- `background.js` — 엔트리 포인트 (이벤트 등록)
- `sw/viewer-launcher.js` — 뷰어 탭 열기/재활용
- `sw/context-menus.js` — HWP 링크 우클릭 메뉴
- `sw/download-interceptor.js` — .hwp/.hwpx 다운로드 감지 → 뷰어 자동 오픈
- `sw/message-router.js` — Content Script ↔ SW 메시지 라우팅 + CORS 우회 fetch

### rhwp-studio 탑재
- Vite 빌드 설정 (`vite.config.ts`) — rhwp-studio를 확장용으로 빌드
- `build.mjs` — Vite 빌드 + WASM/폰트/확장 파일 조합 스크립트
- `viewer.html` — rhwp-studio 전체 기능 (뷰잉 + 편집)
- URL 파라미터 로딩 (`?url=`) — `loadFromUrlParam()` 구현

### Content Script
- `.hwp`/`.hwpx` 확장자 기반 자동 감지
- `data-hwp="true"` 마커 인식
- HWP 링크 옆 배지(H) 자동 삽입
- MutationObserver — 동적 콘텐츠 대응
- 확장 존재 알림 (`data-hwp-extension="rhwp"`)

### 개발자 도구 (rhwpDev)
- `dev-tools-inject.js` — 페이지 + 뷰어 탭 모두에서 사용 가능
- `rhwpDev.inspect()` — 페이지 전체 검사
- `rhwpDev.inspectLink()` — 개별 링크 검사
- `rhwpDev.help()` — 프로토콜 가이드 출력

### 리소스 번들링
- WASM 바이너리 (3.3MB)
- 필수 폰트 14개 (NotoSansKR, NotoSerifKR, Pretendard, GowunBatang, GowunDodum, NanumGothic, NanumMyeongjo, D2Coding)
- 브랜드 아이콘 (assets/logo → 16/32/48/128px)
- toolbar/menu 아이콘 SVG, favicon

### 사용자 설정
- `options.html` — 자동 열기/배지 표시/호버 미리보기 on/off

## 테스트 결과

### 테스트 페이지 (5개 모두 통과)
1. **자동 감지** — .hwp/.hwpx 확장자, 쿼리스트링 포함, 비-HWP 제외 ✓
2. **data-hwp-* 프로토콜** — 최소/표준/고급 적용, 메타데이터 툴팁 ✓
3. **동적 콘텐츠** — AJAX 게시판 시뮬레이션, MutationObserver ✓
4. **개발자 도구** — rhwpDev.inspect(), inspectLink(), help() ✓
5. **정부24 시뮬레이션** — 공공기관 게시판 환경, 혼합 링크 ✓

### 실제 환경 테스트
- Windows 11 Chrome — 확장 등록 + 뷰어 동작 ✓
- Windows 11 Edge — 확장 등록 + 뷰어 동작 ✓
- 실제 정부 사이트(korea.kr) — HWP 다운로드 시 뷰어 자동 오픈 ✓
- 재정경제부 보도자료 렌더링 — 상용 뷰어와 거의 동일한 결과 ✓

## 확인된 제한 사항
- `/download.do?fileId=123` 패턴 — 확장자 없는 URL은 자동 감지 불가 → `data-hwp-*` 프로토콜로 해결 (개발자 가이드 배포 필요)

## 빌드 산출물
- `rhwp-chrome/dist/` — 약 17MB (WASM 3.3MB + 폰트 9MB + JS/CSS/HTML 4MB + 기타)
