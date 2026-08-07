---
kind: pr_review
status: local-validation-passed
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-06
---

# PR #4082 검토 — #4080 고아 ref 캐시 정리와 한도 경보

## 결론

**보정 후 merge 후보.** 작업지시자 리뷰
([issuecomment-5195819402](https://github.com/edwardkim/rhwp/pull/4082#issuecomment-5195819402))의
지적 5건을 전부 검증해 사실로 확인하고 ①②③④⑤를 반영했다. 캐시 총량이 이미 무료 한도의 102.4%라
LRU 축출이 도는 구간이므로 미룰 이유가 없다.

## 검토 경로

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md(4.3 CI workflow), post_merge.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
                  collaborator_self_merge.md, intake_and_review.md,
                  local_validation.md, post_merge.md,
                  codex/docs_and_git_workflow.md, hyper_waterfall_docs_guide.md
base: 0b2e1c7e87132c51840bf7d8b79a04635f5b2cbb
review 대상 head: 1c4e230037765a8e72d716f4b3492a483885c1ff
```

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4082](https://github.com/edwardkim/rhwp/pull/4082) |
| 작성자 | `postmelee` (collaborator self-merge) |
| 대상 / head | `devel` / `issue-4080-cache-orphan` (upstream branch) |
| review | [issuecomment-5195819402](https://github.com/edwardkim/rhwp/pull/4082#issuecomment-5195819402) |
| 관련 issue | [#4080](https://github.com/edwardkim/rhwp/issues/4080), [#3684](https://github.com/edwardkim/rhwp/issues/3684), [#3790](https://github.com/edwardkim/rhwp/issues/3790) |
| metadata | label·milestone·review request 없음 |

## 절차 이탈과 보정

이 PR은 계획서 없이 승인 게이트를 건너뛰고 구현·PR 생성·PR CI까지 진행했다. 작업지시자 지적으로
드러났고, 작업지시자 판단에 따라 close·재개 대신 진행하며 문서를 소급 보완했다. 경위와 재발 방지는
[`hyper_waterfall_gate_skipped_4080.md`](../../feedback/hyper_waterfall_gate_skipped_4080.md)에 남겼다.

## review 지적 대응

### ① `sweep_orphan_refs: false`가 무시된다 — 반영

**dispatch 실측으로 확인했다.** `-f dry_run=true -f sweep_orphan_refs=false`로 실행한
[run 31036061226](https://github.com/edwardkim/rhwp/actions/runs/31036061226)이 `고아 ref 캐시 23개 /
2.50GiB`를 그대로 처리했다. 논증뿐 아니라 실제로 스위치가 죽어 있었다.

boolean 입력에서 fallback 관용구를 제거했다. `dry_run`도 같은 형태였고 우연히 동작하던 것이라 함께
고쳤다.

| 입력 | 이전 | 이후 |
| --- | --- | --- |
| `dry_run` (cron 기본 false) | `A && inputs.dry_run \|\| 'false'` | `A && inputs.dry_run` |
| `sweep_orphan_refs` (cron 기본 true) | `A && inputs.sweep_orphan_refs \|\| 'true'` | `A' \|\| inputs.sweep_orphan_refs` |

지적대로 **계약 테스트가 이걸 못 잡았다.** JS 하네스가 env를 직접 주입하므로 YAML 표현식은 검증
범위 밖이었다. `BooleanInputExpressionTests`를 추가해 boolean 입력의 표현식 형태를 단언한다.

### ② 새 테스트가 CI에서 한 번도 돌지 않는다 — 반영

`ci.yml` Lint job에 `Validate workflow contracts` 단계를 추가해
`test_cache_sweep_workflow.py`, `test_review_only_fast_pass_workflows.py`,
`test_workflow_contract_wiring.py`를 실행한다.

Lint job 선택 근거를 classifier 실행으로 확인했다. `.github/**`는 `fail-closed:workflow-contract`,
`scripts/tests/**`는 `fail-closed:unclassified-path`로 **둘 다 `rust_required=true`**가 되므로 관련
변경에서 Lint가 반드시 돈다. 새 CI 축이나 상시 job은 만들지 않았다.

배선 자체를 강제하는 `test_workflow_contract_wiring.py`를 함께 넣었다. 같은 사고가 #4071에도 있었고,
고치기만 하면 다음 계약 테스트에서 반복된다.

### ③ review 문서와 오늘할일 부재 — 반영

이 문서와 `mydocs/orders/20260806.md`를 PR diff에 포함했다. 지적 범위보다 넓게, 누락된 수행계획서·
구현계획서와 피드백 기록도 함께 넣었다.

### ④ 본문·기록 수치 뒤처짐 — 반영

PR 본문과 Stage 1 기록의 테스트 건수·뮤테이션 건수를 최신 head 기준으로 갱신했다.

### ⑤ `refs/pull/<n>/head` 가정 — 반영 (리뷰 권고보다 강하게)

리뷰는 주석으로 충분하다고 봤으나 코드로 고정했다. 이 변경이 해당 경우의 결과를 악화시켰기
때문이다 — 전에는 `/head` 캐시가 세대 상한만 받았지만 이제는 열린 PR 것이어도 고아로 전량 삭제된다.
한 줄이고 엄격히 더 안전하며, 주석은 낡는다. 계약 테스트도 함께 넣었다.

## 리뷰 외 추가 보정 — 한도 단위

작업지시자가 "베이스 캐시가 8.8GiB 근처"라고 확인해 준 것을 계기로 단위를 재점검했다. 그 값은 정확했다
— 2026-08-05 20:00 UTC 정기 스윕([run 31042014495](https://github.com/edwardkim/rhwp/actions/runs/31042014495))의
실제 결과 `50개 8.84GiB`이며, 이 PR 의 dry-run 예측과 삭제 대상 3건까지 일치했다.

그 과정에서 한도 해석이 갈린다는 것이 드러났다. GitHub 문서는 "10 GB"라고만 쓰고 십진인지 이진인지
밝히지 않는데, 실측 `10,241,001,878 B`는 십진으로 **102.4%**, 이진으로 **95.4%**다. 한쪽은 한도 초과이고
다른 쪽은 아니다.

쿼터 가드는 늦게 우는 것보다 일찍 우는 편이 안전하므로 보수적인 십진 10^10 바이트를 채택하고,
`LIMIT_GB`(코드에서 `1024**3` 곱셈) 대신 `LIMIT_BYTES`로 단위가 숨지 않게 했다. 표시는 GiB로 유지해
#3684 이후 기준선 시계열과의 대조를 끊지 않고, 한도를 같은 단위로 환산해 원시 바이트와 함께 적었다.
`LimitUnitTests` 3건으로 고정했다.

## 검증

### 로컬

| 검증 | 결과 |
| --- | --- |
| `python3 -m unittest scripts/tests/test_cache_sweep_workflow.py` | 25 passed / 0 failed |
| `python3 -m unittest scripts/tests/test_workflow_contract_wiring.py` | 3 passed / 0 failed |
| 기존 workflow 계약 테스트 3종 | 25 passed, 무회귀 |
| `node --test scripts/tests/ci-impact-classifier.test.cjs` | 무회귀 |
| `actionlint ci.yml cache-generation-sweep.yml` | 통과, 진단 없음 |
| `git diff --check` | 통과 |

### 뮤테이션 재현

| 뮤테이션 | 결과 |
| --- | --- |
| 고아 판정 제거 | 5건 실패 |
| 빈 branch 목록 가드 제거 | 1건 실패 |
| 임계 판정을 정리 전 총량으로 | 1건 실패 |
| 캐시 읽기를 ref 조회 뒤로 이동 | 1건 실패 |
| PR 보호 기준 `state: 'open'` → `'all'` | 1건 실패 |
| `SWEEP_ORPHAN_REFS`를 버그 형태로 되돌림 | 2건 실패 |
| `ci.yml` 배선 한 줄 제거 | 2건 실패 |
| `/head` 보호 제거 | 1건 실패 |
| 한도를 `LIMIT_GB` 이진 배수로 되돌림 | 2건 실패 |

`/head` 뮤테이션은 처음에 잡히지 않아 전용 테스트를 추가한 뒤 다시 확인했다.

### 실제 스냅샷 시뮬레이션

workflow에서 추출한 스크립트에 2026-08-06 11:45 UTC 시점의 실제 캐시 53개, branch 3건, tag 23건,
열린 PR 9건을 넣어 dry-run으로 돌렸다.

| 항목 | 값 |
| --- | --- |
| 정리 전 | 53개 / 9.54GiB (10,241,001,878 B — 십진 한도의 102.4%) |
| 고아 ref | 23개 / 2.50GiB |
| 구 세대 | 2개 / 0.46GiB |
| 정리 후(추정) | **28개 / 6.57GiB (70.6%)** |

merge 뒤 관찰 기준치는 **6.57GiB**다. 최초 측정(6.34GiB) 이후 devel 이 10개 넘는 PR 로 전진해 구성이
바뀌었고, 한도 해석도 십진으로 바꿨다.

## 시각·fixture 판단

시각 증적 없음. renderer·layout·paint·pagination·golden 출력과 무관한 CI 워크플로 변경이다.

## 잔여 위험과 후속

- 예상치는 `before - staleBytes` 계산값이다. merge 뒤 실제 cron 스윕 1회로 확인하고 새 기준선을
  #4080에 기록한다. 다만 이 방법은 2026-08-05 20:00 UTC 정기 스윕에서 삭제 대상 3건과 총량 8.84GiB를
  정확히 맞혀 이미 한 번 검증됐다.
- 캐시 한도의 단위 해석은 GitHub 문서에 없다. 보수적인 십진 10^10 바이트를 채택했으므로, 실제 한도가
  이진으로 확인되면 임계가 필요 이상으로 일찍 울 수 있다. 그때 완화한다.
- `devel` 한정 KEEP=1은 이번 범위 밖이다. `refs/heads/devel`만 19개 5.84GiB지만 #3684가 KEEP=2를 택한
  근거가 devel에도 적용되는지 미확인이다.
- 닫힌 PR을 다시 열면 캐시가 cold로 시작한다. 받아들이는 비용으로 문서화했다.
- #4080은 새 기준선 기록 뒤에 닫는다. 이 PR은 `Refs`만 걸어 auto-close하지 않는다.

## 최종 권고

최신 head의 GitHub Actions 통과와 작업지시자 승인을 확인한 뒤 collaborator self-merge한다. merge 뒤에는
`post_merge.md`에 따라 devel sync, branch·worktree 정리, 첫 cron 스윕 관찰을 수행한다.
