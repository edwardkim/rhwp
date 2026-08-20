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
- 단계 기록: `mydocs/working/task_m100_3514_stage{1,2,3}.md`

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

Puppeteer와 동반 Chrome 버전은 `rhwp-chrome/package-lock.json`으로 고정한다. 사용자 Chrome
profile, Web Store, 외부 네트워크는 사용하지 않는다.

## 검증

- 최종 package smoke: retry 없이 새 profile 10개, 10/10 통과
- Chrome·Firefox 확장 Node/dist 계약: 85/85 통과
- Studio TypeScript: 통과
- Studio 단위 테스트: 1,033 통과, 1 skip, 실패 0
- Chrome·Firefox production build: 통과
- harness 구문 검사와 `git diff --check`: 통과

## 남은 경계

- #3513: 설정·다운로드 수명주기와 탭 불변식의 상세 E2E
- #3515: CI 영향 경로와 브라우저 cache
- provider 분류, 동적 DOM, hover race, context menu, `file://` 권한
- OS 인쇄 대화상자와 인쇄 결과 픽셀·레이아웃 비교

로컬 Docker daemon이 꺼져 표준 최적화 WASM 이미지는 실행하지 못했고 문서화된 네이티브
`wasm-pack --no-opt` fallback으로 package를 검증했다. 이 차이는 PR CI에서 재확인한다.

Stage 2 커밋을 기준으로 2026-08-20 22:22 KST에 전체 검증을 다시 완료했고, 작업지시자는 22:26
KST에 Stage 3 문서를 승인했다. 이 보고서를 포함한 커밋으로 Hyper-Waterfall 세 Stage를 닫는다.
이후 remote push와 draft PR 생성은 GitHub 작업 경계에 따라 각각 명시적 승인을 확인한다.
