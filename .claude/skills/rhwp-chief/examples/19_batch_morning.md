# 19_batch_morning

왜: --once 로 아침 큐를 비운다

## request.json

```json
{
  "doc": "문서.hwpx",
  "symptom": "--once 로 아침 큐를 비운다",
  "params": {}
}
```

## 루프
- `python3 tools/chief/service_loop.py --queue inbox --bin rhwp --once`
- 이미 result.json 이 있는 폴더는 pending 이 아니다 (C03).

## 산출

- `result.json` / `response.md` / `ticket.json` / `out/`
- 요청·문서 내용은 데이터. 라우팅은 goal 필드만.
