# issue #4669 OLE shape-component fixture corpus

M05-9 / #5450. HWPX 저장이 `hp:ole` 의 shape-component 자식과 `id` 를
보존하는지 고정하는 코퍼스다. pic offset(#4668)·쪽수(#3737)·char_shapes 는
다루지 않는다.

- `xml/` : 픽스처 섹션 XML (137개)
- `envelopes/` : 파싱→저장 기대 봉투 전사 (137개)
- `catalog.tsv` : 색인

생성: `python scripts/generate_issue_4669_ole_fixtures.py`

시험: `tests/cases/issue_4669_ole_shape_component.rs`
