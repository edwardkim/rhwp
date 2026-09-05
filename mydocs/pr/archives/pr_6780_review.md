# PR #6780 검토 기록

## 최종 판정

판정: 승인

로컬 계약 검증과 최종 head의 GitHub Actions를 통과한 뒤 일반 merge commit으로 병합했다. 실제 devel push 검증과 후속 처리도 완료했다. 이어 같은 저장소의 Frontend 변경 PR #6781에서 Rust duration artifact 없이 CI 재사용이 성공한 사실까지 확인했다. 이 보완 기록은 2026-09-05에 확인한 실행 결과를 기준으로 한다.

## 검토 대상

| 항목 | 내용 |
| --- | --- |
| PR | [#6780](https://github.com/edwardkim/rhwp/pull/6780) |
| 관련 이슈 | [#6779](https://github.com/edwardkim/rhwp/issues/6779) |
| 작성자 및 검토 방식 | jangster77, collaborator 작성자 self-review |
| 대상 base | devel |
| 작업 branch | fix/6779-frontend-postmerge-reuse-20260905 |
| 구현 기준 base | fdba74164ef7003c68fdb14980a3ee1023957fed |
| 검증한 구현 commit | 21f89bf429e10a900f0c9610a268b83e1f12bacf |
| 최종 PR head | ecbf49f2591d7ac257bbeb3742d5d7f678c40517 |
| 실제 merge commit | a7b95f4041ef5d7d3574c4becfea5cb636eaf836 |
| 병합 시각 | 2026-09-05 13:46:20 UTC |
| 구현 변경 규모 | workflow 2개, JavaScript 검증기 1개, 계약 테스트 2개; 314줄 추가, 29줄 삭제 |
| 작성일 | 2026-09-05 |
| 원격 CI 및 merge 상태 | 최종 head의 required check와 실제 worker 결과를 확인하고 MERGEABLE/CLEAN 상태에서 일반 merge했다. PR 및 devel CI 결과는 아래 표에 기록한다. |

최초 검토 기록과 오늘할일은 구현 검증 뒤 같은 PR의 문서 전용 trailing commit에 포함되어 함께 병합됐다. 이번 보완은 작업지시자가 별도로 요청한 문서 전용 후속 PR이며, 오늘할일이나 이미 완료한 issue/PR comment 및 close를 반복하지 않는다. 저장소 owner를 reviewer로 자동 지정하지 않았다.

## 원인과 변경 범위

PR #6777의 [PR CI](https://github.com/edwardkim/rhwp/actions/runs/33967701594)는 Frontend package lane으로 성공했다. 그러나 [devel push CI](https://github.com/edwardkim/rhwp/actions/runs/33968179761)의 재사용 판정은 `candidate-full-lane-evidence-unavailable`로 거부되었다. Rust가 필요 없는 PR에도 Rust B/C/D nextest duration artifact가 있어야 재사용 후보가 되는 결합이 원인이었다.

PR #6772의 `devel -> main` 승격 검증과 이번 devel post-merge 재사용 판정은 다른 경로다.

이번 변경은 다음 범위로 제한했다.

- merge 전 신뢰된 base에서 classifier와 verifier를 로드한다.
- GitHub API의 PR 변경 파일을 분류해 Rust와 Native Skia가 불필요한 Frontend unit/package 변경만 추가 경로의 대상으로 삼는다.
- 최종 PR head의 성공한 CI에서 preflight, aggregate, 선택된 Frontend worker, 예상된 Rust skip을 확인한다.
- 19개 job 계약에서 누락, 중복, 알 수 없는 job, pending, 실패, 취소, 예상과 다른 skip/success를 거부한다.
- 기존 PR identity, repository, head, 실행 시각, merge-tree 및 enforcement-surface 검사를 유지한다.
- 검증되지 않은 최종 review head 뒤에 있는 과거 Frontend run으로 대체하지 않는다.
- Frontend-only 재사용에서는 duration 갱신을 건너뛰고, Rust CI 재사용에는 기존 B/C/D duration artifact 요구를 유지한다.
- 만료된 duration artifact를 증거에서 제외한다.

제품 source, renderer, sample, golden, baseline은 변경하지 않았다.

## 완료한 로컬 검증

| 명령 | 실제 결과 |
| --- | --- |
| `node --check scripts/verify-trusted-postmerge-ci-reuse.mjs` | 통과 |
| `node --test scripts/tests/verify-trusted-postmerge-ci-reuse.test.mjs` | 30개 통과, 실패 0개, skip 0개 |
| `python3 -m unittest scripts.tests.test_trusted_postmerge_ci_reuse_workflow scripts.tests.test_ci_impact_workflow` | 43개 통과 |
| `actionlint -shellcheck= .github/workflows/trusted-postmerge-ci-reuse.yml .github/workflows/ci.yml` | 통과; 외부 ShellCheck는 이 명령에서 비활성화 |
| `git diff --check` | 구현 후보에서 통과 |

기존 계약 테스트를 약화하지 않고 Frontend unit/package 수용과 실패·취소·pending·identity 불일치·증거 누락·과거 head 거부 사례를 추가했다. 완료한 테스트는 커밋·PR 생성 단계에서 다시 실행하지 않았다. 원시 실행 로그는 임시 디렉터리에만 두고 PR에 포함하지 않았다.

## 생략한 검증과 시각 증적

Rust source와 제품 동작을 변경하지 않아 Cargo 전체 회귀 테스트, WASM build, Studio 테스트는 실행하지 않았다. 렌더링·레이아웃·문서 출력 변경이 없어 visual sweep, PDF 변환, 대표 PNG 산출은 필요하지 않다.

위 생략 범위는 #6780의 로컬 검증에 대한 설명이다. 실제 GitHub worker 실행과 #6781의 Frontend-only 재사용 결과는 아래에 별도로 기록하며, 이번 문서 보완을 위해 이미 완료한 테스트를 반복 실행하지 않았다.

## 실제 PR 및 devel 검증: #6780

최종 head `ecbf49f2591d7ac257bbeb3742d5d7f678c40517`의 required check와 aggregate, 실행된 worker를 확인한 뒤 병합했다. 아래 devel 결과는 모두 merge SHA `a7b95f4041ef5d7d3574c4becfea5cb636eaf836`의 push 실행이다.

| Workflow | 최종 PR head | merge SHA의 devel push |
| --- | --- | --- |
| CI | [33969065110](https://github.com/edwardkim/rhwp/actions/runs/33969065110), 성공 | [33969863629](https://github.com/edwardkim/rhwp/actions/runs/33969863629), 성공 |
| CodeQL | [33969065103](https://github.com/edwardkim/rhwp/actions/runs/33969065103), 성공 | [33969863662](https://github.com/edwardkim/rhwp/actions/runs/33969863662), 성공 |
| Adapter inter-diff | [33969065154](https://github.com/edwardkim/rhwp/actions/runs/33969065154), 성공 | [33969863658](https://github.com/edwardkim/rhwp/actions/runs/33969863658), 성공 |
| Proptest | [33969065230](https://github.com/edwardkim/rhwp/actions/runs/33969065230), 성공 | [33969863727](https://github.com/edwardkim/rhwp/actions/runs/33969863727), 성공 |
| Close Issues | 해당 없음 | [33969863494](https://github.com/edwardkim/rhwp/actions/runs/33969863494), 성공 |

- PR과 devel의 CodeQL은 JavaScript, Python, Rust 분석 worker가 실제로 성공했다.
- devel CI는 preflight, Build & Test, Lint, Native Skia, Frontend package, Rust archive A/B/C/D build 및 test가 성공했다. 선택되지 않은 별도 Frontend unit, WASM 및 promotion 경로는 정책상 skip이었다.
- [nextest duration 갱신 job](https://github.com/edwardkim/rhwp/actions/runs/33969863629/job/101317697248)도 성공했다.
- CI enforcement 변경인 #6780에는 Render Diff workflow가 실행되지 않았다. 이를 시각 검증 성공으로 간주하지 않는다.

첫 merge의 [재사용 판정 job](https://github.com/edwardkim/rhwp/actions/runs/33969863629/job/101316335319)은 다음을 출력했다.

```text
reuse=false reason=pr-changes-ci-enforcement-surface source_run_id=none refresh_duration_data=false
```

이는 controller 자체를 변경한 PR의 기존 CI 증거를 무조건 신뢰하지 않는 예상된 안전 동작이다. 재사용 출력의 `refresh_duration_data=false`는 Full CI 이후의 일반 duration 갱신까지 막는다는 뜻이 아니다. 이 실행에서는 재사용이 거부되어 Full CI와 일반 duration 갱신이 실제로 수행됐다.

## 실제 Frontend-only 재사용 검증: #6781

[PR #6781](https://github.com/edwardkim/rhwp/pull/6781)은 원 [PR #5953](https://github.com/edwardkim/rhwp/pull/5953)의 Frontend 변경을 provenance-preserving cherry-pick으로 같은 저장소의 임시 branch에 통합한 검증 사례다. 제품 변경은 Studio command/history와 관련 테스트이며, 검토 문서 및 오늘할일을 함께 포함했다. 코드 후보의 tree는 devel에 정렬한 원 PR과 동일한 `0ac7d0c362342867e488b2dd6ebde1f99184ffb3`임을 확인했다.

| 항목 | 실제 확인값 |
| --- | --- |
| head repository | edwardkim/rhwp |
| 최종 PR head | a95d56a7df6327a76ff928adb9285d9e8184c2f6 |
| merge commit | 656af6bdbdce290a65a00fd2ac35fa18a8f38120 |
| 병합 시각 | 2026-09-05 14:05:20 UTC |
| classifier | rust_required=false, frontend_mode=package, native_skia_required=false, render_required=true, codeql_languages=javascript-typescript |
| merge 직전 상태 | 최종 head의 required check 충족, MERGEABLE/CLEAN |

검토 문서를 포함한 최종 head에서 Frontend package worker가 실제 실행되었다. 원 fork PR의 base-update fast-pass나 이전 코드 head의 성공을 최종 head의 실제 실행으로 대신 기록하지 않았다.

| Workflow | 최종 PR head의 검증 | merge SHA의 devel push |
| --- | --- | --- |
| CI | [33970151406](https://github.com/edwardkim/rhwp/actions/runs/33970151406), Frontend package와 Build & Test 성공 | [33970764979](https://github.com/edwardkim/rhwp/actions/runs/33970764979), 재사용 및 aggregate 성공 |
| CodeQL | [33970151426](https://github.com/edwardkim/rhwp/actions/runs/33970151426), JavaScript 분석 성공 | [33970765020](https://github.com/edwardkim/rhwp/actions/runs/33970765020), 재사용 성공, 분석 worker skip |
| Adapter inter-diff | [33970151397](https://github.com/edwardkim/rhwp/actions/runs/33970151397), worker 성공 | [33970764988](https://github.com/edwardkim/rhwp/actions/runs/33970764988), 재사용 성공, worker skip |
| Proptest | [33970151412](https://github.com/edwardkim/rhwp/actions/runs/33970151412), worker 성공 | [33970765087](https://github.com/edwardkim/rhwp/actions/runs/33970765087), 재사용 성공, worker skip |
| Canvas Render Diff | [33970151348](https://github.com/edwardkim/rhwp/actions/runs/33970151348), 성공 | push에서는 실행되지 않음 |
| Close Issues | 해당 없음 | [33970764743](https://github.com/edwardkim/rhwp/actions/runs/33970764743), 성공 |

#6781 PR의 CodeQL Python/Rust wrapper 표시를 실제 언어 분석 실행으로 해석하지 않았다. 선택된 분석 언어는 JavaScript였고, 플랫폼 CodeQL check의 NEUTRAL은 정책상 허용된 결과였으며 필수 aggregate 조건은 충족했다.

devel CI의 [실제 재사용 판정](https://github.com/edwardkim/rhwp/actions/runs/33970764979/job/101318740975)은 다음과 같다.

```text
reuse=true reason=review-tail-final-head-green-frontend-ci-reused source_run_id=33970151406 refresh_duration_data=false
```

CodeQL, Adapter, Proptest도 각각 위 표의 최종 PR run을 source로 사용했으며, 모두 `reuse=true reason=review-tail-final-head-green-pr-workflow-reused`였다.

- source CI run의 artifact API 결과는 유효한 `trusted-postmerge-merge-tree-v1-...` artifact 1개였으며, Rust B/C/D duration artifact는 없었다. Rust timing artifact 없이 Frontend CI가 실제 재사용됐다.
- devel의 preflight와 Build & Test는 성공했고, [Frontend package](https://github.com/edwardkim/rhwp/actions/runs/33970764979/job/101318789356), Rust archive build/test, Lint, Native Skia, 별도 WASM worker는 skip이었다.
- [Refresh nextest target duration data](https://github.com/edwardkim/rhwp/actions/runs/33970764979/job/101318790117)도 skip이었다. Frontend-only 사례에는 갱신할 Rust duration 증거가 없으므로 이것이 의도한 결과다.
- PR의 [Frontend package worker](https://github.com/edwardkim/rhwp/actions/runs/33970151406/job/101317236095)는 13:53:35~14:00:12 UTC, 6분 37초 동안 실행됐다. devel CI run은 API의 시작·최종 갱신 시각 기준 14:05:22~14:05:53 UTC, 약 31초였다. 서로 다른 범위의 시간값이므로 동일 작업의 정밀 성능 비교로 일반화하지 않는다.

## 검증 범위와 남는 제한

- job 이름이나 topology가 바뀌면 계약이 일치하지 않아 재사용을 거부한다. 조용히 검증 범위를 줄이는 대신 기존 실행 경로로 돌아간다.
- #6780의 최초 merge에서 enforcement 변경에 따른 재사용 거부와 Full CI 성공을 확인했다. #6781에서는 변경된 controller가 적용된 뒤의 Frontend package 재사용 성공을 확인했다.
- 실제 성공 사례는 같은 저장소 `edwardkim/rhwp`의 PR #6781이다. contributor fork의 원 PR #5953 자체에 대한 재사용 허용이나 fork 보안 정책 변경을 검증한 것은 아니다.
- 실제 원격 성공 사례는 Frontend package 경로다. Frontend unit 전용 경로와 실패·취소·pending 등 거부 조건은 위 로컬 계약 테스트 범위이며, 모든 조합을 실제 Actions에서 재현했다고 주장하지 않는다.
- 모든 workflow를 무조건 skip하거나 모든 PR에서 duration 갱신만 실행하도록 변경하지 않는다.
- Rust가 필요한 PR의 기존 B/C/D duration artifact 요구를 유지한다. 새로운 PR은 해당 최종 head의 required check, 관련 aggregate와 worker, MERGEABLE/CLEAN 조건을 별도로 확인해야 한다.

## 완료한 후속 처리와 증적 보존

[post_merge.md](../../manual/pr_review/post_merge.md)에 따라 두 merge SHA의 실제 devel CI 결과를 확인한 뒤 처리했다.

- PR #6780 본문의 `Closes #6779`를 확인했다. GraphQL closingIssuesReferences는 비어 있었지만, [Close Issues bot 기록](https://github.com/edwardkim/rhwp/issues/6779#issuecomment-5552235357)과 API의 CLOSED 상태 및 2026-09-05 13:46:32 UTC 종료 시각을 확인했다.
- [PR #6780 후속 comment](https://github.com/edwardkim/rhwp/pull/6780#issuecomment-5552358693)와 [issue #6779 후속 comment](https://github.com/edwardkim/rhwp/issues/6779#issuecomment-5552358810)에 실제 merge SHA, PR/devel CI, 최초 Full CI의 이유와 #6781 재사용 성공을 기록했다.
- [PR #6781 후속 comment](https://github.com/edwardkim/rhwp/pull/6781#issuecomment-5552358524)를 게시했다. 원 [PR #5953 수용 comment](https://github.com/edwardkim/rhwp/pull/5953#issuecomment-5552358253)에 #6781 체리픽 통합 수용 사실을 남기고 2026-09-05 14:08:30 UTC에 CLOSED 처리했다. 원 PR은 직접 merge하지 않았으며 contributor fork branch는 보존했다.
- comment는 UTF-8 body file로 한 번씩 게시하고 API로 본문 일치를 확인했다. 이번 문서 보완에서 comment, issue close, source PR close를 다시 수행하지 않는다.
- 두 merge SHA의 upstream/devel 포함과 기본 devel 작업공간의 clean 상태를 확인하고, 이 작업 소유 local branch만 정리했다. upstream 임시 remote branch, contributor fork branch, 기본 작업공간 및 공유 target은 보존했다.
- 증적은 위 Actions run/job 및 comment permalink로 보존한다. 시각 비교가 판단 근거가 아니므로 무관한 이미지, PDF, 임시 원시 로그를 추가하지 않는다.
- 이 보완 문서 PR은 작업지시자의 별도 요청에 따른 기록 갱신이다. 제품 source, test, workflow, golden/baseline, sample, 신규 LFS 및 오늘할일은 변경하지 않고, 이미 완료한 PR #6777의 source PR 처리도 반복하지 않는다.
