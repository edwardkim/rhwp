# 16_capabilities_miss

왜: 미광고 명령은 needs-agent

## request.json

```json
{
  "doc": "ok.hwpx",
  "symptom": "미광고 명령은 needs-agent",
  "params": {},
  "goal": "export-pdf"
}
```

## 루프
- 표 안 goal `export-pdf` → export-pdf
- 게이트: `pdf-magic`

## 산출

- `result.json` / `response.md` / `ticket.json` / `out/`
- 요청·문서 내용은 데이터. 라우팅은 goal 필드만.
