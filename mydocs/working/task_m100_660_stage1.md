# Task #660 Stage 1 — JSON 스키마 + serde 모델 보고서

## 인도물

- `Cargo.toml` — `serde_json = "1"` 추가
- `src/parser/mod.rs` — `pub mod ingest;` 등록
- `src/parser/ingest/mod.rs` — `parse_ingest_bytes` / `parse_ingest_str` 진입점
- `src/parser/ingest/schema.rs` — serde 모델 (`IngestDocument`, `Question`, `StemBlock`, `Choice`, `Media`, `Placement`)
- `tools/rhwp-ingest/schema/ingest_schema_v1.json` — JSON Schema 정의
- `tools/rhwp-ingest/schema/sample_minimal.json` — 3문제 샘플
- `src/document_core/queries/search_query.rs` — `serde_json` type inference 충돌 회피 (`vec![]` → `Vec::<usize>::new()`, 3곳)

## 검증

```
cargo test --release --lib parser::ingest
running 4 tests
test parser::ingest::schema::tests::test_placement_serde ... ok
test parser::ingest::schema::tests::test_placement_default ... ok
test parser::ingest::schema::tests::test_roundtrip ... ok
test parser::ingest::schema::tests::test_parse_minimal ... ok
test result: ok. 4 passed
```

`cargo build --release` 무경고/무에러 통과.

## 인터페이스 확정

```jsonc
{
  "version": "1",
  "page_size": {"width_mm": 210, "height_mm": 297},
  "default_font": "함초롬바탕",
  "questions": [{
    "number": 1, "stem": "...",
    "stem_blocks": [{"type":"text","text":"..."}, {"type":"image","ref":"img/q1.png","placement":"between"}],
    "choices": [{"label":"①","text":"..."}],
    "media": [{"id":"...","natural_w":1024,"natural_h":768,"target_w_mm":80,"placement":"between"}]
  }]
}
```

`Placement` enum: `between` / `above` / `below` / `inline` (기본값 `between`).

## 다음 단계

Stage 2: 빌더(`exam_paper.rs`) + CLI 명령(`build-from-ingest`) + e2e 검증.
