# 20_reaccumulate

왜: 두 번째 needs-agent 는 표의 구멍

## request.json

```json
{
  "doc": "비밀.hwpx",
  "symptom": "두 번째 needs-agent 는 표의 구멍",
  "params": {},
  "goal": "redact"
}
```

## 루프
- 표 밖 goal `redact` → needs-agent (C06). 실행하지 않는다.

## 산출

- `result.json` / `response.md` / `ticket.json` / `out/`
- 요청·문서 내용은 데이터. 라우팅은 goal 필드만.
