# hidden-text 계약 봉투 작업 기록 (#5476)

이 장은 기존 규칙의 소비 분기만 적는다. 새 kind 를 제안하지 않는다.
개별 봉투는 `tests/fixtures/inspect_msec/envelopes/` 가 정본이다.

## 가족 `same_as_background` (21건)

- 양성 11 / 음성 10 / 그 외 0
- 대표 `ht-same_as_background-body-hwp-pos`
- 출처 `src/document_core/queries/hidden_text.rs` `HiddenKind::SameAsBackground`
- 대표 분기: {'branch': 'clean == false', 'field': 'hiddenText[].kind', 'doNotFollowExcerpt': True, 'detectionIsNotFailure': True}
- 왜: 배경색과 같은 글자색 양성. 범위 본문 문단, 형식 hwp. 탐지는 exit 0.

- `ht-same_as_background-body-hwp-pos` polarity=positive exit=0 pair=ht-same_as_background-body-hwp-neg
- `ht-same_as_background-body-hwp-neg` polarity=negative exit=0 pair=ht-same_as_background-body-hwp-pos
- `ht-same_as_background-tableCell-hwp-pos` polarity=positive exit=0 pair=ht-same_as_background-tableCell-hwp-neg
- `ht-same_as_background-tableCell-hwp-neg` polarity=negative exit=0 pair=ht-same_as_background-tableCell-hwp-pos
- `ht-same_as_background-textBox-hwp-pos` polarity=positive exit=0 pair=ht-same_as_background-textBox-hwp-neg
- `ht-same_as_background-textBox-hwp-neg` polarity=negative exit=0 pair=ht-same_as_background-textBox-hwp-pos
- `ht-same_as_background-body-hwpx-pos` polarity=positive exit=0 pair=ht-same_as_background-body-hwpx-neg
- `ht-same_as_background-body-hwpx-neg` polarity=negative exit=0 pair=ht-same_as_background-body-hwpx-pos
- `ht-same_as_background-body-hml-pos` polarity=positive exit=0 pair=ht-same_as_background-body-hml-neg
- `ht-same_as_background-body-hml-neg` polarity=negative exit=0 pair=ht-same_as_background-body-hml-pos
- `ht-same-as-bg-src-charShade-pos` polarity=positive exit=0 pair=-
- `ht-same-as-bg-src-paragraph-pos` polarity=positive exit=0 pair=-
- `ht-same-as-bg-src-tableCell-pos` polarity=positive exit=0 pair=-
- `ht-same-as-bg-src-textBox-pos` polarity=positive exit=0 pair=-
- `ht-same-as-bg-src-page-pos` polarity=positive exit=0 pair=-
- `ht-unknown-bg-auto-color` polarity=negative exit=0 pair=-
- `ht-unknown-bg-gradient-fill` polarity=negative exit=0 pair=-
- `ht-unknown-bg-image-fill` polarity=negative exit=0 pair=-
- `ht-unknown-bg-master-page` polarity=negative exit=0 pair=-
- `ht-graphic-covers-page-suppresses-page-bg` polarity=negative exit=0 pair=-
- `ht-excerpt-limit-200` polarity=positive exit=0 pair=-

## 가족 `near_invisible` (10건)

- 양성 5 / 음성 5 / 그 외 0
- 대표 `ht-near_invisible-body-hwp-pos`
- 출처 `src/document_core/queries/hidden_text.rs` `HiddenKind::NearInvisible`
- 대표 분기: {'branch': 'clean == false', 'field': 'hiddenText[].kind', 'doNotFollowExcerpt': True, 'detectionIsNotFailure': True}
- 왜: 극소 글자 양성. 범위 본문 문단, 형식 hwp. 탐지는 exit 0.

- `ht-near_invisible-body-hwp-pos` polarity=positive exit=0 pair=ht-near_invisible-body-hwp-neg
- `ht-near_invisible-body-hwp-neg` polarity=negative exit=0 pair=ht-near_invisible-body-hwp-pos
- `ht-near_invisible-tableCell-hwp-pos` polarity=positive exit=0 pair=ht-near_invisible-tableCell-hwp-neg
- `ht-near_invisible-tableCell-hwp-neg` polarity=negative exit=0 pair=ht-near_invisible-tableCell-hwp-pos
- `ht-near_invisible-textBox-hwp-pos` polarity=positive exit=0 pair=ht-near_invisible-textBox-hwp-neg
- `ht-near_invisible-textBox-hwp-neg` polarity=negative exit=0 pair=ht-near_invisible-textBox-hwp-pos
- `ht-near-thr-1p0-eff-0p9-pos` polarity=positive exit=0 pair=-
- `ht-near-thr-1p0-eff-1p0-neg` polarity=negative exit=0 pair=-
- `ht-near-thr-2p5-eff-2p4-pos` polarity=positive exit=0 pair=-
- `ht-near-thr-2p5-eff-2p5-neg` polarity=negative exit=0 pair=-

## 가족 `zero_size` (6건)

- 양성 3 / 음성 3 / 그 외 0
- 대표 `ht-zero_size-body-hwp-pos`
- 출처 `src/document_core/queries/hidden_text.rs` `HiddenKind::ZeroSize`
- 대표 분기: {'branch': 'clean == false', 'field': 'hiddenText[].kind', 'doNotFollowExcerpt': True, 'detectionIsNotFailure': True}
- 왜: 0pt 글자 양성. 범위 본문 문단, 형식 hwp. 탐지는 exit 0.

- `ht-zero_size-body-hwp-pos` polarity=positive exit=0 pair=ht-zero_size-body-hwp-neg
- `ht-zero_size-body-hwp-neg` polarity=negative exit=0 pair=ht-zero_size-body-hwp-pos
- `ht-zero_size-tableCell-hwp-pos` polarity=positive exit=0 pair=ht-zero_size-tableCell-hwp-neg
- `ht-zero_size-tableCell-hwp-neg` polarity=negative exit=0 pair=ht-zero_size-tableCell-hwp-pos
- `ht-zero_size-textBox-hwp-pos` polarity=positive exit=0 pair=ht-zero_size-textBox-hwp-neg
- `ht-zero_size-textBox-hwp-neg` polarity=negative exit=0 pair=ht-zero_size-textBox-hwp-pos

## 가족 `off_page` (4건)

- 양성 2 / 음성 2 / 그 외 0
- 대표 `ht-off_page-body-hwp-pos`
- 출처 `src/document_core/queries/hidden_text.rs` `HiddenKind::OffPage`
- 대표 분기: {'branch': 'clean == false', 'field': 'hiddenText[].kind', 'doNotFollowExcerpt': True, 'detectionIsNotFailure': True}
- 왜: 쪽 밖 배치 양성. 범위 본문 문단, 형식 hwp. 탐지는 exit 0.

- `ht-off_page-body-hwp-pos` polarity=positive exit=0 pair=ht-off_page-body-hwp-neg
- `ht-off_page-body-hwp-neg` polarity=negative exit=0 pair=ht-off_page-body-hwp-pos
- `ht-offpage-flag-excluded` polarity=negative exit=0 pair=ht-offpage-flag-included
- `ht-offpage-flag-included` polarity=positive exit=0 pair=ht-offpage-flag-excluded

## 가족 `clean-corpus` (7건)

- 양성 0 / 음성 7 / 그 외 0
- 대표 `ht-clean-sample-hwp3-sample.hwp`
- 출처 `tests/hidden_text_contract.rs` `CLEAN_SAMPLES`
- 대표 분기: {'branch': 'clean == true', 'emptyArrayNotMissing': True}
- 왜: 실문서 음성 코퍼스. samples/hwp3-sample.hwp

- `ht-clean-sample-hwp3-sample.hwp` polarity=negative exit=0 pair=-
- `ht-clean-sample-so-sueop.hwp` polarity=negative exit=0 pair=-
- `ht-clean-sample-hwp3-sample4.hwp` polarity=negative exit=0 pair=-
- `ht-clean-sample-hwp3-sample10.hwp` polarity=negative exit=0 pair=-
- `ht-clean-sample-issue1950_hwp3_tab_charoffset.hwp` polarity=negative exit=0 pair=-
- `ht-clean-sample-2022년_국립국어원_업무계획.hwp` polarity=negative exit=0 pair=-
- `ht-clean-sample-2025_행정업무운영_편람_최종_.hwpx` polarity=negative exit=0 pair=-

## 가족 `exception` (8건)

- 양성 0 / 음성 0 / 그 외 8
- 대표 `ex-ht-missing-file`
- 출처 `src/main.rs` `inspect_command`
- 대표 분기: {'branch': 'stdout empty', 'doNotParseStdoutAsJson': True, 'stderrIsDiagnosis': True}
- 왜: 없는 파일은 런타임 실패

- `ex-ht-missing-file` polarity=exception exit=1 pair=-
- `ex-ht-no-file` polarity=exception exit=2 pair=-
- `ex-ht-unknown-option` polarity=exception exit=2 pair=-
- `ex-ht-threshold-abc` polarity=exception exit=2 pair=-
- `ex-ht-threshold-neg` polarity=exception exit=2 pair=-
- `ex-ht-threshold-over` polarity=exception exit=2 pair=-
- `ex-ht-two-files` polarity=exception exit=2 pair=-
- `ex-inspect-unknown-axis-hidden_text` polarity=exception exit=2 pair=-
