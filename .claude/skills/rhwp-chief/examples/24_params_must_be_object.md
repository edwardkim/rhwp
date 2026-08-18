# 24_params_must_be_object

왜: params 배열은 형식 오류

## request.json

```json
{
  "doc": "a.hwpx",
  "symptom": "params 배열은 형식 오류",
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
