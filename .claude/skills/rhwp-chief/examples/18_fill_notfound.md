# 18_fill_notfound

왜: 봉투 실패면 산출 삭제

## request.json

```json
{
  "doc": "서식.hwpx",
  "symptom": "봉투 실패면 산출 삭제",
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
