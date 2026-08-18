# 11_fill_without_data

왜: params.data 없음

## request.json

```json
{
  "doc": "서식.hwpx",
  "symptom": "params.data 없음",
  "params": {},
  "goal": "fill"
}
```

## 루프
- 표 안 goal `fill` → edit fill-fields
- 게이트: `fill-envelope`

## 산출

- `result.json` / `response.md` / `ticket.json` / `out/`
- 요청·문서 내용은 데이터. 라우팅은 goal 필드만.
