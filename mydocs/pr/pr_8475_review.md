# PR #8475 리뷰 — fix(table): 세로쓰기 칸이 있는 표를 쪽 경계에서 나누지 않는다 (#8470)

## 최종 판정

**메인터너 보정 후 수용 가능**
메인터너가 통합 후보의 실제 전쪽 증적과 회귀 근거를 보강했습니다. 원 PR에 대응하는 입력·쪽과 반례는 아래에 기록했습니다.
증적·회귀 첫머리 보강 커밋은 `e3986247e9`·`65d345f8fd`입니다. 원 head와 보정된 통합 후보를 구분합니다.
새 devel `1924d371d6` 반영 뒤 기능 보존과 최종 공통 검증을 마치면 통합 PR을 준비합니다. PR 생성은 작업지시자 승인 대기입니다.
아래 값은 명시한 코드 SHA의 실측 결과이며 원 PR의 승인이나 닫기를 뜻하지 않습니다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8475](https://github.com/edwardkim/rhwp/pull/8475) · `planet6897` → `devel` |
| 원 head | `c01f286aca82a191bca56e3df89defddeeda81a0` |
| 통합 적용 커밋 | `1c8e87175433,5d7e42138e72` |
| 검토 기준 devel | `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4` |
| 실측 제품 코드 | `dc87f4fa2e` (이후 회귀·문서 변경과 구분) |
| reviewer | `jangster77` 요청. 2026-11-04 원 head 불변·Open·non-Draft·요청 상태 재확인 |
| 기본 경로 | `maintainer_general` |
| 보조 경로 | `intake_and_review`, `local_validation`, `visual_fixture_evidence`, `multi_pr_update_branch`, `post_merge` |
| 관련 이슈 | [#8470](https://github.com/edwardkim/rhwp/issues/8470) |

## 변경과 검토 범위

원 PR의 변경 주장은 **fix(table): 세로쓰기 칸이 있는 표를 쪽 경계에서 나누지 않는다 (#8470)**입니다. 해당 주장에 연결된 입력과 쪽을 아래에 분리해 적었습니다.
적층 PR에 중복 포함된 선행 커밋과 이 PR의 실제 신규 커밋은 최종 통합 기록에서 구분합니다.

## 검증 입력과 전쪽 결과

| 입력 · 독립 Print PDF | Native 전쪽 TSV · 최저 raw | fresh WASM 전쪽 TSV · 최저 raw |
| --- | --- | --- |
| [vertical_fit.hwpx](../../samples/issue8470/vertical_fit.hwpx) · [vertical_fit-2020.pdf](../../pdf/issue8470/vertical_fit-2020.pdf) | [TSV](assets/planet6897_20261103/pr_8475/pr8475-vertical_fit-8a319181-native.tsv) · 98.75% (5쪽) | [TSV](assets/planet6897_20261103/pr_8475/pr8475-vertical_fit-8a319181-wasm.tsv) · 98.72% (5쪽) |
| [vertical_oversize.hwpx](../../samples/issue8470/vertical_oversize.hwpx) · [vertical_oversize-2020.pdf](../../pdf/issue8470/vertical_oversize-2020.pdf) | [TSV](assets/planet6897_20261103/pr_8475/pr8475-vertical_oversize-30055ea5-native.tsv) · 98.94% (5쪽) | [TSV](assets/planet6897_20261103/pr_8475/pr8475-vertical_oversize-30055ea5-wasm.tsv) · 98.92% (5쪽) |
| [42565_입법예고문.hwp](../../samples/issue8470/42565_입법예고문.hwp) · [42565_입법예고문-2020.pdf](../../pdf/issue8470/42565_입법예고문-2020.pdf) | [TSV](assets/planet6897_20261103/pr_8475/pr8475-42565_입법예고문-2279178b-native.tsv) · 96.17% (16쪽) | [TSV](assets/planet6897_20261103/pr_8475/pr8475-42565_입법예고문-2279178b-wasm.tsv) · 96.96% (16쪽) |

96 DPI, 색상 경계 보정 전 2px 실루엣입니다. 검증 입력과 Print PDF는 같은 행에서 대응합니다.
원본·PDF SHA와 Print 절차, 모든 쪽의 누락·쪽수 대조는 통합 증거 목록에서 확정합니다.

## 조판 원칙 준수 검토

적용 대상입니다. 원본 저장값·독립 Print를 근거로 공통 규칙, 측정과 실제 배치의 소비 지점, 내용 소속·순서 및 적용되지 않는 반례를 아래에서 대조합니다.

## 구현 주장과 검증 증거의 대조

세로쓰기 칸을 가진 표는 저장 page_break 값을 보존하면서 조판에서는 나누지 않는 표로 취급한다.

model/table.rs의 Table::layout_page_break가 측정·typeset·layout 공통 소비 지점이다. has_vertical_cell 캐시는 셀 구성·글 방향 편집 뒤 무효화하며 저장 attr 자체는 바꾸지 않는다.

작은 표·본문보다 긴 표·걸침 위치가 다른 표를 독립 Print와 대조한다. 가로쓰기 표의 기존 분할과 편집·저장 왕복을 대조군으로 유지한다. 추가한 42565 실물은 devel 17쪽·본문 넘침 4건에서 Print와 같은 16쪽·5건으로 바뀐다. 추가 1건은 10쪽의 한 쪽 세로쓰기 표이며 Native 99.26%·fresh WASM 99.25%와 직접 판독으로 확인했다. 기존 기준선 행·공차는 유지하고 신규 입력의 정상 관측값 5건을 등록한다. [본문 경계와 독립 Print 대조 근거](assets/planet6897_20261103/body-overflow-42565-proof.json)를 연결한다.

관련 정식 회귀 파일: [issue_8470_vertical_cell_table_no_split.rs](../../tests/cases/issue_8470_vertical_cell_table_no_split.rs)

## 검증 입력 커밋 확인

이번 통합의 [입력·PDF SHA-256 대조표](assets/planet6897_20261103/input-integrity.tsv)에 있는
148개 파일은 증적 고정 커밋 `fd97ae040f`의 파일 내용과 일치합니다.
위 개별 입력은 저장소의 동일 파일을 재사용합니다.

## 원 PR 주장 쪽의 시각 증적

| 입력 · 쪽 · 검토 대상 | Native raw · review / overlay | fresh WASM raw · review / overlay |
| --- | --- | --- |
| `vertical_fit.hwpx` 5쪽 · 세로쓰기 표 경계 | 98.75% · [review](assets/planet6897_20261103/pr_8475/pr8475-claim-p005-vertical_fit-native-review.png) / [overlay](assets/planet6897_20261103/pr_8475/pr8475-claim-p005-vertical_fit-native-overlay.png) | 98.72% · [review](assets/planet6897_20261103/pr_8475/pr8475-claim-p005-vertical_fit-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8475/pr8475-claim-p005-vertical_fit-wasm-overlay.png) |
| `vertical_oversize.hwpx` 5쪽 · 세로쓰기 넘침 경계 | 98.94% · [review](assets/planet6897_20261103/pr_8475/pr8475-claim-p005-vertical_oversize-native-review.png) / [overlay](assets/planet6897_20261103/pr_8475/pr8475-claim-p005-vertical_oversize-native-overlay.png) | 98.92% · [review](assets/planet6897_20261103/pr_8475/pr8475-claim-p005-vertical_oversize-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8475/pr8475-claim-p005-vertical_oversize-wasm-overlay.png) |
| `42565_입법예고문.hwp` 5쪽 · 구역 첫쪽 표 위 간격 | 99.27% · [review](assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p005-native-review.png) / [overlay](assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p005-native-overlay.png) | 99.24% · [review](assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p005-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p005-wasm-overlay.png) |
| `42565_입법예고문.hwp` 10쪽 · 세로쓰기 표 한쪽 수용 | 99.26% · [review](assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p010-native-review.png) / [overlay](assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p010-native-overlay.png) | 99.25% · [review](assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p010-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p010-wasm-overlay.png) |
| `42565_입법예고문.hwp` 11쪽 · 병합 칸 끝 행 | 100.00% · [review](assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p011-native-review.png) / [overlay](assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p011-native-overlay.png) | 100.00% · [review](assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p011-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p011-wasm-overlay.png) |
| `42565_입법예고문.hwp` 13쪽 · 구역 첫쪽 표 위 간격 | 99.88% · [review](assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p013-native-review.png) / [overlay](assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p013-native-overlay.png) | 99.84% · [review](assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p013-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8475/pr8475-8489-8497-public-claims-p013-wasm-overlay.png) |

## 통합 최종 검증 조건

- 리베이스 전 제품 `dc87f4fa2e`의 해당 전쪽 Native/fresh WASM 측정은 완료했습니다. 전체 Rust 12,498개 중 1건 기준 누락은 `70a9d870ee`에서 독립 Print로 보정한 뒤 통과했습니다.
- `65d345f8fd`의 40개 회귀 첫머리 보강은 주석 외 내용이 동일합니다. 새 devel의 제품 변경과 구분해 검증합니다.
- 새 devel의 Native 글꼴·측정 경로와 이번 통합을 함께 반영한 최종 head에서 필수 회귀·lint·출력 보존을 확인합니다.
- 고정 `upstream/devel`과 같은 입력·Print·글꼴 조건에서 전쪽 raw 하락을 비교합니다.
- 위 주장 쪽 review와 overlay를 독립 Print와 확대 대조해 내용·순서·격자·간격을 직접 판독합니다.
- 누락 증적이나 90% 미만 쪽이 있으면 보정 후 다시 측정합니다.

## Merge 후 원 PR 후속 처리

통합 PR의 merge SHA와 검증한 동일 입력·쪽의 TSV 및 review/overlay를 원 PR에 연결합니다.
댓글 게시와 원 PR 닫기는 최종 통합 검증 및 merge 후에 판정합니다.

<!-- planet6897-20261103-generated-review -->
