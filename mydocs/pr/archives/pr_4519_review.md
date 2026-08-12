---
kind: report
status: active
last_verified: 2026-08-11
---

# PR #4519 검토 — CodeQL 언어별 선택 실행

## 라우팅

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md,
  multi_pr_update_branch.md, review_only_fast_pass.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
  pr_review/collaborator_self_merge.md, pr_review/intake_and_review.md,
  pr_review/local_validation.md, pr_review/multi_pr_update_branch.md,
  pr_review/review_only_fast_pass.md
current head: 64175764120e97eb9af0f0a55be4da86c072cdc6
```

## Metadata

| 항목 | 검토 기록 |
| --- | --- |
| PR | [#4519](https://github.com/edwardkim/rhwp/pull/4519) |
| 작성자 | `postmelee` |
| base / head | `devel` / `issue-3790-stage5b-codeql-languages` |
| 관련 이슈 | [#3790](https://github.com/edwardkim/rhwp/issues/3790) |
| 코드 candidate 규모 | 8개 파일, +513/-40 |
| 작성 시점 상태 | Draft, reviewer `edwardkim` 지정 |
| 최신 base | `upstream/devel` `32ecfd1136905c7b1bb26b16c47579a16143d305` |
| current-base merge | `64175764120e97eb9af0f0a55be4da86c072cdc6` |

## 변경 범위와 판단

Stage 5A에서 보강한 candidate-bound CodeQL 검증을 유지하면서, PR base SHA의 trusted classifier가 낸
`codeql_languages`에 따라 JavaScript/TypeScript·Python·Rust 분석만 선택 실행한다. 세
`Analyze (...)` matrix job은 항상 생성하며 선택하지 않은 lane은 명시적 no-op success로 끝내 check
identity를 보존한다. push·schedule·수동 실행과 판정 실패는 세 언어 full로 닫는다.

리뷰 [#5243938913](https://github.com/edwardkim/rhwp/pull/4519#issuecomment-5243938913)의
F1·F2·F4를 수용했다. preflight output 자체가 없을 때 consumer에서 세 언어 full로 복구하고,
`actions: read`의 candidate-bound workflow 조회 목적을 기록했으며, fast-pass Summary는 언어 판정을
실행하지 않았음을 `n/a (fast-pass)`로 표시한다. 보정 결과는
[#5244177145](https://github.com/edwardkim/rhwp/pull/4519#issuecomment-5244177145)에 게시했다.
F3에 따라 reviewer `edwardkim`을 지정하고 이 검토 기록을 추가했다.

제품 코드·renderer·fixture·baseline은 바뀌지 않는다. 따라서 Cargo, WASM과 시각 검증은 적용하지
않는다. Stage 5B merge 뒤에는 실제 frontend-only PR의 selective run과 같은 SHA의 수동 full run을
대조하는 canary가 별도로 필요하다.

## 검증

- 리뷰 보정 candidate `b5a9bffd5f04c48c88a8f82fe54d498d98a6cd7e`의 CI run
  `31418214475`와 CodeQL run `31418214336`에서 Build & Test, 세 `Analyze (...)` job과 GHAS
  `CodeQL`을 포함한 모든 check가 성공했다.
- 최신 `upstream/devel` `32ecfd1136905c7b1bb26b16c47579a16143d305`를 current-base merge commit
  `64175764120e97eb9af0f0a55be4da86c072cdc6`으로 충돌 없이 반영했다.
- current-base merge 뒤 `python3 -m unittest scripts/tests/test_codeql_workflow.py` 12/12,
  Codex 번들 Python의 `python3 -m unittest discover -s scripts/tests -p 'test_*.py'` 188/188,
  `node --test scripts/tests/ci-impact-classifier.test.cjs` 28/28이 통과했다.
- `actionlint .github/workflows/ci.yml .github/workflows/codeql.yml`과 `git diff --check`가 통과했다.
- 이 검토 기록을 포함한 최종 head에서는 수동 `workflow_dispatch`로 CI와 CodeQL full lane을 새로
  시작한다. 그 완료 여부는 이 문서 작성 시점에 선판정하지 않는다.

## 잔여 위험과 merge 조건

- preflight cancel·runner 장애로 output이 없는 실제 원격 상황은 강제 재현하지 않았다. consumer의 빈
  값 full fallback, 고정 matrix와 actionlint 계약으로 fail-closed 경계를 검증했다.
- GHAS `CodeQL` check는 세 언어 결과를 대신하지 않는다. 세 `Analyze (...)` job과 GHAS check를 계속
  독립 확인한다.
- workflow 자체 변경은 classifier에서 full로 닫히므로 이 PR에서 selective remote canary를 만들 수
  없다. 선택 실행 효과와 no-op 진리표는 merge 뒤 frontend-only canary에서 실측한다.

**최종 권고: 최신 head의 수동 full CI·CodeQL, `Build & Test`, 세 `Analyze (...)`와 GHAS `CodeQL`이
모두 성공하고 reviewer 검토가 끝난 뒤에만 Draft를 해제하고 merge한다. 이 기록만으로 CI 통과나 merge
승인을 갈음하지 않는다.**
