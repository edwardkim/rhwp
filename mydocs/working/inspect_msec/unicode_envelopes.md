# unicode 계약 봉투 작업 기록 (#5476)

이 장은 기존 규칙의 소비 분기만 적는다. 새 kind 를 제안하지 않는다.
개별 봉투는 `tests/fixtures/inspect_msec/envelopes/` 가 정본이다.

## 가족 `zero_width` (16건)

- 양성 12 / 음성 4 / 그 외 0
- 대표 `uni-zw-u+200b-run1-hwp`
- 출처 `src/document_core/text_security.rs` `is_zero_width`
- 대표 분기: {'branch': 'clean == false', 'compareRenderedRaw': True, 'detectionIsNotFailure': True}
- 왜: ZERO WIDTH SPACE ×1 → severity low. 연속 열이 high.

- `uni-zw-u+200b-run1-hwp` polarity=positive exit=0 pair=-
- `uni-zw-u+200b-run3-hwp` polarity=positive exit=0 pair=-
- `uni-zw-u+200c-run1-hwp` polarity=positive exit=0 pair=-
- `uni-zw-u+200c-run3-hwp` polarity=positive exit=0 pair=-
- `uni-zw-u+200d-run1-hwp` polarity=positive exit=0 pair=-
- `uni-zw-u+200d-run3-hwp` polarity=positive exit=0 pair=-
- `uni-zw-u+2060-run1-hwp` polarity=positive exit=0 pair=-
- `uni-zw-u+2060-run3-hwp` polarity=positive exit=0 pair=-
- `uni-zw-u+feff-run1-hwp` polarity=positive exit=0 pair=-
- `uni-zw-u+feff-run3-hwp` polarity=positive exit=0 pair=-
- `uni-zw-excluded-u+00ad` polarity=negative exit=0 pair=-
- `uni-zw-excluded-u+180e` polarity=negative exit=0 pair=-
- `uni-zw-excluded-u+061c` polarity=negative exit=0 pair=-
- `uni-zw-hangul-pua-typesetting` polarity=negative exit=0 pair=-
- `uni-loc-body` polarity=positive exit=0 pair=-
- `uni-loc-cell_0_0_.para_0` polarity=positive exit=0 pair=-

## 가족 `bidi_override` (10건)

- 양성 10 / 음성 0 / 그 외 0
- 대표 `uni-bidi-lre-hwp`
- 출처 `src/document_core/text_security.rs` `is_bidi_control`
- 대표 분기: {'branch': 'clean == false', 'compareRenderedRaw': True}
- 왜: LEFT-TO-RIGHT EMBEDDING. rendered 와 raw 를 나란히 싣는다.

- `uni-bidi-lre-hwp` polarity=positive exit=0 pair=-
- `uni-bidi-rle-hwp` polarity=positive exit=0 pair=-
- `uni-bidi-pdf-hwp` polarity=positive exit=0 pair=-
- `uni-bidi-lro-hwp` polarity=positive exit=0 pair=-
- `uni-bidi-rlo-hwp` polarity=positive exit=0 pair=-
- `uni-bidi-lri-hwp` polarity=positive exit=0 pair=-
- `uni-bidi-rli-hwp` polarity=positive exit=0 pair=-
- `uni-bidi-fsi-hwp` polarity=positive exit=0 pair=-
- `uni-bidi-pdi-hwp` polarity=positive exit=0 pair=-
- `uni-bidi-rendered-vs-raw-exe-doc` polarity=positive exit=0 pair=-

## 가족 `tag_char` (5건)

- 양성 5 / 음성 0 / 그 외 0
- 대표 `uni-tag-ignore-payload`
- 출처 `src/document_core/text_security.rs` `is_tag_char`
- 대표 분기: {'branch': 'findings[0].hidden', 'hidden': 'Ignore'}
- 왜: 태그 문자로 실어 나른 숨은 지시 Ignore.

- `uni-tag-ignore-payload` polarity=positive exit=0 pair=-
- `uni-tag-range-u+e0000` polarity=positive exit=0 pair=-
- `uni-tag-range-u+e0020` polarity=positive exit=0 pair=-
- `uni-tag-range-u+e0049` polarity=positive exit=0 pair=-
- `uni-tag-range-u+e007f` polarity=positive exit=0 pair=-

## 가족 `confusable` (5건)

- 양성 4 / 음성 1 / 그 외 0
- 대표 `uni-cf-cyr-lower-0430`
- 출처 `src/document_core/text_security.rs` `confusable_to_latin`
- 대표 분기: {'branch': 'clean == false', 'latin': 'a'}
- 왜: 라틴 낱말에 а (정규 a) 가 섞였다. 전체 표는 matrices/unicode_confusable.tsv.

- `uni-cf-cyr-lower-0430` polarity=positive exit=0 pair=-
- `uni-cf-cyr-upper-0422` polarity=positive exit=0 pair=-
- `uni-cf-gr-lower-03b1` polarity=positive exit=0 pair=-
- `uni-cf-gr-upper-0391` polarity=positive exit=0 pair=-
- `uni-cf-neg-pure-cyrillic` polarity=negative exit=0 pair=-

## 가족 `kind-filter` (5건)

- 양성 1 / 음성 0 / 그 외 4
- 대표 `uni-filter-all-mixed`
- 출처 `src/document_core/text_security.rs` `DeceptionKind::ALL`
- 대표 분기: {'branch': 'findingCount == 4'}
- 왜: tests/unicode_deception_contract.rs 의 PAYLOAD 와 같은 네 축.

- `uni-filter-all-mixed` polarity=positive exit=0 pair=-
- `uni-filter-zero-width` polarity=filter exit=0 pair=uni-filter-all-mixed
- `uni-filter-bidi` polarity=filter exit=0 pair=uni-filter-all-mixed
- `uni-filter-tag` polarity=filter exit=0 pair=uni-filter-all-mixed
- `uni-filter-confusable` polarity=filter exit=0 pair=uni-filter-all-mixed

## 가족 `clean-corpus` (3건)

- 양성 0 / 음성 3 / 그 외 0
- 대표 `uni-clean-2026_oss_rst.hwp`
- 출처 `tests/unicode_deception_contract.rs` `clean_document_reports_empty_findings_not_a_missing_key`
- 대표 분기: {'branch': 'clean == true', 'emptyArrayNotMissing': True, 'kindCountsPresent': True, 'scannedCharsPositive': True}
- 왜: 검사했는데 깨끗함 ≠ 검사하지 않음.

- `uni-clean-2026_oss_rst.hwp` polarity=negative exit=0 pair=-
- `uni-clean-hwp3-sample.hwp` polarity=negative exit=0 pair=-
- `uni-clean-2022년_국립국어원_업무계획.hwp` polarity=negative exit=0 pair=-

## 가족 `exception` (5건)

- 양성 0 / 음성 0 / 그 외 5
- 대표 `ex-uni-missing-file`
- 출처 `src/main.rs` `inspect_command`
- 대표 분기: {'branch': 'stdout empty', 'doNotParseStdoutAsJson': True, 'stderrIsDiagnosis': True}
- 왜: 없는 파일은 런타임 실패

- `ex-uni-missing-file` polarity=exception exit=1 pair=-
- `ex-uni-no-file` polarity=exception exit=2 pair=-
- `ex-uni-kind-bad` polarity=exception exit=2 pair=-
- `ex-uni-unknown-option` polarity=exception exit=2 pair=-
- `ex-uni-two-files` polarity=exception exit=2 pair=-
