# PR #8522 리뷰 — fix(composer): 줄 없는 TAC 표 host 줄을 그리는 행 높이로, 누름틀 뒤 넘치는 표는 다음 줄로 (#8507, #8473 위 적층)

## 최종 판정

**메인터너 보정 후 수용 가능**
메인터너가 통합 후보의 실제 전쪽 증적과 회귀 근거를 보강했습니다. 원 PR에 대응하는 입력·쪽과 반례는 아래에 기록했습니다.
원 head와 리베이스·메인터너 보정을 적용한 통합 후보를 구분합니다. 실제 적용 커밋은 아래 표에 기록했습니다.
완료된 검증과 남은 공통 검증을 아래에서 구분합니다. 최종 수용 판정과 PR 생성은 아직 대기입니다.
아래 값은 명시한 코드 SHA의 실측 결과이며 원 PR의 승인이나 닫기를 뜻하지 않습니다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8522](https://github.com/edwardkim/rhwp/pull/8522) · `planet6897` → `devel` |
| 원 head | `98e69aed1707043a1132bee59860126847329f49` |
| 통합 적용 커밋 | `930476e667f4db102dd20ad1c5c97d29ce5e3106, beaff24c1788552570af99834a3e6abbde1580c0, b94ae3190c5f73c9ae886e9d3e523a3759eece98` |
| 최종 검토 기준 devel | `22947f6a90c998e95fa51238d5b907a0ebd37162` |
| 최종 필수 검증 제품 코드 | `a7e5ce7112ea8b227c317a9cbfb9d6fed0b00973` |
| 전쪽 시각 측정 producer / 당시 base | `4b149b6cdfc514b9858ed644be2cf8120c731cd4` / `7fdd5441ff1e77c083818e920a48089f294de35e` |
| reviewer | `jangster77` 요청. 2026-11-04T17:44:08.556Z 원 head 불변·Open·non-Draft·요청 상태 재확인 |
| 기본 경로 | `maintainer_general` |
| 보조 경로 | `intake_and_review`, `local_validation`, `visual_fixture_evidence`, `multi_pr_update_branch`, `post_merge` |
| 관련 이슈 | [#8507](https://github.com/edwardkim/rhwp/issues/8507) |

## 변경과 검토 범위

원 PR의 변경 주장은 **fix(composer): 줄 없는 TAC 표 host 줄을 그리는 행 높이로, 누름틀 뒤 넘치는 표는 다음 줄로 (#8507, #8473 위 적층)**입니다. 해당 주장에 연결된 입력과 쪽을 아래에 분리해 적었습니다.
적층 PR에 중복 포함된 선행 커밋과 이 PR의 실제 신규 커밋은 최종 통합 기록에서 구분합니다.

## 검증 입력과 전쪽 결과

| 입력 · 독립 Print PDF | Native 전쪽 TSV · 최저 raw | fresh WASM 전쪽 TSV · 최저 raw |
| --- | --- | --- |
| [field_fit.hwpx](../../samples/issue8507/field_fit.hwpx) · [field_fit-2020.pdf](../../pdf/issue8507/field_fit-2020.pdf) | [TSV](assets/planet6897_20261103/pr_8522/pr8522-field_fit-cdaa8e18-native.tsv) · 100.0% (1쪽) | [TSV](assets/planet6897_20261103/pr_8522/pr8522-field_fit-cdaa8e18-wasm.tsv) · 100.0% (1쪽) |
| [field_fit_rb.hwpx](../../samples/issue8507/field_fit_rb.hwpx) · [field_fit_rb-2020.pdf](../../pdf/issue8507/field_fit_rb-2020.pdf) | [TSV](assets/planet6897_20261103/pr_8522/pr8522-field_fit_rb-07cac652-native.tsv) · 100.0% (1쪽) | [TSV](assets/planet6897_20261103/pr_8522/pr8522-field_fit_rb-07cac652-wasm.tsv) · 100.0% (1쪽) |
| [field_over1414.hwpx](../../samples/issue8507/field_over1414.hwpx) · [field_over1414-2020.pdf](../../pdf/issue8507/field_over1414-2020.pdf) | [TSV](assets/planet6897_20261103/pr_8522/pr8522-field_over1414-afc47b9f-native.tsv) · 100.0% (1쪽) | [TSV](assets/planet6897_20261103/pr_8522/pr8522-field_over1414-afc47b9f-wasm.tsv) · 100.0% (1쪽) |
| [field_over1414.hwpx](../../samples/issue8507/field_over1414.hwpx) · [field_over1414-2024.pdf](../../pdf/issue8507/field_over1414-2024.pdf) | [TSV](assets/planet6897_20261103/pr_8522/pr8522-field_over1414-c331d27c-native.tsv) · 100.0% (1쪽) | [TSV](assets/planet6897_20261103/pr_8522/pr8522-field_over1414-c331d27c-wasm.tsv) · 100.0% (1쪽) |
| [field_over19.hwpx](../../samples/issue8507/field_over19.hwpx) · [field_over19-2020.pdf](../../pdf/issue8507/field_over19-2020.pdf) | [TSV](assets/planet6897_20261103/pr_8522/pr8522-field_over19-dbf6a6ab-native.tsv) · 100.0% (1쪽) | [TSV](assets/planet6897_20261103/pr_8522/pr8522-field_over19-dbf6a6ab-wasm.tsv) · 100.0% (1쪽) |
| [field_over19.hwpx](../../samples/issue8507/field_over19.hwpx) · [field_over19-2024.pdf](../../pdf/issue8507/field_over19-2024.pdf) | [TSV](assets/planet6897_20261103/pr_8522/pr8522-field_over19-27f08a5e-native.tsv) · 100.0% (1쪽) | [TSV](assets/planet6897_20261103/pr_8522/pr8522-field_over19-27f08a5e-wasm.tsv) · 100.0% (1쪽) |
| [field_over19_rb.hwpx](../../samples/issue8507/field_over19_rb.hwpx) · [field_over19_rb-2020.pdf](../../pdf/issue8507/field_over19_rb-2020.pdf) | [TSV](assets/planet6897_20261103/pr_8522/pr8522-field_over19_rb-8650f44f-native.tsv) · 100.0% (1쪽) | [TSV](assets/planet6897_20261103/pr_8522/pr8522-field_over19_rb-8650f44f-wasm.tsv) · 100.0% (1쪽) |
| [field_over19-saved-2020.hwp](../../samples/issue8507/field_over19-saved-2020.hwp) · [field_over19-2020.pdf](../../pdf/issue8507/field_over19-2020.pdf) | [TSV](assets/planet6897_20261103/pr_8522/pr8522-field_over19-saved-2020-dbf6a6ab-native.tsv) · 100.0% (1쪽) | [TSV](assets/planet6897_20261103/pr_8522/pr8522-field_over19-saved-2020-dbf6a6ab-wasm.tsv) · 100.0% (1쪽) |
| [tac_adjacent-saved.hwpx](../../samples/issue8507/tac_adjacent-saved.hwpx) · [tac_adjacent-saved-2020.pdf](../../pdf/issue8507/tac_adjacent-saved-2020.pdf) | [TSV](assets/planet6897_20261103/pr_8522/pr8522-tac_adjacent-saved-aa2055e5-native.tsv) · 100.0% (1쪽) | [TSV](assets/planet6897_20261103/pr_8522/pr8522-tac_adjacent-saved-aa2055e5-wasm.tsv) · 100.0% (1쪽) |
| [tac_adjacent_generator.hwpx](../../samples/issue8507/tac_adjacent_generator.hwpx) · [tac_adjacent_generator-2020.pdf](../../pdf/issue8507/tac_adjacent_generator-2020.pdf) | [TSV](assets/planet6897_20261103/pr_8522/pr8522-tac_adjacent_generator-271b8297-native.tsv) · 100.0% (1쪽) | [TSV](assets/planet6897_20261103/pr_8522/pr8522-tac_adjacent_generator-271b8297-wasm.tsv) · 100.0% (1쪽) |

96 DPI, 색상 경계 보정 전 2px 실루엣입니다. 검증 입력과 Print PDF는 같은 행에서 대응합니다.
전쪽 TSV·PNG는 위 실제 producer에서 산출한 증거입니다. 최신 head의 재출력으로 표시하지 않습니다.
충돌 없는 리베이스 뒤 영향 소비 경로를 새 제품으로 집중 검증 중이며, 완료한 실행과 남은 대조를 구분합니다.
원본·PDF SHA와 Print 절차, 모든 쪽의 누락·쪽수 대조는 통합 증거 목록에서 확정합니다.

## 조판 원칙 준수 검토

적용 대상입니다. 원본 저장값·독립 Print를 근거로 공통 규칙, 측정과 실제 배치의 소비 지점, 내용 소속·순서 및 적용되지 않는 반례를 아래에서 대조합니다.

## 구현 주장과 검증 증거의 대조

저장 줄 없는 TAC 표 host는 실제 그리는 행 높이로 줄을 구성하고, 누름틀 뒤 가용 너비를 넘는 표는 다음 줄에 배치한다.

composer의 공통 줄 구성 결과에서 개체 순서·행 높이·가용 너비를 만들고 측정과 TAC 표 실제 배치가 이를 소비한다. treat_as_char만으로 모든 개체를 한 줄로 묶지 않는다.

field_fit·field_over·rowbreak·인접 표 변형에서 줄 수용·이월과 뒤 내용 순서를 확인한다. 들어가는 경우와 넘는 경우를 같은 원본 단위로 구분한다.

관련 정식 회귀 파일: [issue_8507_tac_declared_above_rows.rs](../../tests/cases/issue_8507_tac_declared_above_rows.rs)

## 검증 입력 커밋 확인

이번 통합의 [입력·PDF SHA-256 대조표](assets/planet6897_20261103/input-integrity.tsv)에 있는
148개 파일은 증적 고정 커밋 `fd97ae040f`의 파일 내용과 일치합니다.
위 개별 입력은 저장소의 동일 파일을 재사용합니다.

## 원 PR 주장 쪽의 시각 증적

| 입력 · 쪽 · 검토 대상 | Native raw · review / overlay | fresh WASM raw · review / overlay |
| --- | --- | --- |
| `field_fit.hwpx` 1쪽 · 필드 뒤 표 폭이 맞는 정상 대조군 | 100.00% · [review](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-field_fit-native-review.png) / [overlay](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-field_fit-native-overlay.png) | 100.00% · [review](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-field_fit-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-field_fit-wasm-overlay.png) |
| `field_fit_rb.hwpx` 1쪽 · RowBreak 필드 뒤 표 폭이 맞는 대조군 | 100.00% · [review](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-field_fit_rb-native-review.png) / [overlay](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-field_fit_rb-native-overlay.png) | 100.00% · [review](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-field_fit_rb-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-field_fit_rb-wasm-overlay.png) |
| `field_over1414.hwpx` 1쪽 · 1414HU 폭 초과 표의 필드 뒤 줄바꿈 | 100.00% · [review](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-field_over1414-0914b4-native-review.png) / [overlay](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-field_over1414-0914b4-native-overlay.png) | 100.00% · [review](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-field_over1414-0914b4-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-field_over1414-0914b4-wasm-overlay.png) |
| `field_over1414.hwpx` 1쪽 · 2024 Print의 폭 초과 표 줄바꿈 | 100.00% · [review](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-field_over1414-321313-native-review.png) / [overlay](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-field_over1414-321313-native-overlay.png) | 100.00% · [review](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-field_over1414-321313-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-field_over1414-321313-wasm-overlay.png) |
| `field_over19.hwpx` 1쪽 · 19HU 폭 초과 표의 필드 줄 점유 | 100.00% · [review](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-field_over19-033784-native-review.png) / [overlay](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-field_over19-033784-native-overlay.png) | 100.00% · [review](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-field_over19-033784-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-field_over19-033784-wasm-overlay.png) |
| `field_over19.hwpx` 1쪽 · 누름틀 뒤 표 넘침 | 100.00% · [review](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-field_over19-88499a-native-review.png) / [overlay](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-field_over19-88499a-native-overlay.png) | 100.00% · [review](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-field_over19-88499a-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-field_over19-88499a-wasm-overlay.png) |
| `field_over19_rb.hwpx` 1쪽 · RowBreak 폭 초과 표의 자기 줄 소속 | 100.00% · [review](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-field_over19_rb-native-review.png) / [overlay](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-field_over19_rb-native-overlay.png) | 100.00% · [review](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-field_over19_rb-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-field_over19_rb-wasm-overlay.png) |
| `field_over19-saved-2020.hwp` 1쪽 · 한컴 저장본의 빈 필드 줄과 표 줄 보존 | 100.00% · [review](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-field_over19-saved-native-review.png) / [overlay](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-field_over19-saved-native-overlay.png) | 100.00% · [review](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-field_over19-saved-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-field_over19-saved-wasm-overlay.png) |
| `tac_adjacent-saved.hwpx` 1쪽 · 저장 줄이 있는 TAC 표의 행 높이 대조군 | 100.00% · [review](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-tac_adjacent-saved-native-review.png) / [overlay](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-tac_adjacent-saved-native-overlay.png) | 100.00% · [review](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-tac_adjacent-saved-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-tac_adjacent-saved-wasm-overlay.png) |
| `tac_adjacent_generator.hwpx` 1쪽 · 생성 TAC 표의 선언 높이와 실제 행 점유 | 100.00% · [review](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-tac_adjacent_gener-native-review.png) / [overlay](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-tac_adjacent_gener-native-overlay.png) | 100.00% · [review](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-tac_adjacent_gener-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8522/pr8522-claim-p001-tac_adjacent_gener-wasm-overlay.png) |

## 통합 최종 검증 조건

- 제품 `a7e5ce7112ea8b227c317a9cbfb9d6fed0b00973`의 전체 Rust 12,561개 통과와 Native Skia·CI 통합 모듈 21개·Rust lint를 확인했습니다.
- 최적화 fresh WASM의 실제 브라우저 실행과 공개 API 검사를 완료했습니다.
- 전쪽 측정 producer에서 Native/fresh WASM 각 80개 입력·406쪽은 모두 90% 이상입니다.
- Native의 당시 base 대비 406쪽 대조와 PR #8389·#8397의 원 TSV 표시값 무하락 대조는 완료했습니다.
- WASM의 당시 base 대비 전쪽 대조와 최신 base 기능 보존의 남은 실행은 아래와 같습니다.

- 대기: 동일 환경에서 당시 base 대비 WASM 80개 입력·406쪽의 0.1%p 하락 게이트 완료
- 새 base의 실제 Native/WASM 제품과 후보의 집중 대조 14쪽은 모두 같은 raw입니다. [실행·대조 원본](assets/planet6897_20261103/rebase_scope_22947f6a90/README.md)을 확인했습니다.
- 대기: 주장 쪽 직접 판독·회귀 주석·수동 작성 리뷰와 PR 본문 최종 감사

이 대기 항목을 완료하지 않은 상태에서는 최종 검증 완료·승인으로 판정하지 않습니다.
[완료·대기 검증 원본 기록](assets/planet6897_20261103/intermediate-evidence-summary.md)

## Merge 후 원 PR 후속 처리

통합 PR 생성·병합과 원 PR 댓글·닫기는 아직 수행하지 않았습니다.
실제 병합 뒤 원 PR의 주장 문서·페이지에 대응하는 증거와 merge SHA를 연결합니다.

<!-- planet6897-20261103-generated-review -->
