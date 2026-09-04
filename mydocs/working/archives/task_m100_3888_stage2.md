# Task M100 #3888 2단계 — slow shard 단수 Summary 파서 보정

- 이슈: [#3888](https://github.com/edwardkim/rhwp/issues/3888)
- 대상 PR: [#3892](https://github.com/edwardkim/rhwp/pull/3892)
- 실패 run: `30813876861`, job `91689598765`

## 관측

slow worker의 exact filterset은 의도대로 `rhwp::overflow_cell_baseline` binary의
`overflow_cell_lines_do_not_grow`만 선택했다. nextest는 1건을 244.761초에 통과시켰고 Summary는
`1 test run`이었다.

하지만 shard count parser는 `[0-9]+ tests run`만 수용했다. 단수 Summary에서 `run` 값이 비어
GitHub Actions의 `bash -e`가 종료 코드 1을 반환했으며 count artifact도 업로드되지 않았다.

## 보정

정규식을 `[0-9]+ tests? run`으로 바꿨다. 일반 shard의 복수형과 slow shard의 단수형을 모두
수용하며, 빈 값 검증과 8개 artifact 합계 검증은 유지한다.

## 검증 결과

| 검증 | 결과 |
| --- | --- |
| `1 test run` Summary 파싱 | 통과: `1` 추출 |
| `650 tests run` Summary 파싱 | 통과: `650` 추출 |
| `actionlint .github/workflows/ci.yml` | 통과 (v1.7.12) |
| worker run shell 추출 후 `bash -n` | 통과 |
| matrix/filterset 정적 계약 | 통과: slow 1개, regular 7개, artifact 8개, exact filterset과 여집합 |
| `git diff --check` | 통과 |

보정 head의 GitHub Actions에서 slow count artifact와 `Build & Test` 집계를 재확인한다.
