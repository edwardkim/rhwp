# Task M100 #2124 Stage 5 - maintainer 리뷰 반영

- 이슈: #2124
- 단계: Stage 5 - 최종 보고와 GitHub 후속 처리
- 상태: maintainer 최종 승인 / 최신 devel 재검증 후 merge
- 작성일: 2026-07-10
- 브랜치: `task2124-frontend-baseline`
- 기준 커밋: `upstream/devel` `3077f96d1f9931c50d6d62be77b389d4f66470a9`
- 선행 단계: `mydocs/working/task_m100_2124_stage4.md`

## 1. 현재 상태

draft PR #2174를 생성했고, maintainer가 metrics·contract/gate·SOLID 미채점·후속 순서 네 안건을
모두 승인했다. maintainer WSL2에서도 metrics 총량 3종과 함수별 자기 비교가 재현됐고, stale binding
검출 후 repo Docker fresh WASM으로 consumer gate가 통과했다. merge 전 경미 수정은 metrics remote
fallback, 최신 devel rebase, metrics 전용 `node_modules` ignore다.

최신 `upstream/devel` rebase에서 원격의 0.7.18 릴리즈 기록과 Chrome/Firefox/Studio Vite·TypeScript
dependency update를 유지한 뒤 #2124 변경을 적용했다. metrics 도구는 `upstream/devel`, `origin/devel`
순서로 조회하고 둘 다 없으면 `upstreamDevelCommit` 속성을 생략한다. 세 경로와 기존 snapshot 자기
비교를 로컬 검증했다. 최종 head의 CI 상태는 PR #2174 checks를 실시간 source of truth로 사용한다.
Studio/metrics/Chrome/Firefox/VS Code 다섯 패키지의 lockfile로 `npm ci`를 실행하고 Studio·확장
production build와 VS Code compile을 검증했다. build 후 contract/shared 68건과 Studio 185건이 통과했다.

upstream TypeScript 7.0.2는 기존 compiler API를 기본 export하지 않고 metrics parser의 peer 범위도
`<6.1.0`이라 Studio devDependencies 공유 방식이 재현되지 않았다. 제품 TypeScript 7은 유지하고
metrics 의존성을 `scripts/frontend-metrics/` private package로 분리해 TypeScript 6.0.3을 고정했다.

최종 merge 직전 `upstream/devel@6f1bd284`의 #2188 Studio/Rust 변경을 충돌 없이 rebase했다. 직전
snapshot 대비 Total CC +14가 `renderTextRun` +16, `recordTextRunCoverageGaps` -2로 설명됐으며,
Top 20과 CC>25/100은 변하지 않았다. 공식 기준선을 `6f1bd284`로 재생성하고 fresh WASM gate를 다시
적용했다.

그 뒤 `acc841c9`는 `mydocs/manual/memory/`만 바꾼 문서 전용 전진분이었다. 최신 devel로 다시
rebase하고 공식 snapshot provenance를 갱신했으며 정량 지표와 함수별 diff가 모두 0-delta임을
확인했다. frontend/Rust/package 입력이 동일하므로 fresh WASM gate는 재실행하지 않았다.

maintainer 후속 요청대로 `scripts/frontend-metrics/.gitignore`의 `node_modules/` 패턴을 확인했다.
이 패턴은 현재 head에 포함돼 있으며 `npm ci --prefix scripts/frontend-metrics` 후 의존성 디렉터리가
Git status에 남지 않는 것을 검증했다.

push 직후 `3077f96d`까지 #2184/#2191의 Studio CanvasKit과 Rust renderer/layout 변경이 추가됐다.
다시 rebase한 공식 snapshot은 `acc841c9` 대비 code lines +86, functions +3, Total CC +17,
CC>25 count/sum +1/+31을 포착했다. 네 함수 diff로 전부 설명되고 Top 20, CC>100, Max는 유지됐다.
Rust 변경을 포함하므로 repository Docker fresh WASM과 consumer gate를 다시 실행했다.

## 2. 현재 판단

| 항목 | 판단 |
|------|------|
| PR | #2174 maintainer 최종 승인. 최종 head gate 확인 후 collaborator merge |
| maintainer 답변 | rebase·재검증 후 merge 위임 확인 |
| build 후속 이슈 | 생성하지 않음. fresh WASM에서 binding, Studio, VS Code gate가 모두 통과함 |
| #2124 checklist/close | PR merge 전 금지. merge 후에도 사용자 승인 필요 |
| #2022 umbrella update | #2124 승인·close 시점에 근거 링크와 함께 수행 |
| #2183 | #2124 close 후 frontend CI gate로 우선 진행 |
| #2187 | #2183 merge 후 contract snapshot·smoke 보정과 collaborator 리뷰 |
| #2125 | #2187 이후 착수하며 #2190 font subset 규칙 연계 검토 |

## 3. Stage 5 완료 조건

1. #2174 ready 전환과 maintainer 반영 보고를 완료한다.
2. 최종 rebase·재검증·CI를 통과한 승인 PR을 merge한다.
3. 사용자 승인 후 #2124 체크리스트·최종 코멘트·close와 #2022 추적 항목을 갱신한다.

현재 문서는 merge 전 상태를 기록한 단계 보고이며, #2124 완료 선언은 아니다.
