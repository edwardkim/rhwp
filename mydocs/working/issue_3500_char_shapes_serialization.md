# #3500 / #5451 — HWPX char_shapes 직렬화

## 이슈

`samples/re-multisize-10-10-empty-hancom.hwp` 를 HWPX 로 내보낸 뒤 다시 읽으면
문단 0 의 `char_shapes` 가 달라진다.

```
expected=[(0,0),(34,0),(53,0)]
actual=[(0,0)]
```

쪽수(`--verify-pages`) 와 OLE 축이 아니다. PARA_CHAR_SHAPE 직렬화만의 문제다.

## 표본 실측

FileHeader `flags & 1` = 압축. DocInfo CHAR_SHAPE 5칸, BodyText/Section0 문단 1개.

PARA_TEXT 73 UTF-16 유닛:

| 구간 | 유닛 | 내용 |
| --- | --- | --- |
| 0..8 | secd | 구역 정의 슬롯 |
| 8..16 | cold | 단 정의 슬롯 |
| 16..72 | 본문 | `가나다라마바사아자차카타파하` × 4 |
| 72 | `U+000D` | 문단 끝 |

PARA_CHAR_SHAPE 24바이트 = entry 3개:

| start_pos | id | 위치 |
| --- | --- | --- |
| 0 | 0 | 문단 시작(슬롯 포함) |
| 34 | 0 | 본문 두 번째 `마` |
| 53 | 0 | 본문 세 번째 `차` |

DocInfo CHAR_SHAPE:

| id | base_size | font_ids[0] | 의미 |
| --- | --- | --- | --- |
| 0 | 1000 | 1 | 10pt, 본문 글꼴 |
| 1 | 1000 | 0 | 10pt, 기본 글꼴 |
| 2 | 900 | 0 | 9pt |
| 3 | 900 | 1 | 9pt, 본문 글꼴 |
| 4 | 900 | 0 | 9pt |

이름은 `multisize-10-10` 이지만 테이블에 9pt 슬롯이 같이 들어 있다. 본문 run 은
전부 id 0 을 가리킨다. 34·53 경계는 글자 크기가 바뀐 자리가 아니라 **같은 모양의
추가 시작점** 이다. 한컴이 크기 변경 후 되돌리거나 줄 경계에 entry 를 남긴 흔적이다.

## 왜 접히면 안 되는가

렌더 결과는 같다. 그러나 `--verify` 는 `start_pos`+`id` 시퀀스를 그대로 비교한다
(`serializer/hwpx/roundtrip.rs` `diff_paragraph_char_shapes`). HWP5 파서는
원본 entry 를 보존하고, HWPX 는 run 시작으로만 경계를 표현한다. 연속 동일 id 를
한 run 으로 합치면 재파싱이 `[(0,0)]` 만 남긴다.

첫 문단 템플릿은 `secPr` 전용 run 을 하나 더 둔다. 파서는 그 다음 **같은 id**
run 하나를 템플릿 handoff 로 정규화한다 (`issue_3739_secpr_template_handoff`).
handoff 가 삼키는 것은 **텍스트가 없는 템플릿 짝** 뿐이어야 하고, 34·53 처럼
본문 한가운데 있는 동일-id 경계는 별도 `<hp:run>` 으로 남아야 한다.

## 직렬화 계약

1. `plan_run_boundaries` 는 `CharShapeRef` 를 접지 않는다.
2. `RunSplitter` 는 그 시퀀스 그대로 cut 한다. 연속 동일 id 도 새 run.
3. `hh:charPr` 방출은 `char_shapes::write_char_pr` 한곳. 언어 7칸·선 13종·
   외곽선 8종·그림자 3종·강조점 7종 표는 `char_shape_tables`.
4. 음영 `0xFFFFFFFF` 만 `shadeColor="none"`. `0x00000000` 은 `#000000`.
5. 취소선이 꺼져 있으면 `shape="NONE"` (shape 숫자만 보면 파서가 켠다).

## HWP5 ↔ HWPX 표

| HWP5 | 오프셋/비트 | HWPX |
| --- | --- | --- |
| font_ids[7] | 0 | `hh:fontRef@hangul..user` |
| ratios[7] | 14 | `hh:ratio` |
| spacings[7] | 21 | `hh:spacing` |
| relative_sizes[7] | 28 | `hh:relSz` (기본 100) |
| char_offsets[7] | 35 | `hh:offset` |
| base_size | 42 | `hh:charPr@height` |
| attr italic | bit 0 | `hh:italic` |
| attr bold | bit 1 | `hh:bold` |
| underline type | bits 2-3 | `hh:underline@type` |
| underline shape | bits 4-7 | `hh:underline@shape` |
| outline | bits 8-10 | `hh:outline@type` |
| shadow | bits 11-12 | `hh:shadow@type` |
| emboss/engrave | 13/14 | `hh:emboss` / `hh:engrave` |
| super/sub | 15/16 | `hh:supscript` / `hh:subscript` |
| emphasis | bits 21-24 | `hh:charPr@symMark` |
| use_font_space | bit 25 | `@useFontSpace` |
| strike shape | bits 26-29 | `hh:strikeout@shape` |
| kerning | bit 30 | `@useKerning` |
| colors | 52..73 | text/underline/shade/shadow/strike |
| PARA_CHAR_SHAPE | 8바이트×N | `<hp:run charPrIDRef>` 시작점 |

## 픽스처

| 경로 | 내용 |
| --- | --- |
| `tests/fixtures/char_shapes/issue_3500_re_multisize.json` | 표본 전체 IR |
| `tests/fixtures/char_shapes/corpus_same_id_para_char_shapes.jsonl` | 동일-id 문단 |
| `src/serializer/hwpx/char_shape_tables/same_id_corpus.rs` | 시험이 직접 순회 |
| `src/serializer/hwpx/char_shape_tables/shape_catalog.rs` | 동일-id 파일 테이블 |
| `src/serializer/hwpx/char_shape_tables/encoding_matrix.rs` | charPr 토큰 전조합 |

재추출: `python scripts/extract_char_shape_ir.py` 후
`python scripts/gen_char_shape_tables.py`.

## 시험

- `issue_3500_char_shapes_roundtrip` — 표본 parse → serialize_hwpx → reparse
- `issue_3500_same_id_mid_text_emits_three_runs` — 슬롯 없는 본문 구간
- `encoding_matrix_tokens_appear_in_char_pr` — 표 전조합이 XML 에 남는지
- 코퍼스 TSV/JSONL 행 수 일치

쪽수 로직·OLE·gym 은 이 PR 범위 밖이다.
