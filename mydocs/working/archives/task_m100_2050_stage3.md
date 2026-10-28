# Stage 3 완료보고서 — task_m100_2050

- **이슈**: #2050  **브랜치**: `local/task2050`

## 작업: 사이드바 레이아웃 + 썸네일

### 변경 1: `hwp-editor-provider.ts` (HTML/CSS)
- `#scroll-container`를 `#app-shell`(flex row)로 감싸고 그 앞에 `#nav-sidebar` 추가. **`#scroll-container` 내부 구조·좌표계 불변** (스크롤/줌 회귀 방지).
- 사이드바: `#nav-tabs`(썸네일/목차/북마크) + `#nav-body`(패널 3개).
- 상태 표시줄 좌측에 `#stb-sidebar-toggle`(☰) 추가 — 사이드바 접기/펼치기.
- CSS 접두어 `nav-` 신설 (기존 tb-/sb-/stb- 규약과 분리). VSCode 테마 변수(`--vscode-sideBar-*`) 사용.

### 변경 2: `viewer.ts` (로직)
- 사이드바 DOM 참조 추가.
- `scrollToPage(pageNum)`: 편집 영역 좌표 기준 스크롤 (getBoundingClientRect 차이 방식).
- `buildThumbnails()`: 페이지별 `.nav-thumb`(캔버스+쪽번호). **IntersectionObserver 지연 렌더**(rootMargin 200px) — 보이는 썸네일만 `renderPageToCanvas(i, canvas, scale)`.
- `highlightCurrentThumb()`: 현재 페이지 썸네일 `.current` 강조 + 스크롤. `updateCurrentPage`에 연동.
- 탭 전환(`switchTab`), 토글 이벤트 배선.
- `buildSidebar()` 오케스트레이터 → 로드 후 호출 (Stage 4에서 목차/북마크 추가 예정).

## 인코딩 참고
- 이 파일 HTML 템플릿 리터럴은 한국어를 리터럴 `\uXXXX` 이스케이프로 저장하는 기존 컨벤션 사용. 삽입 라벨도 동일하게 저장되며 런타임에 정상 한글 렌더. 라벨 오타(썬네일→썸네일) 수정 완료.

## 검증
- `npm run compile` (webpack production) 성공, 타입 체크 통과. `dist/webview/viewer.js` 산출.

## 상태: ✅ 완료 (목차/북마크 콘텐츠는 Stage 4)
