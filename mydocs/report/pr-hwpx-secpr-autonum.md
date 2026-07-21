# Issue #2742 — HWPX secPr autoNumFormat 10속성 미반영 (구역 각주/미주 모양 소실)

## 개요

HWPX section properties(secPr) 직렬화 시 `footNotePr`/`endNotePr` 내 `<hp:autoNumFormat>` 요소의
10개 속성(각주 5 + 미주 5)이 템플릿 기본값으로 하드코딩되어 IR 값이 반영되지 않는 결함 수정.

해당 속성:
- `type` (number_format) — 번호 모양 (DIGIT/CIRCLED_DIGIT/ROMAN_CAPITAL/…)
- `userChar` (user_char) — 사용자 기호 문자
- `prefixChar` (prefix_char) — 앞 장식 문자
- `suffixChar` (suffix_char) — 뒤 장식 문자
- `supscript` (number_code_superscript) — 위첨자 출력 여부

위 5속성이 각주(footNotePr)와 미주(endNotePr) 각각에 존재하므로 총 10개 속성 미반영.
103 슬롯(secPr notePr 전체 속성) 중 결함 12건.

## 분석

- **파서**: `parser/hwpx/section.rs` `parse_note_pr_children()` — `autoNumFormat`의 5속성을
  `FootnoteShape.number_format`/`user_char`/`prefix_char`/`suffix_char`/`number_code_superscript`로
  정확히 파싱하여 IR에 보존함.
- **직렬화기**: `serializer/hwpx/section.rs` `replace_footnote_shape()` — 기존에는 `noteLine`,
  `noteSpacing`, `numbering`, `placement` 4요소만 치환하고 `autoNumFormat`은 미치환.
  템플릿(`empty_section0.xml`)의 고정값 `type="DIGIT" userChar="" prefixChar="" suffixChar=")" supscript="0"`이
  그대로 방출되어 번호 모양/장식문자/위첨자가 무조건 기본값으로 저장됨.

## 수정 내용

`src/serializer/hwpx/section.rs`:

1. **`note_number_format_to_str()` 함수 추가** (line 246)
   - `NumberFormat` enum → HWPX `type` 토큰 문자열 역매핑
   - 파서 `number_format_from_name()`의 정확한 역함수

2. **`render_auto_num_format()` 함수 추가** (line 273)
   - `FootnoteShape` → `<hp:autoNumFormat type="…" userChar="…" prefixChar="…" suffixChar="…" supscript="…"/>`

3. **`replace_footnote_shape()`에 치환 로직 추가** (line 327–336)
   - 기존 `numbering` 치환 직후 `replace_first_two()`로 각주/미주 `autoNumFormat` 각각 치환
   - fn/en 템플릿 문자열이 동일하므로 위치 기반 2회 치환 사용

## 영향

- `secPr` 내 각주/미주 번호 형식 저장 충실도 복원
- 라운드트립(저장→재파싱) 시 `autoNumFormat` 속성 보존
- 기존 동작에 영향 없음 (미파싱 문서는 템플릿 기본값 유지)
