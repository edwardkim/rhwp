---
kind: report
status: completed
canonical: mydocs/plans/task_m100_3514.md
last_verified: 2026-08-21
---

# Task M100 #3514 Stage 5 — 최신 devel 정합화·재검증

## 1. 승인 대상

Stage 5에서는 기능 코드를 추가하지 않고 다음 정합화 문서만 승인 대상으로 삼는다.

- `mydocs/orders/20260820.md`
- `mydocs/plans/task_m100_3514.md`
- `mydocs/plans/task_m100_3514_impl.md`
- `mydocs/report/task_m100_3514_report.md`
- `mydocs/working/task_m100_3514_stage5.md`

작업지시자가 이 다섯 경로를 승인했다. remote push와 draft PR 생성은 별도 GitHub 승인 경계로
남긴다.

## 2. 최신 기준 정합화

- 리베이스 전 기준: `upstream/devel@00da1ab356d4782fc3bd6320d02e656e7431bc34`
- 리베이스 전 HEAD: `526f08f0a`
- 리베이스 후 기준: `upstream/devel@f26c2e7ca4911a07cb51e2bdec415b8c1b02ee9c`
- 리베이스 후 Stage 4 HEAD: `3c6aa1e8ee927b2ba0e99c1037a60be5e769b4f3`

기존 브랜치는 최신 기준보다 91개 커밋 뒤에 있었고 기능 커밋 네 개를 최신 `upstream/devel` 위로
리베이스했다. 충돌은 여러 작업이 공유하는 당일 작업 목록 `mydocs/orders/20260820.md` 한 파일에서만
발생했다. 최신 base의 기존 작업 기록을 모두 유지하고 #3514의 M100 행을 합쳤다.

| Stage | 리베이스 전 | 리베이스 후 | `git range-diff` |
|---|---|---|---|
| 1 | `081a44af9` | `576b84d44` | 공유 당일 작업 목록의 최신 base 문맥만 반영 |
| 2 | `e2b6ec723` | `f91c75976` | patch 동일 |
| 3 | `1fa80a632` | `da3ab4e74` | patch 동일 |
| 4 | `526f08f0a` | `3c6aa1e8e` | patch 동일 |

`git merge-base --is-ancestor upstream/devel HEAD`도 통과했다. 최신 WASM package 생성 과정에서
`Cargo.lock`의 로컬 package 두 항목 순서만 뒤바뀐 부수 diff는 원상 복구했으며 기능·의존성 변경은
남기지 않았다.

## 3. 최신 기준 재검증 결과

| 영역 | 명령·범위 | 결과 |
|---|---|---|
| WASM | `wasm-pack build --target web --out-dir pkg --no-opt` | 통과, 최신 Rust package 새로 생성 |
| Studio type | `npx tsc --noEmit` (`rhwp-studio`) | 통과 |
| Studio unit | `npm test` (`rhwp-studio`) | 1,047 pass, 1 skip, 0 fail |
| Extension Node | page-budget와 Chrome·Firefox shared/worker/options 계약 | 125/125 통과 |
| Firefox build | production extension build | 통과 |
| Dist contract | `scripts/frontend-extension-dist.test.mjs` | 3/3 통과 |
| Actual Chrome | `RHWP_EXTENSION_SMOKE_REPEAT=10 npm --prefix rhwp-chrome run test:e2e:smoke` | 새 profile 10개, retry 없이 10/10 통과 |
| Git | `git diff --check`, 최신 base 조상 검사, `range-diff` | 통과 |

actual Chrome 명령은 extension production build를 한 번 수행한 뒤 반복마다 새 격리 profile을 만들어
실제 MV3 worker·viewer·options·print·content script와 탭 예산을 함께 검증했다. 최신 base의 frontend
build는 214 modules를 변환했고, 새로 생성한 WASM asset을 사용했다.

## 4. 검증 경계

- Docker daemon이 꺼져 표준 최적화 WASM image는 실행하지 못했다. 네이티브 `wasm-pack --no-opt`
  컴파일과 브라우저 검증은 통과했으며 최적화 WASM은 Docker 사용 가능 환경 또는 release pipeline에서
  재확인한다.
- `cargo fmt --all -- --check`는 일반 작업 checkout에 없는 review-only 파생 suite
  `tests/generated/regression_suite_001.rs`~`032.rs` 때문에 format 검사 진입 전에 중단됐다. 이번
  브랜치에는 Rust diff가 없고 최신 Rust의 WASM 컴파일은 통과했지만, 이를 fmt 성공으로 기록하지
  않는다. 파생 suite가 준비되는 표준 review 환경 또는 CI에서 재확인한다.
- TypeScript 첫 시도에서 `npm --prefix rhwp-studio exec`의 working directory 해석 때문에 compiler
  help만 출력됐다. `rhwp-studio`를 실제 working directory로 지정한 `npx tsc --noEmit`을 다시 실행해
  통과했으며 제품 실패는 아니었다.

## 5. Stage 5 판정

최신 `upstream/devel`과의 코드 계보, 빌드 산출물, extension 계약 및 실제 Chrome 10회 실행은
정합화됐다. 위 두 표준 환경 gate는 성공으로 과장하지 않고 CI 재확인 항목으로 남겼다.

작업지시자가 정합화 문서를 승인했으며, 이 문서가 포함된 별도 커밋으로 Stage 5를 닫았다. remote
push와 draft PR 생성은 수행하지 않았다.
