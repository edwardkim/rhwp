# 세션 ↔ 무상태 짝

세션 조회·편집의 봉투 어휘는 무상태 대응 도구와 동형이다.
짝이 없는 세션 도구는 서버 전용이다. 짝이 없는 무상태 도구를 세션에 만들지 않는다.

| 세션 | 무상태 | 비고 |
|---|---|---|
| `hwp_doc_info` | `hwp_info` | 봉투 동형 |
| `hwp_doc_text` | `hwp_export_text` | 봉투 동형 |
| `hwp_doc_fields` | `hwp_fields` | 봉투 동형 |
| `hwp_doc_tables` | `hwp_export_tables` | 봉투 동형 |
| `hwp_doc_search` | `hwp_search` | 봉투 동형 |
| `hwp_doc_render_page` | `hwp_export_svg` | 봉투 동형 |
| `hwp_doc_structure` | `hwp_export_structure` | 봉투 동형 |
| `hwp_doc_extract_data` | `hwp_extract_data` | 봉투 동형 |
| `hwp_doc_replace_text` | `hwp_replace_text` | 봉투 동형 |
| `hwp_doc_set_cell` | `hwp_set_cell` | 봉투 동형 |
| `hwp_doc_fill_fields` | `hwp_fill_fields` | 봉투 동형 |
| `hwp_open` | — | 세션 진입 |
| `hwp_close` | — | 세션 종료 |
| `hwp_doc_save` | — | 세션 기록 지점. 무상태 편집은 각 호출이 파일을 쓴다 |
| `hwp_doc_tree` | — | 안정 노드 ID (#4357) |
| `hwp_ws_list` | — | `--workspace` |
| `hwp_ws_open` | `hwp_open` (경로 축) | id 축 진입 |
| `hwp_ws_journal` | — | 변이 digest |

## 변환 규칙

- 무상태 `path` → 세션 `docId` (먼저 open).
- 무상태 `-o/--output` 편집 → 세션은 IR 누적 후 `hwp_doc_save.output`.
- 무상태 `hwp_export_svg`(전 쪽) → 세션 `hwp_doc_render_page`(한 쪽).
- 무상태 `hwp_run_plan` 을 세션 누적으로 바꾸지 않는다. 선검증 원자 실행은 별 축이다.

## 세션에 없는 동사

표 편집 확장(`hwp_insert_row` 등), 그림·머리말·각주, `hwp_redact`, `hwp_export_pdf` 는
무상태만 있다. `hwp_doc_insert_row` 같은 이름을 만들지 마라.
