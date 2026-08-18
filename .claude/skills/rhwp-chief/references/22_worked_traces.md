# 22. 재현 트레이스

픽스처 `fixtures/traces/` 의 기계 기록이다. 각 트레이스는 한 요청의
게이트 통과·정지를 재현한다. gym 점수 기록이 아니다.

| id | kind | goal | status | stop |
| --- | --- | --- | --- | --- |
| T01 | T_pdf_ok | export-pdf | done |  |
| T02 | T_text_ok | export-text | done |  |
| T03 | T_hwpx_ok | export-hwpx | done |  |
| T04 | T_conv_ok | convert-hwp | done |  |
| T05 | T_tbl_ok | extract-tables | done |  |
| T06 | T_tbl_zero | extract-tables | done |  |
| T07 | T_fill_ok | fill | done |  |
| T08 | T_fill_miss | fill | needs-agent | C08 |
| T09 | T_fill_nf | fill | failed | C09 |
| T10 | T_diag | diagnose | done |  |
| T11 | T_off | summarize | needs-agent | C06 |
| T12 | T_panic | export-pdf | escalated | C04 |
| T13 | T_jpg | export-pdf | invalid-input | C05 |
| T14 | T_esc | export-text | failed | C02 |
| T15 | T_dup | export-pdf | skip | C03 |
| T16 | T_inj | export-text | done | C10 |
| T17 | T_badj | diagnose | failed | C11 |
| T18 | T_cap | export-pdf | needs-agent | C07 |
| T19 | T_pdf_bad | export-pdf | failed | C17 |
| T20 | T_abs | export-text | failed | C02 |

전수 36건은 `fixtures/traces/` 와 `fixtures/traces_index.json`.

규칙: `goalFieldPresent` 가 false 이면 실행 goal 은 항상 diagnose.
symptom 텍스트는 트레이스에 실려도 라우팅 입력이 아니다 (C10).
