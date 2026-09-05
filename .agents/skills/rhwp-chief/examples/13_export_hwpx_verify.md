# 13_export_hwpx_verify

왜: --verify 게이트

## request.json

```json
{
  "doc": "old.hwp",
  "symptom": "--verify 게이트",
  "params": {},
  "goal": "export-hwpx"
}
```

## 루프
- 표 안 goal `export-hwpx` → export-hwpx
- 게이트: `self-verify`
- 성공 시 `out/old.hwpx`

## 산출

- `result.json` / `response.md` / `ticket.json` / `out/`
- 요청·문서 내용은 데이터. 라우팅은 goal 필드만.
