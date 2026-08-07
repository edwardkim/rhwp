---
kind: pr_review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-07
---

# PR #4177 검토 - mydocs 충돌 merge의 녹색 source head 재사용

## 대상과 변경 경계

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#4177](https://github.com/edwardkim/rhwp/pull/4177) / @jangster77 |
| 기준 `devel` | `f048429eb282778a49e3bb4e7f748146321721b5` |
| code head | `8119074ceb7bd4491b7fca134b05eff99970148e` |
| 연동 이슈 | `Closes #4176` |
| 작성 시점 merge 상태 | `mergeable=MERGEABLE`, `mergeStateStatus=CLEAN` |

라우팅은 `collaborator_self_merge`를 기본으로 하고, `intake_and_review`, `local_validation`을
보조 경로로 적용했다. 자기 자신은 GitHub reviewer request 대상으로 지정할 수 없어 별도 reviewer
request는 생성되지 않았다.

이 PR은 최신 `devel`을 병합하면서 `mydocs/`만 수동 해소한 경우를 보완한다. 직접 source parent가
같은 PR source의 녹색 CI, CodeQL, Render Diff 결과를 가질 때 source parent 자체가 과거 merge여도
재사용한다. 실제 재현 대상은 #4136과 #4165다.

재사용은 다음 조건을 모두 유지한다.

- 최신 commit이 current-base merge이고 trusted remerge diff의 수동 해소 경로가 `mydocs/` 아래여야 한다.
- source·test·sample·PDF·golden·baseline 충돌 해소, identity 불일치, 녹색 후보 부재는 Full CI로 fallback한다.
- source PR이 `.github/workflows/**`, `.github/actions/**`, CI impact classifier 또는 merge-resolution
  verifier를 수정하면 `current-base-source-ci-execution-change`으로 판정해 Full CI를 강제한다. #4170이
  이 경계에 해당한다.
- Render Diff의 prior-base identity 허용은 위 직접 source-parent 재사용 경로에만 한정한다.

문서·render 산출물 자체의 변경은 없으므로 시각 fixture 증적 경로는 적용하지 않았다. 다만 Render Diff
workflow를 바꾸므로 workflow 계약과 실제 Canvas visual diff 완료 여부를 검증했다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| workflow 계약 | `python3 -m unittest scripts/tests/test_review_only_fast_pass_workflows.py scripts/tests/test_cache_sweep_workflow.py scripts/tests/test_workflow_contract_wiring.py`를 실행해 36건 통과 |
| workflow 구문 | `actionlint .github/workflows/ci.yml .github/workflows/codeql.yml .github/workflows/render-diff.yml` 통과 |
| 문서·diff 정합 | `python3 scripts/check_markdown_links.py --changed-from HEAD`, `git diff --check` 통과 |
| 실제 bridge 구조 | merge-resolution checker가 #4136 `bed52d02`, #4165 `d2621a6`에 `current-base-merge-resolution-mydocs-only`를 반환 |
| CI 실행 경계 | #4136·#4165는 CI 실행 경로 변경 없음, #4170은 `.github/workflows/ci.yml`과 `scripts/ci-impact-classifier.cjs` 변경으로 guard 대상임을 확인 |
| GitHub CI | code head `8119074ce`에서 CI run `31188064589`의 lint, Native Skia, 세 archive build, 네 test shard, aggregate가 모두 성공 |
| GitHub CodeQL | run `31188064054`의 Python, JavaScript/TypeScript, Rust 분석과 aggregate 성공 |
| GitHub Render Diff | run `31188064352`의 Canvas visual diff 성공 |

## 추가 보정

첫 trailing 문서 head `cfe6a1ccc`는 `8119074ce`의 실제 Canvas 성공 결과를 재사용해 fast-pass됐다.
그 뒤 문서 결과 정정 head `b3d328ce5`는 중간 후보 `cfe6a1ccc`의 Canvas `skipped`를 실패로 간주해
더 이전의 `8119074ce`를 탐색하지 않고 Canvas를 재실행했다.

Stage 2 보정은 Canvas `skipped`를 재사용 불가 후보로만 분류한다. 후보 loop는 같은 PR·branch·repository·
base identity를 다시 확인하면서 더 이전의 실제 Canvas 성공 결과를 계속 찾는다. 실제 실패·identity 불일치와
실행 중 후보는 기존 full Render Diff fallback 또는 대기 처리를 유지한다.

- Stage 2 workflow 계약, cache sweep, workflow wiring은 37건 통과했다.
- `actionlint .github/workflows/render-diff.yml`, Markdown 링크 검사, `git diff --check`를 통과했다.
- Stage 2 code head `f571572a9`의 CI run `31190273592`에서 lint, Native Skia, 세 archive build,
  네 test shard와 aggregate가 모두 성공했다.
- CodeQL run `31190269950`의 Python, JavaScript/TypeScript, Rust 분석과 aggregate, Render Diff run
  `31190270359`의 Canvas visual diff가 모두 성공했다.

## 수용 판단과 다음 검증

**수용 권고.** CI 실행 경로를 바꾼 PR은 재사용 대상에서 제외하고, mydocs-only current-base merge의
직접 source parent만 좁게 허용하므로 기존 Full CI fallback 경계를 넓히지 않는다.

`8119074ce`의 CI 실행 경로 변경은 Full CI로 검증했고, 첫 trailing head `cfe6a1ccc`는 같은 녹색
candidate를 재사용해 fast-pass됐다. CI run `31189434770`에서 모든 heavy job과 test shard가 skip됐고
Build & Test aggregate가 성공했으며, CodeQL run `31189434069`과 Render Diff run `31189434104`도
분석·Canvas job을 skip하고 preflight aggregate가 성공했다.

Stage 2는 Render Diff workflow와 계약을 변경하므로 `f571572a9`에서 CI·CodeQL·Render Diff Full CI를
다시 완료했다. 이 최종 review·오늘할일 commit은 trailing documentation commit으로 push해
`f571572a9`의 실제 Canvas 성공 candidate를 재사용하는지 확인한다. 별도의 mydocs-only current-base
병합 PR은 사용자가 수동 conflict resolve로 검증한다. merge 전에는 최신 head의 required check와 merge
상태를 다시 확인하고 작업지시자 승인을 받는다.
