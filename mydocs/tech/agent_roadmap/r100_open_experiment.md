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

- **환경을 canonical manifest로 고정한다.** environment manifest는 RFC 8785 JCS
  canonical JSON bytes로 직렬화하고,
  `environment_id = sha256:<SHA-256(canonical environment manifest bytes)>`로
  정의한다. `environment_id`는 운영자가 붙이는 자유 형식 label이 아니다.
- environment manifest에는 OS image digest, architecture, CPU resource class,
  vCPU count, RAM bytes, network profile, clock profile, timestamp authority,
  preinstalled tool manifest, cache policy를 모두 넣는다. timestamp authority의
  identity·endpoint·검증 key 또는 immutable log identity도 이 manifest에 고정한다.
- 각 회차는 이 manifest와 일치하는 새 환경의 빈 workspace에서 시작한다. cache
  policy는 저장소 checkout, `rhwp` binary, 언어별 package, build 산출물,
  dependency/build cache가 모두 없음을 명시해야 한다.

```yaml
schema_version: <environment-manifest-version>
os_image_digest: sha256:<digest>
architecture: <cpu-architecture>
cpu_resource_class: <provider-resource-class>
vcpu_count: <count>
ram_bytes: <bytes>
network_profile:
  id: <profile-id>
  sha256: <canonical-network-profile-sha256>
clock_profile:
  source: <clock-source>
  synchronization: <sync-policy>
timestamp_authority:
  id: <authority-id>
  endpoint: <append-only-receipt-endpoint>
  verification_key_or_log: <key-or-immutable-log-id>
preinstalled_tool_manifest:
  sha256: <canonical-tool-manifest-sha256>
cache_policy:
  repository_checkout: absent
  rhwp_binary_and_packages: absent
  dependency_cache: empty
  build_cache: empty
```

- **source cohort만 집계한다.** run package가 지정한 exact repository SHA를 clone해
  source에서 발견·build/install한다. release binary 지급 경로는 v1에 섞지 않는다.
- environment manifest byte가 하나라도 다르면 `environment_id`도 달라지며, 서로
  다른 `environment_id` 결과는 별도 cohort로 집계한다.
- 공개 저장소·문서·issue·PR 열람과 참가자의 자율 탐색은 `open-book/self-discovery`다.
  진행자나 사람이 실시간으로 명령, 탐색 방향, 해답을 주면 `guided`로 분리한다.
  사람+에이전트 페어에서 사람이 고정 run package를 중계하고 제출만 하는 것은
  허용하지만, 그 경계를 넘는 상호작용은 모두 guidance log에 남긴다.
- 결과의 `guidance_class`는 `open-book/self-discovery | guided` enum이다. 시작값은
  `open-book/self-discovery`이고 live human hint가 한 번이라도 있으면 `guided`로
  단방향 전환한다. 이 분류는 task 의미가 아니라 실행 조건이므로 `task_variant_id`
  바깥에 기록한다.

## 2. organizer run-package manifest

진행자는 참가 준비를 확인한 뒤 아래 필드를 전부 고정한 manifest를 한 번에 전달한다.
manifest도 RFC 8785 JCS canonical JSON bytes로 직렬화한다. 그 bytes에는 self-hash를
넣지 않고, 외부 envelope에
`package_sha256 = sha256:<SHA-256(canonical run-package bytes)>`를 기록한다. 같은
`package_id`의 canonical bytes와 hash는 실행 중 바꾸지 않으며, 둘 다 §6 ledger에
보존한다.

```yaml
package_id: <globally-unique-id>
protocol:
  path: mydocs/tech/agent_roadmap/r100_open_experiment.md
  sha256: <protocol-file-sha256>
repo:
  url: https://github.com/edwardkim/rhwp.git
  sha: <exact-40-hex-commit>
environment_id: sha256:<canonical-environment-manifest-sha256>
prompt:
  exact_natural_language: <verbatim-prompt>
input:
  path: <package-relative-path>
  sha256: <input-sha256>
target: T1|T2|T3|T4|T5
template_version: <result-template-version>
required_artifact_contract:
  type: <participant-visible-artifact-type>
  format: <participant-visible-format-contract>
  semantic_requirements: <participant-visible-semantic-contract>
evidence_contract: <machine-checkable-acceptance-contract>
task_variant_id: sha256:<canonical-task-variant-sha256>
submission_destination: <run-instance-storage-locator>
submission_endpoint: <append-only-receipt-authority-endpoint>
timestamp_authority_id: <must-match-environment-manifest>
aggregation_plan:
  task_mix_by_guidance_class:
    open-book/self-discovery: <mix-id-and-sha256-or-null>
    guided: <separate-mix-id-and-sha256-or-null>
```

`protocol.sha256`와 `repo.sha`는 protocol과 source 기준을 각각 고정한다. input의
경로·SHA-256, target, 결과 template version, required artifact와 evidence contract,
submission endpoint도 같은 package의 일부다.

`required_artifact_contract`는 참가자에게 dispatch 때 보이는 type·format·semantic
requirements다. 반면 `submission_destination`은 회차별 storage locator이므로 package에는
고정하되 variant hash에는 넣지 않는다. 같은 artifact contract에서 bucket/path만 바뀐
실행은 같은 variant일 수 있지만, artifact type·format·semantic requirement가 바뀌면
새 variant다.

`task_variant_id`는 다음 object의 RFC 8785 canonical JSON bytes에 대한 SHA-256이다.
필드 값이 하나라도 바뀌면 새 variant다.

```json
{
  "evidence_contract": "<exact contract>",
  "input_sha256": "<input sha256>",
  "prompt": "<exact natural-language prompt>",
  "required_artifact_contract": {
    "format": "<exact participant-visible format>",
    "semantic_requirements": "<exact participant-visible requirements>",
    "type": "<exact participant-visible type>"
  },
  "target": "<T1..T5>",
  "template_version": "<exact version>"
}
```

즉 `task_variant_id = sha256(canonical target + exact prompt + input SHA-256 +
template version + required_artifact_contract + evidence contract)`이며, 문자열 단순
이어붙이기가 아니라 위 canonical object를 hash한다. `submission_destination`,
receipt sequence와 storage locator는 이 object에 넣지 않는다.

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

- `t0`와 `t1`은 environment manifest에 고정한 **동일 timestamp authority**의
  append-only receipt만 사용한다. authority는 package별 monotonic `sequence`를
  발급하고 receipt를 수정·삭제할 수 없어야 한다.
- receipt schema는 아래 필드를 모두 가진다. `sequence: 0`은 dispatch receipt이며
  artifact/evidence hash는 `null`이다. 제출은 `sequence >= 1`이고 두 hash가 필수다.

```yaml
kind: dispatch|submission
package_id: <package-id>
package_sha256: <canonical-run-package-sha256>
sequence: <monotonic-integer>
artifact_sha256: <sha256-or-null-for-dispatch>
evidence_sha256: <sha256-or-null-for-dispatch>
received_at: <authority-RFC3339-timestamp>
authority: <timestamp-authority-id>
signature: <signature-or-null>
immutable_log_id: <log-id-or-null>
```

  `signature` 또는 `immutable_log_id` 중 하나는 반드시 있어야 하며, receipt의
  `authority`는 run package와 environment manifest의 authority와 일치해야 한다.
- `t0`는 canonical run-package bytes와 `package_sha256`을 결속한 dispatch receipt의
  `received_at`이다. 참가자는 이 receipt 전에는 해당 package의 clone·탐색·명령·GUI
  동작을 시작하지 않는다. clone, dependency 취득, build/install, 공개 자료 탐색과
  추론을 모두 시간에 포함한다.
- validator는 submission receipt의 artifact/evidence SHA-256과 실제 제출 bytes를
  대조하고 sequence 순서대로 판정한다. **사후 valid로 판정된 submission receipt 중
  가장 이른 `received_at`이 `t1`**이다. filesystem file mtime이나 참가자·진행자가
  적은 시각은 `t0`/`t1` 근거로 사용하지 않는다.
- `validation_timestamp`와 `validation_latency`는 별도 기록하며, 검증이 늦어져도
  `t1`을 validation 시각으로 옮기지 않는다.
- 30분 cutoff는 `valid receipt.received_at <= dispatch receipt.received_at + 30분`으로
  판정한다. cutoff 전에 접수됐지만 나중에 valid로 확인된 제출은 완주다. cutoff 전
  receipt가 모두 invalid이고 valid receipt가 cutoff 뒤 처음 생겼으면 미완주다.
- validator의 성공·실패 사유와 수정 힌트를 포함한 **validation feedback은 cutoff
  뒤에만** 참가자에게 전달한다. cutoff 전에는 append-only receipt만 돌려준다.

## 5. 결과 보고 양식

```text
package_id / package_sha256 / protocol_sha256 / repo_sha:
environment_id / task_variant_id:
에이전트/모델:
target / template_version:
required_artifact_contract / evidence_contract:
submission_destination:
dispatch receipt sequence/log-id/received_at (t0):
earliest valid receipt sequence/log-id/received_at (t1):
artifact_sha256 / evidence_sha256:
validation_timestamp / validation_latency:
소요: mm:ss (t1 - t0)
완주: 예/아니오 (30분 cutoff는 receipt received_at 기준)
submitted artifact / evidence:
막힌 지점:
guidance_class: open-book/self-discovery | guided
guidance log: (live human hint가 없으면 "없음")
```

## 6. ledger와 환류 규약

각 행은 canonical run-package bytes, `package_sha256`, canonical environment manifest,
dispatch/submission receipt 원장을 가리켜야 한다.

| 회차 | package/hash | protocol/repo | environment_id | task_variant_id | guidance_class | t0 receipt | t1 receipt | validation timestamp/latency | 소요·완주·막힌 지점 |
|---|---|---|---|---|---|---|---|---|---|
| (첫 회차 대기 — #4355 접수) | | | | | | | | | |

- **막힌 지점 1건 = 이슈 1건** — 해당 트랙(A 봉투 / B 가드 / D 발견 / H MCP)으로
  분리하고 ledger에 연결한다.
- 완주 사례는 온보딩 사례집(`onboarding_cases_2026h2.md` 계열)에 등재한다.
- 기본 집계 key는 `(protocol_sha256, repo_sha, environment_id, task_variant_id,
  guidance_class)`다. 다섯 값 중 하나라도 다른 회차를 같은 cell에 합치지 않는다.
- overall은 첫 dispatch 전에 task variant 목록과 weight를 canonical bytes+SHA-256으로
  고정한 `task_mix_id`가 있을 때만 계산한다. mix와 weight는 guidance class별로 따로
  preregister한다. `open-book/self-discovery`와 `guided` 결과를 하나의 overall로
  합치지 않으며, preregistered mix/weight가 없는 class는 variant별로만 보고한다.
- R100 게이트("30분") 판정은 이 실측과 별도 no-manual protocol로만 한다.

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
- filesystem file mtime, 참가자 입력 시각, 서로 다른 timestamp authority를
  `t0`/`t1`에 사용하기.
- validation feedback을 cutoff 전에 전달하거나 validation timestamp를 `t1`로 사용하기.
- 다른 `task_variant_id`를 합치거나 사후 task mix·weight로 overall을 만들기.
- `required_artifact_contract` 변경을 같은 variant로 두거나 run-instance
  `submission_destination` 변경만으로 새 variant를 만들기.
- 서로 다른 `guidance_class`를 같은 집계 cell이나 overall에 합치기.
- v1 결과만으로 "매뉴얼 없이 30분 달성"을 선언하기.
