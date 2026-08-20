---
kind: report
status: completed
canonical: mydocs/plans/task_m100_3514.md
last_verified: 2026-08-20
---

# Task M100 #3514 Stage 4 — 탭 예산 즉시 중단 보강

- Issue: [#3514](https://github.com/edwardkim/rhwp/issues/3514)
- Stage 3 commit: `1fa80a632`
- 기준 브랜치: `codex/issue-3514-extension-smoke`

## 보강 사유

PR 전 자체 검토에서 `targetcreated` listener가 예상 밖 page URL을 진단 배열에 기록하지만 실제 실패는
다음 `assertPageBudget` checkpoint까지 늦춘다는 사실을 확인했다. surface 대기가 진행 중이면 최대
timeout까지 추가 탭을 허용할 수 있어 #3514의 “예상 밖 탭 생성 시 즉시 실패” 완료 조건을 엄격히
충족하지 못했다.

작업지시자는 이 보강을 별도 Hyper-Waterfall Stage 4로 진행하도록 승인했다.

## 구현

- 탭 예산 controller가 예상 밖 page target 최초 생성 시 실패 promise를 reject한다.
- service worker 준비부터 viewer/options/print/content-script까지 모든 비동기 surface 작업을 이 promise와
  race한다.
- owned page와 service worker 등 non-page target은 허용한다.
- page target은 닫히더라도 진단 배열에 남고, 첫 실패가 즉시 현재 surface를 중단한다.
- controller listener는 smoke `finally` 정리에서 detach한다.
- 실행 파일을 import할 때 실제 smoke가 시작되지 않도록 main 진입점을 구분하고, 별도 Node 계약 테스트를
  추가했다.
- `test:e2e:smoke`가 production build 뒤 탭 예산 계약 테스트와 실제 packaged smoke를 순서대로 실행한다.

## 검증

| 게이트 | 결과 |
|---|---|
| harness·계약 테스트 `node --check` | 통과 |
| 탭 예산 계약 테스트 | 2/2 통과 |
| Chrome·Firefox 확장 Node 계약(탭 예산 포함) | 125/125 통과 |
| Chrome·Firefox·Safari dist 계약 | 3/3 통과 |
| 실제 package smoke | 새 profile 10개, retry 없이 10/10 통과 |

계약 테스트는 완료되지 않는 promise를 surface로 두고 예상 밖 page target 이벤트만 발생시킨다. 기존처럼
진단만 기록하면 1초 timeout으로 실패하고, 현재 구현은 즉시 reject되어 통과한다. owned page와
service worker target이 정상 예산에 포함되는 positive control도 함께 고정했다.

## 범위 경계

Stage 4는 #3514 내부의 탭 예산 실패 시점만 보강한다. 실제 HWP 다운로드, 동일 profile 브라우저 재실행,
service worker 수명주기와 과거 다운로드 기록 불변식은 #3513에서 시나리오별로 구현한다. CI 선택 실행,
Chrome 설치·cache와 실패 artifact는 #3515가 담당한다.

## Stage 4 승인 대상

- `rhwp-chrome/e2e/extension-smoke.test.mjs`
- `rhwp-chrome/e2e/page-budget.test.mjs`
- `rhwp-chrome/package.json`
- `mydocs/manual/chrome_edge_extension_build_deploy.md`
- `mydocs/orders/20260820.md`
- `mydocs/plans/task_m100_3514.md`
- `mydocs/plans/task_m100_3514_impl.md`
- `mydocs/report/task_m100_3514_report.md`
- `mydocs/working/task_m100_3514_stage4.md`

작업지시자 승인 전에는 Stage 4를 커밋하지 않는다. 승인 뒤에도 remote push와 PR 생성은 별도 승인
경계로 유지한다.

## 승인 결과

작업지시자는 2026-08-20 23:20 KST에 “진행해줘”로 Stage 4 구현, 계약 테스트와 실제 smoke 10회
검증 결과를 승인했다. 위 승인 대상만 보정 커밋하며, push와 PR 생성은 별도 승인 경계로 남긴다.
