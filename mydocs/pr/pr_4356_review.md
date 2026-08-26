---
kind: pr_review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# PR #4356 검토 — R100 공개 실험의 재현 가능한 시작 경계

## 라우팅

base route: `maintainer_general.md`

modifiers: `intake_and_review.md`, `local_validation.md`,
`multi_pr_update_branch.md`, `review_only_fast_pass.md`

loaded documents: `pr_review_workflow.md`, `pr_review/README.md`,
`pr_review/maintainer_general.md`, `pr_review/intake_and_review.md`,
`pr_review/local_validation.md`, `pr_review/multi_pr_update_branch.md`,
`pr_review/review_only_fast_pass.md`

## 메타데이터와 적용 경계

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#4356](https://github.com/edwardkim/rhwp/pull/4356) / @kevin9327 |
| base | `devel` |
| 원 PR head | `125176f6eb1b7b78fc1b8a7bf5e58cc63c7322d3` |
| 기준 devel | `e48fe86947fbf9a44b1b98c7037150751af541ab` |
| 가시성 브랜치 | `review/kevin9327-20260810-pr4356` |
| 원 변경 규모 | 실험 프로토콜 1파일, `+77/-0`, contributor 커밋 1개 |
| 1차 메인터너 보정 | `540b2aea8ea03d44f5c6250fb1305fb3f7c85486` — `docs(roadmap): make #4356 experiment start reproducible` |
| 후속 protocol 보정 | `e03935281a9450f529f6fd818ccfb47b256c16d2` — `docs(roadmap): define #4356 open-book cohort` |
| 1차 trailing review | `7c306013da14eef6b20524345bccdfe1c335ee15` — `docs(pr): update #4356 open-book review` |
| 증적 결속 보정 | `e3b4a993c923b60ec6678dadcebc9a08d2522e21` — `docs(roadmap): bind #4356 experiment receipts` |
| 증적 trailing review | `4d4359e46fdb2686c9f028e7a6e1c69be5a5bc39` — `docs(pr): record #4356 receipt review` |
| 집계 identity 보정 | `51e069bd131ff14f901ede1c49f9aaf41b38f5be` — `docs(roadmap): complete #4356 aggregation identity` |

원 변경은 R100의 "30분 첫 유효 산출" 공개 실험 절차를 제안한다. 메인터너
보정은 프로토콜과 이 review·구현 기록만 바꾸며 source, test, workflow, fixture,
baseline에는 영향이 없다. contributor history는 그대로 두고 원 head 뒤에
single-parent 문서 commit만 추가한다.

## 발견한 차단 결함

원 프로토콜은 시작점을 "저장소 클론" 또는 "릴리스 바이너리" 중 하나라고 하면서
설치·클론을 모두 측정에 포함한다고 적었다. 두 시작 상태는 준비 작업과 발견 표면이
달라 같은 대장의 30분 결과를 비교할 수 없다. 또한 "첫 명령"은 명령 전 추론 시간을
제외하고, 참가자에게 과제표의 명령·검증 힌트를 어디까지 전달하는지도 정의하지 않았다.

1차 메인터너 보정도 공개 Git history에 과제·판정표가 있는데 이를 참가자에게 숨기는
private rubric처럼 정의했다. public repository, docs, issue·PR history를 볼 수 있는
source 실험에서는 재현할 수 없는 조건이다. host를 문자열로만 기록해 OS image와
preinstalled toolchain을 고정하지 않았고, `t1`을 진행자의 validation 시각으로 잡아
검증 지연이 참가자 시간에 더해지는 문제도 남았다. run package의 source·input·prompt·
artifact·submission 계약과 30분 cutoff timestamp 기준도 없었다.

open-book 후속 보정에도 측정 무결성 공백이 남았다. `environment_id`가 canonical
manifest hash로 정의되지 않아 CPU·RAM·network·clock·cache 차이를 같은 label로
숨길 수 있었다. run package bytes와 dispatch가 hash로 결속되지 않았고, file
submission timestamp를 허용해 서로 다른 clock과 filesystem mtime이 `t1`에 섞일 수
있었다. prompt·input·template·evidence contract가 다른 task를 구분하는 variant ID와
사전 고정된 overall task mix도 없었다.

증적 결속 보정 뒤에도 `task_variant_id`가 참가자에게 보이는 산출물의 type·format·
semantic requirements를 hash하지 않았다. 한 `required_artifact` 값에 산출물 의미와
회차별 제출 위치를 함께 적으면 의미가 다른 과제를 같은 variant로 집계하거나 storage
locator만 바뀐 실행을 다른 variant로 나눌 수 있다. 또한 4-tuple 집계 key에는
`guidance_class`가 없어 live human hint를 받은 `guided` 결과와 self-discovery 결과가
같은 cell과 overall에 섞일 수 있었고, 사전 등록 task mix·weight도 guidance class별
분리를 강제하지 않았다.

## 메인터너 보정

- v1을 **open-book/self-discovery source cohort**로 재정의했다. public repo, docs,
  issue·PR history 열람은 허용하고 live human hint만 `guided`로 분리한다.
- `environment_id`를 OS image, architecture/CPU class/vCPU/RAM, network, clock·timestamp
  authority, preinstalled tools, cache policy를 담은 RFC 8785 canonical manifest의
  SHA-256으로 정의했다. checkout, binary, package, build artifact, warm cache가 없는
  empty workspace만 같은 environment cohort에 넣는다.
- organizer run-package의 canonical bytes와 `package_sha256`에 package/protocol/repo,
  exact prompt, input path/SHA, target/template, artifact/evidence, environment와
  submission authority를 고정하고 ledger에 원문 bytes를 남긴다.
- canonical run-package bytes와 `package_sha256`을 동일 authority의 append-only
  dispatch/submission receipt에 결속했다. receipt는 package/hash/sequence/artifact·
  evidence hash/`received_at`/authority와 signature 또는 immutable log ID를 가진다.
- dispatch receipt의 `received_at`을 `t0`, 사후 valid인 가장 이른 submission
  receipt의 `received_at`을 `t1`로 고정하고 filesystem mtime을 배제했다.
- target, exact prompt, input SHA, template, participant-visible
  `required_artifact_contract`(type·format·semantic requirements), evidence contract의
  canonical hash인 `task_variant_id`를 추가했다. 회차별 `submission_destination` storage
  locator는 package에는 고정하지만 variant hash에서는 제외한다.
- 기본 집계 key를 `(protocol_sha256, repo_sha, environment_id, task_variant_id,
  guidance_class)` 5-tuple로 고정했다. 결과와 ledger에 `guidance_class`를 기록하고,
  preregistered task mix·weight와 overall도 guidance class별로 분리한다.
- 공개 #4355와 PR body의 repo-or-release/private legacy 문구에 대해 merge/run 전
  maintainer notice가 필요함을 잔여 blocker로 기록했다. 외부 표면은 변경하지 않았다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| cohort 불변식 검사 | open-book/self-discovery, live hint=`guided`, source-only empty workspace가 존재 |
| environment identity 검사 | RFC 8785 canonical manifest SHA-256 정의와 OS/arch/CPU/vCPU/RAM/network/clock-authority/tool/cache 필드가 존재 |
| run-package schema 검사 | canonical package SHA와 protocol/repo/prompt/input/target/template/artifact/evidence/environment/authority 필드가 모두 존재 |
| receipt·시간 계약 검사 | append-only package/sequence/artifact/evidence/received_at/authority/signature-or-log schema, 같은 authority `t0`/`t1`, mtime 금지와 feedback-after-cutoff가 존재 |
| artifact-bound variant 검사 | canonical `task_variant_id`가 participant-visible artifact type·format·semantic requirements와 evidence contract를 포함하고 `submission_destination`·storage locator를 제외함 |
| guidance·집계 검사 | 결과·ledger의 `guidance_class`, 5-tuple 기본 집계, guidance class별 preregistered task mix·weight와 분리된 overall이 존재 |
| stale private rubric 제거 | "참가자에게 전달 금지", "비공개 기준", 진행자 validation 시각=`t1` 문구가 없음 |
| external residual 확인 | #4355/PR body legacy notice가 merge/run 전 blocker이고 외부 mutation은 미수행으로 기록됨 |
| Markdown 상대 링크 검사 | 프로토콜과 review·구현 기록의 저장소 내부 링크 통과 |
| `python scripts/check_document_metadata.py` | 통과. 문서 522개의 front matter·canonical 관계 이상 없음 |
| `git diff --check origin/pr/4356..HEAD` | 통과 |
| Cargo·시각 검증 | 생략. `mydocs` 아래 Markdown만 변경하며 실행 코드·렌더 출력 영향 없음 |

## 리스크와 권고

- v1은 open-book source cohort이므로 R100의 blind/no-manual 달성을 증명하지 않는다.
- OS image 또는 toolchain manifest가 달라지면 새 `environment_id`와 별도 cohort가
  필요하다. 식별자만 같게 두고 환경을 바꾸면 결과를 집계할 수 없다.
- 공개 issue #4355와 PR body에 maintainer notice가 게시·확인되기 전에는 merge 또는
  첫 run을 진행하지 않는다. 이 로컬 보정에는 외부 mutation 권한이 포함되지 않았다.
- 첫 run 전 canonical environment/package serializer와 append-only timestamp authority를
  실제로 준비하고 receipt signature 또는 immutable log 검증을 smoke해야 한다. 문서만
  존재하는 상태에서는 측정 dispatch를 시작하지 않는다.
- live hint 발생 여부를 guidance log에 빠짐없이 남기고 `guided` 전환은 되돌리지 않는다.
  이 분류를 신뢰할 수 없는 결과는 어느 guidance class 집계에도 넣지 않는다.
- artifact contract가 결속된 variant ID, `guidance_class`, class별 preregistered mix hash가
  없는 과거 결과는 새 cell이나 overall에 소급 혼합하지 않는다.
- 최신 PR head의 required checks와 mergeability는 실제 push 뒤 다시 확인해야 한다.
- 이 로컬 보정은 remote에 push하거나 GitHub 상태를 바꾸지 않았다.

**maintainer notice·receipt infrastructure·최신 CI가 확인될 때까지 merge/run 보류.**
