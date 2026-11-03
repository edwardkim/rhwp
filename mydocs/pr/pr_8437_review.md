# PR #8437 리뷰 — fix(layout): TAC 표 다음 줄을 밀어낸 자리차지 로고 개체를 앵커 줄 자리에 둔다 (#8435)

## 최종 판정

**머지 보류 — 통합 최종 head의 Native/fresh WASM 전쪽 측정과 기준 devel 대비 대조, 전체 로컬 검증이 진행 중입니다.**
아래 값은 명시한 코드 SHA의 실측 결과이며 원 PR의 승인이나 닫기를 뜻하지 않습니다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8437](https://github.com/edwardkim/rhwp/pull/8437) · `planet6897` → `devel` |
| 원 head | `30bdfa39b6c5c93fdeed7fa5aa69bc9f6f6ae01e` |
| 통합 적용 커밋 | `a19b71ecbccf,3a884a23950d,66151761f274` |
| 검토 기준 devel | `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4` |
| 실측 제품 코드 | `dc87f4fa2e` (이후 회귀·문서 변경과 구분) |
| reviewer | `jangster77` 요청. 2026-11-04 원 head 불변·Open·non-Draft·요청 상태 재확인 |
| 기본 경로 | `maintainer_general` |
| 보조 경로 | `intake_and_review`, `local_validation`, `visual_fixture_evidence`, `multi_pr_update_branch`, `post_merge` |
| 관련 이슈 | [#8435](https://github.com/edwardkim/rhwp/issues/8435) |

## 변경과 검토 범위

원 PR의 변경 주장은 **fix(layout): TAC 표 다음 줄을 밀어낸 자리차지 로고 개체를 앵커 줄 자리에 둔다 (#8435)**입니다. 해당 주장에 연결된 입력과 쪽을 아래에 분리해 적었습니다.
적층 PR에 중복 포함된 선행 커밋과 이 PR의 실제 신규 커밋은 최종 통합 기록에서 구분합니다.

## 검증 입력과 전쪽 결과

| 입력 · 독립 Print PDF | Native 전쪽 TSV · 최저 raw | fresh WASM 전쪽 TSV · 최저 raw |
| --- | --- | --- |
| [156618114-logo-textbox-offset.hwpx](../../samples/issue8435/156618114-logo-textbox-offset.hwpx) · [156618114-logo-textbox-offset-2022.pdf](../../pdf/issue8435/156618114-logo-textbox-offset-2022.pdf) | [TSV](assets/planet6897_20261103/pr_8437/pr8437-156618114-logo-textbox-offset-0d68dd70-native.tsv) · 99.14% (2쪽) | [TSV](assets/planet6897_20261103/pr_8437/pr8437-156618114-logo-textbox-offset-0d68dd70-wasm.tsv) · 99.1% (2쪽) |
| [156638890-logo-textbox.hwpx](../../samples/issue8435/156638890-logo-textbox.hwpx) · [156638890-logo-textbox-2022.pdf](../../pdf/issue8435/156638890-logo-textbox-2022.pdf) | [TSV](assets/planet6897_20261103/pr_8437/pr8437-156638890-logo-textbox-68da8d3c-native.tsv) · 99.95% (3쪽) | [TSV](assets/planet6897_20261103/pr_8437/pr8437-156638890-logo-textbox-68da8d3c-wasm.tsv) · 99.95% (3쪽) |
| [156770078-logo-table.hwpx](../../samples/issue8435/156770078-logo-table.hwpx) · [156770078-logo-table-2022.pdf](../../pdf/issue8435/156770078-logo-table-2022.pdf) | [TSV](assets/planet6897_20261103/pr_8437/pr8437-156770078-logo-table-b137fa2e-native.tsv) · 99.93% (3쪽) | [TSV](assets/planet6897_20261103/pr_8437/pr8437-156770078-logo-table-b137fa2e-wasm.tsv) · 99.96% (3쪽) |

96 DPI, 색상 경계 보정 전 2px 실루엣입니다. 검증 입력과 Print PDF는 같은 행에서 대응합니다.
원본·PDF SHA와 Print 절차, 모든 쪽의 누락·쪽수 대조는 통합 증거 목록에서 확정합니다.

## 구현 주장과 검증 증거의 대조

TAC 표 다음 줄을 밀어낸 자리차지 로고는 밀린 글줄의 새 원점으로 다시 내려가지 않고 자신의 앵커 줄 원점에 놓인다.

layout.rs의 tac_sibling_float_anchor_offset_px와 typeset의 TableHostShapeFlow·place_table_host_shape_with_storage가 저장 앵커 소속과 흐름 이동을 나눠 소비한다. 뒤에서 별도 원점으로 덮어쓰는 경로를 함께 대조한다.

서로 다른 세 로고 입력의 주장 쪽에서 표·로고·뒤 글줄 관계를 확인한다. 표 소유 줄과 뒤 텍스트 줄이 다른 입력을 사용해 helper의 수치만 같은 테스트로 대체하지 않는다.

관련 정식 회귀 파일: [issue_8435_pushed_anchor_line_float.rs](../../tests/cases/issue_8435_pushed_anchor_line_float.rs)

## 검증 입력 커밋 확인

이번 통합의 [입력·PDF SHA-256 대조표](assets/planet6897_20261103/input-integrity.tsv)에 있는
148개 파일은 증적 고정 커밋 `fd97ae040f`의 파일 내용과 일치합니다.
위 개별 입력은 저장소의 동일 파일을 재사용합니다.

## 원 PR 주장 쪽의 시각 증적

| 입력 · 쪽 · 검토 대상 | Native raw · review / overlay | fresh WASM raw · review / overlay |
| --- | --- | --- |
| `156618114-logo-textbox-offset.hwpx` 2쪽 · 세 번째 TAC 로고 사례 | 99.95% · [review](assets/planet6897_20261103/pr_8437/pr8437-claim-p002-156618114-logo-tex-native-review.png) / [overlay](assets/planet6897_20261103/pr_8437/pr8437-claim-p002-156618114-logo-tex-native-overlay.png) | 99.95% · [review](assets/planet6897_20261103/pr_8437/pr8437-claim-p002-156618114-logo-tex-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8437/pr8437-claim-p002-156618114-logo-tex-wasm-overlay.png) |
| `156638890-logo-textbox.hwpx` 3쪽 · TAC 로고 원점 | 99.95% · [review](assets/planet6897_20261103/pr_8437/pr8437-claim-p003-156638890-logo-tex-native-review.png) / [overlay](assets/planet6897_20261103/pr_8437/pr8437-claim-p003-156638890-logo-tex-native-overlay.png) | 99.95% · [review](assets/planet6897_20261103/pr_8437/pr8437-claim-p003-156638890-logo-tex-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8437/pr8437-claim-p003-156638890-logo-tex-wasm-overlay.png) |
| `156770078-logo-table.hwpx` 3쪽 · 두 번째 TAC 로고 사례 | 99.98% · [review](assets/planet6897_20261103/pr_8437/pr8437-claim-p003-156770078-logo-tab-native-review.png) / [overlay](assets/planet6897_20261103/pr_8437/pr8437-claim-p003-156770078-logo-tab-native-overlay.png) | 99.98% · [review](assets/planet6897_20261103/pr_8437/pr8437-claim-p003-156770078-logo-tab-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8437/pr8437-claim-p003-156770078-logo-tab-wasm-overlay.png) |

## 남은 검증

- 현재 통합 최종 head의 집중·전체 Rust 회귀, lint와 fresh WASM을 완료하고 로그를 연결합니다.
- 고정 `upstream/devel`과 같은 입력·Print·글꼴 조건에서 전쪽 raw 하락을 비교합니다.
- 위 주장 쪽 review와 overlay를 독립 Print와 확대 대조해 내용·순서·격자·간격을 직접 판독합니다.
- 누락 증적이나 90% 미만 쪽이 있으면 보정 후 다시 측정합니다.

## Merge 후 원 PR 후속 처리

통합 PR의 merge SHA와 검증한 동일 입력·쪽의 TSV 및 review/overlay를 원 PR에 연결합니다.
댓글 게시와 원 PR 닫기는 최종 통합 검증 및 merge 후에 판정합니다.

<!-- planet6897-20261103-generated-review -->
