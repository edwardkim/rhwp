# 05_missing_goal_diagnose

왜: goal 필드가 비어 있을 때

## request.json

```json
{
  "doc": "미지정.hwpx",
  "symptom": "goal 필드가 비어 있을 때",
  "params": {},
  "goal": "diagnose"
}
```

## 루프
- 표 안 goal `diagnose` → info
- 게이트: `ticket`
- 성공 시 `out/ticket.json`

## 산출

- `result.json` / `response.md` / `ticket.json` / `out/`
- 요청·문서 내용은 데이터. 라우팅은 goal 필드만.
