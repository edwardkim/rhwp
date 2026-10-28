# 작업 2493 단계 1 - HML 편집기 및 확장 발견 경로 통합

## 범위

- 기여자 PR #2493, #2495, #2511을 통합한다.
- 이미 지원하는 HML 형식을 VS Code custom editor, Marketplace metadata, 브라우저 확장 link
  detection에 노출한다.

## 검토 근거

- PR 본문: core, Studio, CLI는 이미 HML을 불러올 수 있지만 VS Code에는 `*.hml` custom-editor
  selector가 없었고, 브라우저 확장의 URL 검사는 HWP/HWPX만 받아들였다.
- PR 코멘트: 세 PR 모두 검토 당시 없음.
- 공유 URL resolver는 일반 URL과 GitHub raw URL의 공통 안전 gate다. 기존 테스트는 HWP/HWPX만
  다뤘으므로 HML 허용에는 명시적인 회귀 검증이 필요하다.

## 메인터너 보강

- 기존 HWP/HWPX 사례와 함께 HML path 인식과 GitHub blob-to-raw resolution을 검증한다.
- query만 있는 pseudo extension과 문서가 아닌 GitHub path는 계속 거부한다.

## 검증 계획

1. 공유 URL resolver Node 테스트 모음을 실행한다.
2. VS Code package JSON을 검증하고 publish 없이 로컬에서 package를 만든다.
3. 변경된 모든 브라우저 content script의 문법을 검사하고 Chrome/Firefox 확장을 로컬 빌드한다.
4. 최종 통합 PR 전 통합 전체 회귀에 이 그룹을 포함한다.
