# Task M100 #4962 W3 Stage 4-B — 최신 devel 통합·checkpointed pilot

- **Issue**: [#4962](https://github.com/edwardkim/rhwp/issues/4962)
- **상위 추적**: [#4960](https://github.com/edwardkim/rhwp/issues/4960)
- **계획**: [`task_m100_4962.md`](../plans/task_m100_4962.md)
- **선행 결과**: [`task_m100_4962_stage4a_finalizer_manifest.md`](task_m100_4962_stage4a_finalizer_manifest.md)
- **최종 실행 source HEAD**: `4a6c9e24234375e5bba5477c5d96bbed3261bc6c`
- **날짜**: 2026-08-21 KST
- **단계 상태**: Stage 4-B 완료, private 10k decision worker 전건 미착수

## 1. 승인 범위와 결론

승인된 Stage 4-B 범위에서 최신 `upstream/devel`을 task branch에 정상 병합하고, 기존 private 고위험
cohort 32건을 새 checkpoint runner·finalizer로 재실행했다. 최초 R1 뒤 전체 보호 suite가 발견한 schema
registry 누락을 고치면서 source identity가 바뀌었으므로 같은 입력을 최종 source에서 R2로 다시 결박했다.
목적은 coverage 기준선을 다시 만드는 것이 아니라 merge source의 semantic drift와 10k journal 저장공간
적합성을 전건 전에 검사하는 것이었다.

결과는 통과다.

- 32/32 성공, 모든 failure·excluded·truncation 0
- Stage 3의 문서 결과·7개 기준 count·7개 category exact
- Stage 3 결합 산식으로 복원한 combined decision hash exact
- journal lower-p90 stress 10k 외삽 5.23 GiB, 16 GiB hard maximum 이내
- private document identity와 개별 hash는 로컬 `0600`·gitignored 경계 밖으로 내보내지 않음

따라서 최신 devel 통합으로 인한 coverage semantic drift와 checkpoint 저장공간 blocker는 발견되지
않았다. 이 승인으로 private 10k 전건은 실행하지 않았다.

## 2. 최신 devel 통합

실행 전 `upstream/devel`은 `7df17a0ca9b8070192a230878fc9f56313ecae83`, task branch는 24 commit
앞·9 commit 뒤였다. merge-tree에서 예고된 유일한 text conflict는 양쪽이 각각 추가한
`mydocs/orders/20260821.md`였다.

충돌은 task branch의 #4962 W3 기록 뒤에 원격 devel의 CI 통과 외부 PR 통합 검토 기록을 이어 붙여 양쪽
내용을 모두 보존했다. merge commit은 `308e38f73`이며 부모는 기존 task HEAD `82af4ab51`과
`upstream/devel@7df17a0ca`다. merge 뒤 원격 devel이 task branch의 ancestor임을 확인했다.

## 3. merge source 검증

merge HEAD에서 다음 검증을 다시 수행했다.

| 검증 | 결과 |
| --- | --- |
| `cargo fmt --all` + `cargo fmt --all -- --check` | 통과 |
| `cargo build --locked --bin rhwp-agent --example font_metric_coverage_worker` | 통과 |
| contract·selector·supervisor·runner·finalizer·manifest Node test | 31/31 통과 |
| `regression_suite_024` 전체, merge source | 119/119 통과 |
| `regression_suite_024` #4962, 최종 source | 8/8 통과 |
| `regression_suite_011` 전체, 최종 source | 127 통과, 진단 1건 의도적 ignore |

integration source는 검증 전용 detached worktree에서 manifest를 준비해 실행했고, generated suite와
manifest를 source branch에 남기지 않았다. 검증 worktree는 제거했다. checkout 중 기존 LFS 관리 PDF가
pointer 형식이 아니라는 비차단 경고가 있었지만 이번 변경이나 두 focused suite의 결과에는 영향을 주지
않았으며 해당 파일은 수정하지 않았다. 최종 `cargo fmt --all`은 #4962 integration source의 기존 assertion
네 곳을 현재 rustfmt 형식으로만 정규화했으며 실행 의미는 바꾸지 않았다.

전체 `regression_suite_011`은 최초 실행에서 `font_metric_coverage.rs`가 legacy projection schema version을
직접 정의한 단일 출처 위반을 발견했다. 값을 바꾸지 않고 `schema_registry.rs`의 전용 상수로 옮겨
`4a6c9e242`에 고정했다. 해당 contract와 전체 suite를 다시 통과시킨 뒤 worker를 재빌드하고 아래 32건
R2를 최종 source에서 다시 실행했다.

## 4. private 32건 입력 preflight

기존 Stage 3 cohort의 식별 manifest를 저장소 밖으로 복사하거나 공개하지 않고, merge HEAD에서 현재
bytes를 다시 읽어 다음 항목을 전건 검사했다.

| 항목 | 결과 |
| --- | ---: |
| documents | 32 |
| HWP / HWPX | 16 / 16 |
| input bytes | 554,287,921 |
| corpus-root confinement | 32/32 |
| regular file·symlink 검사 | 32/32 |
| size match | 32/32 |
| BLAKE3 match | 32/32 |

최종 preflight source HEAD는 `4a6c9e242`, manifest SHA-256은
`0a7c0355e1f15514a69092ea4800578418a63a1d3ca7f9a0fefe8d08d63bd6f5`다. 비식별 preflight에는 문서
identity가 없고 권한 `0600`의 gitignored local output으로만 남겼다.

## 5. checkpoint·finalizer 결과

최종 source에서 같은 32건을 manifest 순서대로 R2로 실행했다.

| 결과 | 값 |
| --- | ---: |
| attempted / success | 32 / 32 |
| 전체 failure / resource-limit | 0 / 0 |
| worker elapsed 합 | 204,098 ms |
| peak worker RSS | 595,742,720 bytes |
| layout / coverage 문자 | 6,856,718 / 6,849,574 |
| not applicable / excluded / truncated | 7,144 / 0 / 0 |
| final legacy / decision usage row | 5,896 / 11,635 |
| final aggregate SHA-256 | `141d10a27f9d1f6df7b9e8c2a5a1574b1d8ecb34a4a2b8b5f79282dabcfcf9e7` |
| checkpoint chain | `37330a0d706da8d78f859c2f6376e3410a836a47a7fddee94e142302291f7bee` |

`joins.joined`는 layout 6,856,718자와 일치했고 `layoutOnly`와 `excluded`는 모두 0이다. backend 관찰은
이번 portable layout pass에서 요청하지 않았으므로 coverage 성공으로 가장하거나 miss에 섞지 않았다.

## 6. Stage 3 기준선 exact 대조

과거 Stage 3의 combined decision hash 산식은 각 문서 순서대로 다음 bytes를 SHA-256에 누적하는 방식임을
당시 실행 기록에서 복구했다.

```text
format:complete:document-aggregate-hash\n
```

새 journal의 문서별 aggregate hash를 공개하지 않고 이 산식을 로컬에서 재계산했다. 그 결과는 Stage 3의
`bb3e29267ba5ac364196a3eb868c7883b710732597fdbe619a74eb621cc11ae6`와 exact였다.

| 비교 항목 | 판정 |
| --- | --- |
| attempted / success / failure vector | exact |
| 7개 기준 count | exact |
| 7개 coverage category | exact |
| combined decision hash | exact |
| merge-source R1 대비 identity 제외 semantic body | exact |

category exact 값은 measured-overlay 769,493, identity-alias-hit 0, metric-surrogate 0, exact-hit
5,474,219, char-miss 36,588, face-miss 565,571, heuristic 3,703이다. merge 뒤 source에서도 의미론적
coverage drift가 없음을 count와 문서별 aggregate 결합 해시 양쪽으로 확인했다.

R1과 R2의 전체 final aggregate hash와 checkpoint chain은 다르다. 이 envelope에는 source HEAD,
runner·worker hash 등 실행 identity가 들어가므로 source 중앙화 commit 뒤 달라져야 한다. `aggregateHash`,
`checkpoint`, `finalizer` 계보를 제외한 semantic body는 exact였고, Stage 3 산식은 문서별 semantic
aggregate만 결합하므로 과거 combined hash도 exact였다.

## 7. journal 저장공간 gate

각 NDJSON record의 UTF-8 bytes에 trailing newline을 포함해 측정했다. quantile은 Stage 3 시간 모델과
같이 정렬된 값의 `floor((n - 1) × fraction)` index를 사용한다.

| 형식 | 문서 | 최소 | p50Lower | p90Lower | p95Lower | 최대 | 평균 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| HWP | 16 | 132,199 | 308,485 | 619,464 | 831,701 | 1,017,717 | 403,811 bytes |
| HWPX | 16 | 49,724 | 170,902 | 449,655 | 473,794 | 490,626 | 244,949 bytes |
| 전체 | 32 | 49,724 | 274,173 | 588,752 | 619,464 | 1,017,717 | 324,380 bytes |

최종 R2 실측 journal은 10,380,169 bytes다. 포맷별 분포에 기존 10k 구성 HWP 6,582건·HWPX 3,418건을 적용한
외삽은 다음과 같다.

| 모델 | 예상 journal | 16 GiB 대비 |
| --- | ---: | ---: |
| lower-p50 | 2,614,591,306 bytes, 2.44 GiB | 통과 |
| arithmetic mean | 3,495,122,200 bytes, 3.25 GiB | 통과 |
| lower-p90 stress | 5,614,232,838 bytes, 5.23 GiB | 통과 |

이 cohort는 고위험 편향이므로 실제 10k 분포를 확정하는 표본은 아니다. 그럼에도 p90 stress 외삽도
16 GiB hard maximum의 약 32.7%이며, 현재 filesystem 여유 220,110,974,976 bytes와 append 뒤 4 GiB
reserve 조건을 함께 충족한다. Stage 4-C는 기존 fail-closed storage gate를 유지한다.

## 8. privacy와 로컬 증거

다음 evidence는 모두 권한 `0600`의 gitignored local output이다.

```text
output/poc/font-metric-coverage/preflight-stage4-b-pilot-r2.json
output/poc/font-metric-coverage/checkpoint-stage4-b-pilot-r2/
output/poc/font-metric-coverage/final-stage4-b-pilot-r2.json
output/poc/font-metric-coverage/comparison-stage4-b-pilot-r2.json
```

비교 결과는 문서 identity·경로·filename·개별 BLAKE3·개별 aggregate hash·raw trace를 포함하지 않는지
재귀 검사했다. 식별 manifest와 journal은 Issue·PR·CI artifact에 게시하지 않는다. merge source의 R1
evidence도 덮어쓰지 않고 같은 local-only 경계에 보존했다.

## 9. 종료 판정과 다음 승인 후보

Stage 4-B 종료 조건은 충족됐다.

- 최신 devel 정상 병합과 충돌 양쪽 기록 보존
- merge source의 build·focused regression 통과
- 기존 32건 current-byte preflight 통과
- checkpoint/finalizer 32/32 성공
- Stage 3 count·category·combined decision hash exact
- p90 stress journal 외삽이 16 GiB hard maximum 이내
- private identity의 local-only 격리

source 최소 정정은 local commit `4a6c9e242`에 고정했다. metric DB, fallback 순서, font asset과 제품
renderer 출력 정책은 변경하지 않았다.

다음 승인 후보는 **Stage 4-C — private 10k decision delta 1차 checkpoint pass**다. Stage 4-A manifest는
병합 전 source HEAD에 결박되어 있으므로 덮어쓰거나 그대로 재사용하지 않는다. 먼저 Stage 4-B 보고 commit
까지 포함한 현재 HEAD에 결박된 새 local-only full manifest와 비식별 preflight를 별도 이름으로
생성·검증한 뒤 1차 전건만 실행한다.

예상 순차 시간은 Stage 3 두 회차 기준 lower-p50 17.63시간, arithmetic mean 21.84시간, lower-p90 stress
36.80시간이다. 첫 전건 결과를 보고하고 별도 승인받기 전에는 두 번째 결정성 pass, Stage 5, 원격 push와
PR을 수행하지 않는다.
