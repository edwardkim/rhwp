# PR #8477 리뷰 — fix(조판): 칸 안 자리차지 그림 — 앵커 줄 이중 계상 제거, 분할 시 그림과 앵커 줄을 함께 넘긴다 (#8472)

## 최종 판정

**머지 보류 — 통합 최종 head의 Native/fresh WASM 전쪽 측정과 기준 devel 대비 대조, 전체 로컬 검증이 진행 중입니다.**
아래 값은 명시한 코드 SHA의 실측 결과이며 원 PR의 승인이나 닫기를 뜻하지 않습니다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8477](https://github.com/edwardkim/rhwp/pull/8477) · `planet6897` → `devel` |
| 원 head | `d53da96d75daf912d3e53d9137eadc7770732bbf` |
| 통합 적용 커밋 | `32cb512908d4,d2535a52ecbb,313a9960b743,76ceb2574cb0` |
| 검토 기준 devel | `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4` |
| 실측 제품 코드 | `dc87f4fa2e` (이후 회귀·문서 변경과 구분) |
| reviewer | `jangster77` 요청. 2026-11-04 원 head 불변·Open·non-Draft·요청 상태 재확인 |
| 기본 경로 | `maintainer_general` |
| 보조 경로 | `intake_and_review`, `local_validation`, `visual_fixture_evidence`, `multi_pr_update_branch`, `post_merge` |
| 관련 이슈 | [#8472](https://github.com/edwardkim/rhwp/issues/8472) |

## 변경과 검토 범위

원 PR의 변경 주장은 **fix(조판): 칸 안 자리차지 그림 — 앵커 줄 이중 계상 제거, 분할 시 그림과 앵커 줄을 함께 넘긴다 (#8472)**입니다. 해당 주장에 연결된 입력과 쪽을 아래에 분리해 적었습니다.
적층 PR에 중복 포함된 선행 커밋과 이 PR의 실제 신규 커밋은 최종 통합 기록에서 구분합니다.

## 검증 입력과 전쪽 결과

| 입력 · 독립 Print PDF | Native 전쪽 TSV · 최저 raw | fresh WASM 전쪽 TSV · 최저 raw |
| --- | --- | --- |
| [cell_picture_row_split.hwp](../../samples/issue8472/cell_picture_row_split.hwp) · [issue8472_cell_picture_row_split-2020.pdf](../../pdf/issue8472_cell_picture_row_split-2020.pdf) | [TSV](assets/planet6897_20261103/pr_8477/pr8477-cell_picture_row_split-f5c3c377-native.tsv) · 90.29% (2쪽) | [TSV](assets/planet6897_20261103/pr_8477/pr8477-cell_picture_row_split-f5c3c377-wasm.tsv) · 90.64% (2쪽) |

96 DPI, 색상 경계 보정 전 2px 실루엣입니다. 검증 입력과 Print PDF는 같은 행에서 대응합니다.
원본·PDF SHA와 Print 절차, 모든 쪽의 누락·쪽수 대조는 통합 증거 목록에서 확정합니다.

## 조판 원칙 준수 검토

적용 대상입니다. 원본 저장값·독립 Print를 근거로 공통 규칙, 측정과 실제 배치의 소비 지점, 내용 소속·순서 및 적용되지 않는 반례를 아래에서 대조합니다.

## 구현 주장과 검증 증거의 대조

칸 안 문단 기준 자리차지 그림은 앵커 줄 높이를 위에 한 번 더 쌓지 않고, 행 분할 때 그 그림과 소유 앵커 줄을 같은 조각으로 넘긴다.

칸의 저장 사다리에서 점유·원점을 구하고 측정·분할 컷·실제 칸 paint가 같은 소유 범위를 소비한다. 글자가 있는 앵커와 글자 없는 앵커를 구분한다.

2쪽 조각의 그림 두 개가 표 밖으로 나가지 않는지 확인한다. 저장 사다리가 되감기는 그림 칸을 본문 표 전체의 정상 단조 사다리로 오인하지 않는 반례를 유지한다.

관련 정식 회귀 파일: [issue_8472_cell_picture_anchor.rs](../../tests/cases/issue_8472_cell_picture_anchor.rs)

## 검증 입력 커밋 확인

이번 통합의 [입력·PDF SHA-256 대조표](assets/planet6897_20261103/input-integrity.tsv)에 있는
148개 파일은 증적 고정 커밋 `fd97ae040f`의 파일 내용과 일치합니다.
위 개별 입력은 저장소의 동일 파일을 재사용합니다.

## 원 PR 주장 쪽의 시각 증적

| 입력 · 쪽 · 검토 대상 | Native raw · review / overlay | fresh WASM raw · review / overlay |
| --- | --- | --- |
| `cell_picture_row_split.hwp` 2쪽 · 그림 칸 앵커 | 100.00% · [review](assets/planet6897_20261103/pr_8477/pr8477-claim-p002-cell_picture_row_s-native-review.png) / [overlay](assets/planet6897_20261103/pr_8477/pr8477-claim-p002-cell_picture_row_s-native-overlay.png) | 100.00% · [review](assets/planet6897_20261103/pr_8477/pr8477-claim-p002-cell_picture_row_s-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8477/pr8477-claim-p002-cell_picture_row_s-wasm-overlay.png) |

## 남은 검증

- 현재 통합 최종 head의 집중·전체 Rust 회귀, lint와 fresh WASM을 완료하고 로그를 연결합니다.
- 고정 `upstream/devel`과 같은 입력·Print·글꼴 조건에서 전쪽 raw 하락을 비교합니다.
- 위 주장 쪽 review와 overlay를 독립 Print와 확대 대조해 내용·순서·격자·간격을 직접 판독합니다.
- 누락 증적이나 90% 미만 쪽이 있으면 보정 후 다시 측정합니다.

## Merge 후 원 PR 후속 처리

통합 PR의 merge SHA와 검증한 동일 입력·쪽의 TSV 및 review/overlay를 원 PR에 연결합니다.
댓글 게시와 원 PR 닫기는 최종 통합 검증 및 merge 후에 판정합니다.

<!-- planet6897-20261103-generated-review -->
