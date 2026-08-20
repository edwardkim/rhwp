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
- 기준: `upstream/devel@f26c2e7ca4911a07cb51e2bdec415b8c1b02ee9c`
- Stage 1: `576b84d44` — 조사·범위·계획·절차 복구
- Stage 2: `f91c75976` — packaged smoke 구현
- Stage 3: `da3ab4e74` — 전체 재검증·운영 문서
- Stage 4: `3c6aa1e8e` — 탭 예산 즉시 중단 보강
- Stage 5: 최신 `devel` 정합화·재검증 — 승인 완료
- 단계 기록: `mydocs/working/task_m100_3514_stage{1,2,3,4,5}.md`

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

## 최신 `devel` 정합화

PR 생성 전 최신 `upstream/devel`을 다시 fetch하자 기존 기준보다 91개 커밋이 앞서 있었다. 기능
커밋 네 개를 `f26c2e7ca` 위로 리베이스했고, 충돌은 여러 작업이 함께 사용하는
`mydocs/orders/20260820.md` 한 파일에서만 발생했다. 최신 작업 기록을 모두 유지하고 #3514의 M100 행을
합쳤다.

| Stage | 리베이스 전 | 리베이스 후 | `range-diff` 판정 |
|---|---|---|---|
| 1 | `081a44af9` | `576b84d44` | 당일 작업 목록의 최신 base 문맥만 반영 |
| 2 | `e2b6ec723` | `f91c75976` | 동일 |
| 3 | `1fa80a632` | `da3ab4e74` | 동일 |
| 4 | `526f08f0a` | `3c6aa1e8e` | 동일 |

따라서 Stage 2~4의 기능·운영 문서 patch는 그대로이고 Stage 1만 공유 당일 작업 목록의 최신 내용이
추가됐다. `upstream/devel`이 현재 HEAD의 조상임도 별도로 확인했다. 네이티브 WASM 재생성 중
`Cargo.lock`의 로컬 package 두 항목 순서만 바뀐 부수 diff는 원상 복구해 의도하지 않은 lockfile
변경을 남기지 않았다.

## 최신 기준 재검증

- `wasm-pack build --target web --out-dir pkg --no-opt`: 최신 Rust 기준 package 생성 통과
- Studio TypeScript: 통과
- Studio 단위 테스트: 1,047 통과, 1 skip, 실패 0
- Chrome·Firefox 확장 Node 계약: 탭 예산 2개를 포함해 125/125 통과
- Firefox production build: 통과
- Chrome·Firefox·Safari dist 계약: 3/3 통과
- 실제 packaged Chrome smoke: 새 profile 10개, retry 없이 10/10 통과
- `git diff --check`, 최신 base 조상 검사, 리베이스 `range-diff`: 통과

Docker daemon이 꺼져 문서화된 표준 최적화 WASM image는 로컬에서 실행하지 못했다. 대신 최신 Rust를
네이티브 `wasm-pack --no-opt`로 실제 컴파일해 extension build와 브라우저 검증에 사용했으며, 최적화
WASM은 PR CI에서 재확인한다.

`cargo fmt --all -- --check`는 최신 base가 일반 작업 checkout에 두지 않는 review-only 파생 suite
`tests/generated/regression_suite_001.rs`~`032.rs`를 참조해 format 검사 진입 전에 중단됐다. 이번
브랜치에는 Rust diff가 없고 동일 최신 Rust가 위 WASM build를 통과했지만, 이를 `cargo fmt` 성공으로
기록하지 않는다. 파생 suite를 준비하는 표준 review 환경 또는 CI에서 이 gate를 재확인한다.

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

Stage 2 커밋을 기준으로 2026-08-20 22:22 KST에 전체 검증을 다시 완료했고, 작업지시자는 22:26
KST에 Stage 3 문서를 승인했다. 이후 PR 전 자체 검토에서 탭 예산 지연 실패를 발견해 작업지시자가
Stage 4 진행을 승인했고, 구현·검증 결과도 23:20 KST에 승인했다. 이 보고서를 포함한 보정 커밋으로
네 Stage를 닫았다. 이후 최신 `devel` 정합화 요청에 따라 Stage 5 리베이스와 전체 재검증을 완료했고,
작업지시자 승인 뒤 정합화 커밋으로 Stage 5를 닫았다. remote push와 PR 생성은 GitHub 작업 경계에
따라 각각 명시적 승인을 확인한다.
