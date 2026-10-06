# Task M100 #1516 Stage 1: 분석 및 계획

## 확인 내용

- #1516은 #1515에 의존한다.
- 현재 브랜치는 #1515 작업 브랜치에서 분기한 `local/task1516-firefox-download-state-machine`이다.
- Firefox `download-interceptor.js`는 아직 `seen`/`handled`/`workerStartedAt` 전역 상태를 직접 관리한다.
- Firefox build script는 `sw/` 복사 시 `dereference: true`를 사용하므로 shared symlink 추가가 배포본에 실체 파일로 포함된다.

## 구현 방향

- Chrome #1515 구현과 같은 공통 상태 머신 API를 사용한다.
- Firefox 파일에는 `browser.*` adapter와 뷰어 오픈만 남긴다.
- Firefox 테스트는 Chrome 테스트의 의미상 같은 케이스를 `browser` mock으로 검증한다.

## 리스크

- Firefox `browser.storage.session` 지원 여부는 런타임마다 다를 수 있으므로 feature detection과 Map fallback이 필요하다.
- Firefox의 `tabs.create()`는 Promise 기반이므로 mock도 Promise를 반환해야 한다.
- #1516은 #1515 위 stacked 브랜치이므로 PR 생성 시 base/head 관계를 명확히 해야 한다.
