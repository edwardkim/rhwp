---
kind: report
status: completed
canonical: mydocs/plans/task_m100_3514.md
last_verified: 2026-08-22
---

# Task M100 #3514 Stage 6 — PR 직전 최신 base·locked WASM 재검증

## 1. 승인 범위

Stage 6에서는 기능 코드를 추가하지 않고 최신 `upstream/devel` 정합화와 재검증 결과를 문서화한다.
작업지시자는 2026-08-22 전진한 upstream 반영이 필요하면 반영하고 PR 본문을 정합화한 뒤 게시까지
진행하라고 승인했다.

## 2. 최신 base 정합화

- 이전 기준: `upstream/devel@d5f0f8dc96d41f324a13488976b4f589b5da8e6a`
- 이전 Stage 5 HEAD: `4e4f1c4a3b00e225eeadf5696b974d0a2ebff534`
- 최종 기준: `upstream/devel@65f71270f`
- 최종 Stage 5 HEAD: `694f9059c`
- 전진량: 71커밋

기존 다섯 Stage 커밋을 최신 base 위로 충돌 없이 리베이스했다. `range-diff`에서 Stage 2~5 기능·문서
patch는 동일했고, Stage 1은 공유 당일 작업 목록 `mydocs/orders/20260820.md`의 최신 upstream 내용을
보존한 채 #3514 행을 적용해 문맥만 달라졌다.

| Stage | `d5f0f8dc9` 기준 | `65f71270f` 기준 | `git range-diff` |
|---|---|---|---|
| 1 | `9c51b75e7` | `48e89416e` | 공유 당일 작업 목록의 최신 base 문맥 반영 |
| 2 | `1ceab60b3` | `fd22153af` | patch 동일 |
| 3 | `0519db47e` | `612bbe496` | patch 동일 |
| 4 | `040fdb344` | `f090fc0a7` | patch 동일 |
| 5 | `4e4f1c4a3` | `694f9059c` | patch 동일 |

`upstream/devel`이 현재 HEAD의 조상이며 문서 변경 전 branch는 `0 behind / 5 ahead`다.

## 3. 최신 base 재검증

| 영역 | 명령·범위 | 결과 |
|---|---|---|
| Rust format 준비 | `node scripts/rust-test-suite-manifest.mjs --prepare`·`--check` | review-only harness 32개 준비, 864 source 확인 통과 |
| Rust format | `cargo fmt --all`·`cargo fmt --all -- --check` | 통과, 추적 Rust diff 없음 |
| WASM | `CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg --no-opt` | fresh build 1분 56초 통과 |
| Cargo lock | 실행 전후 `git hash-object Cargo.lock` | `e0ad3758affc57b170a03cfbe2f1c8294c89d7aa`, 동일 |
| Studio type | `npx tsc --noEmit` (`rhwp-studio`) | 통과 |
| Studio unit | `npm test` (`rhwp-studio`) | 1,065 pass, 1 skip, 0 fail |
| Extension Node | page-budget와 Chrome·Firefox shared/worker/options 계약 | 132/132 통과 |
| Firefox build | production extension build | 통과, 215 modules |
| Dist contract | `scripts/frontend-extension-dist.test.mjs` | 3/3 통과 |
| Actual Chrome | `RHWP_EXTENSION_SMOKE_REPEAT=10 npm --prefix rhwp-chrome run test:e2e:smoke` | build·page-budget 2/2 뒤 첫 profile에서 `ECONNRESET`으로 실패 |

표준 Docker 최적화 WASM은 daemon이 꺼진 기존 환경 경계를 유지한다. 문서화된 native `--no-opt`
fallback으로 최신 Rust/WASM과 이를 소비하는 package를 실제 빌드했다.

## 4. 실패 판정

Chrome production build와 page-budget 계약 2건은 성공했지만 1/10 profile의 제품 surface assertion 전에
loopback 차단 proxy의 CONNECT socket이 `read ECONNRESET`을 처리하지 못해 Node 프로세스가 종료됐다.
이 실행은 10회 통과로 인정하지 않으며 성공 재시도를 하지 않았다.

오류는 extension product code가 아니라 smoke harness의 proxy socket 수명주기 결함이다. Chrome이
차단 응답을 다 읽기 전에 연결을 닫는 정상 client abort에는 socket 오류 listener가 필요하고, 예상 밖
socket 오류는 진단 실패로 남겨야 한다. 이 보강은 #3514의 “사용자·외부망에 의존하지 않는 반복 가능한
smoke” 완료 조건에 포함되므로 Stage 7로 분리한다.

## 5. Stage 6 판정

최신 base 리베이스, patch 계보, mandatory format, fresh locked WASM, Studio·extension Node·Firefox·dist
검증은 정합하다. 실제 Chrome 10회 gate는 harness 결함으로 실패했으므로 PR 게시 조건은 아직 충족하지
못했다. Stage 6은 이 실패와 Stage 7 필요성을 고정하는 문서 커밋으로 닫고, 다음 커밋에서 proxy socket
정정과 새 code head의 retry 없는 10회를 수행한다.
