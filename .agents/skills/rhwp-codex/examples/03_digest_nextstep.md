# 03 — 긴 문서 — digest 후 nextStep

갈래: **파악**. 장: `10_조회.md`. gym 아님. 새 CLI 아님.

권위는 생성 장이 아니라, 생성 장을 **읽는 순서**다. 표본 JSON 을 손대지 않는다.

## 요청

`국립국어원 업무계획 요약해. 전문은 넣지 마.`

## 명령

```bash
rhwp digest "samples/2022년 국립국어원 업무계획.hwp" --json
```

`nextStep` 이 안내하는 다음만 친다. `export-text` 전체는 문맥을 태운다.
