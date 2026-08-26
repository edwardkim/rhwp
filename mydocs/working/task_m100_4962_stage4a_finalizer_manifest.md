# Task M100 #4962 W3 Stage 4-A — checkpoint finalizer·full manifest preflight

- **Issue**: [#4962](https://github.com/edwardkim/rhwp/issues/4962)
- **상위 추적**: [#4960](https://github.com/edwardkim/rhwp/issues/4960)
- **계획**: [`task_m100_4962.md`](../plans/task_m100_4962.md)
- **선행 결과**: [`task_m100_4962_stage3_checkpoint_resume.md`](task_m100_4962_stage3_checkpoint_resume.md)
- **구현·검증 source HEAD**: `29c6a20a1407888d7b71fb3c832bbfc8e8c08145`
- **날짜**: 2026-08-21 KST
- **단계 상태**: Stage 4-A 완료, private 10k decision worker 전건 미착수

## 1. 승인 범위와 결론

승인된 Stage 4-A 범위에서 두 결손을 닫았다.

1. checkpoint journal의 문서별 aggregate를 최종 비식별 corpus aggregate로 병합하는 finalizer
2. 기존 10k 입력을 재계측하지 않고 현재 bytes와 저장공간을 고정하는 local-only manifest preflight

finalizer는 같은 usage key의 수치만 합산하고 `format`을 key에 주입해 HWP/HWPX 축을 보존한다. 실패
문서는 이유별 document count에만 들어가며 성공 usage로 가장하지 않는다. 공개 실제 worker 3건을 2건
commit 뒤 강제 중단한 결과는 재개·finalize 후 무중단 final aggregate와 exact였다.

full manifest preflight는 private 문서를 renderer로 열지 않고 5.47 GB를 읽기 전용 BLAKE3 pass로
고정했다. 문서 수·format·bytes·symlink·regular file·worker input 상한과 checkpoint filesystem을
검사했다. HWP 6,582건과 HWPX 3,418건, 합계 10,000건이 기존 POC 기준선과 일치했다.

Stage 4-A는 통과했다. 다만 최신 `upstream/devel`이 작업 브랜치보다 앞서 있고 renderer·parser 변경을
포함하며, 대표성 있는 32건 pilot의 journal 크기는 과거 결과에 남아 있지 않다. 따라서 바로 10k worker를
시작하지 않고 최신 devel 통합과 같은 32건의 checkpointed 1회 재검증을 Stage 4-B로 분리한다.

## 2. 원격 기준 재감사

Stage 4-A 시작 시 fetch한 원격 기준은 `upstream/devel@7df17a0ca9b8070192a230878fc9f56313ecae83`다.
현재 task branch는 기존 W3 작업 21개 commit이 앞서고 원격은 PR #5811 merge가 추가된 분기 상태다.

read-only `git merge-tree` 결과 source 파일의 직접 text conflict는 확인되지 않았지만 양쪽에서 새로 만든
`mydocs/orders/20260821.md`가 add/add conflict다. 두 문서 모두 독립된 오늘할일 기록이므로 어느 한쪽을
버리지 않고 순서대로 합치는 것이 다음 통합 원칙이다.

PR #5811에는 `rendering.rs`, HWPX parser, table layout·typeset과 serializer 변경이 있다. W3 collector
파일과 직접 겹치지는 않지만 실제 layout·문자 결정 결과의 무변경을 추정할 수는 없다. Stage 4 전건
실행 source는 최신 devel 통합 뒤 다시 고정해야 한다.

## 3. finalizer 계약

### 3.1 입력 무결성

finalizer는 `status=complete` checkpoint만 읽는다. state가 기록한 committed byte보다 journal이 짧거나
길면 모두 거부하고, 읽기 작업에서 tail을 자동 수정하지 않는다. journal 전건을 replay한 summary가
state와 exact가 아니어도 거부한다.

checkpoint runner도 다음 검사를 추가했다.

- 문서 aggregate의 canonical SHA-256을 재계산해 envelope hash와 exact 대조
- 문서 aggregate는 `attempted=1`, `success=1`, failure 전 항목 0만 허용
- 같은 source가 manifest에 두 번 등장하면 실행 전 거부
- 같은 BLAKE3의 서로 다른 source는 corpus 빈도 보존을 위해 허용

### 3.2 usage 병합

문서 aggregate에는 format이 top-level에만 있으므로 finalizer가 각 legacy·decision usage identity에
`format`을 주입한다. 다음 네 count만 같은 identity끼리 더한다.

- `documentCount`
- `paragraphCount`
- `runCount`
- `charCount`

나머지 필드가 추가·삭제되면 schema drift로 실패한다. 최종 row 순서는 `charCount` 내림차순 뒤 전체
canonical identity 순으로 고정해 journal 입력 순서와 무관하다. 최종 `legacyUsageRows`와
`decisionUsageRows`는 문서별 row 수의 합이 아니라 병합 뒤 row 수다.

finalizer는 다음 독립 대사를 수행한다.

- legacy·decision usage의 `charCount` 합 = `joins.joined`
- legacy usage의 `runCount` 합 = `counts.sourceRunsSeen`
- layout·coverage·category·join·document·backend 분모
- aggregate 전체 privacy recursive scan
- volatile runtime resource를 제외한 canonical aggregate SHA-256

## 4. 공개 실제 worker 검증

공개 HWP 2건과 HWPX 1건을 같은 source HEAD의 격리 worker로 실행했다. 2건 checkpoint commit 뒤 3번째
worker 시작 전에 강제 중단하고 재개한 final aggregate를 별도 무중단 checkpoint의 final aggregate와
비교했다.

| 항목 | 결과 |
| --- | ---: |
| attempted / success | 3 / 3 |
| layout / coverage 문자 | 1,546 / 1,546 |
| legacy / decision 최종 row | 26 / 45 |
| resumed vs uninterrupted final aggregate | exact |
| aggregate SHA-256 | `02bbf13c9439b62ede2e68f58312f4e4142ef69f87d6923ef1b87fcfe5fc699e` |
| journal bytes | 39,604 |
| final aggregate bytes | 39,599 |
| checkpoint 문서 identity 잔존 | 0 |

두 임시 checkpoint directory는 검증 직후 제거했다. 3건의 평균 journal 크기는 약 13.2 KB지만 문서가
작고 공개 표본이므로 10k 저장공간 예상치로 외삽하지 않는다.

## 5. full manifest preflight

기존 POC aggregate에는 전체 document hash 목록이 없고 상위 위험 200건만 식별 자료로 남아 있었다.
따라서 기존 hash를 재사용한다고 주장하지 않고 현재 10k bytes를 한 번 읽어 BLAKE3를 계산했다. 이는
renderer decision 재계측이 아니라 이후 중단·재개의 입력 identity를 고정하는 13.409초 read-only pass다.

| 항목 | 결과 |
| --- | ---: |
| documents | 10,000 |
| HWP / HWPX | 6,582 / 3,418 |
| candidate bytes | 5,471,422,390 |
| 최대 document bytes | 184,719,360 |
| ignored metadata files / bytes | 1 / 6,017 |
| symlink | 0 |
| unique source | 10,000 |
| 동일 content 그룹 | 14 |
| 추가 content 인스턴스 | 39 |
| manifest bytes | 3,401,216 |
| manifest SHA-256 | `a8a776b9382e3a2ba2e2f0043a0af37eb529f2ae2659525f64f6ff1edc538a6f` |

동일 content 39개 추가 인스턴스는 삭제하지 않았다. 기존 POC의 10k 문서 빈도와 비교 가능성을 유지하려면
같은 bytes가 다른 source에 있다는 이유로 deduplicate하면 안 된다. manifest 정렬은 format, BLAKE3,
size, source 순이며 같은 source만 중복 오류다.

local-only 결과는 다음 gitignored 파일에 권한 `0600`으로 남겼다.

```text
output/poc/font-metric-coverage/full-manifest-stage4-a-v1.json
output/poc/font-metric-coverage/full-manifest-preflight-stage4-a-v1.json
```

manifest만 corpus root, source와 개별 BLAKE3를 가진다. 870-byte preflight에는 문서 identity가 없고 이
보고서에도 옮기지 않았다.

## 6. 저장공간 판정

| 경계 | 값 |
| --- | ---: |
| preflight 시 filesystem available | 220,687,495,168 bytes |
| journal hard maximum | 17,179,869,184 bytes, 16 GiB |
| append 뒤 최소 reserve | 4,294,967,296 bytes, 4 GiB |
| maximum + reserve 충족 | 통과 |
| 10k에서 허용되는 평균 journal record 상한 | 약 1,717,987 bytes/document |

worker 자체는 문서당 aggregate row 20,000개와 output 32 MiB를 기본 상한으로 두므로 최악값을 10k에
곱한 크기는 운영 계획으로 사용할 수 없다. journal은 16 GiB에서 먼저 fail-closed하지만, 전건 도중
상한에 닿는 것을 정상 운영으로 볼 수도 없다.

과거 32건 pilot 결과는 count·시간·RSS만 보존했고 문서 aggregate/journal bytes는 보존하지 않았다.
따라서 공개 3건을 억지로 외삽하지 않고, 최신 devel 통합 뒤 같은 32건을 checkpoint runner로 한 번
재실행해 실제 총량·최대·p50·p90 record bytes를 측정하는 것을 Stage 4-B hard gate로 둔다.

## 7. 검증 결과

다음 31건이 모두 통과했다.

- checkpoint runner 6건
- checkpoint finalizer 2건
- full manifest builder 2건
- coverage contract·privacy 10건
- deterministic pilot selector 4건
- Linux process isolation supervisor 7건

검증은 강제 중단 exact replay, incomplete/tail/corruption, source·policy·contract drift, aggregate hash
위조, usage schema drift, symlink·inventory drift·해시 중 파일 변조, duplicate content 보존을 포함한다.

```text
tests 31
pass 31
fail 0
```

`cargo fmt --all`과 `cargo fmt --all -- --check`, `git diff --check`도 통과했다.

## 8. 변경 경계와 로컬 증거

| 파일 | 책임 |
| --- | --- |
| `font_metric_coverage_finalizer_policy.json` | usage identity·count·결정적 병합 계약 |
| `font_metric_coverage_full_manifest_policy.json` | 10k inventory·BLAKE3·storage·privacy 계약 |
| `font_metric_coverage_checkpoint_finalizer.mjs` | completed journal replay·최종 aggregate·hash |
| `font_metric_coverage_full_manifest.mjs` | 읽기 전용 발견·병렬 hash·stable-stat·preflight |
| 두 focused test | finalizer와 manifest fail-closed 회귀 |

구현은 local commit `d06e1774c`, raw bytes·parsed policy와 실제 Git HEAD 계보 보강은 `29c6a20a1`에
고정했다. Rust 제품 source, metric DB, fallback, paint와 font asset은 변경하지 않았다. private 10k
decision worker 전건, remote push, Issue·PR 변경도 수행하지 않았다.

## 9. 종료 판정과 다음 승인 후보

Stage 4-A 종료 조건은 충족됐다.

- checkpoint replay에서 최종 format 보존 usage aggregate 생성
- 재개·무중단 final aggregate exact
- 10k 현재 bytes의 local-only identity 동결
- 기존 10k 문서·format 분모와 exact 대사
- content duplicate를 삭제하지 않고 빈도 보존
- journal hard maximum + filesystem reserve 충족
- manifest와 식별 정보의 gitignored `0600` 격리

다음 승인 후보는 **Stage 4-B — 최신 devel 통합과 checkpointed 32건 pilot 재검증**이다.

1. `upstream/devel@7df17a0ca`를 task branch에 병합하고 add/add 오늘할일 문서를 양쪽 보존으로 해결한다.
2. worker·runner·contract를 최신 merge HEAD에서 다시 빌드하고 identity를 고정한다.
3. 기존 local-only 32건 manifest를 current bytes로 preflight한다.
4. 같은 32건을 checkpoint/finalizer로 한 번 실행해 이전 count·failure·combined hash와 비교한다.
5. journal record bytes 총량·최대·p50·p90으로 10k 저장공간 범위를 보고한다.

이 결과와 별도 승인 전에는 private 10k decision worker 전건 실행, 원격 push와 PR을 수행하지 않는다.
