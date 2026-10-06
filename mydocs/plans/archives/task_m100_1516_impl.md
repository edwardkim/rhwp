# Task M100 #1516 구현 계획

## 1. Firefox adapter 정리

- `rhwp-firefox/sw/download-observer-state.js`를 shared 파일 symlink로 추가한다.
- `download-interceptor.js`에서 전역 `seen`/`handled`/`workerStartedAt` 기반 로직을 제거한다.
- `evaluateDownloadCreated()` / `evaluateDownloadChanged()` / `markDownloadHandled()` / `markDownloadTerminal()`를 사용한다.

## 2. 상태 저장

- 상태 key는 Chrome과 같은 `rhwpDownloadState:${id}` 형식을 쓴다.
- `browser.storage.session`이 있으면 session storage를 사용한다.
- session storage가 없으면 Firefox 호환성을 위해 Map fallback을 둔다.
- terminal delta가 들어오면 상태를 terminal로 표시하고 30초 뒤 제거한다.

## 3. 이벤트 처리

- `onCreated`: 새 다운로드로 판단되면 track 저장 후 HWP 후보를 처리한다.
- `onChanged`: 추적 중이고 아직 handled가 아니며 filename/finalUrl/complete 변경이면 `downloads.search({ id })`로 최신 항목을 재조회한다.
- HWP 후보이면 `autoOpen` 설정을 확인하고 handled 상태를 저장한다.
- `autoOpen=true`이면 기존 `openViewer({ url, filename })` 경로를 유지한다.

## 4. 테스트

- observer 등록 테스트
- 직접 HWP 다운로드 1회 오픈
- `autoOpen=false` 미오픈
- filename 확정 후 재판정
- 과거 `onChanged` 단독 무시
- 과거 완료 항목 `onCreated` 무시
- 재조회 결과가 과거 항목이면 무시
- event page 재기동 후 추적 상태 유지
- handled 상태가 재기동 후 중복 오픈을 방지
- 동일 id 다중 changed 이벤트 1회 오픈
