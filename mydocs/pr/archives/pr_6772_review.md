---
kind: pr_review
status: approved
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-05
pr: 6772
issue: 6689
author: edwardkim
---

# PR #6772 self-review — workflow 승격 전 exact-head 실행 증적 강제

## 결론

**승인.** PR #6772의 code candidate
`ccb732752cb7a8b0bda45d22833967811b9c08b8`은 `devel → main` 승격 전에 변경된 workflow의 exact-head
실행 증적을 수집·검증하고, 누락·stale·실패·정책 위반이면 기존 required context `Build & Test`를
fail-closed 한다. Pages 실제 배포, Gym full benchmark와 Oracle advisory 발행을 검증 실행과 분리해
수동 dogfood가 외부 변경을 일으키지 않도록 했다.

로컬 계약과 exact-head Full CI가 모두 성공했다. 8개 workflow 수동 dogfood도 verifier가 전부 수락했다.
Fuzz smoke의 `parse_wmf` overflow panic은 숨기지 않고 제품 결함으로 분리 보존했으며, 이 실패를 #6689
promotion gate의 성공으로 표현하지 않는다. 범위 안 구현 blocker는 없다.

이 문서의 `승인`은 작성자 self-review 판정이다. 자기 PR이므로 reviewer 지정이나 GitHub approve event를
만들지 않는다. 이 review·오늘할일·최종 보고서 상태만 추가하는 review-only trailing head의 Actions,
최신 `devel` 정합과 `MERGEABLE`·`CLEAN`을 다시 확인한 뒤 메인테이너의 별도 merge 승인을 받아야 한다.

## 라우팅과 메타데이터

- 기본 경로: `collaborator_self_merge.md`
- 보조 경로: `intake_and_review.md`, `local_validation.md`, `rework_and_exceptions.md`,
  `review_only_fast_pass.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`와 위 자식 문서
- 대형 변경 예외: 1,000줄을 넘는 workflow·검증 도구 작업이므로 코드 후보 CI와 review-only trailing
  문서 주기를 분리했다. admin merge나 branch protection 우회는 사용하지 않는다.
- 구현·검증 계보: [수행계획](../../plans/task_m100_6689.md),
  [Stage 1](../../working/task_m100_6689_stage1.md),
  [Stage 2](../../working/task_m100_6689_stage2.md),
  [Stage 3](../../working/task_m100_6689_stage3.md),
  [Stage 4](../../working/task_m100_6689_stage4.md),
  [Stage 5](../../working/task_m100_6689_stage5.md),
  [최종 보고서](../../report/task_m100_6689_report.md)

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#6772](https://github.com/edwardkim/rhwp/pull/6772) / @edwardkim |
| 관련 이슈 | [#6689](https://github.com/edwardkim/rhwp/issues/6689) (`Closes #6689`) |
| base | `devel@2c144b180dd776aa450c499778510199ae6cdf89` |
| code candidate | `ccb732752cb7a8b0bda45d22833967811b9c08b8` |
| 규모 | 26 files, `+4717/-14`, 16 commits |
| 작성 시점 GitHub 상태 | Open, 비 Draft, `MERGEABLE`·`CLEAN`, candidate checks 완료 |
| assignee / label / milestone | `edwardkim` / `bug`, `ci`, `test` / `v1.0.0` |
| reviewer | self PR이므로 지정하지 않음 |

## 문제 원인과 구현 검토

1. PR #5366은 Fuzz 원격 확인 필요성을 텍스트로 남겼지만 merge 종료 gate로 만들지 않았다.
2. 통합 PR #5425의 일반 CI 성공은 새 workflow 자체의 실행을 보증하지 못했고, Fuzz는 `main` schedule에서
   처음 실행돼 `parse_wmf` panic을 뒤늦게 드러냈다.
3. `workflow_promotion_preflight.py`는 `main..candidate`의 workflow·local action을 Git blob과 SHA-256으로
   inventory화하고 executable 변경과 민감 표면을 구조화한다.
4. `workflow_promotion_evidence.py`는 GitHub API에서 exact candidate SHA의 run·job·artifact만 읽고,
   pagination·byte 상한·digest·필수 verdict·actor와 event까지 검증한다.
5. CI promotion job은 same-repository `devel → main` PR에서만 실행된다. 일반 PR·fork·push에는 skipped를
   요구하므로 새 gate가 기존 개발 주기마다 원격 workflow 전건 실행을 강요하지 않는다.
6. waiver는 메인테이너·SHA·workflow hash·scope·만료가 일치해야 하며 permission·secret·security·deployment
   표면이나 실패한 exact run에는 사용할 수 없다.
7. Pages는 build/artifact까지만, Gym은 contracts-only, Oracle은 구조화 verdict까지만 증적으로 인정해
   검증과 실제 배포·full benchmark·advisory 발행 권한을 분리했다.

## 보호 불변식

| 불변식 | self-review 결과 |
| --- | --- |
| exact-head | candidate SHA, workflow content hash와 run head가 다르면 거부 |
| stale 방지 | live PR/base와 맞지 않는 과거 run·job·artifact를 재사용하지 않음 |
| fail-closed | 누락·실패·API 상한·artifact 변조·정책 부재를 성공으로 축소하지 않음 |
| 권한 최소화 | collector는 read-only이며 workflow dispatch·branch·issue·artifact를 변경하지 않음 |
| required context 보존 | 기존 `Build & Test` 이름과 branch protection을 유지한 채 결과를 집계 |
| 외부 효과 분리 | Pages deploy, Gym full, Oracle publish는 증적용 기본 실행에서 제외 |
| 일반 PR 비용 경계 | canonical same-repository `devel → main`이 아니면 promotion job은 expected skip |
| Fuzz 사실성 | 5 success와 `parse_wmf` failure를 분리해 기록하고 retry·waiver로 은폐하지 않음 |

## 로컬 및 원격 dogfood 검증

| 검증 | 결과 |
| --- | --- |
| CI impact classifier | Node 44건 통과 |
| CI impact policy | Node 37건 통과 |
| aggregate workflow 상태 | Python 35건 통과 |
| CI workflow contract 묶음 | 196건 통과 |
| Gym 직접 영향 | 8건 통과 |
| Python 구문·JSON·actionlint | 모두 통과; actionlint v1.7.12, 변경 workflow 4개 |
| Markdown 링크·변경 정본 metadata·diff check | 신규 이상 없음 |
| Stage 4 promotion workflow | exact candidate `76334ea1a`, 8개 전부 verifier 수락 |
| Stage 5 Fuzz smoke | exact `devel@2c144b1`, 6 matrix 생성; 5 success, `parse_wmf` failure |

Stage 4의 직접 실행은 Adapter, CI, CodeQL, Deploy Pages, Gym, Oracle advisory, Proptest와 Render Diff다.
run ID와 artifact 판정은 [최종 보고서](../../report/task_m100_6689_report.md#32-stage-4-exact-head-workflow-dogfood)에
고정했다. 해당 세트는 구현 dogfood이며 다음 릴리스 candidate의 증적으로 재사용하지 않는다.

Stage 5 run
[33959858373](https://github.com/edwardkim/rhwp/actions/runs/33959858373)의 `parse_wmf`는
`attempt to negate with overflow`로 실패했다. #6689의 목적은 workflow 실행 증적 누락 방지이므로 제품
panic 수정은 범위 밖이다. 그렇다고 실패를 성공이나 허용된 waiver로 바꾸지 않고 별도 제품 결함의 입력
digest와 artifact ID를 보존했다.

## PR exact-head GitHub Actions

| workflow | run | 판정 |
| --- | ---: | --- |
| CI | [33962237206](https://github.com/edwardkim/rhwp/actions/runs/33962237206) | success; Lint/WASM Clippy, Native Skia, frontend package와 4개 archive 통과 |
| CodeQL | [33962237194](https://github.com/edwardkim/rhwp/actions/runs/33962237194) | success; JavaScript/TypeScript, Python, Rust 분석 통과 |
| Adapter inter-diff | [33962237185](https://github.com/edwardkim/rhwp/actions/runs/33962237185) | success |
| Proptest roundtrip | [33962237238](https://github.com/edwardkim/rhwp/actions/runs/33962237238) | success |
| Gym Benchmark Audit | [33962237096](https://github.com/edwardkim/rhwp/actions/runs/33962237096) | contracts success; full benchmark expected skip |
| CI Impact Policy 최종 audit | [33962975688](https://github.com/edwardkim/rhwp/actions/runs/33962975688) | exact head status success |

최종 status 설명은
`v=6;cv=7;mode=full;rfp=0;wf=110;rust=1;fe=package;render=1;skia=1;ql=js,py,rs`다.
`wf=110`의 bit 순서는 CI/CodeQL/Render Diff이므로 이 PR에서는 CI와 CodeQL run만 필수이고 별도 Render
Diff workflow는 변경 경로상 비대상이다. `render=1`은 classifier가 CI 내부 renderer 검증을 요구한다는
별도 필드이며, Native Skia가 실제 성공했다. 두 값을 혼동해 미실행 workflow를 성공으로 세지 않았다.

`WASM Build`, Frontend unit, nextest duration refresh와 CI의 promotion preflight는 이번 PR event·변경
범위상 expected skip이다. WASM 전용 Clippy는 `Lint (fmt, clippy, WASM check)` 안에서 실제 성공했다.
candidate check에는 pending이나 failure가 없고 GitHub는 PR을 `MERGEABLE`·`CLEAN`으로 판정했다.

## 성능·비용과 시각 검증

- 제품 source·renderer·WASM 출력·Studio UI와 sample을 변경하지 않아 사용자 실행 성능과 시각 결과에는
  직접 영향이 없다. 별도 스크린샷·PDF 비교는 해당 없음이다.
- 일반 PR·push에서는 promotion job을 skip한다. canonical release PR에서만 최대 10분의 read-only
  preflight가 추가된다.
- 일회성 dogfood의 job wall-time 합계는 Stage 4 약 131.5분, Fuzz 약 62.1분, 총 약 193.6분이다.
  병렬 job wall-time 합계이며 GitHub 청구액이나 릴리스 경과시간을 뜻하지 않는다.

## 잔여 위험과 릴리스 경계

- 현재 `upstream/devel@2c144b180dd776aa450c499778510199ae6cdf89`은 candidate의 조상이다.
- `upstream/main@e8800c8def63449808a4092798442652ed460552`에는 devel에 없는 v0.8.6 release merge와
  release CI hotfix가 있다. 다음 릴리스 PR 전에 **main을 devel에 먼저 동기화**해야 한다.
- 동기화로 SHA와 inventory가 달라지므로 새 exact `devel` SHA에서 필요한 workflow를 다시 실행해야 한다.
  이번 task branch의 dogfood나 PR CI를 release promotion 증적으로 재사용하지 않는다.
- `parse_wmf` panic과 release 게시 뒤 package workflow 미기동 #6634는 #6689 gate의 완료와 별개다.
- 4,700줄 이상 추가된 보안·운영 gate이므로 post-merge `devel` CI까지 성공한 뒤에만 #6689를 닫는다.

## Merge 후 계획

정상 merge commit이 `devel`에 반영된 뒤 다음 순서로 처리한다.

1. merge SHA의 `devel` CI와 workflow contract 검사가 성공했는지 확인한다.
2. PR #6772에 code candidate·trailing head·merge SHA와 post-merge run을 구분해 기록한다.
3. #6689에 exact-head gate, dogfood, Fuzz 분리와 릴리스 선행조건을 요약한다.
4. 실제 완료조건을 재확인한 뒤 #6689를 close하고 로컬 `devel`을 fast-forward한다.
5. 이번 task의 local·remote branch만 정리하며 다른 WIP나 contributor branch는 건드리지 않는다.

게시 뒤 API로 한글·선두 BOM·`??` 치환과 SHA·run URL을 검증한다. 같은 사실의 메인테이너 comment가 이미
있으면 중복 게시하지 않는다.

## 최종 판정과 다음 조건

- 판정: **승인**
- 판정 대상: code candidate `ccb732752cb7a8b0bda45d22833967811b9c08b8`
- trailing 조건: report 상태·review·오늘 기록만 추가하고 Actions fast-pass, 최신 `upstream/devel`,
  `MERGEABLE`·`CLEAN` 재확인
- merge 조건: 최신 head SHA 고정과 메인테이너의 별도 merge 승인
- GitHub review: self PR이므로 approve event와 reviewer 지정 없음
- merge 방식: branch protection을 우회하지 않는 정상 merge commit
- merge 뒤: `devel` post-merge CI 성공 후 #6689 결과 comment·close와 task branch 정리
