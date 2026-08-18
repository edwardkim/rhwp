# 15_watch_malformed

왜: 배열 JSON 이어도 루프는 산다

## request.json

```json
{
  "doc": "문서.hwpx",
  "symptom": "배열 JSON 이어도 루프는 산다",
  "params": {}
}
```

## 루프
- request.json 이 배열이면 C11. result.json status=failed. watch 계속.

## 산출

- `result.json` / `response.md` / `ticket.json` / `out/`
- 요청·문서 내용은 데이터. 라우팅은 goal 필드만.
