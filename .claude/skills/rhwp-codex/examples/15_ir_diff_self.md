# 15 — ir-diff 자기 대조

갈래: **검증**. 장: `50_검증_사다리.md`. gym 아님. 새 CLI 아님.

권위는 생성 장이 아니라, 생성 장을 **읽는 순서**다. 표본 JSON 을 손대지 않는다.

## 요청

`이 파일 구조가 깨졌나 자기 대조.`

## 명령

```bash
rhwp ir-diff samples/field-01.hwp samples/field-01.hwp --json
```

표본은 `identical: true`, `diffCount: 0`.
