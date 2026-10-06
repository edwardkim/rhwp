# Task M100 #3888 1단계 — slow shard 분리 구현·정적 검증

- 이슈: [#3888](https://github.com/edwardkim/rhwp/issues/3888)
- 브랜치: `task/3888-ci-slow-shard`
- 기준: `upstream/devel` `85d334728`

## 구현

`.github/workflows/ci.yml`의 `test-shard`는 별도 job으로 쪼개지 않고 하나의 matrix를 유지한다.
matrix는 `slow shard`와 `shard 1/7`~`shard 7/7`로 구성되어 총 GitHub Actions runner 수가 기존과
같은 8개다. 이 구조는 `strategy.fail-fast: true`가 slow worker까지 포함한 여덟 worker에 적용되게
한다.

- slow worker filterset:
  `binary_id(=rhwp::overflow_cell_baseline) & test(=overflow_cell_lines_do_not_grow)`
- 일반 worker filterset: 위 filterset의 여집합을 `hash:m/7`로 분배
- count artifact: `shard-count-slow`, `shard-count-1`~`shard-count-7`

`Build & Test` 집계는 기존과 같이 8개 artifact의 실행 건수 합계와 archive 단계의 runnable 수를
대조한다. 따라서 filterset 누락 또는 중복은 성공으로 위장할 수 없다.

## 검증

| 검증 | 결과 |
| --- | --- |
| `actionlint .github/workflows/ci.yml` | 통과 (v1.7.12, 공식 release checksum 확인) |
| worker run shell 추출 후 `bash -n` | 통과 |
| matrix 정적 검사 | slow 1개 + 일반 7개 + count label 8개 확인 |
| filterset 정적 검사 | exact slow filterset 1개와 일반 worker의 여집합 선언 확인 |
| `git diff --check` | 통과 |

이번 변경은 CI workflow와 운영 문서만 수정한다. 제품 Rust·TypeScript·fixture를 변경하지 않았으므로
제품 Cargo 테스트를 다시 실행하지 않았다. 최종 검증은 PR 최신 head의 GitHub Actions에서 archive 기반
nextest 실행 건수, 8개 artifact 합계, 일반 shard 재분배 시간과 slow shard 시간을 확인하는 것이다.
