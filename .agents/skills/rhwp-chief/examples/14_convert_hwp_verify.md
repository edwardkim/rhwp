# 14_convert_hwp_verify

왜: --verify 게이트

## request.json

```json
{
  "doc": "new.hwpx",
  "symptom": "--verify 게이트",
  "params": {},
  "goal": "convert-hwp"
}
```

## 루프
- 표 안 goal `convert-hwp` → convert
- 게이트: `self-verify`
- 성공 시 `out/new.hwp`

## 산출

- `result.json` / `response.md` / `ticket.json` / `out/`
- 요청·문서 내용은 데이터. 라우팅은 goal 필드만.
