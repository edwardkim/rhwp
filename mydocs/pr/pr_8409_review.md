# PR #8409 리뷰 — test(#8153): 편람 원문 기준점을 run 경계와 무관한 연속 글 조각으로 대조한다

## 최종 판정

**머지 보류 — 통합 최종 head의 Native/fresh WASM 전쪽 측정과 기준 devel 대비 대조, 전체 로컬 검증이 진행 중입니다.**
아래 값은 명시한 코드 SHA의 실측 결과이며 원 PR의 승인이나 닫기를 뜻하지 않습니다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8409](https://github.com/edwardkim/rhwp/pull/8409) · `planet6897` → `devel` |
| 원 head | `a09816f7a1aa3a42b147602bbfce998f477af8d1` |
| 통합 적용 커밋 | `b93afd2b4e3e` |
| 검토 기준 devel | `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4` |
| 실측 제품 코드 | `dc87f4fa2e` (이후 회귀·문서 변경과 구분) |
| reviewer | `jangster77` 요청. 2026-11-04 원 head 불변·Open·non-Draft·요청 상태 재확인 |
| 기본 경로 | `maintainer_general` |
| 보조 경로 | `intake_and_review`, `local_validation`, `visual_fixture_evidence`, `multi_pr_update_branch`, `post_merge` |
| 관련 이슈 | 원 PR 본문 확인 |

## 변경과 검토 범위

원 PR의 변경 주장은 **test(#8153): 편람 원문 기준점을 run 경계와 무관한 연속 글 조각으로 대조한다**입니다. 해당 주장에 연결된 입력과 쪽을 아래에 분리해 적었습니다.
적층 PR에 중복 포함된 선행 커밋과 이 PR의 실제 신규 커밋은 최종 통합 기록에서 구분합니다.

## 검증 입력과 전쪽 결과

| 입력 · 독립 Print PDF | Native 전쪽 TSV · 최저 raw | fresh WASM 전쪽 TSV · 최저 raw |
| --- | --- | --- |
| [2025 행정업무운영 편람(최종).hwp](<../../samples/2025 행정업무운영 편람(최종).hwp>) · [2025 행정업무운영 편람(최종)-hwp-kopub-2024.pdf](<../../pdf/2025 행정업무운영 편람(최종)-hwp-kopub-2024.pdf>) | [TSV](assets/planet6897_20261103/pr_8409/historical-handbook-hwp-native.tsv) · 99.96% (383쪽) | [TSV](assets/planet6897_20261103/pr_8409/historical-handbook-hwp-wasm.tsv) · 99.96% (383쪽) |
| [2025 행정업무운영 편람(최종).hwpx](<../../samples/2025 행정업무운영 편람(최종).hwpx>) · [2025 행정업무운영 편람(최종)-hwpx-kopub-2024.pdf](<../../pdf/2025 행정업무운영 편람(최종)-hwpx-kopub-2024.pdf>) | [TSV](assets/planet6897_20261103/pr_8409/historical-handbook-hwpx-native.tsv) · 99.96% (383쪽) | 산출 중 · 미측정 (?쪽) |

96 DPI, 색상 경계 보정 전 2px 실루엣입니다. 검증 입력과 Print PDF는 같은 행에서 대응합니다.
원본·PDF SHA와 Print 절차, 모든 쪽의 누락·쪽수 대조는 통합 증거 목록에서 확정합니다.

## 구현 주장과 검증 증거의 대조

원문 기준점은 같은 물리 기준선에서 맞닿은 연속 run의 글로 식별한다. 글꼴 슬롯 때문에 나뉜 run 개수는 출력의 정확성 기준이 아니다.

tests/cases/issue_8153_all_pages.rs의 text_anchor_runs가 같은 기준선·수평 접점을 확인하고 원문 문자열과 대조한 첫 run의 원점을 기존 독립 좌표 검사에 전달한다. 제품 renderer는 변경하지 않는다.

공백 run을 기준점 시작으로 삼지 않고, 여러 run을 이었을 때 조각 뒤에 다른 본문이 남는 경우는 배제한다. 241쪽의 다른 글줄 앞부분을 잘못 찾는 반례와 두 편람의 전쪽 내용·좌표 검사를 유지한다.

관련 정식 회귀 파일: [issue_8153_all_pages.rs](../../tests/cases/issue_8153_all_pages.rs) · [issue_8153_kopub_circled_symbol.rs](../../tests/cases/issue_8153_kopub_circled_symbol.rs)

## 검증 입력 커밋 확인

두 편람 원본과 독립 Print PDF는 기존 저장소·로컬 기준 자료를 재사용합니다.
파일별 SHA·대용량 PDF의 보관 조건은 전쪽 대조의 입력 명세에 함께 기록합니다.

## 원 PR 주장 쪽의 시각 증적

제품 renderer를 바꾸지 않는 기준점 식별 검사이므로 위 두 원본의 전쪽 TSV와
기존 독립 Print 원점 계약을 재사용합니다. 연속 run의 식별과 원문 기준점 보존을 검증합니다.

## 남은 검증

- 현재 통합 최종 head의 집중·전체 Rust 회귀, lint와 fresh WASM을 완료하고 로그를 연결합니다.
- 고정 `upstream/devel`과 같은 입력·Print·글꼴 조건에서 전쪽 raw 하락을 비교합니다.
- 위 주장 쪽 review와 overlay를 독립 Print와 확대 대조해 내용·순서·격자·간격을 직접 판독합니다.
- 누락 증적이나 90% 미만 쪽이 있으면 보정 후 다시 측정합니다.

## Merge 후 원 PR 후속 처리

통합 PR의 merge SHA와 검증한 동일 입력·쪽의 TSV 및 review/overlay를 원 PR에 연결합니다.
댓글 게시와 원 PR 닫기는 최종 통합 검증 및 merge 후에 판정합니다.

<!-- planet6897-20261103-generated-review -->
