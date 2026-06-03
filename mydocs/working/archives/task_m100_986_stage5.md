# Stage 5 완료 보고서 — Task #986

## 범위

이슈 #986 제보 파일을 저장소 fixture 로 추가하고, 같은 문제가 다시 발생하지 않도록
통합 회귀 테스트를 추가했다.

## 변경 파일

- `samples/issue-986-receipt.hwp`
  - 이슈 #986 제보 첨부 `receipt.hwp` fixture 추가
- `tests/issue_986.rs`
  - page count 가 다시 3페이지로 늘어나는 회귀 방지
  - `ci=2..8` 표가 page 1 에만 렌더되는지 검증
  - `ci=4`, `ci=6` 오른쪽 표가 `ci=2` 왼쪽 큰 표 아래로 밀리지 않고 독립 lane 에서
    시작하는지 render tree bbox 로 검증

## 테스트 설계

현재 구현은 trailing 빈 문단 `pi=2` 때문에 최종 page count 가 2이다. 아직 Stage 6
전이므로 `page_count == 1` 을 단언하지 않고, 이번 단계에서는 본 이슈의 핵심 회귀인
3페이지 분할과 오른쪽 표 밀림을 막는 조건만 고정했다.

단언:

- `page_count <= 2`
- `pi=0 ci=2..8` 의 Table node 가 모두 page 1 에 존재
- `pi=0 ci=2..8` 이 page 2 이후에 렌더되지 않음
- `ci=4`, `ci=6` 의 x lane 이 각각 `ci=2`, `ci=4` 오른쪽에 있음
- `ci=4`, `ci=6` 의 y top 이 `ci=2` 와 같은 시작선 근처에 있음

## 검증

```bash
cargo fmt --all --check
TMPDIR=/private/tmp/rhwp-issue-986-rusttmp cargo test --test issue_986 -- --nocapture
TMPDIR=/private/tmp/rhwp-issue-986-rusttmp cargo test --test issue_712
TMPDIR=/private/tmp/rhwp-issue-986-rusttmp cargo test --test issue_713
TMPDIR=/private/tmp/rhwp-issue-986-rusttmp cargo test --test issue_775
```

결과:

- `cargo fmt --all --check`: 통과
- `issue_986`: 2개 테스트 통과
- `issue_712`: 1개 테스트 통과
- `issue_713`: 1개 테스트 통과
- `issue_775`: 1개 테스트 통과

`issue_986` bbox 출력:

```text
ci2=[x 37.8..707.4, y 18.9..631.0]
ci4=[x 719.1..886.9, y 18.9..297.1]
ci6=[x 898.1..1084.7, y 18.9..342.7]
```

## 판단

Stage 5 기준 회귀 테스트 추가는 완료됐다. 이 테스트는 현재 해결된 핵심 결함
(`PartialTable` 로 인한 3페이지 분할, 오른쪽 표의 세로 밀림)을 고정하면서도,
후속 Stage 에서 trailing 빈 문단을 제거해 1페이지로 줄어드는 개선을 막지 않는다.

## 남은 문제

page 2 의 빈 문단 `pi=2` 가 아직 남아 있다. Stage 6 에서 fallback 경로와 기존 회귀
세트를 확인하면서, trailing 빈 문단 처리까지 마무리할지 범위를 확정해야 한다.

## 다음 단계

Stage 6 에서 `RHWP_USE_PAGINATOR=1` fallback 결과와 기존 회귀 테스트 세트를 확인한다.
필요하면 trailing 빈 문단 흡수 보정을 별도 소단계로 진행한다.
