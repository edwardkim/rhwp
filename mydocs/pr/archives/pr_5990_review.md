---
kind: pr-review
status: accepted-pending-integration-pr-approval
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-24
---

# PR #5990 self-review — 암호 문서 저장 보호 의도 보존

## 라우팅과 접수 메타데이터

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`,
  `multi_pr_update_branch.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본·보조 문서
- 작성자 본인 self-review이므로 reviewer를 지정하지 않는다.
- 기능 commit: `bdc90ded9ff608e5fd163ce78ca064e0931cd82f`
- 최초 PR candidate: `ecb63c1c7d5b250a37a3d73b5e43f3b54c16b9d9`
- current-base bridge: `b12512f8a` (`upstream/devel@01e2e7422`)

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR / 작성자 | [#5990](https://github.com/edwardkim/rhwp/pull/5990) / [@postmelee](https://github.com/postmelee) |
| base / head | `devel` / `codex/issue-5986-save-protection` |
| 최초 candidate 규모 | 14 files, +417 / -13, 2 commits |
| 상태 | Open, non-draft; base 전진 bridge와 trailing commit push 뒤 최신 상태 재확인 필요 |
| 관련 issue | closes [#5986](https://github.com/edwardkim/rhwp/issues/5986); embed RPC는 [#5987](https://github.com/edwardkim/rhwp/issues/5987) |

1,000줄 기준 아래이며 제품 변경은 `rhwp-studio`의 상태 전이와 회귀 테스트에 한정된다. 검토 중
`devel`이 `01e2e7422`로 전진해 원격 candidate가 `CONFLICTING/DIRTY`가 됐으나, merge-tree에서 확인된
충돌은 `mydocs/orders/20260824.md` 하나뿐이었다. current-base bridge는 #5985의 최신 오늘할일과 이 PR의
#5986 기록을 모두 보존하며 source·test 충돌 해소를 포함하지 않는다.

## 변경 범위와 self-review

- `WasmBridge.loadDocumentAtomically()`가 성공한 문서의 저장 보호 의도를 명시적으로 받아 atomic commit
  구간에서만 `_requiresPasswordForSave`를 교체한다.
- 평문 load는 `false`, 암호 load 성공은 `true`를 전달한다. 오답·손상 등 commit 전 실패는 기존 문서와
  보호 의도를 함께 보존한다.
- 평문 Save As fallback은 download 성공 뒤에만 파일명과 보호 의도를 갱신해 저장 실패가 보호 상태와
  dirty 상태를 바꾸지 않게 한다.
- 실제 HWP3/HWP5/HWPX 암호 fixture, 평문 load, 새 문서, release, 저장 실패 상태 전이를 계약 test와
  실제 browser E2E로 고정했다.
- 암호 문자열을 state, log, DOM, URL 또는 storage에 추가하지 않았다. 장기 상태는 boolean 보호 의도
  하나뿐이며 기존 암호 serializer와 content-loss reporter를 변경하지 않는다.

renderer, layout, paint, sample, 기준 PDF, golden, Rust source와 workflow 변경은 없다. 따라서 별도 visual
sweep은 적용하지 않았다. host-managed password save embed 표면은 #5987의 독립 범위다.

## 발견 사항과 잔여 위험

### blocker 없음

- load 성공 뒤 보호 상태와 실패 시 기존 상태 보존이 같은 atomic commit 경계에 있다.
- fallback 저장의 파일명·보호 상태 갱신이 download 호출 성공 뒤에 있어 실패 경로가 기존 상태를 보존한다.
- 암호 문자열을 장기 보관하거나 자동 재사용하는 새 경로가 없다.
- 최초 PR candidate의 Frontend package gate, CodeQL JavaScript/TypeScript, Proptest, Adapter inter-diff와
  Canvas visual diff가 모두 성공했다.

### 공개한 기준선 예외

`python3 scripts/check_e2e_manifest.py`는 이번 diff 이전부터 미등재된
`loading-busy-cursor.test.mjs`, `status-page-number.test.mjs`, `toolbox-visibility.test.mjs` 세 건 때문에
실패했다. 이번에 변경한 두 E2E manifest 행은 갱신했다. 작업지시자에게 이 상태를 보고한 뒤 PR 생성
지시를 받았으므로 기존 세 건은 범위 밖 예외로 유지하되, CI와 후속 검토에서 새 누락이 없는지 확인한다.

원 구현은 계획·구현·보고가 한 commit에 들어갔고 계획 작성 뒤 별도 승인과 동시점 stage report가 없었다.
감사 뒤 이를 소급 승인하지 않고 `ecb63c1c7`에서 구현계획, Stage 1~3 사후 재구성, 계획 대비 실제와 절차
이탈을 기록했다. 이 보정은 감사 가능성을 높이지만 과거 단계 경계를 새로 만든 것으로 간주하지 않는다.

## 완료한 로컬 검증

| 검증 | 결과 |
| --- | --- |
| focused Node 계약 테스트 | 11/11 통과 |
| `npm test` | 1,071 통과, 1 skip, 실패 0 |
| locked WASM wrapper `--no-opt` | fresh binding 생성 통과 |
| `npm run build` | TypeScript와 Vite production build 통과 |
| `npm run e2e:hwp-password-open` | HWP3/HWP5/HWPX와 상태 수명주기 통과 |
| `npm run e2e:issue-4430-content-loss` | 저장 성공·실패·암호·fallback 통과 |
| `cargo fmt --all` / `cargo fmt --all -- --check` | generated suite를 준비한 별도 review worktree에서 통과 |
| `git diff --check` | 통과 |
| current-base merge-tree | 충돌은 오늘할일 1개뿐, `mydocs/` bridge로 양쪽 기록 보존 |

Rust source·fixture·renderer 변경이 없어 release-test 전체, clippy, Native Skia와 별도 시각 검증은 적용하지
않았다. source PR worktree의 첫 formatter 시도는 정책상 생성하지 않는 `tests/generated/` 32개가 없어
중단됐고, 가이드에 따라 별도 review worktree에서 `--prepare`한 뒤 두 formatter 명령을 통과시켰다. 파생
suite와 manifest는 PR에 포함하지 않았다.

## GitHub Actions

최초 candidate `ecb63c1c7`에서 다음 결과가 성공했다.

| workflow | run | 판정 |
| --- | --- | --- |
| CI | [32690539820](https://github.com/edwardkim/rhwp/actions/runs/32690539820) | Frontend package gates와 Build & Test 성공, Rust heavy lane은 영향 분류상 skip |
| CodeQL | [32690539673](https://github.com/edwardkim/rhwp/actions/runs/32690539673) | JavaScript/TypeScript·Rust·Python Analyze 성공, aggregate neutral |
| Proptest roundtrip | [32690539626](https://github.com/edwardkim/rhwp/actions/runs/32690539626) | prop roundtrip 성공 |
| Adapter inter-diff | [32690539702](https://github.com/edwardkim/rhwp/actions/runs/32690539702) | 성공 |
| Render Diff | [32690539724](https://github.com/edwardkim/rhwp/actions/runs/32690539724) | Canvas visual diff와 CanvasKit readiness 성공 |

current-base bridge와 이 self-review·오늘할일은 candidate 뒤의 허용된 `mydocs/` 변경이다. push 뒤
review-only fast-pass가 같은 candidate를 재사용하는지, 최신 aggregate가 성공하는지, head SHA와
`MERGEABLE/CLEAN`이 일치하는지 다시 확인해야 한다.

## 최종 권고

암호 문서의 보호 의도 수명주기, 실패 원자성, 암호 비보존 경계가 구현과 회귀 테스트에서 일치하며 추가
코드 blocker는 발견하지 않았다. self-review는 **완료 / 조건부 merge 권고**다. 최신 trailing head의
fast-pass required checks, `MERGEABLE/CLEAN`, manifest 기준선 예외 공개 상태와 작업지시자의 별도 merge
승인을 확인하기 전에는 merge하지 않는다.

## 통합 체리픽 검토 - 2026-08-24

open PR 중 CI가 통과한 항목만 모은 `review/open-ci-green-20260824` 통합 후보에 #5990 최신 head
`2047ad9f6d8fccf431b9a08b602c81efff616722`를 포함했다. `mydocs/orders/20260824.md` 충돌은 #5985,
#5776, #5986 기록을 모두 보존하는 방식으로 해소했다.

통합 후보의 전체 nextest 8292 passed / 42 skipped, Studio test/build, CI impact tests, render-diff workflow
unittest, 기본 all-targets clippy가 통과했다. #5990은 통합 PR에서 수용 권고로 처리한다. 통합 PR 생성은
작업지시자 사전 승인 전까지 진행하지 않는다.
