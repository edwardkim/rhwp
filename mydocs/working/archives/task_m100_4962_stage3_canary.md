# Task M100 #4962 W3 Stage 3-P0 — private pilot canary

- **Issue**: [#4962](https://github.com/edwardkim/rhwp/issues/4962)
- **상위 추적**: [#4960](https://github.com/edwardkim/rhwp/issues/4960)
- **계획**: [`task_m100_4962.md`](../../plans/archives/task_m100_4962.md)
- **선정 기준**: [`task_m100_4962_stage3_pilot_selection.md`](task_m100_4962_stage3_pilot_selection.md)
- **실행 HEAD**: `468349617d0eb29b459260ea3912e1ae26667245`
- **날짜**: 2026-08-21 KST
- **단계 상태**: Stage 3 canary 완료, full pilot 미착수

## 1. 승인 범위와 결론

승인된 범위대로 local-only manifest 32건 전체를 preflight하고 그중 canary 8건만 W3 worker로
실행했다. canary는 HWP 4건과 HWPX 4건 모두 성공했으며 failure·resource-limit·truncation·excluded는
0이었다. 최대 관찰 RSS는 595,714,048 bytes로 1.5 GiB canary gate 아래였다.

canary gate는 통과했다. 그러나 고위험 4건씩의 실행시간 분산이 커서 이 결과만으로 10k 전수 시간을
확정하거나 corpus 전체 coverage 비율을 주장하지 않는다. full 32건과 반복 실행은 수행하지 않았다.

## 2. 정확한 HEAD와 preflight

작업공간이 clean이고 HEAD가 `468349617d0eb29b459260ea3912e1ae26667245`임을 확인한 뒤 다음 binary를
같은 HEAD와 추적 중인 lockfile에서 빌드했다.

```bash
cargo fmt --all -- --check
cargo build --locked --bin rhwp-agent --example font_metric_coverage_worker
```

32건 전체에 대해 식별 값을 출력하지 않고 다음을 검사했다.

| preflight 항목 | 결과 |
| --- | --- |
| 승인된 corpus root confinement | 32/32 |
| symlink가 아닌 regular file | 32/32 |
| manifest format·파일 크기 일치 | 32/32 |
| 현재 bytes의 BLAKE3 일치 | 32/32 |
| HWP / HWPX | 16 / 16 |
| 입력 bytes 합 | 554,287,921 |

preflight 결과는 권한 `0600`의 gitignored
`output/poc/font-metric-coverage/preflight-stage3-p1.json`에 두었다. 원본 path와 개별 hash는
보고서·stdout·tracked file에 옮기지 않았다.

## 3. 격리 실행

manifest의 `tier=canary` 8건을 선정 순서대로 각각 별도 worker process에서 순차 실행했다. Stage 3-I의
기본 한도인 문서별 2 GiB address space, 75초 CPU, 90초 wall timeout과 128 MiB stdout budget을 그대로
사용했다.

| 결과 | 값 |
| --- | ---: |
| attempted / success | 8 / 8 |
| HWP / HWPX | 4 / 4 |
| 전체 failure | 0 |
| resource-limit | 0 |
| worker elapsed 합 | 71,550 ms |
| peak worker RSS | 595,714,048 bytes |

포맷별 관찰 시간은 다음과 같다. 문서 4건의 짝수 표본에서 결과 봉투의 `median`은 정렬 후 아래쪽
중앙값을 사용한다.

| 형식 | 최소 | lower median | 최대 |
| --- | ---: | ---: | ---: |
| HWP | 3,200 ms | 4,978 ms | 35,192 ms |
| HWPX | 289 ms | 642 ms | 4,810 ms |

HWP의 최대/최소 차이가 약 11배이고 HWPX도 약 17배다. `6,582 × HWP lower median + 3,418 × HWPX
lower median`의 provisional 계산은 약 9.71시간이지만, 포맷별 4건·고위험 편향·lower median이라는
조건 때문에 전수 계획값으로 승인할 수 없다. full 32건의 분포를 먼저 측정해야 한다.

## 4. W3 분모와 category

| 분모 | 문자 수 |
| --- | ---: |
| layout | 2,108,443 |
| coverage | 2,105,856 |
| not applicable | 2,587 |
| excluded | 0 |
| truncated | 0 |

`layout = coverage + not applicable + excluded`가 일치한다. coverage의 일곱 상호배타 category도 다음과
같이 합계가 2,105,856자로 일치한다.

| category | 문자 수 |
| --- | ---: |
| measured-overlay | 3,755 |
| identity-alias-hit | 0 |
| metric-surrogate | 0 |
| exact-hit | 1,914,907 |
| char-miss | 12,340 |
| face-miss | 174,479 |
| heuristic | 375 |

이 비율은 대형·압축·kerning을 의도적으로 과표집한 canary의 관찰값이다. 특히 face-miss 174,479자를
10k corpus의 발생률로 외삽하거나 font backlog 순위로 바로 사용하지 않는다.

## 5. gate와 privacy 판정

| gate | 기준 | 결과 |
| --- | ---: | --- |
| 전체 failure | 2 이하 | 0, 통과 |
| resource-limit | 1 이하 | 0, 통과 |
| peak RSS | 1.5 GiB 이하 | 595,714,048 bytes, 통과 |
| supervisor fatal error | 없어야 함 | 없음 |
| 민감 출력 | 없어야 함 | 통과 |

canary 비식별 결과는 권한 `0600`의 gitignored
`output/poc/font-metric-coverage/canary-stage3-p1-r1.json`에 두었다. 결과는 source path, filename,
개별 BLAKE3, raw character·trace·record를 포함하지 않는다. 정확한 금지 key와 Linux·macOS·Windows
home-path 패턴을 재귀 검사했고 분모·category·failure·hash를 독립 재계산했다.

결정 순서의 8개 문서 aggregate hash를 SHA-256으로 결합한 canary digest는 다음과 같다. 이는 개별 문서
hash가 아니라 같은 cohort·순서·실행 결과를 다음 실행과 비교하기 위한 local aggregate 증거다.

```text
9bfd4f86c2d5a0ff7237b09152a82e5f72b9228abd404c1063bdd94890037df6
```

## 6. 다음 승인 게이트

다음 절편은 동일 manifest의 full 32건을 **한 번만** 실행한다. canary 8건은 full cohort에 포함되므로
다시 실행되며, 포맷별 elapsed/RSS 분포와 전체 W3 aggregate를 비식별 집계한다.

1차 full 결과에서 실제 비용·failure·resource-limit·privacy를 보고한 뒤에만 동일 32건의 2차 실행을
승인 요청한다. 2차 결과가 1차의 count·failure vector·combined decision hash와 일치해야 Stage 3
결정성 gate가 닫힌다. 현재 승인으로는 full 1차·2차, 10k 전수, 원격 push와 PR을 수행하지 않았다.
