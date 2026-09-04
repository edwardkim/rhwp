# Task M100 #4962 W3 Stage 4-C — private 10k decision delta 1차

- **Issue**: [#4962](https://github.com/edwardkim/rhwp/issues/4962)
- **상위 추적**: [#4960](https://github.com/edwardkim/rhwp/issues/4960)
- **계획**: [`task_m100_4962.md`](../plans/archives/task_m100_4962.md)
- **선행 결과**: [`task_m100_4962_stage4b_devel_pilot.md`](task_m100_4962_stage4b_devel_pilot.md)
- **실행 source HEAD**: `c1ec759f97eab0152fe8b38cef2e9fbbc50a5c6c`
- **실행 완료**: 2026-08-21 22:28 KST
- **finalizer 검증**: 2026-08-22 KST
- **단계 상태**: Stage 4-C 1차 완료, 2차 결정성 실행·W4 미착수

## 1. 승인 범위와 결론

승인된 Stage 4-C 범위에서 기존 private 10k corpus를 변경하지 않고 실제 renderer의 문자별 font metric
decision delta를 전건 실행했다. 첫 실행이 확장자 분류와 실제 컨테이너 형식을 혼동한 manifest 계약
결함으로 13건 뒤 fail-closed한 뒤, 두 의미를 분리하는 최소 정정을 `c1ec759f9`에 고정했다. 기존 32건
ingress를 다시 통과시킨 다음 새 manifest와 새 checkpoint 이름으로 10k 1차를 처음부터 실행했다.

결과는 W3 1차 기준선으로 유효하다.

- 10,000건 전부 시도, 9,909건 성공, 이유가 고정된 실패 91건
- parser·cancelled·truncation 0, layout/source join 분모 exact
- coverage 문자 54,326,042자 중 renderer metric 성공 계열 96.2049%
- `face-miss` 3.0230%, `char-miss` 0.7198%, `heuristic` 0.0523%
- finalizer 계약 대사 오류 0, 민감 값 0, canonical aggregate SHA-256 재계산 일치
- 실제 journal 510.04 MiB로 16 GiB hard maximum 이내

이번 결과는 W3 coverage 기준선이다. 장평·자간·고정 frame·stored LineSeg의 기존 POC를 폐기하지 않고,
W4 조판 위험 순위·fresh LineSeg cohort·backend snapshot·metric DB와 fallback 변경은 수행하지 않았다.

## 2. 전건 ingress에서 발견한 형식 계약 결함

정정 전 r1은 13건을 정상 checkpoint한 뒤 manifest의 확장자 기반 `hwp`와 worker aggregate의 실제
`hwpx`가 달라 중단됐다. 불일치 거부는 올바른 보호 불변식이므로 완화하지 않았다.

정정된 `stage4-c-v2` manifest는 다음 두 축을 분리한다.

| 축 | 의미 | 결과 |
| --- | --- | ---: |
| `inputFormat` | 동결 inventory의 확장자 분모 | HWP 6,582 / HWPX 3,418 |
| 실행 `format` | 지원 컨테이너 시그니처 | HWP OLE 6,491 / HWPX ZIP 3,424 |
| 입력·컨테이너 교차 | 이름과 실제 지원 형식 불일치 | 45 |
| 지원 컨테이너 미인식 | HWP5/HWPX가 아닌 입력 | 85 |

지원 컨테이너가 교차된 45건은 corpus 이름을 바꾸지 않고 실제 컨테이너 형식으로 실행했다. 미인식
85건은 local-only aggregate 진단에서 HWP3 38, unknown 15, DRM 8, empty 24로 분해됐다. HWP3·HML·unknown은
이번 W3 HWP5/HWPX 실행 범위 밖이므로 worker가 명시적 `unsupported` 실패로 회수한다. HML은 0건이었다.

정정 commit에서 기존 private 32건을 별도 r2 checkpoint로 다시 실행한 결과 32/32 성공했고 Stage 3의
문서·count·7개 category·combined decision hash와 Stage 4-B semantic body가 모두 exact였다. 따라서
manifest 정정이 renderer coverage 의미를 바꾸지 않았음을 전건 전에 확인했다.

## 3. 실행 identity와 preflight

식별 manifest는 gitignored `output/`에만 보존하고 공개 보고에는 비식별 preflight만 사용한다.

| 항목 | 값 |
| --- | ---: |
| source HEAD | `c1ec759f97eab0152fe8b38cef2e9fbbc50a5c6c` |
| policy | `stage4-c-v2` |
| documents | 10,000 |
| candidate bytes | 5,471,422,390 |
| maximum input | 184,719,360 bytes |
| ignored regular files / bytes | 1 / 6,017 |
| duplicate content groups / extra instances | 14 / 39 |
| manifest SHA-256 | `88eac0436082e85f8f74a547f01b83b9386e9babdd56f51c49a2098dc35431e2` |
| checkpoint maximum + 4 GiB reserve | 통과 |

동일 content의 추가 인스턴스 39건은 편집 습관 빈도를 보존하기 위해 제거하지 않았다. manifest와
checkpoint는 각각 source·runner·worker·coverage contract·policy·분석 옵션·격리 한도를 hash로 고정한다.

## 4. checkpoint 실행 결과

| 항목 | 값 |
| --- | ---: |
| attempted / success / failure | 10,000 / 9,909 / 91 |
| worker elapsed 합 | 1,712,782 ms, 28.55분 |
| peak worker RSS | 595,988,480 bytes, 568.38 MiB |
| journal entries / bytes | 10,000 / 534,811,486, 510.04 MiB |
| checkpoint chain | `94d9bbe3…6b03649` |

문서 실패 분모는 다음과 같다.

| 실패 | 건수 |
| --- | ---: |
| unsupported | 53 |
| empty | 24 |
| drm | 8 |
| encrypted | 5 |
| resource-limit | 1 |
| parser / cancelled | 0 / 0 |

지원 HWP/HWPX 컨테이너는 9,915건이며, 그중 9,909건이 성공했다. 나머지 6건은 encrypted 5건과
resource-limit 1건이다. 지원 컨테이너 밖 85건은 unsupported 53건 중 HWP3 38건과 unknown 15건,
DRM 8건, empty 24건으로 정확히 대사된다.

단일 resource-limit 문서는 1.22초·약 43 MiB RSS에서 실패해 OS memory·wall timeout 유형이 아니었다.
모든 조정 가능 상한을 최대값으로 높인 local-only 재실행에서도 같은 실패가 재현됐고, debug breakpoint로
폰트 관련 dimension 문자열의 4,096바이트 hard cap에서 거부됐음을 확인했다. 문자열 값과 문서 identity는
출력하거나 보존하지 않았다. 이는 partial aggregate를 수용하지 않는 입력 방어의 정상 작동이며 W3
기준선에서는 이유가 있는 제외 1건으로 유지한다.

## 5. finalizer 분모와 coverage

finalizer는 10,000개 journal record를 format이 포함된 usage identity로 결정론적으로 병합했다.

| 분모 | 값 |
| --- | ---: |
| paragraphs / source runs | 3,764,899 / 5,602,126 |
| layout / coverage | 54,399,371 / 54,326,042 |
| not applicable / excluded / truncated | 73,329 / 0 / 0 |
| joined / layoutOnly | 54,399,371 / 0 |
| legacy / decision usage rows | 66,447 / 138,965 |

`layout = coverage + notApplicable + excluded`, `layout = joined + layoutOnly + excluded`, `attempted = success +
failures`가 모두 exact다. backend는 이번 portable pass에서 요청하지 않았으므로 requested 0이며 coverage
성공이나 miss에 섞지 않았다.

| coverage category | 문자 | coverage 대비 |
| --- | ---: | ---: |
| exact-hit | 48,071,620 | 88.4872% |
| measured-overlay | 4,192,417 | 7.7171% |
| metric-surrogate | 273 | 0.0005% |
| identity-alias-hit | 0 | 0.0000% |
| face-miss | 1,642,295 | 3.0230% |
| char-miss | 391,018 | 0.7198% |
| heuristic | 28,419 | 0.0523% |

성공 계열 네 분류의 합은 52,264,310자, 96.2049%다. 이 수치는 기존 POC의 face-level
`metricMappedChars` 91.16%와 분모·판정 단위가 다르므로 개선율로 직접 비교하지 않는다. W3는 실제 문자별
renderer decision이고 기존 값은 선언 face/metric mapping 기준선이다.

최종 aggregate는 110,097,106 bytes, mode `0600`이며 SHA-256은
`378671053ae94c02cda4caf8b8a4a1ac7866c3711fdf72c14aa527f8d9bc2888`이다. 계약 reconcile 오류 0,
민감 값 탐지 0, canonical hash 재계산 일치를 확인했다.

## 6. 기존 POC와의 대사

기존 POC와 새 W3는 같은 10,000개 input bytes를 사용하지만 성공 집합이 다르다.

| 항목 | 기존 POC v2 | W3 1차 | 차이 |
| --- | ---: | ---: | ---: |
| success | 9,948 | 9,909 | -39 |
| failure | 52 | 91 | +39 |
| DRM / empty / encrypted | 8 / 24 / 5 | 8 / 24 / 5 | exact |
| unsupported | 15 | 53 | +38 |
| resource-limit | 0 | 1 | +1 |

unsupported 증가 38건은 기존 POC가 확장자 분모 안에서 성공으로 포함했던 HWP3 38건과 정확히 같다.
나머지 1건은 위의 dimension hard cap이다. parser 실패는 양쪽 모두 0이므로 +39를 HWP5/HWPX parser
regression으로 해석할 근거가 없다.

성공 집합 차이와 함께 새 legacy projection은 기존보다 문단 11,407개, 문자 539,388자 적다. 기존 합본의
usage row 57,395개는 HWP/HWPX 공통 key를 합친 값이므로 format을 identity에 주입한 새 결과와 직접 비교할
수 없다. 기존 포맷별 usage row 합 68,095개와 비교하면 새 66,447개는 1,648개 적다. 이 세 delta는 제외된
HWP3 38건과 hard-cap 1건을 포함한 집합 차이 위의 값이며, 보존된 기존 aggregate만으로 공통 9,909건의
문서별 차이를 역산할 수는 없다. 이를 parser/model drift로 과잉 해석하거나 다시 10k를 계측해 덮지 않는다.

## 7. privacy와 보존 경계

다음 증거는 모두 gitignored local output이며 Issue·PR·CI artifact로 게시하지 않는다.

```text
output/poc/font-metric-coverage/full-manifest-stage4-c-r2.json
output/poc/font-metric-coverage/checkpoint-stage4-c-10k-r2/
output/poc/font-metric-coverage/final-stage4-c-10k-r2.json
```

보고서에는 aggregate count와 실행 identity hash만 기록했다. corpus path·filename·개별 BLAKE3·개별
aggregate hash·원문·font dimension 문자열은 포함하지 않았다. 정정 전 r1과 ingress r1/r2도 원인 계보를
위해 덮어쓰지 않고 같은 local-only 경계에 유지한다.

## 8. 종료 판정과 다음 승인 후보

Stage 4-C 1차 종료 조건은 충족됐다.

- 현재 source에 결박된 full manifest와 10k checkpoint complete
- 모든 문서 실패가 고정된 이유별 count로 설명됨
- coverage·join·document 분모 exact, excluded·truncation 0
- finalizer privacy·canonical hash 통과
- 기존 POC의 재사용 범위와 새 decision delta의 책임 분리
- metric DB·fallback·font asset·제품 renderer 정책 변경 없음

다음 hard gate 후보는 같은 source·manifest·worker identity로 **10k 2차 결정성 pass**를 실행해 문서
failure vector, 모든 count, final aggregate의 semantic body와 문서 aggregate 결합 hash를 대조하는 것이다.
별도 승인 전에는 2차 실행, W4 위험 순위, fresh/backend cohort, 원격 push와 PR을 진행하지 않는다.
