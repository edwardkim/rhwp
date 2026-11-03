# PR #8505 리뷰 — fix(table): #8501 셀 단위 나눔 표의 쪽 끝 행은 셀 저장 재시작 자리에서 나눈다

## 최종 판정

**머지 보류 — 통합 최종 head의 Native/fresh WASM 전쪽 측정과 기준 devel 대비 대조, 전체 로컬 검증이 진행 중입니다.**
아래 값은 명시한 코드 SHA의 실측 결과이며 원 PR의 승인이나 닫기를 뜻하지 않습니다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8505](https://github.com/edwardkim/rhwp/pull/8505) · `planet6897` → `devel` |
| 원 head | `4b3426fcd6f91c16a4d03e1ab5b293ebe1956948` |
| 통합 적용 커밋 | `5a091f479c46,ed9b6f848968,624aeaed3d21,552f40d3c31a,eb4c7c50bd8e` |
| 검토 기준 devel | `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4` |
| 실측 제품 코드 | `dc87f4fa2e` (이후 회귀·문서 변경과 구분) |
| reviewer | `jangster77` 요청. 2026-11-04 원 head 불변·Open·non-Draft·요청 상태 재확인 |
| 기본 경로 | `maintainer_general` |
| 보조 경로 | `intake_and_review`, `local_validation`, `visual_fixture_evidence`, `multi_pr_update_branch`, `post_merge` |
| 관련 이슈 | [#8501](https://github.com/edwardkim/rhwp/issues/8501) |

## 변경과 검토 범위

원 PR의 변경 주장은 **fix(table): #8501 셀 단위 나눔 표의 쪽 끝 행은 셀 저장 재시작 자리에서 나눈다**입니다. 해당 주장에 연결된 입력과 쪽을 아래에 분리해 적었습니다.
적층 PR에 중복 포함된 선행 커밋과 이 PR의 실제 신규 커밋은 최종 통합 기록에서 구분합니다.

## 검증 입력과 전쪽 결과

| 입력 · 독립 Print PDF | Native 전쪽 TSV · 최저 raw | fresh WASM 전쪽 TSV · 최저 raw |
| --- | --- | --- |
| [child_product_safety_rule.hwp](../../samples/issue8501/child_product_safety_rule.hwp) · [issue8501_child_product_safety_rule-2020.pdf](../../pdf/issue8501_child_product_safety_rule-2020.pdf) | [TSV](assets/planet6897_20261103/pr_8505/pr8505-child_product_safety_rule-5645416e-native.tsv) · 99.78% (19쪽) | 산출 중 · 미측정 (?쪽) |

96 DPI, 색상 경계 보정 전 2px 실루엣입니다. 검증 입력과 Print PDF는 같은 행에서 대응합니다.
원본·PDF SHA와 Print 절차, 모든 쪽의 누락·쪽수 대조는 통합 증거 목록에서 확정합니다.

## 구현 주장과 검증 증거의 대조

셀 저장 줄이 쪽 끝에서 vpos 0으로 다시 시작하면 그 소유 경계를 행 분할 컷에 반영한다.

typeset/table의 저장 재시작 경계·요구 높이·예산과 실제 이어받는 행 조각을 대조한다. 전체 행 내용이 근사 예산에 들어간다는 이유로 명시된 저장 컷을 버리지 않는다.

어린이제품 안전 규칙의 6·7쪽에서 행 16 꼬리와 행 17의 소속·순서를 확인한다. 진단 stderr를 검증 근거로 대신하지 않고 실제 배치와 전쪽 Print를 사용한다.

관련 정식 회귀 파일: [issue_8493_table_tail_rewind.rs](../../tests/cases/issue_8493_table_tail_rewind.rs) · [issue_8501_rowbreak_stored_restart.rs](../../tests/cases/issue_8501_rowbreak_stored_restart.rs)

## 검증 입력 커밋 확인

이번 통합의 [입력·PDF SHA-256 대조표](assets/planet6897_20261103/input-integrity.tsv)에 있는
148개 파일은 증적 고정 커밋 `fd97ae040f`의 파일 내용과 일치합니다.
위 개별 입력은 저장소의 동일 파일을 재사용합니다.

## 원 PR 주장 쪽의 시각 증적

| 입력 · 쪽 · 검토 대상 | Native raw · review / overlay | fresh WASM raw · review / overlay |
| --- | --- | --- |
| `child_product_safety_rule.hwp` 6쪽 · 셀 단위 나눔 저장 재시작 | 99.86% · [review](assets/planet6897_20261103/pr_8505/pr8505-claim-p006-child_product_safe-native-review.png) / [overlay](assets/planet6897_20261103/pr_8505/pr8505-claim-p006-child_product_safe-native-overlay.png) | 미측정 · 산출 중 / 산출 중 |

## 남은 검증

- 현재 통합 최종 head의 집중·전체 Rust 회귀, lint와 fresh WASM을 완료하고 로그를 연결합니다.
- 고정 `upstream/devel`과 같은 입력·Print·글꼴 조건에서 전쪽 raw 하락을 비교합니다.
- 위 주장 쪽 review와 overlay를 독립 Print와 확대 대조해 내용·순서·격자·간격을 직접 판독합니다.
- 누락 증적이나 90% 미만 쪽이 있으면 보정 후 다시 측정합니다.

## Merge 후 원 PR 후속 처리

통합 PR의 merge SHA와 검증한 동일 입력·쪽의 TSV 및 review/overlay를 원 PR에 연결합니다.
댓글 게시와 원 PR 닫기는 최종 통합 검증 및 merge 후에 판정합니다.

<!-- planet6897-20261103-generated-review -->
