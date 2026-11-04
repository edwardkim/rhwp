# PR #8422 리뷰 — fix(layout): 앞 줄에 TAC 개체가 있는 문단의 TAC 표 줄이 자기 줄간격을 소비한다 (#8418)

## 최종 판정

**메인터너 보정 후 수용 가능**
메인터너가 통합 후보의 실제 전쪽 증적과 회귀 근거를 보강했습니다. 원 PR에 대응하는 입력·쪽과 반례는 아래에 기록했습니다.
증적·회귀 첫머리 보강 커밋은 `e3986247e9`·`65d345f8fd`입니다. 원 head와 보정된 통합 후보를 구분합니다.
새 devel `1924d371d6` 반영 뒤 기능 보존과 최종 공통 검증을 마치면 통합 PR을 준비합니다. PR 생성은 작업지시자 승인 대기입니다.
아래 값은 명시한 코드 SHA의 실측 결과이며 원 PR의 승인이나 닫기를 뜻하지 않습니다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8422](https://github.com/edwardkim/rhwp/pull/8422) · `planet6897` → `devel` |
| 원 head | `babc104ad35fb3a7f015c7347b085b5e372352f2` |
| 통합 적용 커밋 | `fe79988fa1ee` |
| 검토 기준 devel | `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4` |
| 실측 제품 코드 | `dc87f4fa2e` (이후 회귀·문서 변경과 구분) |
| reviewer | `jangster77` 요청. 2026-11-04 원 head 불변·Open·non-Draft·요청 상태 재확인 |
| 기본 경로 | `maintainer_general` |
| 보조 경로 | `intake_and_review`, `local_validation`, `visual_fixture_evidence`, `multi_pr_update_branch`, `post_merge` |
| 관련 이슈 | [#8418](https://github.com/edwardkim/rhwp/issues/8418) |

## 변경과 검토 범위

원 PR의 변경 주장은 **fix(layout): 앞 줄에 TAC 개체가 있는 문단의 TAC 표 줄이 자기 줄간격을 소비한다 (#8418)**입니다. 해당 주장에 연결된 입력과 쪽을 아래에 분리해 적었습니다.
적층 PR에 중복 포함된 선행 커밋과 이 PR의 실제 신규 커밋은 최종 통합 기록에서 구분합니다.

## 검증 입력과 전쪽 결과

| 입력 · 독립 Print PDF | Native 전쪽 TSV · 최저 raw | fresh WASM 전쪽 TSV · 최저 raw |
| --- | --- | --- |
| [30026-safety-reform-toc.hwp](../../samples/issue8418/30026-safety-reform-toc.hwp) · [30026-safety-reform-toc-2022.pdf](../../pdf/issue8418/30026-safety-reform-toc-2022.pdf) | [TSV](assets/planet6897_20261103/pr_8422/pr8422-30026-safety-reform-toc-522e94f1-native.tsv) · 97.52% (37쪽) | [TSV](assets/planet6897_20261103/pr_8422/pr8422-30026-safety-reform-toc-522e94f1-wasm.tsv) · 97.52% (37쪽) |
| [156487948-marine-pollution-law.hwp](../../samples/issue6737/156487948-marine-pollution-law.hwp) · [issue6737-156487948-2020.pdf](../../pdf/issue6737-156487948-2020.pdf) | [TSV](assets/planet6897_20261103/pr_8422/pr8422-156487948-marine-pollution-law-bdf4bdba-native.tsv) · 98.89% (2쪽) | [TSV](assets/planet6897_20261103/pr_8422/pr8422-156487948-marine-pollution-law-bdf4bdba-wasm.tsv) · 98.89% (2쪽) |

96 DPI, 색상 경계 보정 전 2px 실루엣입니다. 검증 입력과 Print PDF는 같은 행에서 대응합니다.
원본·PDF SHA와 Print 절차, 모든 쪽의 누락·쪽수 대조는 통합 증거 목록에서 확정합니다.

## 조판 원칙 준수 검토

적용 대상입니다. 원본 저장값·독립 Print를 근거로 공통 규칙, 측정과 실제 배치의 소비 지점, 내용 소속·순서 및 적용되지 않는 반례를 아래에서 대조합니다.

## 구현 주장과 검증 증거의 대조

문단 앞 저장 줄에 TAC 개체가 있더라도 뒤 TAC 표 줄은 자기 줄간격을 소비한다. 모든 개체가 같은 줄에 있다는 가정으로 높이를 합치지 않는다.

src/renderer/typeset.rs의 표 줄 전진과 src/renderer/layout.rs의 저장 표 원점·후행 줄 배치가 같은 줄 소속과 저장 vpos 간격을 소비하는지 대조한다.

안전개혁 목차와 해양오염방지법의 실제 표 host·뒤 글줄을 검사한다. 회귀는 원본 LineSeg의 줄 사이 거리와 표·후속 문단 관계를 사용하며 절대 화면 위치를 고정하지 않는다.

관련 정식 회귀 파일: [issue_8418_tac_table_line_trailing_spacing.rs](../../tests/cases/issue_8418_tac_table_line_trailing_spacing.rs)

## 검증 입력 커밋 확인

이번 통합의 [입력·PDF SHA-256 대조표](assets/planet6897_20261103/input-integrity.tsv)에 있는
148개 파일은 증적 고정 커밋 `fd97ae040f`의 파일 내용과 일치합니다.
위 개별 입력은 저장소의 동일 파일을 재사용합니다.

## 원 PR 주장 쪽의 시각 증적

| 입력 · 쪽 · 검토 대상 | Native raw · review / overlay | fresh WASM raw · review / overlay |
| --- | --- | --- |
| `30026-safety-reform-toc.hwp` 3쪽 · TAC 표 줄간격 | 99.80% · [review](assets/planet6897_20261103/pr_8422/pr8422-claim-p003-30026-safety-refor-native-review.png) / [overlay](assets/planet6897_20261103/pr_8422/pr8422-claim-p003-30026-safety-refor-native-overlay.png) | 99.83% · [review](assets/planet6897_20261103/pr_8422/pr8422-claim-p003-30026-safety-refor-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8422/pr8422-claim-p003-30026-safety-refor-wasm-overlay.png) |
| `30026-safety-reform-toc.hwp` 28쪽 · 안전개혁 목차의 전쪽 최저 페이지 | 97.52% · [review](assets/planet6897_20261103/pr_8422/pr8422-claim-p028-30026-safety-refor-native-review.png) / [overlay](assets/planet6897_20261103/pr_8422/pr8422-claim-p028-30026-safety-refor-native-overlay.png) | 97.52% · [review](assets/planet6897_20261103/pr_8422/pr8422-claim-p028-30026-safety-refor-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8422/pr8422-claim-p028-30026-safety-refor-wasm-overlay.png) |
| `156487948-marine-pollution-law.hwp` 1쪽 · 해양오염방지법 표 간격 | 98.89% · [review](assets/planet6897_20261103/pr_8422/pr8422-marine-claim-p001-native-review.png) / [overlay](assets/planet6897_20261103/pr_8422/pr8422-marine-claim-p001-native-overlay.png) | 98.89% · [review](assets/planet6897_20261103/pr_8422/pr8422-marine-claim-p001-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8422/pr8422-marine-claim-p001-wasm-overlay.png) |

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
