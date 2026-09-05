---
kind: snapshot
status: active
canonical: mydocs/plans/task_m100_6731.md
issue: 6731
last_verified: 2026-09-05
---

# #6731 Stage 4 — 로컬 검증과 exact-head 준비

## 기준선 E2E 최소 정정

Stage 3에서 확인한 #4430 E2E 기준선 불일치를 제품 동작 변경 없이 test 한 곳에서 정정했다.

- 보호 문서의 평문 사본 시나리오가 Save As 기본 `확인` 대신 현재 UI 계약의
  `암호 없이 저장`을 선택한다.
- helper에 `plaintext` action을 추가했으며 다른 Save As 시나리오의 선택은 바꾸지 않았다.
- 이는 2026-08-30 통합 커밋 `2ca9aa90f7`의 `inheritPassword` 계약을 뒤늦게 반영한 test 현행화다.

## 로컬 검증 결과

| 검증 | 결과 |
| --- | --- |
| `node --check e2e/content-loss-save-issue4430.test.mjs` | 성공 |
| `npm run e2e:issue-4430-content-loss` | 전체 시나리오 성공, 종료 코드 0 |
| `npm test` | 1,376건 중 1,375건 성공, 실패 0, 기존 skip 1 |
| `npm run build` | TypeScript compile·Vite production build 성공 |
| `npm run e2e:hwp-password-open` | HWP3·HWP5·HWPX 암호 open과 무유출 계약 성공 |
| `git diff --check` | 성공 |

Studio 변경 범위이므로 [로컬 검증 표](../manual/pr_review/local_validation.md#43-변경-범위별-기본-검증)의
TypeScript 검사, `npm test`, 실제 browser 동작을 충족한다. Rust source·Rust test·WASM binding은
변경하지 않아 Rust lint·release-test·WASM build 대상이 아니다. Vite의 CanvasKit `fs`·`path`
externalize와 500 kB chunk 메시지는 실패가 아닌 기존 build warning이다.

## 원격 기준선과 병렬 변경

- 확인 시점의 `upstream/devel`은 계획 기준선
  `9bf5bcfd061b491c89be1ea28ef1fff8a892b6d1`에서 이동하지 않았다.
- 현재 branch는 기준선보다 세 commit 앞서고 원격 선행 commit은 없다.
- PR #6725는 `wasm-bridge.ts`, PR #6637은 `main.ts`를 계속 변경하지만 둘 다 open이며 현재
  `MERGEABLE`·`CLEAN`이다. 이번 변경과 직접 겹치는 hunk는 없지만 push 직전과 merge 직전에
  상태를 다시 확인한다.

## 남은 Stage 4 게이트

로컬 CodeQL CLI가 없으므로 alert #186의 최종 판정은 GitHub의 PR exact head 분석이 필요하다.
remote push와 PR 생성 승인을 받은 뒤 다음 순서로 진행한다.

1. 최신 `upstream/devel`을 다시 fetch하고 선행 commit이 생겼으면 먼저 병합·재검증한다.
2. branch를 push하고 PR을 생성한다.
3. exact-head JavaScript/TypeScript CodeQL SARIF에서 alert #186 flow 소멸을 확인한다.
4. CodeQL workflow·query·path가 기준선과 같고 다른 password source 탐지를 제외하지 않았음을
   diff와 run 설정으로 재확인한다.
5. flow가 남으면 sanitizer·query 제외를 추가하지 않고 SARIF delta와 수정 수행계획을 보고한다.
