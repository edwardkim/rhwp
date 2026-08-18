# 규칙 2 — 새 편집·조회 로직을 만들지 않는다

플레이북 §1 규칙 2. 맵: [`../fixtures/reuse/core_map.json`](../fixtures/reuse/core_map.json).

## 왜

서버 전용 `set_cell` 을 새로 짜면, CLI `edit set-cell` 과 넘침·병합·앵커
계약이 갈라진다. 에이전트는 같은 일을 두 경로로 부르므로 **한 경로만
고친 수정**이 다른 경로에서 재현된다.

표면은 얇은 껍데기다. 검증된 코어와 봉투 helper 를 부른다.

## 재사용 표 (대표)

| 표면 | 코어 | 봉투 |
|---|---|---|
| `hwp_fill_fields` / `hwp_doc_fill_fields` | `set_field_value_by_name_at` · `collect_field_records` | fill 봉투 (`notFound`/`ambiguous`) |
| `hwp_replace_text` / `hwp_doc_replace_text` | `replace_all_native` | `replacedCount` |
| `hwp_search` / `hwp_doc_search` | `grep` | `matchCount`/`totalMatchCount` |
| `hwp_export_tables` / `hwp_doc_tables` | `extract_tables` | `tables[]`/`rowSpan` |
| `hwp_set_cell` / `hwp_doc_set_cell` | 셀 기록 + overflow probe | `overflow`/`outputFormat` |
| `hwp_fields` / `hwp_doc_fields` | `collect_field_records` | `fields[]` |
| `hwp_extract_data` / `hwp_doc_extract_data` | extract-data | `counts`/`items` |
| `hwp_export_structure` / `hwp_doc_structure` | export-structure | `sections` |
| `hwp_ir_diff` | ir-diff | `identical`/`categories` |
| `hwp_doc_save` / `edit -o` | `edit_serialize` | `verify` |

함수 이름이 소스에서 바뀌면 이 표보다 소스가 이긴다. 요지는
**표면 파일에 알고리즘을 새로 쓰지 말라**는 것이다.

## 세션은 인메모리 면

`hwp_doc_fill_fields` 는 `hwp_fill_fields` 와 같은 코어를 열린 문서에
적용한다. 저장은 `hwp_doc_save` 가 `edit_serialize` 를 한 번 더 부른다.
세션 전용 직렬화를 만들지 마라.

## DocumentCore 를 이 스킬에서 열지 않는다

새 편집 의미론이 필요하면 **별도 이슈**로 코어를 먼저 검증하고, 표면
PR 은 그 함수를 배선만 한다. 이 스킬의 PR 범위는 스킬·픽스처·계약
시험이다. `src/document_core/` 를 건드리지 않는다.

## 새 CLI 도 이 스킬에서 만들지 않는다

"표면을 더하는 절차"를 안내할 뿐, 이 작업 자체는 명령을 추가하지 않는다.
구현 PR 은 플레이북 §2 를 따른다.
