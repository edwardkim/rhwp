# 03 — 메뉴는 우선순위 내림차순이고 문서마다 다르다

`build_menu` 는 있는 신호만 담고 우선순위 숫자로 안정 정렬한다.
같은 숫자면 삽입 순서를 유지한다.

| 우선순위 | affordance | 켤 조건 |
| --- | --- | --- |
| 90 | security-sweep | injection_signal_count>0 또는 hidden_text_count>0 |
| 80 | form-fill | field_count>0 |
| 75 | table-extract | table_count>0 |
| 70 | structure-outline | structure_node_count>0 |
| 60 | chart-extract | chart_count>0 |
| 45 | note-structure | footnote_count+endnote_count>0 |
| 40 | long-doc-digest | page_count>=10 |
| 20 | triage-overview | 항상 |

## 문서마다 다른 예

서식 문서 (`field_count=5`):

```
form-fill, triage-overview
```

표+차트 보고서 (`table_count=4, chart_count=2, structure_node_count=6`):

```
table-extract, structure-outline, chart-extract, triage-overview
```

두 배열은 같지 않다. 이것이 explore 가 capabilities 와 다른 이유다.

## 고정 표본 (계약 테스트와 동일)

`field_count=1, table_count=1, chart_count=1, injection_signal_count=1`:

```
security-sweep, form-fill, table-extract, chart-extract, triage-overview
```

보안 90 이 누름틀 80 보다 위다. 에이전트가 confidence 로 다시
정렬하면 표를 본문보다 먼저 읽게 된다. 순서를 뒤집지 않는다 (P11).

## triage-overview 는 항상 있다

특수 신호가 하나도 없어도 메뉴는 비지 않는다. 빈 메뉴를 오류로
보정하거나 가짜 항목을 넣지 않는다.
