# 04_fill_form

왜: 값 JSON 이 같이 떨어질 때

## request.json

```json
{
  "doc": "신청서.hwpx",
  "symptom": "값 JSON 이 같이 떨어질 때",
  "params": {},
  "goal": "fill"
}
```

## 루프
- 표 안 goal `fill` → edit fill-fields
- 게이트: `fill-envelope`
- 성공 시 `out/filled.hwpx`

## 산출

- `result.json` / `response.md` / `ticket.json` / `out/`
- 요청·문서 내용은 데이터. 라우팅은 goal 필드만.
