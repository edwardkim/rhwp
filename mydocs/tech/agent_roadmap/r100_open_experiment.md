---
kind: guide
status: active
canonical: mydocs/tech/agent_roadmap/r100_open_experiment.md
last_verified: 2026-08-10
---

# R100 공개 실험 프로토콜 — "30분 첫 유효 산출"을 지금 연다 (#4355)

R100(트랙 J 엔드게임)의 판정 문장은 하나다: **"새로 온 에이전트가 매뉴얼 없이
30분 안에 첫 유효 산출을 만든다."** 종전 계획은 하위 트랙 완성 뒤의 최종
실험이었지만, 이 프로토콜은 그 실험을 **탑다운으로 지금 공개**한다 — 완성을
기다리면 측정이 없고, 측정이 없으면 어느 트랙이 진짜 병목인지 끝까지 추정으로
남는다. 실험 자체가 로드맵의 계기판이 된다.

참가 접수·결과 수집처는 [#4355](https://github.com/edwardkim/rhwp/issues/4355)
(상시 개방 이슈)다. 이 문서는 절차의 canonical 이다.

공개 Git history에 과제·검증 기준이 남으므로 이를 private rubric 또는 blind test로
운영할 수 없다. v1은 **open-book/self-discovery source cohort**다. 참가자는 공개
저장소, README·`llms.txt`를 포함한 문서, issue·PR history와 실행 파일의 `--help`·
`capabilities`를 열람할 수 있다. 따라서 v1 결과만으로 R100의 "매뉴얼 없이"를
달성했다고 판정하지 않는다.

## 1. cohort와 시작 상태

- **환경을 content-addressed identity로 고정한다.** 각 회차는 exact
  `environment_id`, OS image digest, preinstalled toolchain manifest를 가진 새
  환경의 빈 workspace에서 시작한다. 저장소 checkout, `rhwp` binary, 언어별
  package, build 산출물, warm dependency/build cache는 없어야 한다.
- **source cohort만 집계한다.** run package가 지정한 exact repository SHA를 clone해
  source에서 발견·build/install한다. release binary 지급 경로는 v1에 섞지 않는다.
- OS image digest나 preinstalled toolchain manifest가 다르면 `environment_id`도
  달라야 하며, 서로 다른 `environment_id` 결과는 별도 cohort로 집계한다.
- 공개 저장소·문서·issue·PR 열람과 참가자의 자율 탐색은 `open-book/self-discovery`다.
  진행자나 사람이 실시간으로 명령, 탐색 방향, 해답을 주면 `guided`로 분리한다.
  사람+에이전트 페어에서 사람이 고정 run package를 중계하고 제출만 하는 것은
  허용하지만, 그 경계를 넘는 상호작용은 모두 guidance log에 남긴다.

## 2. organizer run-package manifest

진행자는 참가 준비를 확인한 뒤 아래 필드를 전부 고정한 manifest를 한 번에 전달한다.
같은 `package_id`의 필드는 실행 중 바꾸지 않으며, 원문 manifest를 §6 ledger에 함께
보존한다.

```yaml
package_id: <globally-unique-id>
protocol:
  path: mydocs/tech/agent_roadmap/r100_open_experiment.md
  sha256: <protocol-file-sha256>
repo:
  url: https://github.com/edwardkim/rhwp.git
  sha: <exact-40-hex-commit>
environment:
  environment_id: <immutable-environment-id>
  os_image_digest: sha256:<digest>
  preinstalled_toolchain_manifest:
    path: <manifest-path>
    sha256: <manifest-sha256>
prompt:
  exact_natural_language: <verbatim-prompt>
input:
  path: <package-relative-path>
  sha256: <input-sha256>
target: T1|T2|T3|T4|T5
template_version: <result-template-version>
required_artifact: <artifact-type-and-location>
evidence_contract: <machine-checkable-acceptance-contract>
submission_endpoint: <timestamping-server-or-file-drop>
dispatch_timestamp: <RFC3339-server-timestamp>
```

`protocol.sha256`와 `repo.sha`는 protocol과 source 기준을 각각 고정한다. input의
경로·SHA-256, target, 결과 template version, required artifact와 evidence contract,
submission endpoint도 같은 package의 일부다. `dispatch_timestamp`가 §4의 `t0`다.

## 3. 공개 과제·evidence contract

이 표는 공개 reference다. 실제 회차의 exact prompt와 검증 계약은 run-package
manifest에 verbatim으로 고정한다.

| # | 과제 | evidence contract 예시 | 난이도 |
|---|---|---|---|
| T1 | 임의 HWP 1개의 쪽수·형식·표 개수를 기계 출력으로 보고 | `info --json` 봉투 값과 일치 | ★ |
| T2 | 문서에서 특정 문자열이 "몇 쪽 어디"인지 찾기 | `search --json` 결과와 일치 | ★ |
| T3 | 서식 문서의 누름틀을 채워 저장하고 스스로 검증 | 저장본 `--verify` 통과 봉투 제시 | ★★ |
| T4 | 문서 1개를 Markdown 으로 변환해 표가 살아 있음을 보이기 | 변환물의 표 행·열이 원본 `export-tables`와 일치 | ★★ |
| T5 | MCP 호스트에 rhwp 를 붙여 세션으로 열고 3회 이상 왕복 | 호스트 로그의 `initialize`→도구 호출 기록 | ★★★ |

과제는 "기여"가 아니라 **사용**부터 잰다. R100 원문의 "첫 유효 기여" 이전
단계인 "첫 유효 산출"을 1차 관문으로 낮춘 것이며, v1은 그중에서도 source
open-book/self-discovery cohort다. "기여 30분"과 blind/no-manual 실험은 별도
protocol version으로 연다.

## 4. 시간·제출·검증 계약

- `t0`는 timestamping server가 기록한 manifest의 `dispatch_timestamp`다. 참가자는
  그 전에는 해당 package의 clone·탐색·명령·GUI 동작을 시작하지 않는다. clone,
  dependency 취득, build/install, 공개 자료 탐색과 추론을 모두 시간에 포함한다.
- 제출 endpoint는 각 artifact/evidence 제출에 변조하기 어려운 server timestamp
  또는 file submission timestamp를 붙인다. validator는 제출을 순서대로 검사한다.
- `t1`은 **사후에 valid로 판정된 제출 중 가장 이른 제출의 server/file submission
  timestamp**다. `validation_timestamp`와 `validation_latency`는 별도 기록하며,
  검증이 늦어져도 `t1`을 validation 시각으로 옮기지 않는다.
- 30분 cutoff는 `submission_timestamp <= t0 + 30분`으로 판정한다. cutoff 전에
  제출됐지만 나중에 valid로 확인된 artifact는 완주다. cutoff 전 제출이 모두
  invalid이고 valid artifact가 cutoff 뒤 처음 제출됐으면 미완주다.
- validator의 성공·실패 사유와 수정 힌트를 포함한 **validation feedback은 cutoff
  뒤에만** 참가자에게 전달한다. cutoff 전에는 제출 receipt와 timestamp만 돌려준다.

## 5. 결과 보고 양식

```text
package_id / protocol_sha256 / repo_sha:
environment_id / os_image_digest / toolchain_manifest_sha256:
에이전트/모델:
target / template_version:
dispatch_timestamp (t0):
earliest_valid_submission_timestamp (t1):
validation_timestamp / validation_latency:
소요: mm:ss (t1 - t0)
완주: 예/아니오 (30분 cutoff는 submission timestamp 기준)
required artifact / evidence:
막힌 지점:
cohort: open-book/self-discovery | guided
guidance log: (live human hint가 없으면 "없음")
```

## 6. ledger와 환류 규약

각 행은 원문 run-package manifest와 submission receipt 목록을 가리켜야 한다.

| 회차 | package_id | environment_id | repo SHA | target | t0 | t1 | validation timestamp/latency | 소요·완주 | cohort·막힌 지점 |
|---|---|---|---|---|---|---|---|---|---|
| (첫 회차 대기 — #4355 접수) | | | | | | | | | |

- **막힌 지점 1건 = 이슈 1건** — 해당 트랙(A 봉투 / B 가드 / D 발견 / H MCP)으로
  분리하고 ledger에 연결한다.
- 완주 사례는 온보딩 사례집(`onboarding_cases_2026h2.md` 계열)에 등재한다.
- 같은 protocol SHA, repo SHA, `environment_id` cohort의 중앙값 소요·완주율만 먼저
  집계한다. R100 게이트("30분") 판정은 이 실측과 별도 no-manual protocol로만 한다.

## 7. merge/run 전 blocker와 residual

공개 issue [#4355](https://github.com/edwardkim/rhwp/issues/4355)와 PR #4356 본문에는
repo-or-release 시작점 또는 private/no-manual rubric을 전제로 한 legacy 문구가 남아
있을 수 있다. **이 PR을 merge하거나 첫 회차를 dispatch하기 전에** maintainer는
두 공개 표면에 v1이 source-only open-book/self-discovery cohort이고 이 문서의
manifest·timestamp 계약을 따른다는 notice를 남겨야 한다. 이 문서 보정은 외부
issue/PR 본문이나 comment를 변경하지 않았으므로, notice 게시와 게시 내용 확인은
잔여 blocker다.

## 8. 하지 않는 것

- 공개 history에 있는 검증표를 private rubric이라고 주장하거나 v1을 blind test로
  보고 — 공개 자료 열람은 허용된 self-discovery다.
- 서로 다른 `environment_id`, 사전 checkout, release binary, package, warm cache
  결과를 같은 source cohort에 섞기.
- validation feedback을 cutoff 전에 전달하거나 validation timestamp를 `t1`로 사용하기.
- v1 결과만으로 "매뉴얼 없이 30분 달성"을 선언하기.
