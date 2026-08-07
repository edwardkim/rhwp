# task_m100_4080 Stage 1 — 고아 ref 정리와 한도 경보

- **이슈**: [#4080](https://github.com/edwardkim/rhwp/issues/4080)
- **브랜치**: `issue-4080-cache-orphan`
- **분기 기준**: `upstream/devel` `0b2e1c7e87132c51840bf7d8b79a04635f5b2cbb`
- **선행**: [#3684](https://github.com/edwardkim/rhwp/issues/3684) / [PR #3810](https://github.com/edwardkim/rhwp/pull/3810)
- **기록일**: 2026-08-06 KST

## 1. 문제

[#3810](https://github.com/edwardkim/rhwp/pull/3810)이 고정한 캐시 기준선 4.73GB가 사흘 만에 8.84GB로
올랐다(무료 한도의 88%). 세대 상한은 지켜지고 있었다 — (ref, 그룹) 쌍 42개 중 2세대를 초과한 쌍은
3개뿐이었다. 회귀의 실제 원인은 두 가지다.

1. 세대 상한이 (ref, 그룹) 쌍마다 적용되므로 **총량 하한이 쌍 수에 비례한다.**
2. 브랜치가 삭제되거나 PR이 닫혀도 **그 ref의 캐시 최신 N개는 영구히 정리 대상이 아니었다.** GitHub의
   7일 미사용 만료까지 자리를 차지한다.

조사 시점의 **실제** 총량은 53개 10.01GB로 무료 한도의 100.1%였고, GitHub의 LRU 축출이 이미 도는
구간이었다. 그동안 아무 신호도 없었다는 점도 문제였다.

## 2. 이번 범위

- **고아 ref 정리**: 실재하지 않는 branch/tag, 열려 있지 않은 PR의 캐시를 세대와 무관하게 삭제한다.
- **한도 경보**: 정리 후 총량을 무료 한도와 대조해 80% 이상이면 경고, 95% 이상이면 job을 실패시킨다.
- **계약 테스트**: 스윕 판정을 회귀 테스트로 고정한다.

`devel` 한정 KEEP=1은 이번 범위에서 제외했다. 근거는 5절에 적는다.

## 3. 구현

`.github/workflows/cache-generation-sweep.yml` 한 파일만 바꿨다. 스윕 로직은 checkout 금지 안전
경계 때문에 workflow 안에 인라인으로 유지한다.

### 3.1 고아 판정

살아 있는 ref 집합 = 열린 PR의 `refs/pull/<n>/{merge,head}` + 실재 branch의 `refs/heads/<name>` +
실재 tag의 `refs/tags/<name>`. 이 집합에 없는 ref의 캐시는 세대와 무관하게 삭제 대상이다.

권한은 ref 목록 조회를 위해 `contents: read`를 더했다. `actions: write`와 함께 둘뿐이다.

### 3.1.1 PR 보호 기준은 merge 여부가 아니라 열림 여부다

`pulls.list`를 `state: 'open'`으로 조회하므로, merge된 PR·그냥 닫은 PR·**체리픽이나 통합 PR로
내용만 반영하고 닫은 PR**이 전부 같게 취급된다. 셋 다 `refs/pull/<n>/merge`로는 더 이상 CI가 돌지
않고, 그 캐시를 다른 ref가 읽지도 못한다(캐시 scope는 생성 ref와 base로 제한된다). 따라서 전부
고아다.

실제 표본이 이 구분을 요구한다. 시뮬레이션에서 고아로 잡힌 PR ref 19건에 merge된 #3919·#3868·
#3816·#3801·#3690와 merge 없이 닫힌 #3951·#3858·#3853 등이 섞여 있고, 그중 #3779·#3775는 내용이
통합 PR #3801로 반영된 뒤 닫힌 경우다.

닫힌 PR이 다시 열리면 캐시는 cold로 시작한다. 받아들이는 비용으로 문서에 남긴다.

### 3.2 조회 순서가 안전 계약이다

**캐시를 먼저 읽고 ref를 나중에 읽는다.** 캐시는 자기 ref보다 먼저 생길 수 없으므로, 캐시 스냅샷
이후에 만들어진 branch/PR은 그 스냅샷에 캐시가 없다. 반대 순서로 읽으면 두 조회 사이에 열린 PR의
캐시를 고아로 오인해 지울 수 있다. 이 순서를 계약 테스트로 고정했다.

### 3.3 fail-closed 가드

살아 있는 ref 목록 수집이 실패하거나 branch가 0건이면 고아 정리를 통째로 건너뛴다. 목록을 못 믿는
상태에서 삭제하면 전량 삭제가 되기 때문이다. 이때 세대 상한 정리는 그대로 동작한다.

`sweep_orphan_refs` 입력으로 수동 비활성화도 가능하다(기본 true). 이 스위치는 최초 구현에서 boolean
입력 표현식 오류로 죽어 있었고, 작업지시자 리뷰와 dispatch 실측으로 드러나 보정했다 — 4.4절.

### 3.4 한도 경보

`LIMIT_GB=10`, `WARN_PERCENT=80`, `FAIL_PERCENT=95`. 판정은 **정리 전이 아니라 정리 후** 총량으로
한다. 정리로 내려갈 양을 미리 실패로 처리하지 않기 위해서다. summary에 한도 대비 비율과 고아/구세대
내역을 분리해 남긴다.

## 4. 검증

### 4.1 로컬 시뮬레이션 — 실제 저장소 스냅샷

workflow에서 추출한 스크립트에 실제 저장소 스냅샷을 그대로 넣어 dry-run으로 돌렸다. devel 이
빠르게 전진하므로 최초 측정과 merge 직전 측정을 함께 남긴다.

| 항목 | 2026-08-05 18:00 UTC | 2026-08-06 11:45 UTC (최신) |
| --- | --- | --- |
| 정리 전 | 53개 / 10.01GiB | 53개 / 9.54GiB |
| 고아 ref | 23개 / 2.50GiB | 23개 / 2.50GiB |
| 구 세대 | 3개 / 1.17GiB | 2개 / 0.46GiB |
| 정리 예정 | 26개 / 3.67GiB | 25개 / 2.97GiB |
| 정리 후(추정) | 27개 / 6.34GiB | **28개 / 6.57GiB** |
| 한도 대비 | 63.4% (이진 기준) | **70.6%** (십진 기준) |
| 보호한 열린 PR | 13 | 9 |

**merge 뒤 관찰 기준치는 6.57GiB**다. 왼쪽 열은 최초 측정으로, 그 사이 devel 이 10개 넘는 PR 로
전진해 구성이 바뀌었다. 한도 대비 백분율은 4.5절의 십진 해석 변경도 함께 반영된 값이다.

고아 23건의 내역은 **닫힌 PR의 `refs/pull/<n>/merge` 19건, 삭제된 branch 4건**이다.

이슈 #4080 최초 기록의 "고아 약 0.53GB"는 `refs/heads/*`만 센 값이라 과소평가였다. 실제 고아의
대부분은 닫힌 PR ref이며 총량은 2.50GB다.

### 4.2 계약 테스트

`scripts/tests/test_cache_sweep_workflow.py` — workflow YAML에서 github-script 본문을 추출해 node
스텁 위에서 실행하고 판정만 단언한다. `test_ci_impact_workflow.py`의 aggregate shell 추출과 같은
방식이다.

| 검증 | 결과 |
| --- | --- |
| `python3 -m unittest scripts/tests/test_cache_sweep_workflow.py` | 25 passed / 0 failed |
| `python3 -m unittest scripts/tests/test_workflow_contract_wiring.py` | 3 passed / 0 failed |
| `actionlint .github/workflows/cache-generation-sweep.yml` | 통과, 진단 없음 |
| `python3 -c "yaml.safe_load(...)"` | 통과 |
| `git diff --check` | 통과 |

테스트가 다루는 계약은 고아 삭제(삭제된 branch·닫힌 PR), merge 여부와 무관한 열림 기준, tag ref
보존, 열린 PR 보호, ref별 독립 세대 상한, dry-run 무삭제, 삭제 실패의 경고 처리, 빈 branch 목록·
조회 실패의 fail-closed, 조회 순서, 임계 경고·실패, summary 항목이다.

node 스텁은 `pulls.list`의 `state` 파라미터를 실제 API처럼 존중한다. 처음에는 파라미터를 무시해서
`state: 'open'` → `'all'` 뮤테이션이 잡히지 않았고, 스텁을 고쳐 잡히게 했다.

### 4.3 RED 재현

아홉 가지 뮤테이션이 모두 테스트에 잡히는 것을 확인했다. `/head` 보호 제거는 처음에 잡히지 않아
전용 테스트를 추가한 뒤 다시 확인했다.

| 뮤테이션 | 결과 |
| --- | --- |
| 고아 판정 제거 | 5건 실패 |
| 빈 branch 목록 가드 제거 | 1건 실패 |
| 임계 판정을 정리 전 총량으로 | 1건 실패 |
| 캐시 읽기를 ref 조회 뒤로 이동 | 1건 실패 |
| PR 보호 기준을 `state: 'open'` → `'all'` | 1건 실패 |
| `SWEEP_ORPHAN_REFS` 를 버그 형태로 되돌림 | 2건 실패 |
| 한도를 `LIMIT_GB` 이진 배수로 되돌림 | 2건 실패 |
| `ci.yml` 배선 한 줄 제거 | 2건 실패 |
| `/head` 보호 제거 | 1건 실패 |

### 4.4 예측 검증 — 실제 cron 스윕과 일치

2026-08-05 20:00 UTC 의 정기 스윕([run 31042014495](https://github.com/edwardkim/rhwp/actions/runs/31042014495))이
`53개 10.01GiB → 3개 1.17GiB 삭제 → 50개 8.84GiB` 를 기록했다. 삭제 대상 3건과 총량 모두 이 PR 의
dry-run 예측과 정확히 일치했다. 예측 방법 자체는 이로써 검증됐다.

그 뒤 약 16시간 동안 10개 넘는 PR 이 merge 되며 8.84GiB → 9.54GiB 로 다시 올랐다. 고아 정리 없이는
하루 만에 0.7GiB 가 쌓인다는 뜻이다.

### 4.5 한도 단위를 바이트로 명시

GitHub 문서는 캐시 한도를 "10 GB" 라고만 쓰고 십진(10^9)인지 이진(2^30)인지 밝히지 않는다. 차이가
7.4% 라 임계 발화 시점이 달라진다. 2026-08-06 실측 `10,241,001,878 B` 는 십진으로 **102.4%**,
이진으로 **95.4%** 로 읽힌다 — 한쪽은 한도 초과이고 다른 쪽은 아니다.

쿼터 가드는 늦게 우는 것보다 일찍 우는 편이 안전하므로 보수적인 **십진 10^10 바이트**를 채택하고,
`LIMIT_GB` 대신 `LIMIT_BYTES` 로 단위가 코드에 숨지 않게 했다. 표시는 GiB 로 유지한다 — #3684 이후
기준선 시계열(4.73, 8.84 …)이 전부 GiB 라 단위를 바꾸면 대조가 끊긴다. 대신 한도를 같은 단위로
환산해 원시 바이트와 함께 적어 백분율 산술이 눈으로 맞도록 했다.

### 4.6 작업지시자 리뷰 보정

[issuecomment-5195819402](https://github.com/edwardkim/rhwp/pull/4082#issuecomment-5195819402)의 지적
5건을 전부 검증해 사실로 확인하고 반영했다. 상세는
[PR #4082 검토](../pr/archives/pr_4082_review.md)에 있다.

- `sweep_orphan_refs: false`가 무시되던 boolean 입력 표현식 오류 — dispatch
  [run 31036061226](https://github.com/edwardkim/rhwp/actions/runs/31036061226)으로 실증 후 보정
- 새 계약 테스트가 `ci.yml`에 배선되지 않아 한 번도 돌지 않던 문제 — Lint job 배선과 배선 강제 테스트
- 열린 PR의 `refs/pull/<n>/head` 미보호 — 리뷰 권고(주석)보다 강하게 코드로 고정
- 본문·기록의 뒤처진 수치 갱신

## 5. 이번에 하지 않은 것과 이유

- **`devel` 한정 KEEP=1**: `refs/heads/devel`만 19개 5.84GB로 가장 크지만, 줄이려면 devel 캐시의
  실제 restore 주체와 빈도를 먼저 봐야 한다. #3684가 KEEP=2를 택한 근거(진행 중 job이 방금 만든
  캐시를 지울 위험, lock 변경 직후 PR cold)가 devel에도 그대로 적용되는지 미확인이다. 고아 정리만으로
  70.6%까지 내려가므로 급하지 않다.
- **새 기준선 수치 확정**: 이 PR merge 뒤 실제 cron 스윕 1회를 관찰해 정한다. 시뮬레이션 예상치
  6.57GiB를 그대로 기준선으로 박지 않는다.

## 6. 다음 단계

1. merge 뒤 첫 cron 스윕의 summary로 실제 정리 후 총량을 확인하고 새 기준선을 #4080에 기록한다.
   관찰 기준치는 **6.57GiB**, 한도 대비 70.6%다.
2. 총량이 경고 임계 아래로 안정되는지 며칠 관찰한다.
3. 필요하면 `devel` KEEP 재검토를 별도로 연다.
