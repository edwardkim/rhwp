---
kind: report
status: final
canonical: mydocs/report/task_m100_6689_report.md
last_verified: 2026-09-05
---

# #6689 workflow 승격 전 실실행 공백 방지 — 최종 보고서

## 0. 결론

`devel`에서 추가되거나 실행 의미가 바뀐 GitHub Actions workflow가 실제 실행되지 않은 채 `main`으로
승격되는 공백을 fail-closed 하는 장치를 구현했다. inventory는 Git tree에서 workflow 변경과 위험 표면을
결정적으로 계산하고, collector는 GitHub API에서 candidate exact SHA의 run·job·artifact만 읽으며,
verifier는 필수 결과가 하나라도 없거나 stale이면 거부한다. same-repository `devel -> main` PR에서만 이
검사가 실행되고 기존 required context인 `Build & Test`가 결과를 강제한다.

구현·계약·원격 dogfood는 완료됐다. 다만 현재 `main`에는 `devel`에 없는 릴리즈 커밋 2개가 있으므로 다음
릴리스에서는 먼저 `main`을 `devel`에 동기화해야 한다. 그 뒤의 새 exact `devel` SHA에서 inventory에 나온
workflow를 다시 실행해야 하며, 이번 task branch의 dogfood run을 릴리스 증적으로 재사용하면 안 된다.

## 1. 문제와 원인 계보

PR #5366은 Fuzz smoke의 원격 확인 필요성을 리뷰 문서에 남겼지만 merge 종료조건으로 만들지 않았다.
통합 PR #5425에서는 일반 CI·CodeQL 등이 성공했으나 신규 Fuzz workflow 자체는 한 번도 실행되지 않았다.
그 결과 workflow는 `main`에 들어간 뒤 schedule에서 처음 실행됐고 `parse_wmf` panic이 뒤늦게 발견됐다.

문제의 핵심은 검토자가 위험을 몰랐던 것이 아니라, 텍스트로 남긴 확인 의무가 candidate SHA·workflow
content·필수 job을 묶는 기계 계약으로 승격되지 않은 데 있었다. 수행계획과 단계별 근거는
[`task_m100_6689.md`](../plans/task_m100_6689.md) 및
[`issue-6689`](../tech/investigations/issue-6689/README.md)에 보존했다.

## 2. 구현 결과

### 2.1 결정적 inventory와 offline verifier

- `scripts/workflow_promotion_preflight.py`가 base·candidate Git tree의 `.github/workflows/**`와
  `.github/actions/**`를 Git blob·SHA-256으로 비교한다.
- 주석·빈 줄만 바뀌었다고 증명 가능한 경우만 `comment-only`로 인정하고 나머지는 executable로 닫는다.
- trigger, permission, secret, matrix, action ref, cache, artifact, timeout, concurrency, deployment와
  job command 변화를 각각 구조화한다.
- exact candidate SHA, workflow content hash, event, actor, run URL·ID, pagination, job conclusion,
  artifact와 구조화 verdict를 함께 검증한다.
- waiver는 메인테이너·SHA·hash·scope·사유·URL·만료가 맞아야 하며 permission·secret·security·deployment
  표면과 실패한 exact run에는 사용할 수 없다.

보고서 작성 직전 source/evidence HEAD `c61f0748ea6ce7cdb797e04fbc7414357164a8e9`에서
`main@e8800c8def63449808a4092798442652ed460552` 대비 inventory를 두 번 생성한 결과는 byte-identical했다.

| 항목 | 결과 |
| --- | --- |
| executable workflow | 8개 |
| inventory SHA-256 | `33193c79f1b3c17defd0cba0e0a1267b09c983f8509b17b04a27dc4250b99355` |
| policy SHA-256 | `8bdd86975c6b69502caa7a959c14579f88117ee70904972f2217d74b1d384d0b` |
| policy violation | 0건 |
| merge-base | `51043f5f8d0453b9bc929233de443fa60cb3df4b` |

최종 보고서 commit은 문서만 추가하므로 workflow·policy content는 바뀌지 않는다. 그러나 candidate SHA가
달라지는 자기참조 문제 때문에 위 hash를 다음 릴리스의 exact-head 증적으로 해석하지 않는다. 실제 승격
증적은 release PR의 GitHub Actions run과 `workflow-promotion-evidence-<run-id>` artifact가 정본이다.

### 2.2 live collector와 CI gate

`scripts/workflow_promotion_evidence.py`는 read-only GitHub REST API만 사용하며 다음 경계를 둔다.

- API·artifact byte 상한과 10 page pagination 상한을 두고 상한 도달·총계 변화·누락을 실패로 처리한다.
- artifact ZIP은 메모리에서만 읽고 파일 수·파일 크기·digest·필수 JSON 경로를 검증한다.
- 신뢰 waiver의 작성자와 URL은 comment 본문이 아니라 API 응답에서 가져온다.
- workflow를 dispatch하거나 branch·issue·artifact를 변경하지 않는다.

CI의 `Workflow promotion preflight`는 같은 저장소의 `devel -> main` PR에서만 실행된다. candidate SHA를
직접 checkout하고 `main` ancestor·merge tree 동일성을 먼저 확인하며, 권한은 `actions: read`,
`contents: read`, `issues: read`뿐이다. `Build & Test`는 canonical promotion이면 gate success를,
그 밖의 PR·push이면 gate skipped를 요구하므로 fast-pass가 이 검사를 우회하지 못한다.

### 2.3 배포·벤치마크·advisory 보호 경계

- Pages 수동 검증은 build와 artifact 생성까지만 수행한다. `pages: write`, `id-token: write`와 실제 Deploy는
  `main` push에서만 가능하다.
- Gym은 contracts-only를 기본으로 하고 full benchmark는 명시적 수동 `mode=full`에서만 실행한다. 제품
  release gate로 되돌리지 않았다.
- Oracle advisory는 신규 workflow identity가 default branch에 없을 때 workflow 파일을 바꾼 신뢰 push로
  bootstrap할 수 있다. sparse checkout의 production compile-time 입력을 최소 목록으로 보완했고,
  advisory run success와 별도로 `verdict.json=completed`를 요구한다.
- 변경 과정에서 새로 유입된 Gym checkout·toolchain의 가변 action ref는 full commit SHA로 고정했다.

## 3. 검증 결과

### 3.1 로컬 최종 검증

| 검사 | 결과 |
| --- | --- |
| CI impact classifier | Node 44건 통과 |
| CI impact policy | Node 37건 통과 |
| aggregate workflow 상태 | Python 35건 통과 |
| CI `Validate workflow contracts` 동일 묶음 | 196건 통과 |
| Gym 직접 영향 | 8건 통과 |
| Python `py_compile` | 통과 |
| 변경 workflow actionlint | v1.7.12, 4개 통과 |
| JSON parse | policy·baseline·Fuzz 영수증 3개 통과 |
| Markdown 상대 링크 | upstream/devel 대비 변경 25개, 이상 없음 |
| 정본 문서 metadata | 변경한 manual 2개·investigation README 통과 |
| `git diff --check` | 통과 |

문서 metadata 전수 검사가 보고하는 16건은 기존 문서 4개의 front matter 누락으로, Stage 4 이전과 동일하다.
이번 변경 문서에는 새 오류가 없다. 최신 `upstream/devel` 대비 Rust source·test와 Cargo delta는 0건이라
Rust lint·WASM·렌더링 제품 검증을 의례적으로 반복하지 않았다.

### 3.2 Stage 4 exact-head workflow dogfood

구현 candidate `76334ea1a640ebc688a468f4c6dd52d37fb7ac75`에서 8개 workflow를 전건 실행했다.

| workflow | run | 결과 |
| --- | ---: | --- |
| Adapter inter-diff | [33959070793](https://github.com/edwardkim/rhwp/actions/runs/33959070793) | success |
| CI | [33959072221](https://github.com/edwardkim/rhwp/actions/runs/33959072221) | success, 19개 job 보존 |
| CodeQL | [33959073606](https://github.com/edwardkim/rhwp/actions/runs/33959073606) | success, 3개 언어 분석 |
| Deploy Pages | [33959074831](https://github.com/edwardkim/rhwp/actions/runs/33959074831) | Build success, Deploy skipped |
| Gym | [33959076202](https://github.com/edwardkim/rhwp/actions/runs/33959076202) | contracts success, full skipped |
| Oracle advisory | [33959077439](https://github.com/edwardkim/rhwp/actions/runs/33959077439) | success, verdict `completed` |
| Proptest roundtrip | [33959078719](https://github.com/edwardkim/rhwp/actions/runs/33959078719) | success |
| Render Diff | [33959079906](https://github.com/edwardkim/rhwp/actions/runs/33959079906) | success |

collector는 8개 run을 모두 수락했고 waiver 0건, 오류 0건으로 판정했다. 당시 inventory hash는
`c3cf6f58c47bb05234290e2713f12523544d8277880f003fe68afdef2d84b974`다. 이후 문서 commit으로
task head가 바뀌었으므로 이 세트는 구현 dogfood이며 다음 release promotion의 증적은 아니다.

### 3.3 Stage 5 Fuzz smoke dogfood

exact `devel@2c144b180dd776aa450c499778510199ae6cdf89`에서 [run
33959858373](https://github.com/edwardkim/rhwp/actions/runs/33959858373)을 한 번 실행했다. 6개 matrix가
모두 생성됐고 `parse_hwp`, `parse_hwp3`, `parse_hwpx`, `parse_hml`, `parse_ooxml_chart`는 성공했다.
`parse_wmf`는 `attempt to negate with overflow` panic으로 실패했다.

Stage 5의 증적 완결성은 통과했지만 Fuzz 제품 건전성은 실패다. 재시도·waiver·`continue-on-error`로
숨기지 않았으며, 84 byte 재현 입력의 digest와 GitHub artifact ID를
[`fuzz-smoke-dogfood.json`](../tech/investigations/issue-6689/fuzz-smoke-dogfood.json)에 고정했다. 같은 target의
반복 실패는 확인됐지만 이전 입력과 digest가 다르므로 동일 code-path 여부는 별도 WMF 제품 결함에서
분석해야 한다.

## 4. runner 비용

GitHub API의 job 시작·완료 시각 차이를 합산한 실측은 다음과 같다.

| 구간 | job wall-time 합계 |
| --- | ---: |
| Stage 4, 8 workflow | 7,889초, 약 131.5분 |
| Stage 5, Fuzz 6-matrix | 3,728초, 약 62.1분 |
| 합계 | 11,617초, 약 193.6분 |

이는 병렬 job의 wall-time 합계이며 GitHub 청구 분이나 금액을 뜻하지 않는다. 정상 운영에서는 모든 PR마다
이 세트를 실행하지 않는다. 일반 PR·push는 promotion job을 skip하고, exact-head 8-workflow 실행은
`devel -> main` 릴리스 후보에서 workflow executable 변경이 있을 때만 수행한다.

## 5. 현재 drift와 잔여 위험

- 확인 시점의 `upstream/devel`은 `2c144b180dd776aa450c499778510199ae6cdf89`이며 task branch의 조상이다.
  열린 PR 10건 중 workflow·action·promotion 경로와 충돌하는 PR은 0건이다.
- `upstream/main@e8800c8def63449808a4092798442652ed460552`에는 devel에 없는 v0.8.6 release merge와 release
  CI hotfix 두 커밋이 있다. 새 gate는 이 상태의 release PR을 ancestor 검사에서 거부한다.
- #6689 task PR이 devel에 병합된 뒤에도 release 직전에는 `main`을 devel에 먼저 동기화하고, 새 exact
  devel SHA에서 inventory와 필수 workflow 실행을 다시 해야 한다.
- `parse_wmf` panic은 #6689 범위에서 고치지 않았다. Fuzz 실패는 제품 결함으로 유지한다.
- Release 게시 뒤 package workflow 자동 기동 문제 #6634는 promotion preflight와 독립된 상태다.

## 6. rollback

문제가 생기면 source·renderer를 건드리지 않고 #6689 task PR 전체를 일반 revert PR로 되돌린다. 특히
required context 이름 `Build & Test`는 유지하고, branch protection을 먼저 바꾸지 않는다.

1. 실패 run의 `workflow-promotion-evidence-<run-id>`에서 inventory·runs·waivers·verdict를 보존한다.
2. collector·verifier·CI promotion job과 세 workflow 보호 경계를 동일 revert에서 복구한다.
3. 운영 문서도 구현과 함께 되돌려 문서가 존재하지 않는 gate를 지시하지 않게 한다.
4. devel CI가 회복된 것을 확인한 뒤에만 다음 release promotion을 재개한다.

## 7. 완료 판정과 남은 절차

구현, 로컬 검증, 8-workflow dogfood, Fuzz 6-matrix 결과 보존까지 완료됐다. PR
[#6772](https://github.com/edwardkim/rhwp/pull/6772)의 code candidate
`ccb732752cb7a8b0bda45d22833967811b9c08b8`은 exact-head Full CI와 self-review를 통과했고, 메인테이너의
결과 승인을 반영해 이 보고서를 `final`로 전환했다.

review-only trailing 문서의 push와 새 head Actions 확인, merge, post-merge 검증 및 이슈 close는 아직
수행하지 않았다. 각 외부 변경은 거버넌스에 따라 별도 승인 뒤 진행한다.
