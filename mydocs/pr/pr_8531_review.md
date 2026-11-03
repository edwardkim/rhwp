# PR #8531 리뷰 — fix(typeset): 쪽 아래 고정 틀 띠 높이에서 host 문단 간격을 뺀다 (#8525, #8522 위 적층)

## 최종 판정

**머지 보류 — 통합 최종 head의 Native/fresh WASM 전쪽 측정과 기준 devel 대비 대조, 전체 로컬 검증이 진행 중입니다.**
아래 값은 명시한 코드 SHA의 실측 결과이며 원 PR의 승인이나 닫기를 뜻하지 않습니다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8531](https://github.com/edwardkim/rhwp/pull/8531) · `planet6897` → `devel` |
| 원 head | `2f14656a35b0934a247fd5b6d300ce3e24f98c48` |
| 통합 적용 커밋 | `222ce1de6de0,23c651919a86,2508c8205da3,0c0d19d911aa` |
| 검토 기준 devel | `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4` |
| 실측 제품 코드 | `dc87f4fa2e` (이후 회귀·문서 변경과 구분) |
| reviewer | `jangster77` 요청. 2026-11-04 원 head 불변·Open·non-Draft·요청 상태 재확인 |
| 기본 경로 | `maintainer_general` |
| 보조 경로 | `intake_and_review`, `local_validation`, `visual_fixture_evidence`, `multi_pr_update_branch`, `post_merge` |
| 관련 이슈 | [#8525](https://github.com/edwardkim/rhwp/issues/8525) |

## 변경과 검토 범위

원 PR의 변경 주장은 **fix(typeset): 쪽 아래 고정 틀 띠 높이에서 host 문단 간격을 뺀다 (#8525, #8522 위 적층)**입니다. 해당 주장에 연결된 입력과 쪽을 아래에 분리해 적었습니다.
적층 PR에 중복 포함된 선행 커밋과 이 PR의 실제 신규 커밋은 최종 통합 기록에서 구분합니다.

## 검증 입력과 전쪽 결과

| 입력 · 독립 Print PDF | Native 전쪽 TSV · 최저 raw | fresh WASM 전쪽 TSV · 최저 raw |
| --- | --- | --- |
| [band_host_spacing_22.hwpx](../../samples/issue8525/band_host_spacing_22.hwpx) · [band_host_spacing_22-2020.pdf](../../pdf/issue8525/band_host_spacing_22-2020.pdf) | [TSV](assets/planet6897_20261103/pr_8531/pr8531-band_host_spacing_22-7af5cf51-native.tsv) · 100.0% (1쪽) | [TSV](assets/planet6897_20261103/pr_8531/pr8531-band_host_spacing_22-7af5cf51-wasm.tsv) · 100.0% (1쪽) |
| [band_host_spacing_23.hwpx](../../samples/issue8525/band_host_spacing_23.hwpx) · [band_host_spacing_23-2020.pdf](../../pdf/issue8525/band_host_spacing_23-2020.pdf) | [TSV](assets/planet6897_20261103/pr_8531/pr8531-band_host_spacing_23-f7de3bc9-native.tsv) · 100.0% (1쪽) | [TSV](assets/planet6897_20261103/pr_8531/pr8531-band_host_spacing_23-f7de3bc9-wasm.tsv) · 100.0% (1쪽) |
| [band_host_spacing_23.hwpx](../../samples/issue8525/band_host_spacing_23.hwpx) · [band_host_spacing_23-2024.pdf](../../pdf/issue8525/band_host_spacing_23-2024.pdf) | [TSV](assets/planet6897_20261103/pr_8531/pr8531-band_host_spacing_23-182b6064-native.tsv) · 100.0% (1쪽) | [TSV](assets/planet6897_20261103/pr_8531/pr8531-band_host_spacing_23-182b6064-wasm.tsv) · 100.0% (1쪽) |
| [band_host_spacing_24.hwpx](../../samples/issue8525/band_host_spacing_24.hwpx) · [band_host_spacing_24-2020.pdf](../../pdf/issue8525/band_host_spacing_24-2020.pdf) | [TSV](assets/planet6897_20261103/pr_8531/pr8531-band_host_spacing_24-06a37a54-native.tsv) · 100.0% (2쪽) | [TSV](assets/planet6897_20261103/pr_8531/pr8531-band_host_spacing_24-06a37a54-wasm.tsv) · 100.0% (2쪽) |

96 DPI, 색상 경계 보정 전 2px 실루엣입니다. 검증 입력과 Print PDF는 같은 행에서 대응합니다.
원본·PDF SHA와 Print 절차, 모든 쪽의 누락·쪽수 대조는 통합 증거 목록에서 확정합니다.

## 구현 주장과 검증 증거의 대조

쪽 아래 고정 틀 밴드의 높이에서 host 문단 간격을 빼 실제 밴드 내용이 차지한 몫만 계상한다.

typeset의 하단 틀 높이 생산과 예약·남은 본문 예산·실제 표 원점을 함께 대조한다. 문단 간격을 표 밴드와 본문 흐름에서 두 번 쓰지 않는다.

서로 다른 band_host_spacing 변형과 뒤 본문을 같은 Print 쪽에 연결한다. 선행 TAC 표·누름틀 줄 구성 PR의 정상 경계도 함께 유지한다.

## 검증 입력 커밋 확인

이번 통합의 [입력·PDF SHA-256 대조표](assets/planet6897_20261103/input-integrity.tsv)에 있는
148개 파일은 증적 고정 커밋 `fd97ae040f`의 파일 내용과 일치합니다.
위 개별 입력은 저장소의 동일 파일을 재사용합니다.

## 원 PR 주장 쪽의 시각 증적

| 입력 · 쪽 · 검토 대상 | Native raw · review / overlay | fresh WASM raw · review / overlay |
| --- | --- | --- |
| `band_host_spacing_23.hwpx` 1쪽 · 쪽 아래 고정 틀 띠 높이 | 100.00% · [review](assets/planet6897_20261103/pr_8531/pr8531-claim-p001-band_host_spacing_-c20c16-native-review.png) / [overlay](assets/planet6897_20261103/pr_8531/pr8531-claim-p001-band_host_spacing_-c20c16-native-overlay.png) | 100.00% · [review](assets/planet6897_20261103/pr_8531/pr8531-claim-p001-band_host_spacing_-c20c16-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8531/pr8531-claim-p001-band_host_spacing_-c20c16-wasm-overlay.png) |
| `band_host_spacing_24.hwpx` 1쪽 · 다른 쪽 아래 고정 틀 띠 높이 | 100.00% · [review](assets/planet6897_20261103/pr_8531/pr8531-claim-p001-band_host_spacing_-ce6196-native-review.png) / [overlay](assets/planet6897_20261103/pr_8531/pr8531-claim-p001-band_host_spacing_-ce6196-native-overlay.png) | 100.00% · 산출 중 / 산출 중 |

## 남은 검증

- 현재 통합 최종 head의 집중·전체 Rust 회귀, lint와 fresh WASM을 완료하고 로그를 연결합니다.
- 고정 `upstream/devel`과 같은 입력·Print·글꼴 조건에서 전쪽 raw 하락을 비교합니다.
- 위 주장 쪽 review와 overlay를 독립 Print와 확대 대조해 내용·순서·격자·간격을 직접 판독합니다.
- 누락 증적이나 90% 미만 쪽이 있으면 보정 후 다시 측정합니다.

## Merge 후 원 PR 후속 처리

통합 PR의 merge SHA와 검증한 동일 입력·쪽의 TSV 및 review/overlay를 원 PR에 연결합니다.
댓글 게시와 원 PR 닫기는 최종 통합 검증 및 merge 후에 판정합니다.

<!-- planet6897-20261103-generated-review -->
