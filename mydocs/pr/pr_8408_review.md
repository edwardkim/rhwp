# PR #8408 리뷰 — fix(pdf-import): 공백만 있는 한 줄 문단이 다음 관측 줄까지의 띠만 차지해 원본 PDF 줄 위치를 지킨다 (#8381, #8380 위)

## 최종 판정

**메인터너 보정 후 수용 가능 — 최종 공통 검증 대기**

PDF PostScript 이름의 family 보정과 새 실제 API 저장본을 같은 독립 PDF로 재검증했습니다.
새 base `7fdd5441ff` 반영 뒤 Native 전46쪽과 최적화 fresh WASM 전46쪽 모두 90% 이상이며,
누락·쪽수 불일치가 없습니다. Native 최저는 2010판 97.80%, 2020판 99.19%, 무표기판 98.54%,
fresh WASM은 각각 98.08%, 99.18%, 98.44%입니다.

실제 WASM `importPdf → exportHwpx` 산출물 세 개도 현재 제출 HWPX와 각각 바이트가 같습니다.
무표기판 현재 SHA `5a13ddd9…`와 이전 `7a54ea97…` 저장본을 구분하고 아래 표·PNG를 현재 파일로
갱신했습니다. [입력·전쪽 TSV·같은 쪽 증적](assets/issue_8381/integration_20261103/native-postscript-family.md)에 근거를 연결합니다.

원 head와 메인터너 보정을 적용한 통합 후보를 구분합니다. 남은 최종 Rust·lint·새 base 대비
전쪽 하락 대조와 PR #8389·#8397 보존 검증은 완료 후 확정합니다. PR 생성·push는 작업지시자
승인 대기이며 원 PR의 원격 승인·닫기를 실행하지 않았습니다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8408](https://github.com/edwardkim/rhwp/pull/8408) · `planet6897` → `devel` |
| 원 head | `b10341be243cf605f7d05ef81d29e0f065999329` |
| 통합 적용 커밋 | `070120f551`, `e172dbe0d5`, `e8978a71c3` |
| 검토 기준 devel | `7fdd5441ff1e77c083818e920a48089f294de35e` |
| 실측 제품 코드 | `4b149b6cdf` / 최적화 WASM 실제 빌드 `7bb9e66fd0`(같은 runtime source) |
| reviewer | `jangster77` 요청. 2026-11-04 원 head 불변·Open·non-Draft·요청 상태 재확인 |
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
원본·PDF SHA와 기존 독립 Print의 출처는 위 전쪽 측정 기록에 있습니다. 현재 Git의 파일 바이트와 실행 입력이 같음을 확인했습니다.

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
| `editable-plain.hwpx` 5쪽 · 무표기 PDF 편집본의 공백 줄 뒤 본문 | 99.67% · [review](assets/planet6897_20261103/pr_8408/pr8408-editable-plain-p005-native-review.png) / [overlay](assets/planet6897_20261103/pr_8408/pr8408-editable-plain-p005-native-overlay.png) | 99.68% · [review](assets/planet6897_20261103/pr_8408/pr8408-editable-plain-p005-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8408/pr8408-editable-plain-p005-wasm-overlay.png) |
| `editable-plain.hwpx` 13쪽 · 무표기 PDF 편집본의 공백 줄 뒤 본문 | 99.68% · [review](assets/planet6897_20261103/pr_8408/pr8408-editable-plain-p013-native-review.png) / [overlay](assets/planet6897_20261103/pr_8408/pr8408-editable-plain-p013-native-overlay.png) | 99.69% · [review](assets/planet6897_20261103/pr_8408/pr8408-editable-plain-p013-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8408/pr8408-editable-plain-p013-wasm-overlay.png) |

## 통합 최종 검증 조건

- 기본 최소 글꼴 전체 실행은 `895205d5a7`에서 12,557 PASS·1 FAIL·45 skipped입니다. 이 PR의 관련 검사들은 통과했습니다. 별도 부적합 검사 #713의 이관과 현재 신규 회귀를 다른 검사의 통과로 덮어 쓰지 않습니다.
- 새 최적화 WASM의 브라우저 계약 271개 assertion과 실제 PDF 가져오기·저장 3종은 통과했습니다.
- 현재 제품의 같은 입력·쪽 review/overlay와 전쪽 TSV를 사용합니다. 이전 저장본의 96.14%를 현재 결과로 재사용하지 않습니다.
- 새 base 대비 페이지별 raw 0.1%p 초과 하락과 PR #8389·#8397 원 TSV 표시값 하락이 없음을 최종 목록 전체에서 확인합니다.
- 필수 lint·Native Skia 3종·나머지 통합 입력의 fresh WASM과 최종 개별 리뷰를 마친 뒤 통합 후보 판정을 확정합니다.

## Merge 후 원 PR 후속 처리

통합 PR의 merge SHA와 검증한 동일 입력·쪽의 TSV 및 review/overlay를 원 PR에 연결합니다.
댓글 게시와 원 PR 닫기는 최종 통합 검증 및 merge 후에 판정합니다.

<!-- planet6897-20261103-generated-review -->
