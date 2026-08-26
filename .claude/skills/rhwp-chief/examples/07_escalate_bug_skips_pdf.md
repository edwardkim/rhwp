# 07_escalate_bug_skips_pdf

왜: 패닉이면 변환하지 않는다

## request.json

```json
{
  "doc": "crash.hwpx",
  "symptom": "패닉이면 변환하지 않는다",
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
