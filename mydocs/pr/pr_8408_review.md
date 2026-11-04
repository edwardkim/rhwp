# PR #8408 리뷰 — fix(pdf-import): 공백만 있는 한 줄 문단이 다음 관측 줄까지의 띠만 차지해 원본 PDF 줄 위치를 지킨다 (#8381, #8380 위)

## 최종 판정

**메인터너 보정 후 수용 가능**
메인터너가 통합 후보의 실제 전쪽 증적과 회귀 근거를 보강했습니다. 원 PR에 대응하는 입력·쪽과 반례는 아래에 기록했습니다.
원 head와 리베이스·메인터너 보정을 적용한 통합 후보를 구분합니다. 실제 적용 커밋은 아래 표에 기록했습니다.
완료된 검증과 남은 공통 검증을 아래에서 구분합니다. 최종 수용 판정과 PR 생성은 아직 대기입니다.
아래 값은 명시한 코드 SHA의 실측 결과이며 원 PR의 승인이나 닫기를 뜻하지 않습니다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8408](https://github.com/edwardkim/rhwp/pull/8408) · `planet6897` → `devel` |
| 원 head | `b10341be243cf605f7d05ef81d29e0f065999329` |
| 통합 적용 커밋 | `60ef42d03155c7b5f7a1eb7947012e996d849098, 046e200c42b6c873298112e869859ba2182f0f22, a84f1c4808656cbf57f80e18f69f98a176eaa953` |
| 최종 검토 기준 devel | `22947f6a90c998e95fa51238d5b907a0ebd37162` |
| 최종 필수 검증 제품 코드 | `a7e5ce7112ea8b227c317a9cbfb9d6fed0b00973` |
| 전쪽 시각 측정 producer / 당시 base | `4b149b6cdfc514b9858ed644be2cf8120c731cd4` / `7fdd5441ff1e77c083818e920a48089f294de35e` |
| reviewer | `jangster77` 요청. 2026-11-04T17:44:08.556Z 원 head 불변·Open·non-Draft·요청 상태 재확인 |
| 기본 경로 | `maintainer_general` |
| 보조 경로 | `intake_and_review`, `local_validation`, `visual_fixture_evidence`, `multi_pr_update_branch`, `post_merge` |
| 관련 이슈 | [#8381](https://github.com/edwardkim/rhwp/issues/8381) |

## 변경과 검토 범위

원 PR의 변경 주장은 **fix(pdf-import): 공백만 있는 한 줄 문단이 다음 관측 줄까지의 띠만 차지해 원본 PDF 줄 위치를 지킨다 (#8381, #8380 위)**입니다. 해당 주장에 연결된 입력과 쪽을 아래에 분리해 적었습니다.
적층 PR에 중복 포함된 선행 커밋과 이 PR의 실제 신규 커밋은 최종 통합 기록에서 구분합니다.

## 검증 입력과 전쪽 결과

| 입력 · 독립 Print PDF | Native 전쪽 TSV · 최저 raw | fresh WASM 전쪽 TSV · 최저 raw |
| --- | --- | --- |
| [editable-2010.hwpx](assets/issue_8381/integration_20261103/editable-2010.hwpx) · [21_언어_기출_편집가능본-2010.pdf](../../samples/21_언어_기출_편집가능본-2010.pdf) | [TSV](assets/planet6897_20261103/pr_8408/pr8408-editable-2010-b05322b0-native.tsv) · 97.8% (16쪽) | [TSV](assets/planet6897_20261103/pr_8408/pr8408-editable-2010-b05322b0-wasm.tsv) · 98.08% (16쪽) |
| [editable-2020.hwpx](assets/issue_8381/integration_20261103/editable-2020.hwpx) · [21_언어_기출_편집가능본-2020.pdf](../../samples/21_언어_기출_편집가능본-2020.pdf) | [TSV](assets/planet6897_20261103/pr_8408/pr8408-editable-2020-678a41ff-native.tsv) · 99.19% (15쪽) | [TSV](assets/planet6897_20261103/pr_8408/pr8408-editable-2020-678a41ff-wasm.tsv) · 99.18% (15쪽) |
| [editable-plain.hwpx](assets/issue_8381/integration_20261103/editable-plain.hwpx) · [21_언어_기출_편집가능본.pdf](../../samples/21_언어_기출_편집가능본.pdf) | [TSV](assets/planet6897_20261103/pr_8408/pr8408-editable-plain-0a7d5835-native.tsv) · 98.54% (15쪽) | [TSV](assets/planet6897_20261103/pr_8408/pr8408-editable-plain-0a7d5835-wasm.tsv) · 98.44% (15쪽) |

96 DPI, 색상 경계 보정 전 2px 실루엣입니다. 검증 입력과 Print PDF는 같은 행에서 대응합니다.
전쪽 TSV·PNG는 위 실제 producer에서 산출한 증거입니다. 최신 head의 재출력으로 표시하지 않습니다.
충돌 없는 리베이스 뒤 영향 소비 경로를 새 제품으로 집중 검증 중이며, 완료한 실행과 남은 대조를 구분합니다.
원본·PDF SHA와 Print 절차, 모든 쪽의 누락·쪽수 대조는 통합 증거 목록에서 확정합니다.

## 조판 원칙 준수 검토

적용 대상입니다. 원본 저장값·독립 Print를 근거로 공통 규칙, 측정과 실제 배치의 소비 지점, 내용 소속·순서 및 적용되지 않는 반례를 아래에서 대조합니다.

## 구현 주장과 검증 증거의 대조

공백만 있는 한 줄 문단은 다음 관측 PDF 줄까지의 띠를 점유한다. 보이는 잉크가 있는 줄의 상자까지 줄이는 규칙으로 확대하지 않는다.

src/parser/pdf/editable.rs의 reflow에서 관측 줄 사이의 간격을 저장 LineSeg와 마지막 줄의 고정 피치에 함께 반영한다. 생성된 HWPX를 다시 열어 출력한 결과를 원본 PDF와 대조하므로 가져오기 중간값만 확인하는 검사와 구분한다.

무표기본 5·13쪽의 제목→본문 간격은 PDF bbox의 2424·2364HU라는 독립 근거를 쓴다. 2010판 14쪽의 위 첨자와 기존 행·paint 순서 검사도 함께 보호한다. fresh WASM의 `HwpDocument.importPdf(bytes, {layout: "editable"}) → exportHwpx()` 실제 실행 결과 3종이 Native importer 산출물과 바이트 단위로 일치했다. [WASM 실제 API 산출물 대조](assets/planet6897_20261103/wasm-producer-parity.md)를 연결한다.

관련 정식 회귀 파일: [issue_8355_pdf_editable_fixed_trailing.rs](../../tests/cases/issue_8355_pdf_editable_fixed_trailing.rs) · [issue_8362_pdf_editable_hancom_rows.rs](../../tests/cases/issue_8362_pdf_editable_hancom_rows.rs) · [issue_8362_pdf_editable_paint_order.rs](../../tests/cases/issue_8362_pdf_editable_paint_order.rs) · [issue_8381_pdf_editable_blank_row.rs](../../tests/cases/issue_8381_pdf_editable_blank_row.rs)

## 변경 전의 실제 가져오기·편집 결과

[동일 원본의 base/candidate API 산출물 대조](assets/planet6897_20261103/native-producer-before-after.md)와
[실제 WASM API와 Native 산출물 일치](assets/planet6897_20261103/wasm-producer-parity.md)를 함께 확인했습니다.

2010·2020판은 base/candidate importer 결과가 바이트 단위로 같은 정상 대조군입니다.
무표기본은 같은 원본 PDF를 base importer로 가져온 HWPX의 5쪽 75.59%·13쪽 72.85%에서
현재 importer의 실제 산출물로 개선됐습니다. 아래 TSV는 변경 전의 주장 쪽 두 쪽만 포함합니다.

[변경 전 HWPX](assets/planet6897_20261103/before-producer/editable-plain-base.hwpx) · [변경 전 선택 쪽 TSV](assets/planet6897_20261103/before-producer/editable-plain-base-selected-native.tsv)

5쪽: [review](assets/planet6897_20261103/before-producer/editable-plain-base-p005-review.png) / [overlay](assets/planet6897_20261103/before-producer/editable-plain-base-p005-overlay.png)
13쪽: [review](assets/planet6897_20261103/before-producer/editable-plain-base-p013-review.png) / [overlay](assets/planet6897_20261103/before-producer/editable-plain-base-p013-overlay.png)

## 검증 입력 커밋 확인

이번 통합의 [입력·PDF SHA-256 대조표](assets/planet6897_20261103/input-integrity.tsv)에 있는
148개 파일은 증적 고정 커밋 `fd97ae040f`의 파일 내용과 일치합니다.
위 개별 입력은 저장소의 동일 파일을 재사용합니다.

## 원 PR 주장 쪽의 시각 증적

| 입력 · 쪽 · 검토 대상 | Native raw · review / overlay | fresh WASM raw · review / overlay |
| --- | --- | --- |
| `editable-2010.hwpx` 14쪽 · 2010판 PDF 편집본의 위 첨자와 다음 본문 줄 | 98.93% · [review](assets/planet6897_20261103/pr_8408/pr8408-editable-2010-p014-native-review.png) / [overlay](assets/planet6897_20261103/pr_8408/pr8408-editable-2010-p014-native-overlay.png) | 98.91% · [review](assets/planet6897_20261103/pr_8408/pr8408-editable-2010-p014-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8408/pr8408-editable-2010-p014-wasm-overlay.png) |
| `editable-2020.hwpx` 1쪽 · 2020판 PDF 편집 가져오기의 정상 대조군 | 99.76% · [review](assets/planet6897_20261103/pr_8408/pr8408-editable-2020-p001-native-review.png) / [overlay](assets/planet6897_20261103/pr_8408/pr8408-editable-2020-p001-native-overlay.png) | 99.73% · [review](assets/planet6897_20261103/pr_8408/pr8408-editable-2020-p001-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8408/pr8408-editable-2020-p001-wasm-overlay.png) |
| `editable-plain.hwpx` 5쪽 · 무표기 PDF 편집본의 공백 줄 뒤 본문 | 99.67% · [review](assets/planet6897_20261103/pr_8408/pr8408-editable-plain-p005-native-review.png) / [overlay](assets/planet6897_20261103/pr_8408/pr8408-editable-plain-p005-native-overlay.png) | 99.68% · [review](assets/planet6897_20261103/pr_8408/pr8408-editable-plain-p005-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8408/pr8408-editable-plain-p005-wasm-overlay.png) |
| `editable-plain.hwpx` 13쪽 · 무표기 PDF 편집본의 공백 줄 뒤 본문 | 99.68% · [review](assets/planet6897_20261103/pr_8408/pr8408-editable-plain-p013-native-review.png) / [overlay](assets/planet6897_20261103/pr_8408/pr8408-editable-plain-p013-native-overlay.png) | 99.69% · [review](assets/planet6897_20261103/pr_8408/pr8408-editable-plain-p013-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8408/pr8408-editable-plain-p013-wasm-overlay.png) |

## 통합 최종 검증 조건

- 제품 `a7e5ce7112ea8b227c317a9cbfb9d6fed0b00973`의 전체 Rust 12,561개 통과와 Native Skia·CI 통합 모듈 21개·Rust lint를 확인했습니다.
- 최적화 fresh WASM의 실제 브라우저 실행과 공개 API 검사를 완료했습니다.
- 전쪽 측정 producer에서 Native/fresh WASM 각 80개 입력·406쪽은 모두 90% 이상입니다.
- Native의 당시 base 대비 406쪽 대조와 PR #8389·#8397의 원 TSV 표시값 무하락 대조는 완료했습니다.
- WASM의 당시 base 대비 전쪽 대조와 최신 base 기능 보존의 남은 실행은 아래와 같습니다.

- 대기: 동일 환경에서 당시 base 대비 WASM 80개 입력·406쪽의 0.1%p 하락 게이트 완료
- 새 base의 실제 Native/WASM 제품과 후보의 집중 대조 14쪽은 모두 같은 raw입니다. [실행·대조 원본](assets/planet6897_20261103/rebase_scope_22947f6a90/base-comparison.json)을 확인했습니다.
- 대기: 주장 쪽 직접 판독·회귀 주석·수동 작성 리뷰와 PR 본문 최종 감사

이 대기 항목을 완료하지 않은 상태에서는 최종 검증 완료·승인으로 판정하지 않습니다.
[완료·대기 검증 원본 기록](assets/planet6897_20261103/review-preparation-a7e5ce7112.json)

## Merge 후 원 PR 후속 처리

통합 PR 생성·병합과 원 PR 댓글·닫기는 아직 수행하지 않았습니다.
실제 병합 뒤 원 PR의 주장 문서·페이지에 대응하는 증거와 merge SHA를 연결합니다.

<!-- planet6897-20261103-generated-review -->
