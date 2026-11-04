# PR #8429 리뷰 — fix(scaffold): 글 문단에 측정하지 않은 저장 줄 정보를 쓰지 않는다 — 긴 셀 텍스트 잘림·쪽수 불일치 (#8424)

## 최종 판정

**머지 보류 — 통합 최종 head의 Native/fresh WASM 전쪽 측정과 기준 devel 대비 대조, 전체 로컬 검증이 진행 중입니다.**
아래 값은 명시한 코드 SHA의 실측 결과이며 원 PR의 승인이나 닫기를 뜻하지 않습니다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8429](https://github.com/edwardkim/rhwp/pull/8429) · `planet6897` → `devel` |
| 원 head | `311a8dce50adc9098cab2e6b68181c80871be38d` |
| 통합 적용 커밋 | `87c47e80b716,756ca7812f26,f384d80a9a57` |
| 검토 기준 devel | `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4` |
| 실측 제품 코드 | `dc87f4fa2e` (이후 회귀·문서 변경과 구분) |
| reviewer | `jangster77` 요청. 2026-11-04 원 head 불변·Open·non-Draft·요청 상태 재확인 |
| 기본 경로 | `maintainer_general` |
| 보조 경로 | `intake_and_review`, `local_validation`, `visual_fixture_evidence`, `multi_pr_update_branch`, `post_merge` |
| 관련 이슈 | [#8424](https://github.com/edwardkim/rhwp/issues/8424) |

## 변경과 검토 범위

원 PR의 변경 주장은 **fix(scaffold): 글 문단에 측정하지 않은 저장 줄 정보를 쓰지 않는다 — 긴 셀 텍스트 잘림·쪽수 불일치 (#8424)**입니다. 해당 주장에 연결된 입력과 쪽을 아래에 분리해 적었습니다.
적층 PR에 중복 포함된 선행 커밋과 이 PR의 실제 신규 커밋은 최종 통합 기록에서 구분합니다.

## 검증 입력과 전쪽 결과

| 입력 · 독립 Print PDF | Native 전쪽 TSV · 최저 raw | fresh WASM 전쪽 TSV · 최저 raw |
| --- | --- | --- |
| [business_plan.hwpx](assets/issue_8424/inputs/after/business_plan.hwpx) · [business_plan-2024.pdf](../../pdf/issue-8424/business_plan-2024.pdf) | [TSV](assets/planet6897_20261103/pr_8429/pr8429-business_plan-b47c75a0-native.tsv) · 99.47% (3쪽) | [TSV](assets/planet6897_20261103/pr_8429/pr8429-business_plan-b47c75a0-wasm.tsv) · 99.51% (3쪽) |
| [cellwrap.hwpx](assets/issue_8424/inputs/after/cellwrap.hwpx) · [cellwrap-2024.pdf](../../pdf/issue-8424/cellwrap-2024.pdf) | [TSV](assets/planet6897_20261103/pr_8429/pr8429-cellwrap-4f4c9ba7-native.tsv) · 100.0% (1쪽) | [TSV](assets/planet6897_20261103/pr_8429/pr8429-cellwrap-4f4c9ba7-wasm.tsv) · 100.0% (1쪽) |

96 DPI, 색상 경계 보정 전 2px 실루엣입니다. 검증 입력과 Print PDF는 같은 행에서 대응합니다.
원본·PDF SHA와 Print 절차, 모든 쪽의 누락·쪽수 대조는 통합 증거 목록에서 확정합니다.

## 조판 원칙 준수 검토

적용 대상입니다. 원본 저장값·독립 Print를 근거로 공통 규칙, 측정과 실제 배치의 소비 지점, 내용 소속·순서 및 적용되지 않는 반례를 아래에서 대조합니다.

## 구현 주장과 검증 증거의 대조

scaffold가 실제로 측정하지 않은 저장 줄 하나를 긴 본문·셀 문단의 확정 조판 정보처럼 만들지 않는다.

src/scaffold/builder.rs의 make_text_para·make_cell_para에서 UTF-16 길이와 내용은 보존하고 측정하지 않은 줄 정보는 비워 실제 줄 구성 경로가 높이와 줄바꿈을 결정하게 한다.

business_plan의 긴 셀 본문, cellwrap의 줄바꿈과 scaffold 범위 대조군을 사용한다. 뒤 내용의 누락·잘림 및 쪽수 차이를 전쪽 독립 PDF와 함께 검사한다.

관련 정식 회귀 파일: [issue_8424_scaffold_cell_wrap.rs](../../tests/cases/issue_8424_scaffold_cell_wrap.rs)

## 검증 입력 커밋 확인

이번 통합의 [입력·PDF SHA-256 대조표](assets/planet6897_20261103/input-integrity.tsv)에 있는
148개 파일은 증적 고정 커밋 `fd97ae040f`의 파일 내용과 일치합니다.
위 개별 입력은 저장소의 동일 파일을 재사용합니다.

## 원 PR 주장 쪽의 시각 증적

| 입력 · 쪽 · 검토 대상 | Native raw · review / overlay | fresh WASM raw · review / overlay |
| --- | --- | --- |
| `business_plan.hwpx` 2쪽 · 긴 셀 문단의 추가 주장 페이지 | 100.00% · [review](assets/planet6897_20261103/pr_8429/pr8429-claim-p002-business_plan-native-review.png) / [overlay](assets/planet6897_20261103/pr_8429/pr8429-claim-p002-business_plan-native-overlay.png) | 100.00% · [review](assets/planet6897_20261103/pr_8429/pr8429-claim-p002-business_plan-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8429/pr8429-claim-p002-business_plan-wasm-overlay.png) |
| `cellwrap.hwpx` 1쪽 · 긴 셀 문단 저장 줄 | 100.00% · [review](assets/planet6897_20261103/pr_8429/pr8429-claim-p001-cellwrap-native-review.png) / [overlay](assets/planet6897_20261103/pr_8429/pr8429-claim-p001-cellwrap-native-overlay.png) | 100.00% · [review](assets/planet6897_20261103/pr_8429/pr8429-claim-p001-cellwrap-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8429/pr8429-claim-p001-cellwrap-wasm-overlay.png) |

## 남은 검증

- 현재 통합 최종 head의 집중·전체 Rust 회귀, lint와 fresh WASM을 완료하고 로그를 연결합니다.
- 고정 `upstream/devel`과 같은 입력·Print·글꼴 조건에서 전쪽 raw 하락을 비교합니다.
- 위 주장 쪽 review와 overlay를 독립 Print와 확대 대조해 내용·순서·격자·간격을 직접 판독합니다.
- 누락 증적이나 90% 미만 쪽이 있으면 보정 후 다시 측정합니다.

## Merge 후 원 PR 후속 처리

통합 PR의 merge SHA와 검증한 동일 입력·쪽의 TSV 및 review/overlay를 원 PR에 연결합니다.
댓글 게시와 원 PR 닫기는 최종 통합 검증 및 merge 후에 판정합니다.

<!-- planet6897-20261103-generated-review -->
