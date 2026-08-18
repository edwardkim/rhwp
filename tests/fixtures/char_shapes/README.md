# char_shapes IR 픽스처 (#3500)

`scripts/extract_char_shape_ir.py` 가 `samples/` 의 HWP5·HWPX 에서 뽑는다.

- `issue_3500_re_multisize.json` — 재현 표본 한 파일의 CHAR_SHAPE + PARA_CHAR_SHAPE
- `corpus_same_id_para_char_shapes.jsonl` — 연속 동일 id 문단

Rust 카탈로그는 `scripts/gen_char_shape_tables.py` 가
`src/serializer/hwpx/char_shape_tables/` 로 다시 쓴다.
