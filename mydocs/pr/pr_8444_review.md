# PR #8444 리뷰 — fix(layout): TAC 표 문단 뒤 줄의 TAC 개체를 줄 높이로 쪽 끝에 수용하고 그 줄 그림을 저장 줄 원점에 둔다 (#8441)

## 최종 판정

**머지 보류 — 통합 최종 head의 Native/fresh WASM 전쪽 측정과 기준 devel 대비 대조, 전체 로컬 검증이 진행 중입니다.**
아래 값은 명시한 코드 SHA의 실측 결과이며 원 PR의 승인이나 닫기를 뜻하지 않습니다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8444](https://github.com/edwardkim/rhwp/pull/8444) · `planet6897` → `devel` |
| 원 head | `14ee038f2194e255f7ca3c14d74f7bfefd480936` |
| 통합 적용 커밋 | `8eaf7e441a92,fabb6523873c` |
| 검토 기준 devel | `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4` |
| 실측 제품 코드 | `dc87f4fa2e` (이후 회귀·문서 변경과 구분) |
| reviewer | `jangster77` 요청. 2026-11-04 원 head 불변·Open·non-Draft·요청 상태 재확인 |
| 기본 경로 | `maintainer_general` |
| 보조 경로 | `intake_and_review`, `local_validation`, `visual_fixture_evidence`, `multi_pr_update_branch`, `post_merge` |
| 관련 이슈 | [#8441](https://github.com/edwardkim/rhwp/issues/8441) |

## 변경과 검토 범위

원 PR의 변경 주장은 **fix(layout): TAC 표 문단 뒤 줄의 TAC 개체를 줄 높이로 쪽 끝에 수용하고 그 줄 그림을 저장 줄 원점에 둔다 (#8441)**입니다. 해당 주장에 연결된 입력과 쪽을 아래에 분리해 적었습니다.
적층 PR에 중복 포함된 선행 커밋과 이 PR의 실제 신규 커밋은 최종 통합 기록에서 구분합니다.

## 검증 입력과 전쪽 결과

| 입력 · 독립 Print PDF | Native 전쪽 TSV · 최저 raw | fresh WASM 전쪽 TSV · 최저 raw |
| --- | --- | --- |
| [156578788-tac-logo-line.hwp](../../samples/issue8441/156578788-tac-logo-line.hwp) · [156578788-tac-logo-line-2022.pdf](../../pdf/issue8441/156578788-tac-logo-line-2022.pdf) | [TSV](assets/planet6897_20261103/pr_8444/pr8444-156578788-tac-logo-line-359ee733-native.tsv) · 99.61% (2쪽) | [TSV](assets/planet6897_20261103/pr_8444/pr8444-156578788-tac-logo-line-359ee733-wasm.tsv) · 99.56% (2쪽) |
| [156669499-tac-logo-line.hwpx](../../samples/issue8441/156669499-tac-logo-line.hwpx) · [156669499-tac-logo-line-2022.pdf](../../pdf/issue8441/156669499-tac-logo-line-2022.pdf) | [TSV](assets/planet6897_20261103/pr_8444/pr8444-156669499-tac-logo-line-2f73c60e-native.tsv) · 99.98% (1쪽) | [TSV](assets/planet6897_20261103/pr_8444/pr8444-156669499-tac-logo-line-2f73c60e-wasm.tsv) · 99.98% (1쪽) |
| [156715172-tac-table-picture.hwpx](../../samples/issue8441/156715172-tac-table-picture.hwpx) · [156715172-tac-table-picture-2022.pdf](../../pdf/issue8441/156715172-tac-table-picture-2022.pdf) | [TSV](assets/planet6897_20261103/pr_8444/pr8444-156715172-tac-table-picture-2651c3f0-native.tsv) · 99.09% (4쪽) | [TSV](assets/planet6897_20261103/pr_8444/pr8444-156715172-tac-table-picture-2651c3f0-fresh-wasm.tsv) · 99.1% (4쪽) |

96 DPI, 색상 경계 보정 전 2px 실루엣입니다. 검증 입력과 Print PDF는 같은 행에서 대응합니다.
원본·PDF SHA와 Print 절차, 모든 쪽의 누락·쪽수 대조는 통합 증거 목록에서 확정합니다.

## 구현 주장과 검증 증거의 대조

TAC 표 뒤 줄의 그림·도형은 그 줄의 점유 높이로 쪽 끝 수용을 판단하고, 실제 paint는 저장된 소유 줄 원점을 따른다.

layout.rs·layout/tac_picture_item.inc.rs·layout/shape_item.inc.rs의 원점과 typeset/controls/shape_flow.rs의 TableHostShapeFlow 요구 높이를 함께 대조한다. 그림의 높이와 host 줄의 전진을 중복 계상하지 않는다.

세 실물 로고 입력의 표·로고·뒤 문단 순서를 확인한다. 쪽 끝에서 이미 놓인 줄과 다음 쪽으로 이월할 줄을 별도로 검사해 한 경로의 통과를 다른 경로의 증거로 대신하지 않는다.

관련 정식 회귀 파일: [issue_8441_tac_shape_line_page_fit.rs](../../tests/cases/issue_8441_tac_shape_line_page_fit.rs)

## 검증 입력 커밋 확인

이번 통합의 [입력·PDF SHA-256 대조표](assets/planet6897_20261103/input-integrity.tsv)에 있는
148개 파일은 증적 고정 커밋 `fd97ae040f`의 파일 내용과 일치합니다.
위 개별 입력은 저장소의 동일 파일을 재사용합니다.

## 원 PR 주장 쪽의 시각 증적

| 입력 · 쪽 · 검토 대상 | Native raw · review / overlay | fresh WASM raw · review / overlay |
| --- | --- | --- |
| `156578788-tac-logo-line.hwp` 1쪽 · TAC 그림 줄 높이 | 99.88% · [review](assets/planet6897_20261103/pr_8444/pr8444-claim-p001-156578788-tac-logo-native-review.png) / [overlay](assets/planet6897_20261103/pr_8444/pr8444-claim-p001-156578788-tac-logo-native-overlay.png) | 99.95% · [review](assets/planet6897_20261103/pr_8444/pr8444-claim-p001-156578788-tac-logo-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8444/pr8444-claim-p001-156578788-tac-logo-wasm-overlay.png) |

## 남은 검증

- 현재 통합 최종 head의 집중·전체 Rust 회귀, lint와 fresh WASM을 완료하고 로그를 연결합니다.
- 고정 `upstream/devel`과 같은 입력·Print·글꼴 조건에서 전쪽 raw 하락을 비교합니다.
- 위 주장 쪽 review와 overlay를 독립 Print와 확대 대조해 내용·순서·격자·간격을 직접 판독합니다.
- 누락 증적이나 90% 미만 쪽이 있으면 보정 후 다시 측정합니다.

## Merge 후 원 PR 후속 처리

통합 PR의 merge SHA와 검증한 동일 입력·쪽의 TSV 및 review/overlay를 원 PR에 연결합니다.
댓글 게시와 원 PR 닫기는 최종 통합 검증 및 merge 후에 판정합니다.

<!-- planet6897-20261103-generated-review -->
