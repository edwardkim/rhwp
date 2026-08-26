# 13 — 변환 + verify

갈래: **변환**. 장: `40_변환과_렌더.md`. gym 아님. 새 CLI 아님.

권위는 생성 장이 아니라, 생성 장을 **읽는 순서**다. 표본 JSON 을 손대지 않는다.

## 요청

`HWP5 로 바꾸고 재파싱까지.`

## 명령

```bash
rhwp convert samples/field-01.hwp out.hwp --verify --json
```

`verify.identical` 이 false 면 exit 3. 고장이 아니라 판정(C1).
