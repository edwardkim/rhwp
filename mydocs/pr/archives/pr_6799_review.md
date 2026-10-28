---
kind: pr_review
status: approved
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-06
pr: 6799
issue: 6634
author: edwardkim
---

# PR #6799 self-review — Release 이후 package publish 직접 호출

## 결론

**승인.** PR #6799의 current candidate
`82d92923573c6cbcf4e879a989ec9a4d989103a6`은 `release.published` 간접 이벤트를 제거하고, stable tag의
Release Binary가 다섯 플랫폼 build와 Release 게시 성공 뒤 같은 commit의 reusable package workflow를
직접 호출한다. exact tag source guard, 네 채널의 멱등·독립 재시도, 구조화 완료 evidence와
verify-only promotion 계약이 함께 닫혀 있어 범위 안 blocker는 없다.

exact executable head의 비게시 Actions와 current PR head의 Full CI·CodeQL이 모두 성공했다. 이 문서와
오늘할일만 추가하는 trailing review-only head의 Actions, 최신 `devel` 정합, `MERGEABLE`·`CLEAN`을 다시
확인한 뒤 메인테이너의 별도 merge 승인을 받아야 한다.

이 문서의 `승인`은 작성자 self-review 판정이다. 자기 PR이므로 reviewer 지정과 GitHub approve event를
만들지 않는다. PR 본문은 `Refs #6634`이며, 다음 stable release canary 전에 이슈를 자동 close하지 않는다.

## 라우팅과 메타데이터

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`,
  `rework_and_exceptions.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`와 위 자식 문서
- 대형 PR 예외: 1,000줄을 넘는 release workflow·검증 도구 변경이므로 current candidate Full CI와
  review-only trailing cycle을 분리한다. admin merge나 branch protection 우회는 사용하지 않는다.
- 별도 `review_impl`은 만들지 않는다. 원인·구현·rollback 순서는 승인된
  [수행계획](../../plans/task_m100_6634.md)과
  [최종 보고서](../../report/task_m100_6634_report.md)에 이미 고정됐고 추가 보정 선택지가 없다.

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#6799](https://github.com/edwardkim/rhwp/pull/6799) / @edwardkim |
| 관련 이슈 | [#6634](https://github.com/edwardkim/rhwp/issues/6634) (`Refs #6634`) |
| base | `devel@ff1ce007b428547da74e0d6b7e9a196592c60ff6` |
| executable dogfood head | `559edb06826e7b8bfa2348d5951d78cf18d066e9` |
| current candidate | `82d92923573c6cbcf4e879a989ec9a4d989103a6` |
| 규모 | 23 files, `+3148/-94`, 12 commits |
| 작성 시점 GitHub 상태 | Open, non-draft, `MERGEABLE`·`CLEAN`, candidate checks 완료 |
| assignee / labels / milestone | `edwardkim` / `bug`, `ci`, `packaging` / `v1.0.0` |
| reviewer | self PR이므로 지정하지 않음 |

## 원인과 구현 검토

1. v0.8.4·v0.8.6은 workflow의 `GITHUB_TOKEN`으로 draft Release를 게시했다. 이 토큰이 만든
   `release.published` 이벤트는 새 workflow를 기동하지 않아 package run이 생성되지 않았다.
2. 그 이전 간접 package run은 binary attachment보다 먼저 시작했고, v0.8.6 수동 복구는 release tag가
   아니라 후속 main hotfix에서 실행됐다.
3. 새 Release Binary caller는 `build=success`이며 production에서는 `release=success`, manual에서는
   `release=skipped`일 때만 reusable workflow를 호출한다. release 실패나 native matrix 실패를 package
   publish가 우회하지 못한다.
4. production publish 입력은 stable `v*` tag push에서만 true다. guard는 checkout·GitHub·tag SHA,
   Cargo/editor/VS Code version과 published non-draft, non-prerelease Release를 모두 대조한다.
5. npm core/editor, VS Code Marketplace, Open VSX job은 exact version을 먼저 조회한다. 기게시 채널은
   secret 사용 step 전 `already-present`로 종료하고, 조회 오류는 미게시로 완화하지 않는다.
6. VSIX는 한 번만 만들고 두 extension channel이 같은 artifact를 소비한다. 한 채널 실패 후 재실행해도
   성공한 채널은 건드리지 않는다.
7. aggregate는 source guard, WASM, VSIX와 네 채널을 `completed | failed`로 구조화하며 오류가 있으면
   workflow를 실패시킨다. `release.published` 별도 run을 관찰할 필요 없이 Release Binary 한 run 안에서
   package 완료 여부가 드러난다.
8. #6689 promotion policy는 exact candidate의 실제 REST job 이름, success·skip 배열과 artifact 안의
   `verdict=completed`를 확인한다. 최초 dogfood의 caller 표시명 차이를 실제 API 값으로 정정했으며
   verifier나 waiver를 완화하지 않았다.

## 보호 불변식

| 불변식 | self-review 결과 |
| --- | --- |
| binary 선행 | 5-platform build와 Release 성공 전 production package publish 불가 |
| exact source | tag/ref/SHA/version/Release metadata 중 하나라도 다르면 publish 전 실패 |
| 최소 권한 | 기본 `contents: read`; Release만 write, npm만 OIDC; secret 두 이름만 명시 전달 |
| 멱등 재시도 | exact version 기게시 채널 skip, 미완료 채널만 독립 재시도 |
| 조회 실패-폐쇄 | timeout·5xx·JSON·identity 오류를 미게시로 해석하지 않음 |
| 비게시 dogfood | branch manual run에서 Release와 외부 publish 4개가 반드시 skipped |
| 구조화 완료 | build·채널 누락은 aggregate 실패; artifact에 token·인증 URL 없음 |
| promotion | exact-head run·job·artifact·verdict가 없으면 #6689 verifier가 거부 |

## 로컬 검증

| 검증 | 결과 |
| --- | --- |
| release/channel/promotion focused Python | 62건 PASS |
| workflow 전체 Python 계약 | 161건 PASS |
| Python·JSON 구문 | PASS |
| actionlint 1.7.7 | 변경 workflow 2개 PASS |
| Docker optimized WASM | release build와 `wasm-opt -O` PASS, 6분 53초 |
| editor/package/VSIX | editor 32건, npm core/editor pack, VSIX 37개 파일 PASS |
| package 금지 경로 | `.env*`, `node_modules/`, `target/`, token 이름 없음 |
| Markdown link / `git diff --check` | PASS |

Rust source·test·fixture·renderer를 바꾸지 않아 Rust 제품 전체와 시각 회귀는 비대상이다. Stage 5-A 뒤
병합된 devel 범위에도 package·workflow·Rust 관련 파일은 없었고, exact executable head의 원격 run이
WASM과 VSIX를 다시 만들었다. 따라서 고비용 package build를 current 문서 head에서 중복 실행하지 않았다.

## exact-head 원격 실증

| workflow | run | 판정 |
| --- | ---: | --- |
| Release Binary | [34001610087](https://github.com/edwardkim/rhwp/actions/runs/34001610087) | success; native 5종·nested package artifact 8개 |
| Publish All Packages | [34001611474](https://github.com/edwardkim/rhwp/actions/runs/34001611474) | success; package artifact 3개 |

두 run은 executable head `559edb06826e7b8bfa2348d5951d78cf18d066e9`에서 실행됐다. Release attach와
외부 publish 4개는 skipped됐고 evidence는 세 gate `success`, 네 채널 `skipped/verify-only`,
`errors=[]`, `accepted=true`, `verdict=completed`였다. offline verifier는 pagination 완결, waiver 0건,
오류 0건, `ok=true`로 두 run을 수락했다.

실행 전후 공개 네 채널은 기존 `0.8.6 already-present`였고 `test` Git tag와 Release는 생성되지 않았다.
이후 Stage 5·최종 보고서만 추가된 current candidate를 위 run의 실행 SHA로 표현하지 않는다.

## current candidate GitHub Actions

| workflow | run | 판정 |
| --- | ---: | --- |
| CI Impact Policy Controller | [34004184124](https://github.com/edwardkim/rhwp/actions/runs/34004184124) | success; `mode=full`, `rfp=0`, `wf=110` |
| CI | [34004184522](https://github.com/edwardkim/rhwp/actions/runs/34004184522) | success; lint, Native Skia, frontend package, archive build·test 4종, aggregate 통과 |
| CodeQL | [34004184429](https://github.com/edwardkim/rhwp/actions/runs/34004184429) | success; JavaScript/TypeScript, Python, Rust 분석 통과 |
| Adapter inter-diff | [34004184458](https://github.com/edwardkim/rhwp/actions/runs/34004184458) | success |
| Proptest roundtrip | [34004184490](https://github.com/edwardkim/rhwp/actions/runs/34004184490) | success |

`wf=110`은 CI·CodeQL은 Full 실행하고 별도 Render Diff workflow는 비대상이라는 뜻이다. CI 내부
Native Skia와 frontend package는 실제 성공했다. `WASM Build`, Frontend unit, duration refresh와 일반
PR의 promotion preflight는 정책상 expected skip이다. required `Build & Test`는 성공했고 pending·failure가
없다.

## 성능·비용·잔여 위험

- 제품 실행 코드와 공개 API를 바꾸지 않아 사용자 실행 성능에는 직접 영향이 없다.
- 비게시 dogfood wall time은 Release Binary 26분 34초, direct package 11분 23초였다. 두 run은
  direct workflow와 nested caller를 각각 입증하기 위한 promotion 증적이며 일상 PR CI에 넣지 않는다.
- `publish_extensions=false`도 VSIX build까지는 수행하고 외부 extension publish만 막는다. npm-only 복구의
  안전한 공통 build 검증 비용이며, 시간 최적화가 필요하면 별도 범위에서 다룬다.
- `upstream/main@e8800c8def63449808a4092798442652ed460552`에는 devel에 없는 v0.8.6 release merge와 release
  hotfix 두 commit이 남아 있다. 다음 release 전에 main을 devel에 동기화하고 새 exact devel SHA의 #6689
  promotion 증적을 만들어야 한다.
- 비게시 run은 stable tag의 production publish 성공을 대신하지 않는다. 다음 release canary에서 같은 tag
  SHA의 package 호출 1회, 네 채널과 evidence를 확인하기 전에는 #6634를 닫지 않는다.

## Rollback과 merge 후 계획

production 결함이면 일반 PR로 reusable caller를 제거하고 기존 Release·package version은 보존한다.
복구는 공개 채널 상태를 확인한 뒤 exact release tag에서 `Publish All Packages publish=true`를 실행한다.
움직이는 branch, 권한 확대, verifier waiver를 사용하지 않는다.

정상 merge commit이 devel에 반영된 뒤에는 다음 순서를 적용한다.

1. merge SHA의 devel post-merge CI와 workflow 계약을 확인한다.
2. PR #6799에 candidate·trailing head·merge SHA와 post-merge run을 구분해 기록한다.
3. #6634에는 구현·비게시 dogfood·잔여 release canary를 요약하되 이슈를 close하지 않는다.
4. 로컬 devel을 fast-forward하고 이번 task branch만 정리한다. 원격 branch 삭제는 별도 승인 없이는 하지 않는다.
5. 다음 stable release canary가 통과하면 별도 승인 뒤 #6634를 close한다.

## 최종 판정과 다음 조건

- 판정: **승인**
- 판정 대상: current candidate `82d92923573c6cbcf4e879a989ec9a4d989103a6`
- trailing 조건: 이 review와 오늘할일만 추가하고 trusted fast-pass 또는 Full fallback, 최신 devel merge
  simulation, `MERGEABLE`·`CLEAN`을 재확인
- merge 조건: 최신 head SHA 고정, required check 성공, 메인테이너의 별도 merge 승인
- GitHub review: self PR이므로 approve event와 reviewer 지정 없음
- merge 방식: branch protection을 우회하지 않는 정상 merge commit
- issue close: merge 때 닫지 않고 다음 stable release canary 승인 뒤 별도 처리
