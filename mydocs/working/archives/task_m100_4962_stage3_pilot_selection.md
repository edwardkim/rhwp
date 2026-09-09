# Task M100 #4962 W3 Stage 3-C — 결정적 pilot cohort 선정

- **Issue**: [#4962](https://github.com/edwardkim/rhwp/issues/4962)
- **상위 추적**: [#4960](https://github.com/edwardkim/rhwp/issues/4960)
- **계획**: [`task_m100_4962.md`](../../plans/archives/task_m100_4962.md)
- **선행 hard gate**: [`task_m100_4962_stage3_public_fixtures.md`](task_m100_4962_stage3_public_fixtures.md)
- **구현 commit**: `2e148f584`
- **날짜**: 2026-08-21 KST
- **단계 상태**: Stage 3의 세 번째 hard gate 완료, private pilot 미착수

## 1. 승인 범위와 결론

기존 POC aggregate만 재사용해 private pilot의 선택 규칙과 규모를 고정하고 실제 로컬 cohort manifest를
생성했다. 원본 HWP/HWPX는 열지 않았고, 파일 hash 재계산·W3 worker 실행·새 전수 계측·원격 작업을
수행하지 않았다.

선정 결과는 HWP 16건과 HWPX 16건, 합계 32건이다. 각 포맷에서 4건씩, 합계 8건을 canary로 먼저
실행한다. 이 cohort는 전체 corpus 발생률을 추정하는 확률 표본이 아니라 대형·다문자·압축 고정 프레임·
극단 압축·kerning을 의도적으로 많이 포함한 **고위험 feasibility cohort**다.

## 2. 기존 자료 재사용 판정

다음 검사를 현재 HEAD에서 다시 통과했다.

```bash
node scripts/font_metric_coverage_contract.mjs check \
  --poc output/poc/font-layout-habits/summary-10k-v2.json \
  --poc-hwp output/poc/font-layout-habits/summary-hwp-v2.json \
  --poc-hwpx output/poc/font-layout-habits/summary-hwpx-v2.json
```

결과: `POC v2 baseline ok`. 전체·포맷별 totals와 비식별 projection을 새로 수집할 이유가 없음을 다시
확인했다.

합본 `summary-10k-v2.json`의 위험 상위 200건은 HWP 181/HWPX 19로 기울어 있다. 이를 그대로 쓰면
HWPX parser와 container 비용을 판단할 표본이 너무 작아지므로 후보 pool로 사용하지 않는다. 대신
포맷별 aggregate의 상위 200건을 각각 사용한다.

| 기존 포맷별 후보 | HWP | HWPX |
| --- | ---: | ---: |
| risk row | 200 | 200 |
| 고유 content hash | 200 | 200 |
| kerning 양성 | 42 | 13 |
| 압축 고정 프레임 양성 | 199 | 199 |
| 극단 압축 양성 | 200 | 199 |
| 최대 입력 크기 | 184,719,360 bytes | 25,362,288 bytes |
| 최대 문자 수 | 706,419 | 261,424 |

필요한 quota는 모든 포맷에서 공집합 없이 구성할 수 있다.

## 3. 결정적 선택 정책

machine-readable 정본은
[`font_metric_coverage_pilot_policy.json`](../../tech/investigations/issue-4962/font_metric_coverage_pilot_policy.json)이다.
각 포맷에 다음 quota를 같은 순서로 적용하며 이미 선택된 content hash는 뒤 quota에서 제외한다.

| tier | 축 | 포맷별 수 | 합계 |
| --- | --- | ---: | ---: |
| canary | risk score 최대 | 1 | 2 |
| canary | 파일 크기 최대 | 1 | 2 |
| canary | 문자 수 최대 | 1 | 2 |
| canary | kerning 문자 최대 | 1 | 2 |
| full | 압축 고정 프레임 문자 | 3 | 6 |
| full | 극단 압축 문자 | 2 | 4 |
| full | 잔여 risk score | 2 | 4 |
| full | 잔여 파일 크기 | 2 | 4 |
| full | 잔여 문자 수 | 2 | 4 |
| full | 상위 200 내부의 낮은 risk control | 1 | 2 |
| **합계** |  | **16** | **32** |

각 축은 해당 값, risk score, 문자 수, 파일 크기, BLAKE3 순으로 정렬한다. content hash가 중복되면 UTF-8
경로 정렬에서 앞선 로컬 항목 하나를 대표로 삼는다. 따라서 입력 배열 순서나 worker thread 완료 순서가
달라도 같은 cohort가 나온다. 실제 식별 경로와 hash는 공개 정책이나 이 보고서에 넣지 않는다.

## 4. selector와 private manifest 경계

`scripts/font_metric_coverage_pilot_selector.mjs`는 다음을 fail closed한다.

- 포맷별 후보 200건 미만
- 필수 numeric field·extension·BLAKE3 형식 오류
- kerning 등 양성 quota의 후보 부족
- quota 합과 16/32/8 분모 불일치
- private manifest를 저장소의 `output/` 밖에 쓰려는 요청

실제 manifest는 다음 로컬 경로에 생성했다.

```text
output/poc/font-metric-coverage/pilot-cohort-stage3-p1.json
```

이 경로는 `.gitignore`의 `/output/` 규칙에 포함되고 파일 권한은 `0600`이다. manifest는 source와
BLAKE3를 포함하므로 커밋·Issue·PR·CI artifact 대상이 아니다.

원본 문서의 현재 존재 여부와 BLAKE3 일치는 아직 검사하지 않았다. 해당 검사는 문서 bytes를 읽는
행위이므로 private pilot 승인 뒤 preflight에서 수행하며, 하나라도 다르면 임의 대체하지 않고 cohort를
stale 상태로 중단한다.

## 5. 선정된 부하 범위

식별 필드를 제외한 manifest 집계는 다음과 같다.

| cohort | 문서 | 입력 bytes 합 | 문자 합 | 단일 최대 bytes | 단일 최대 문자 |
| --- | ---: | ---: | ---: | ---: | ---: |
| canary | 8 | 247,388,142 | 2,108,443 | 184,719,360 | 706,419 |
| full | 32 | 554,287,921 | 6,856,718 | 184,719,360 | 706,419 |
| full HWP | 16 | 437,151,744 | 5,799,392 | 184,719,360 | 706,419 |
| full HWPX | 16 | 117,136,177 | 1,057,326 | 25,362,288 | 261,424 |

입력 bytes 합은 peak memory가 아니다. 문서마다 별도 worker가 순차 실행되며 기존 2 GiB address-space,
75초 CPU, 90초 wall timeout을 각각 적용한다. canary 8건과 full 32건 2회를 모두 timeout까지 사용하면
최대 72회, 108분 wall-clock이므로 실제 canary 결과를 보기 전에 full 2회를 자동 진행하지 않는다.

## 6. canary와 full pilot 승인 게이트

다음 승인을 받으면 먼저 아래 preflight와 canary까지만 실행한다.

1. manifest의 모든 경로가 승인된 corpus root 아래 regular file인지 확인한다.
2. 현재 bytes의 BLAKE3가 manifest와 모두 일치하는지 확인한다.
3. 정확한 task HEAD에서 Rust worker를 빌드한다.
4. canary 8건을 문서별 격리 supervisor로 한 번 실행한다.
5. 식별 출력이 없고 supervisor fatal error가 없으며, 전체 실패 2건 이하,
   `resource-limit` 1건 이하, 관찰 peak RSS 1.5 GiB 이하인지 판정한다.

canary 통과 뒤에도 full pilot 2회는 별도 진행 경계다. full 결과는 포맷별 elapsed/RSS 분포와 W3
aggregate count·hash 반복 일치를 비교한다. 전수 예상 시간은
`6,582 × HWP 중앙 실행시간 + 3,418 × HWPX 중앙 실행시간`으로 산출하되, 고위험 표본의 보수적 계획값일
뿐 불편향 추정치라고 부르지 않는다.

## 7. 대표성 한계

기존 risk rows는 POC 성공 문서만으로 만들었고 parser 실패 52건을 식별 연결하지 않는다. 따라서 이
pilot의 실패율을 10k corpus 실패율로 외삽하지 않는다. LineSeg 누락 여부도 문서별 risk row에 없으므로
이번 cohort는 fresh-layout 또는 LineSeg 유효성 표본이라고 주장하지 않는다.

이 결손을 채우기 위해 지금 corpus를 다시 전수 계측하지 않는다. W3 portable coverage의 고위험 실행
가능성·자원 비용·결정성을 먼저 판단하고, parser 실패율과 LineSeg/fresh-layout은 기존 전체 totals와
별도 승인된 목적별 cohort로 분리한다.

## 8. 검증 결과

```bash
node --test \
  scripts/tests/font_metric_coverage_pilot_selector.test.mjs \
  scripts/tests/font_metric_coverage_contract.test.mjs \
  scripts/tests/font_metric_coverage_supervisor.test.mjs
```

결과: **21 passed, 0 failed**.

- 입력 순서 역전에도 같은 32건 선택
- duplicate content의 대표 경로 결정성
- 희귀 양성 quota 부족 시 fail closed
- private output의 `output/` 경계
- 기존 W3 분류·privacy·hash 계약과 process 격리 회귀

추가 검증:

- selector와 test `node --check`: 통과
- policy JSON parse: 통과
- POC v2 전체·포맷별 기준선 대사: 통과
- `cargo fmt --all -- --check`: 통과
- `git diff --check`: 통과

현재 완료 범위는 cohort 선정과 로컬 manifest 생성까지다. private 원본 preflight, canary·full pilot,
10k 전수 실행, 원격 push와 PR은 수행하지 않았다.
