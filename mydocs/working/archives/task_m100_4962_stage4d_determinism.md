# Task M100 #4962 W3 Stage 4-D — private 10k 결정성 반복

- **Issue**: [#4962](https://github.com/edwardkim/rhwp/issues/4962)
- **상위 추적**: [#4960](https://github.com/edwardkim/rhwp/issues/4960)
- **계획**: [`task_m100_4962.md`](../../plans/archives/task_m100_4962.md)
- **1차 결과**: [`task_m100_4962_stage4c_full_pass.md`](task_m100_4962_stage4c_full_pass.md)
- **실행 source HEAD**: `c1ec759f97eab0152fe8b38cef2e9fbbc50a5c6c`
- **날짜**: 2026-08-22 KST
- **단계 상태**: Stage 4-D 완료, W3 종료 조건 충족

## 1. 승인 범위와 결론

승인된 범위에서 Stage 4-C 1차와 동일한 source·runner·worker·manifest·contract·policy·분석 옵션·격리
한도로 private 10k decision delta를 한 번 더 실행했다. 1차 artifact를 덮어쓰지 않고 별도 r3
checkpoint와 final aggregate를 만들었다.

결정성 hard gate는 통과다.

- 10,000개 문서의 status·failure 또는 document aggregate hash token 전부 exact
- failure vector, 모든 count·category·join과 checkpoint chain exact
- final aggregate SHA-256 exact
- 110,097,106 bytes final JSON 전체가 byte-for-byte exact
- r3 계약 reconcile 오류 0, 민감 값 0, canonical hash 재계산 일치
- 차이는 실행 시간·peak RSS와 이를 포함한 checkpoint journal bytes뿐

따라서 W3 collector와 finalizer는 고정된 입력·정책에서 결정론적인 renderer font metric coverage
기준선을 재현했다. 세 번째 10k 반복이나 새 계측 기능은 필요하지 않다.

## 2. 동일 실행 identity 고정

1차 source `c1ec759f9` 뒤 현재 branch에는 Stage 4-C 계획·보고 문서 commit `d248a6450`만 추가돼 있었다.
실행 관련 tracked source가 바뀌지 않았음을 diff로 확인하고 다음 identity를 1차 state와 대조했다.

| identity | 판정 |
| --- | --- |
| source HEAD | `c1ec759f97eab0152fe8b38cef2e9fbbc50a5c6c`, exact |
| runner SHA-256 | `93958a3b…e6468b`, exact |
| worker SHA-256 | `4909d338…312fe6`, exact |
| manifest SHA-256 | `88eac043…5431e2`, exact |
| checkpoint policy SHA-256 | `b71c1c29…ccf5dd`, exact |
| coverage contract SHA-256 | `7a831988…4c4707`, exact |
| analysis options / isolation limits | exact |
| document count / order | 10,000 / exact |

CLI의 current-HEAD guard에 문서 commit을 실행 source로 거짓 기록하거나 primary checkout을 detached 상태로
바꾸지 않았다. 동일 runner script가 export하는 검증된 `runResumableCoverage` entrypoint에 원래 source
identity를 전달했다. 실행 전에 `c1ec759f9..HEAD` 변경이 계획서와 Stage 4-C 보고서뿐이고 runner·worker
hash가 1차 state와 exact임을 확인했으므로 실행 code identity를 유지한다.

## 3. r2/r3 checkpoint 비교

| 항목 | r2 1차 | r3 2차 | 판정 |
| --- | ---: | ---: | --- |
| attempted / success / failure | 10,000 / 9,909 / 91 | 10,000 / 9,909 / 91 | exact |
| unsupported / empty / DRM | 53 / 24 / 8 | 53 / 24 / 8 | exact |
| encrypted / resource-limit | 5 / 1 | 5 / 1 | exact |
| parser / cancelled | 0 / 0 | 0 / 0 | exact |
| layout / coverage | 54,399,371 / 54,326,042 | 54,399,371 / 54,326,042 | exact |
| notApplicable / excluded / truncated | 73,329 / 0 / 0 | 73,329 / 0 / 0 | exact |
| paragraphs / source runs | 3,764,899 / 5,602,126 | 3,764,899 / 5,602,126 | exact |
| joined / layoutOnly | 54,399,371 / 0 | 54,399,371 / 0 | exact |

7개 coverage category도 전부 exact다.

| category | r2 | r3 |
| --- | ---: | ---: |
| measured-overlay | 4,192,417 | 4,192,417 |
| identity-alias-hit | 0 | 0 |
| metric-surrogate | 273 | 273 |
| exact-hit | 48,071,620 | 48,071,620 |
| char-miss | 391,018 | 391,018 |
| face-miss | 1,642,295 | 1,642,295 |
| heuristic | 28,419 | 28,419 |

각 journal record에서 변동 metrics를 제외하고 다음 token을 10,000개 순서대로 비교했다.

```text
complete: format:complete:document-aggregate-hash
failed:   format:failed:failure-reason
```

10,000/10,000 token이 exact였고 SHA-256 chain도 양쪽 모두
`94d9bbe3cce4218728ff29def8ab24abb51ffc434ec395d8be663fdab6b03649`로 일치했다. 즉 합계가 우연히
같은 것이 아니라 각 문서의 의미 결과와 순서가 같다.

## 4. final aggregate byte exact

두 finalizer output은 다음 항목이 모두 같았다.

| 항목 | 결과 |
| --- | --- |
| documents·formats·failure vector | exact |
| counts·categories·joins·backends | exact |
| legacy / decision usage rows | 66,447 / 138,965, exact |
| checkpoint identity·chain | exact |
| finalizer identity | exact |
| aggregate SHA-256 | `378671053ae94c02cda4caf8b8a4a1ac7866c3711fdf72c14aa527f8d9bc2888` |
| JSON bytes | 110,097,106 / 110,097,106, byte-for-byte exact |
| JSON file SHA-256 | `24eb1d1587079aa664eb265e4dda6e160fa55a9b41b7b359e385ddcfe49352a1` |

r3에 대해 contract reconcile 오류 0, 민감 값 탐지 0, canonical aggregate hash 재계산 일치를 다시
확인했다. final output은 mode `0600`의 gitignored local artifact다.

## 5. 허용된 변동 metrics

| 변동 항목 | r2 | r3 | 차이 |
| --- | ---: | ---: | ---: |
| worker elapsed 합 | 1,712,782 ms | 1,662,605 ms | -50,177 ms, -2.9296% |
| peak worker RSS | 595,988,480 bytes | 595,996,672 bytes | +8,192 bytes |
| journal bytes | 534,811,486 | 534,811,432 | -54 bytes |

elapsed와 RSS는 OS scheduling·sampling에 따라 변하는 운영 metric이므로 aggregate hash와 semantic
결정성에서 의도적으로 제외돼 있다. journal은 이 metrics를 문서별로 기록하므로 54 bytes 차이가 났지만,
모든 semantic record token과 chain은 exact다. 이 차이는 결과 drift가 아니다.

## 6. privacy와 보존 경계

2차 evidence도 1차와 같은 local-only 경계에 보존한다.

```text
output/poc/font-metric-coverage/checkpoint-stage4-c-10k-r3/
output/poc/font-metric-coverage/final-stage4-c-10k-r3.json
```

보고서에는 aggregate와 실행 identity hash만 기록했다. corpus path·filename·개별 BLAKE3·개별 document
aggregate hash·원문은 포함하지 않았다. r2/r3 journal과 final JSON은 Issue·PR·CI artifact로 게시하지
않는다.

## 7. W3 종료 판정과 다음 승인 후보

W3 종료 조건은 충족됐다.

- 기존 POC와 새 decision delta의 책임 분리
- 10k 분모·failure·coverage·join 합 exact
- 모든 실패 이유 설명, parser·truncation 은폐 없음
- 1·2차 문서별 semantic token과 final bytes exact
- privacy·resource·checkpoint 보호 불변식 통과
- metric DB·fallback·font asset·제품 renderer 변경 없음

다음 후보는 같은 #4962의 **W4 조판 위험 순위**다. W4는 새 10k 전수 계측을 먼저 반복하지 않고 보존된
W3 decision aggregate와 기존 POC의 장평·자간·고정 frame·stored LineSeg 축을 결합할 수 있는지
필드·분모 compatibility부터 감사해야 한다. 별도 계획·승인 전에는 ranking 구현, fresh/backend cohort,
원격 push와 PR을 진행하지 않는다.
