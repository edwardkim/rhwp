# rhwp-exam-ingest references

SKILL.md 가 인덱스다. 장은 번호 순으로 연다.

| 장 | 파일 | 한 줄 |
| --- | --- | --- |
| 00 | 00_tree.md | 판단 트리, 살아 있는 동사 |
| 01 | 01_input_normalize.md | PDF/IMG/MD/DOCX → TMP |
| 02 | 02_pdf_to_pngs.md | poppler/magick, page_001 |
| 03 | 03_extract_docx.md | python-docx / zip fallback |
| 04 | 04_image_passthrough.md | 변환 없이 Read |
| 05 | 05_md_image_refs.md | `![alt](path)` → media |
| 06 | 06_ingest_schema_v1.md | deny_unknown_fields |
| 07 | 07_passages_questions.md | 공유 지문 |
| 08 | 08_stem_blocks_boxed.md | 보기 박스 |
| 09 | 09_media_placement.md | between/above/below/inline |
| 10 | 10_auto_number.md | true/false 정책 |
| 11 | 11_crop_bbox.md | 픽셀 정수 bbox |
| 12 | 12_build_from_ingest.md | `--media-dir -o` |
| 13 | 13_check_deps.md | DEP_MISS_* 봉투 |
| 14 | 14_failure_envelopes.md | helper code 목록 |
| 15 | 15_known_limits.md | Picture·수식·표 |
| 16 | 16_pitfalls.md | 중복 번호 등 |
| 17 | 17_sample_transcripts.md | T01–T10 |
| 18 | 18_verify_gate.md | export-text / dump |
| 19 | 19_intent_matrix.md | 발화 → 동작 |
| 20 | 20_exit_codes.md | helper vs rhwp |

예제 워크스루: `../examples/`.
기계 픽스처: `../fixtures/`.
정본 스키마: `../../../../tools/rhwp-ingest/schema/ingest_schema_v1.json`.

이 디렉터리는 gym 과제를 담지 않는다. 새 rhwp 서브커맨드를 제안하지 않는다.
