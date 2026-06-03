# Task #76: Chrome/Edge 확장 프로그램 HWP 뷰어 & 에디터 — 최종 보고서

> **이슈**: [#76](https://github.com/edwardkim/rhwp/issues/76)
> **브랜치**: `local/task76`
> **기간**: 2026-04-07 ~ 2026-04-08
> **Discussion**: [#66](https://github.com/edwardkim/rhwp/discussions/66)

---

## 수행 목표

Chromium 계열 브라우저(Chrome, Edge)에서 HWP/HWPX 파일을 별도 프로그램 설치 없이 브라우저 내에서 열어보고 편집할 수 있는 확장 프로그램을 구현하고, Chrome Web Store와 Microsoft Edge Add-ons에 배포한다.

## 완료 요약

### 1단계: 확장 기본 구조 + rhwp-studio 탑재
- Manifest V3 확장 프로그램 프로젝트 (`rhwp-chrome/`)
- rhwp-studio 전체 기능 탑재 (WASM + Canvas 렌더링 + 편집)
- Service Worker 모듈 구조 (viewer-launcher, context-menus, download-interceptor, message-router)
- Content Script: HWP 링크 자동 감지 + `data-hwp-*` 프로토콜 + 배지 삽입
- 개발자 도구 (rhwpDev): inspect(), inspectLink(), help()
- i18n (한국어/영어), URL 파라미터 파일 자동 로드

### 2단계: 파일 연결 + 인쇄
- 드래그 & 드롭 (document 레벨 기본 드롭 방지)
- HWP 저장 (File System Access API + Blob 폴백)
- 인쇄: SVG → 브라우저 인쇄 (다중 페이지, 진행률 표시, 인쇄/닫기 버튼)
- Ctrl+P 단축키 → rhwp 인쇄 (브라우저 기본 인쇄 인터셉트)
- 다운로드 가로채기 (.hwp/.hwpx → 뷰어 자동 오픈)
- 컨텍스트 메뉴 ("rhwp로 열기")

### 3단계: Content Script 고도화
- 호버 미리보기 카드 (제목, 페이지, 크기, 작성자, 날짜, 카테고리, 설명, 썸네일)
- 사용자 설정 연동 (showBadges, hoverPreview)
- CORS 우회 파일 로딩 (Service Worker fetch 프록시)
- MutationObserver 동적 콘텐츠 대응

### 4단계: 마무리 + 스토어 배포
- 개발자 가이드 (DEVELOPER_GUIDE.md): `data-hwp-*` 프로토콜 사양, CMS 적용 가이드
- 개인정보 처리방침 (PRIVACY.md): 한국어/영어
- README.md
- 스크린샷 (Chrome/Edge 각 4장, 1280x800)
- Promotional tile: Small 440x280 + Large 1400x560
- Chrome Web Store 심사 제출
- Microsoft Edge Add-ons 심사 제출 (Store ID: 0RDCKCKDPBQR)

## 기술 아키텍처

```
rhwp-chrome/
├── manifest.json           ← Manifest V3
├── background.js           ← Service Worker 엔트리
├── sw/
│   ├── viewer-launcher.js  ← 뷰어 탭 관리
│   ├── context-menus.js    ← 우클릭 메뉴
│   ├── download-interceptor.js ← 다운로드 감지
│   └── message-router.js   ← 메시지 라우팅 + CORS 프록시
├── content-script.js/css   ← 웹페이지 HWP 링크 감지
├── dev-tools-inject.js     ← 개발자 도구 (rhwpDev)
├── options.html            ← 사용자 설정
├── _locales/ko, en/        ← 국제화
├── build.mjs               ← Vite 빌드 + 리소스 조합
├── vite.config.ts          ← rhwp-studio → 확장 빌드 설정
└── dist/                   ← 빌드 산출물 (확장 패키지)
    ├── viewer.html + assets/  ← rhwp-studio
    ├── wasm/                  ← WASM 바이너리 (3.3MB)
    └── fonts/                 ← 필수 폰트 14개 (9MB)
```

## 확장 프로그램의 핵심 차별점

| 기존 HWP 뷰어 서비스 | rhwp 확장 |
|---------------------|----------|
| 회원가입 필요 | 불필요 |
| 파일을 서버로 업로드 | 로컬 처리 (WASM) |
| 광고 있음 | 광고 없음 |
| 열람만 가능 | 편집 + 인쇄 + 저장 |
| 기업 유료 | MIT 무료 |
| 100쪽 제한 (웨일) | 제한 없음 |
| 온라인 필수 | 오프라인 동작 |

## 확장 프로그램 보안 권한 이점

| 보안 유형 | 일반 웹의 한계 | 확장에서 해결 |
|----------|--------------|-------------|
| WASM 실행 | 사이트 CSP 차단 가능 | wasm-unsafe-eval로 보장 |
| CORS 우회 | 서버 허용 필요 | host_permissions로 우회 |
| 로컬 파일 접근 | file:// 차단 | 사용자 승인 하에 접근 |
| 다운로드 제어 | 트리거만 가능 | 감지 + 자동 뷰어 오픈 |
| 웹페이지 HWP 감지 | 불가 | Content Script로 자동 감지 |

## HWP 웹 통합 프로토콜 (data-hwp-*)

공공기관 웹 개발자가 HTML에 속성 1개만 추가하면 확장이 HWP 링크를 인식:

```html
<a href="/download.do?fileId=123" data-hwp="true">공문.hwp</a>
```

개발자 디버깅 도구 제공: `rhwpDev.inspect()`, `rhwpDev.inspectLink()`, `rhwpDev.help()`

## 테스트 결과

| 항목 | Chrome | Edge |
|------|--------|------|
| 확장 등록 + 뷰어 동작 | ✓ | ✓ |
| HWP 다운로드 자동 뷰어 오픈 | ✓ | ✓ |
| 드래그 & 드롭 | ✓ | ✓ |
| 파일 저장 (Ctrl+S) | ✓ | ✓ |
| 인쇄 (Ctrl+P) + PDF 저장 | ✓ | ✓ |
| Content Script 자동 감지 + 배지 | ✓ | ✓ |
| 호버 미리보기 카드 | ✓ | ✓ |
| 개발자 도구 (rhwpDev) | ✓ | ✓ |
| 실제 정부 사이트 (korea.kr) | ✓ | ✓ |
| 상용 뷰어 대비 렌더링 품질 | 거의 동일 | 거의 동일 |

## 확인된 이슈

- [#77](https://github.com/edwardkim/rhwp/issues/77): SVG 렌더러 특정 페이지에서 탭 리더/페이지 번호 누락 (별도 이슈로 추적)

## 배포 상태

| 스토어 | 상태 | 비고 |
|--------|------|------|
| Chrome Web Store | 심사 대기 중 | |
| Microsoft Edge Add-ons | 심사 대기 중 | Store ID: 0RDCKCKDPBQR |

## 시장 영향

X(Twitter) 152건 여론 분석 결과 HWP 뷰어에 대한 불만이 56%에 달하며, 이 문제는 20년 이상 지속되어 왔다. 한글과컴퓨터 창업자 이찬진도 "제대로 된 뷰어와 컨버터라도 플랫폼별로 제공되면 불편을 줄여드릴 수 있을 텐데"라고 [언급](https://x.com/chanjin/status/1868391055747584222)한 바 있다.

rhwp Chrome/Edge 확장은 이 20년 묵은 문제에 대한 오픈소스의 답이다. Chrome + Edge 점유율 80% 이상의 한국 시장에서 HWP 파일을 다루는 모든 사용자가 대상이며, rhwp 프로젝트의 첫 일반 사용자 대상 앱이다.
