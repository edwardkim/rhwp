# Stage 6 완료 보고서 — Task #986

## 범위

이슈 #986 구현 후 fallback pagination 경로와 기존 렌더링 회귀 세트를 검증했다.
검증 중 발견된 trailing empty paragraph 와 단일 table-only float 회귀를 함께 정리했다.

## 추가 보정

- `src/renderer/typeset.rs`
  - 마지막 빈 문단이 직전 trailing line spacing drift 로 이미 안전 영역을 약간 넘은 경우,
    직전 항목도 빈 문단일 때만 숨김 처리해 단독 빈 페이지를 만들지 않도록 보정했다.
  - `fit_fail_within_safety` 기존 동작은 보존해, 기존 문서의 visible trailing paragraph 가
    다음 페이지로 밀리는 회귀를 막았다.
  - 새 lane pagination 은 동일 host 문단 안에 TopAndBottom para-float 표가 2개 이상 있는
    경우에만 적용한다. issue #986 의 다중 표 lane 문제는 처리하면서, issue #157 같은
    단일 table-only float 문단의 기존 페이지 분배는 보존한다.
- `src/renderer/pagination/engine.rs`
  - fallback `RHWP_USE_PAGINATOR=1` 경로에도 마지막 빈 문단 prior-drift 흡수 조건을 맞췄다.
- `tests/issue_986.rs`
  - Stage 5 의 임시 `page_count <= 2` 단언을 최종 목표인 `page_count == 1` 로 강화했다.

## 확인 결과

`receipt.hwp` 기본 경로:

```text
문서 로드: /private/tmp/rhwp-issue-986/receipt.hwp (1페이지)
page 1 items=8
pi=0 ci=2..8 Table
pi=1 FullParagraph "(빈)"
```

`receipt.hwp` fallback 경로:

```text
RHWP_USE_PAGINATOR=1 ... dump-pages /private/tmp/rhwp-issue-986/receipt.hwp
문서 로드: /private/tmp/rhwp-issue-986/receipt.hwp (1페이지)
page 1 items=8
pi=0 ci=2..8 Table
pi=1 FullParagraph "(빈)"
```

issue #986 bbox:

```text
ci2=[x 37.8..707.4, y 18.9..631.0]
ci4=[x 719.1..886.9, y 18.9..297.1]
ci6=[x 898.1..1084.7, y 18.9..342.7]
```

오른쪽 표 `ci=4`, `ci=6` 이 왼쪽 큰 표 `ci=2` 아래로 밀리지 않고 같은 y top 에서
독립 lane 으로 배치된다.

## 회귀 확인

최초 Stage 6 검증 후 `upstream/devel` 이 `bc5683ff` 로 갱신되어, 작업 브랜치를
`git rebase --autostash upstream/devel` 로 다시 맞췄다. rebase 후 HEAD 는
`bc5683ff` 기반이며 브랜치명은 `issue-986-landscape-table-flow` 이다.

```bash
cargo fmt --all --check
TMPDIR=/private/tmp/rhwp-issue-986-rusttmp cargo test --test issue_986 -- --nocapture
TMPDIR=/private/tmp/rhwp-issue-986-rusttmp cargo test --test issue_676_trailing_empty_para
TMPDIR=/private/tmp/rhwp-issue-986-rusttmp cargo test --test issue_712 --test issue_713 --test issue_775
TMPDIR=/private/tmp/rhwp-issue-986-rusttmp cargo test --test svg_snapshot
TMPDIR=/private/tmp/rhwp-issue-986-rusttmp cargo test --lib renderer::float_placement
TMPDIR=/private/tmp/rhwp-issue-986-rusttmp cargo test --lib renderer::composer::tests
```

결과:

- `cargo fmt --all --check`: 통과
- `issue_986`: 2개 통과
- `issue_676_trailing_empty_para`: 3개 통과
- `issue_712`, `issue_713`, `issue_775`: 각 1개 통과
- `svg_snapshot`: 8개 통과
- `renderer::float_placement`: 6개 통과
- `renderer::composer::tests`: 36개 통과

rebase 후 재확인:

- `cargo fmt --all --check`: 통과
- `issue_986`: 2개 통과
- `svg_snapshot`: 8개 통과

## 판단

Stage 6 기준으로 기본 TypesetEngine 과 fallback Paginator 모두 제보 문서를 1페이지로
분배한다. issue #157 snapshot 회귀는 새 lane path 를 다중 para-float host 문단으로
제한해 해소했다. 남은 작업은 최종 diff 정리, 필요 시 전체 테스트 범위 확대, 커밋/PR
준비이다.
