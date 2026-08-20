---
kind: report
status: active
canonical: mydocs/plans/task_m100_3514.md
last_verified: 2026-08-20
---

# Task M100 #3514 완료 보고서 — Chrome 실제 패키지 핵심 smoke

- Issue: [#3514](https://github.com/edwardkim/rhwp/issues/3514)
- Parent: [#3512](https://github.com/edwardkim/rhwp/issues/3512)
- 브랜치: `codex/issue-3514-extension-smoke`
- 기준: `upstream/devel@00da1ab356d4782fc3bd6320d02e656e7431bc34`
- Stage 1: `081a44af9` — 조사·범위·계획·절차 복구
- Stage 2: `e2b6ec723` — packaged smoke 구현
- Stage 3: `1fa80a632` — 전체 재검증·운영 문서
- Stage 4: 탭 예산 즉시 중단 보강 — 승인 완료
- 단계 기록: `mydocs/working/task_m100_3514_stage{1,2,3,4}.md`

## 결과

`npm --prefix rhwp-chrome run test:e2e:smoke` 한 명령으로 최종 Chrome 확장을 빌드하고 실제
Chrome for Testing에 설치해 다음 경계를 검증한다.

1. 동적 extension ID, MV3 service worker 시작과 실제 `fetch-file` 정책 응답
2. 실제 HWP3 fixture의 viewer canvas·최종 파일명 상태
3. 다크 모드의 CSP·정적 SVG 자산
4. options의 비동기 설정 hydration과 네 입력 활성화
5. viewer와 같은 extension origin의 print surface
6. loopback 페이지 content script와 HWP 배지 정확히 1개
7. page/worker 오류·외부 요청·로컬 4xx/5xx·추가 탭과 자원 정리

PR 전 자체 검토에서 추가 탭은 생성 즉시 기록되지만 실제 실패가 다음 surface checkpoint까지 늦어지는
완료 조건 불일치를 확인했다. Stage 4에서는 예상 밖 page target을 공유 실패 gate로 연결하고 모든 비동기
surface 작업을 이 gate와 race해, 다음 checkpoint나 timeout을 기다리지 않고 정리 경로로 이동하게 했다.

Puppeteer와 동반 Chrome 버전은 `rhwp-chrome/package-lock.json`으로 고정한다. 사용자 Chrome
profile, Web Store, 외부 네트워크는 사용하지 않는다.

## 검증

- 최종 package smoke: retry 없이 새 profile 10개, 10/10 통과
- Chrome·Firefox 확장 Node/dist 계약: 85/85 통과
- Studio TypeScript: 통과
- Studio 단위 테스트: 1,033 통과, 1 skip, 실패 0
- Chrome·Firefox production build: 통과
- harness 구문 검사와 `git diff --check`: 통과
- 탭 예산 계약 테스트: 2/2 통과. 끝나지 않는 surface도 예상 밖 page 이벤트만으로 reject
- Stage 4 확장 Node 계약: 탭 예산 포함 125/125 통과
- Stage 4 Chrome·Firefox·Safari dist 계약: 3/3 통과
- Stage 4 실제 package smoke: 새 profile 10개, retry 없이 10/10 통과

## PR 활용과 후속 구현 경계

PR 본문에는 이 명령이 확장 변경의 로컬 사전 점검과 릴리즈 후보 package smoke에 사용되고, 후속 #3515에서
영향 경로 기반 CI job의 실행 단위가 된다는 점을 명시한다. 이 PR이 보증하는 것은 실제 package 설치,
핵심 surface 초기화, CSP·정적 자산·worker/content-script 배선이며 실제 다운로드와 프로필 수명주기는
아니다.

#3513에서는 시나리오마다 독립 profile을 쓰되, 브라우저 재실행을 포함하는 한 시나리오 내부에서는 같은
profile을 보존해 다음을 추가한다.

1. 설정 저장 뒤 options 재진입·worker 종료·브라우저 재실행에서 설정 유지
2. `autoOpen=false` 새 HWP/HWPX 다운로드의 viewer 0개
3. 과거 다운로드 기록이 있는 profile의 확장 시작에서 viewer 0개
4. `autoOpen=true` 새 다운로드와 worker 재기동에서 viewer 정확히 1개
5. 동일 profile에서 HWP 다운로드 뒤 Chrome을 종료·재실행해 과거 기록으로 새 viewer가 열리지 않음

#3515에서는 관련 경로 선택 실행, Chrome for Testing 설치·cache, PR 1회와 release/nightly 반복 정책,
실패 screenshot·worker·열린 extension URL artifact를 연결한다.

## 남은 경계

- #3513: 설정·다운로드 수명주기와 탭 불변식의 상세 E2E
- #3515: CI 영향 경로와 브라우저 cache
- provider 분류, 동적 DOM, hover race, context menu, `file://` 권한
- OS 인쇄 대화상자와 인쇄 결과 픽셀·레이아웃 비교

로컬 Docker daemon이 꺼져 표준 최적화 WASM 이미지는 실행하지 못했고 문서화된 네이티브
`wasm-pack --no-opt` fallback으로 package를 검증했다. 이 차이는 PR CI에서 재확인한다.

Stage 2 커밋을 기준으로 2026-08-20 22:22 KST에 전체 검증을 다시 완료했고, 작업지시자는 22:26
KST에 Stage 3 문서를 승인했다. 이후 PR 전 자체 검토에서 탭 예산 지연 실패를 발견해 작업지시자가
Stage 4 진행을 승인했고, 구현·검증 결과도 23:20 KST에 승인했다. 이 보고서를 포함한 보정 커밋으로
네 Stage를 닫는다. 이후 remote push와 PR 생성은 GitHub 작업 경계에 따라 각각 명시적 승인을 확인한다.
