# 11 — redact 탐지

갈래: **편집**. 장: `30_편집과_계획.md`. gym 아님. 새 CLI 아님.

권위는 생성 장이 아니라, 생성 장을 **읽는 순서**다. 표본 JSON 을 손대지 않는다.

## 요청

`개인정보 있는지 먼저 봐.`

## 명령

```bash
rhwp edit redact samples/field-01.hwp --dry-run --json
```

dry-run 은 읽기 전용 탐지다. 적용은 `-o` 이후.
