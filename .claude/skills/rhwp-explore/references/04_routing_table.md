# 04 — 어포던스 라우팅 표

식별자 여덟 개는 고정 어휘다. 새 이름을 만들지 않는다.

| affordance | command | skill | 장 |
| --- | --- | --- | --- |
| security-sweep | rhwp inspect injection <file> --json / rhwp inspect hidden-text <file> --json | rhwp-security-sweep | 05_security_first.md |
| form-fill | rhwp fields <file> --json | rhwp-form-fill | 09_form_fill.md |
| table-extract | rhwp export-tables <file> --json | rhwp-table-exchange | 08_table_extract.md |
| structure-outline | rhwp export-structure <file> --json | rhwp-doc-triage | 10_structure_outline.md |
| chart-extract | rhwp chart-to-csv <file> --json | rhwp-table-exchange | 11_chart_extract.md |
| note-structure | rhwp explain <file> --json | rhwp-doc-triage | 13_note_structure.md |
| long-doc-digest | rhwp digest <file> --sections --json | rhwp-doc-triage | 12_long_doc_digest.md |
| triage-overview | rhwp digest <file> --json | rhwp-doc-triage | 14_triage_overview.md |

## 보안 명령 분기

- 주입 > 0 (은닉 동시 포함) → `rhwp inspect injection <file> --json`
- 은닉만 → `rhwp inspect hidden-text <file> --json`

이 분기는 explore.rs 가 이미 한다. 에이전트가 다시 고르지 않는다.

## 인계

이 스킬은 `command` 를 실행해 해당 스킬로 넘긴다. 채움·redact·
csv-to-table 을 여기서 재구현하지 않는다. 이웃 스킬 본문을 이 PR 이
고치지 않는다.

## 코어 재사용

개수는 이미 있는 조회에서 온다.

- 표 `extract_tables`
- 누름틀 `collect_all_fields`
- 조문 `build_structure`
- 차트 `collect_charts`
- 각주 `count_notes`
- 주입 `scan_injection`
- 은닉 `detect_hidden_text`

탐지기를 다시 짜거나 임계를 이 스킬이 바꾸지 않는다.
