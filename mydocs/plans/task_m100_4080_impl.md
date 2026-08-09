# 구현계획서 — task_m100_4080

- **이슈**: [#4080](https://github.com/edwardkim/rhwp/issues/4080)
- **수행계획서**: [`task_m100_4080.md`](task_m100_4080.md)
- **기록 시각**: 2026-08-06 KST

> 수행계획서와 같은 소급 작성 고지가 적용된다.

## 1. 파일별 변경

| 파일 | 변경 |
| --- | --- |
| `.github/workflows/cache-generation-sweep.yml` | 고아 ref 정리, 한도 경보, boolean 입력 표현식 보정, `/head` 보호 |
| `.github/workflows/ci.yml` | Lint job 에 workflow 계약 테스트 실행 단계 추가 |
| `scripts/tests/test_cache_sweep_workflow.py` | 스윕 판정·YAML 표현식 계약 테스트 (신규) |
| `scripts/tests/test_workflow_contract_wiring.py` | 계약 테스트의 CI 배선 강제 (신규) |
| `mydocs/working/task_m100_4080_stage1.md` | 단계 기록 |
| `mydocs/plans/task_m100_4080{,_impl}.md` | 계획서 |
| `mydocs/pr/archives/pr_4082_review.md` | PR 검토 기록 |
| `mydocs/orders/20260806.md` | 오늘할일 |

## 2. `cache-generation-sweep.yml`

### 2.1 고아 판정

살아 있는 ref 집합을 만들고, 여기 없는 ref 의 캐시는 세대와 무관하게 삭제한다.

```
liveRefs = 열린 PR 의 refs/pull/<n>/{merge,head}
         + 실재 branch 의 refs/heads/<name>
         + 실재 tag 의 refs/tags/<name>
```

`repos.listBranches`·`repos.listTags` 조회를 위해 `contents: read` 를 더한다. `actions: write` 와
합쳐 둘뿐이고 checkout 은 하지 않는다.

**PR 보호 기준은 열림 여부다.** merge 된 PR, 그냥 닫은 PR, 체리픽·통합 PR 로 반영되고 닫힌 PR 을
구분하지 않는다. 셋 다 해당 ref 로 CI 가 돌지 않고 캐시 scope 상 다른 ref 가 읽지도 못한다.

### 2.2 조회 순서

**캐시를 먼저 읽고 ref 를 나중에 읽는다.** 캐시는 자기 ref 보다 먼저 생길 수 없으므로, 캐시 스냅샷
이후 만들어진 branch/PR 은 그 스냅샷에 캐시가 없다. 반대 순서는 두 조회 사이에 열린 PR 의 캐시를
고아로 오인한다.

### 2.3 fail-closed

`listBranches` 가 throw 하거나 branch 가 0건이면 `liveRefsUsable=false` 로 두어 고아 정리를 통째로
건너뛴다. 세대 상한 정리는 그대로 동작한다. tag 0건은 실패로 보지 않는다 — 태그 없는 저장소가 정상이다.

### 2.4 boolean 입력 표현식

`A && inputs.x || '<기본>'` 관용구를 쓰지 않는다. 기본이 true 인 입력에서 false 를 넣으면
`true && false` → falsy → `|| 'true'` 로 되살아난다. fallback 없이 쓰면 falsy 결과가 그대로 `'false'`
로 렌더되어 기본값이 저절로 맞는다.

| 기본값 | 형태 |
| --- | --- |
| cron 에서 false | `github.event_name == 'workflow_dispatch' && inputs.x` |
| cron 에서 true | `github.event_name != 'workflow_dispatch' \|\| inputs.x` |

`keep_generations` 는 string 이라 빈 값이 falsy 이므로 fallback 관용구가 유효하다.

### 2.5 한도 경보

`LIMIT_GB=10`, `WARN_PERCENT=80`, `FAIL_PERCENT=95`. 판정은 **정리 후** 총량으로 한다 — 정리로 내려갈
양을 미리 실패로 처리하지 않기 위해서다. 삭제 실패 시 `deletedBytes` 를 올리지 않아 `after` 가
보수적이고, dry-run 은 대상 전량 삭제를 가정한다.

## 3. `ci.yml`

Lint job 에 `Validate workflow contracts` 단계를 추가해 `test_cache_sweep_workflow.py`,
`test_review_only_fast_pass_workflows.py`, `test_workflow_contract_wiring.py` 를 실행한다.

Lint job 을 고른 근거: `.github/**` 변경은 classifier 가 `fail-closed:workflow-contract`,
`scripts/tests/**` 는 `fail-closed:unclassified-path` 로 판정해 **두 경로 모두 `rust_required=true`** 가
된다(classifier 실행으로 확인). 따라서 이 테스트들이 관련되는 모든 변경에서 Lint 가 실행된다.
새 CI 축이나 상시 job 은 만들지 않는다.

## 4. 테스트 설계

### 4.1 `test_cache_sweep_workflow.py`

스윕 로직은 checkout 금지 경계 때문에 workflow YAML 인라인이다. `test_ci_impact_workflow.py` 의
aggregate shell 추출과 같은 방식으로 github-script 본문을 꺼내 node 스텁 위에서 실행한다.

스텁은 `pulls.list` 의 `state` 파라미터를 실제 API 처럼 존중한다. 처음에는 무시해서
`state: 'open'` → `'all'` 회귀를 잡지 못했다.

별도 클래스로 YAML 표현식 자체를 단언한다. JS 하네스는 env 를 직접 주입하므로 표현식은 그 검증
범위 밖이고, 실제로 `sweep_orphan_refs` 스위치가 죽었는데 `test_orphan_sweep_can_be_disabled` 는
통과했다.

### 4.2 `test_workflow_contract_wiring.py`

`scripts/tests/` 에서 `test_*workflow*.py` 를 찾아 전부 `ci.yml` 에서 호출되는지, 그리고 Lint job
안에 있는지 단언한다. 계약 테스트를 추가하고 배선을 잊는 사고가 두 번(#4071, #4080) 있었다.

패턴 자체가 망가지면 조용히 무의미해지므로, 알려진 파일이 발견되는지도 함께 단언한다.

## 5. 검증 계획

| 검증 | 기대 |
| --- | --- |
| `test_cache_sweep_workflow.py` | 전건 통과 |
| `test_workflow_contract_wiring.py` | 전건 통과 |
| 기존 workflow 계약 테스트 3종 | 무회귀 |
| `node --test ci-impact-classifier.test.cjs` | 무회귀 |
| `actionlint` 두 workflow | 진단 없음 |
| 실제 스냅샷 dry-run 시뮬레이션 | 정리 후 총량이 경고 임계 아래 |
| 뮤테이션 | 각 안전 계약마다 최소 1건 실패 |

## 6. 롤백

workflow 단일 파일 변경이라 revert 로 즉시 되돌아간다. 운영 중 고아 정리만 끄려면
`workflow_dispatch` 의 `sweep_orphan_refs=false` 로 세대 상한만 남길 수 있다 — 이번에 고친 그 스위치다.
