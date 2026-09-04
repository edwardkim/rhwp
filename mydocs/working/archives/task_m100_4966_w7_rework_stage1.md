---
kind: report
status: active
canonical: mydocs/plans/archives/task_m100_4966.md
last_verified: 2026-08-23
---

# Task M100 #4966 — Stage W7-R1 PR CI 실패 원인 귀속

## 1. 판정

PR #5950의 실패는 font runtime 결과 회귀로 확인되지 않았다. 현재까지 확인된 직접 원인은 두 가지다.

1. W7에서 수기 mapping 4개와 새 helper 2개를 `#[cfg(test)]` support로 만들었지만 PR base를 지정한
   unit-tier 검사를 사전에 실행하지 않았다.
2. W1 역사 계측을 현재 source에 계속 적용하는 구조가 W6/W7 소유권 이전 뒤에도 남았고, W7 검증 묶음은
   W3 coverage 계약을 포함하지 않았다.

따라서 수기 함수를 runtime scope로 되돌리거나 unit-tier 기준선을 높이지 않는다. 역사 기준선과 현재
canonical authority를 분리하고, 제품 source의 신규 `#[cfg(test)]` support를 제거한다. 회귀 원본은
`CONTRIBUTING.md`가 정한 `tests/cases/`에 두고 generated suite·manifest를 제출하지 않는 W7-R2~R4를
수행한다.

## 2. 증거 동결

- PR: #5950, head `4a7c0f431a5323d90f1ad417d64fca056ce4b4ee`
- base: `upstream/devel@343ed2c013606319b6418dd8c637c5e04047e304`
- CI: Lint 실패, 나머지 Rust shard·Native Skia·CodeQL·Render Diff·Adapter·Proptest 성공
- unit-tier: `font_metrics_data.rs` 2개, `style_resolver.rs` 4개의 신규 support와 허용 기준선 추가를 거부
- 우회 source 변경: 커밋·push하지 않고 철회

## 3. 이전 계측 신뢰성 감사

W1 `sourceCommit`은 `795e7b5fac24cfef79017e9120516570851a03b2`다. 해당 Git object의 실제
source SHA-256은 snapshot과 일치했다.

| source | Git object SHA-256 | W1 snapshot | 판정 |
| --- | --- | --- | --- |
| `src/renderer/style_resolver.rs` | `b9f4dbe0c497f0d9b818e5a2e7ce32776546a50f364023879782d5c7944cc7f6` | 동일 | 입력 보존 |
| `src/renderer/font_metrics_data.rs` | `b829c75f50e801b09e4ae440b780bf24c33f144b79684bb7c8b8142e0f6aaea6` | 동일 | 입력 보존 |

W1에서 W7으로 이관된 Rust 후보 238개를 별도 JSON 대사로 확인했다.

| 검사 | 결과 |
| --- | ---: |
| W1 Rust 후보 / unique | 238 / 238 |
| canonical registry Rust rule | 238 |
| migration candidate link / unique | 238 / 238 |
| 누락·추가 candidate ID | 0 / 0 |
| boundary·decision plane·source·target·condition·order 불일치 | 0 |
| migration manifest와 registry rule 누락·추가 | 0 / 0 |

따라서 이전 W1 계측 입력과 238개 유한 mapping의 이행 결과가 잘못됐다는 증거는 없다.

## 4. 발견된 계측 회귀

`scripts/tests/font_metric_coverage_contract.test.mjs` 10건 중 다음 계약 1건이 실패했다.

```text
current W1 source keeps all candidate meanings while reporting digest-only drift
```

1,352개 후보의 ID와 수는 같지만 `rust-metric.metric-table` 600개가 `changedCandidateIds`로 보고됐다.
실제 차이는 metric identity·값·순서가 아니라 다음 source selector 이동이다.

```text
W1: static FONT_METRICS: [FontMetric;
W6: static FONT_METRICS: FontMetrics
```

현 `semanticCandidate`가 source digest만 제외하고 path·symbol·selector는 의미 비교에 포함했기 때문에,
승인된 W6 ownership migration을 600개 metric 의미 회귀로 오판했다. 이 실패는 W7의 W1·W2·W6·W7
77개 목록에 W3 계약이 없어서 최종 검증에서 누락됐다.

## 5. 원인 귀속

| 축 | 판정 | 근거 |
| --- | --- | --- |
| font runtime 의미 회귀 | 현재 증거 없음 | Rust 후보 238개 tuple·migration link 불일치 0, 기존 runtime/renderer gate 성공 |
| unit-tier 오류 | 정상 탐지 | PR base에 없던 `#[cfg(test)]` support 6개가 실제 존재 |
| W3 계측 | 계측 회귀 | selector ownership 이동만 600개 의미 변경으로 분류 |
| W7 검증 범위 | 누락 | W3 coverage contract가 77개 목록에 없음 |
| W7 소유권 이전 | 불완전 | 역사 selector와 현재 registry authority의 수명주기가 분리되지 않음 |

## 6. W7-R2 인계

다음 절편은 구현 전에 다음 보호 불변식을 고정한다.

1. W1 selector는 기록된 Git ref에서 검증하고 현재 source 존속 조건으로 사용하지 않는다.
2. 현재 runtime 규칙은 registry·migration manifest·generated projection에서 검증한다.
3. source owner 이동과 semantic tuple 변화는 별도 결과로 보고한다.
4. 미이관 selector 누락과 current authority 누락은 계속 실패한다.
5. 수기 mapping을 runtime scope로 되돌리거나 unit-tier baseline을 상향하지 않는다.
6. W1·W2·W3·W6·W7을 교차 검증 목록에서 다시 분리하지 않는다.
7. 신규 회귀 원본은 `tests/cases/`에만 두고 제품 source의 `#[cfg(test)]` support와 generated
   suite·manifest를 PR 변경에 남기지 않는다.
