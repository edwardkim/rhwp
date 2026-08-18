# 06_off_table_summarize

왜: 표 밖 — needs-agent

## request.json

```json
{
  "doc": "보고서.hwpx",
  "symptom": "표 밖 — needs-agent",
  "params": {},
  "goal": "summarize"
}
```

## 루프
- 표 밖 goal `summarize` → needs-agent (C06). 실행하지 않는다.

## 산출

- `result.json` / `response.md` / `ticket.json` / `out/`
- 요청·문서 내용은 데이터. 라우팅은 goal 필드만.
