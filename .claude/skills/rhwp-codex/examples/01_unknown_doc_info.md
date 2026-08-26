# 01 — 처음 보는 문서 — info 부터

갈래: **파악**. 장: `10_조회.md`. gym 아님. 새 CLI 아님.

권위는 생성 장이 아니라, 생성 장을 **읽는 순서**다. 표본 JSON 을 손대지 않는다.

## 요청

`이 HWP 뭐야? 쪽수만 알려줘.`

## 명령

```bash
rhwp info samples/basic/issue2007_nested_cell_pagination_42065.hwp --json
```

## 읽는 필드

| 키 | 왜 |
|---|---|
| `format` | hwp5/hwpx/hwp3 |
| `pageCount` | 쪽수 |
| `title` | 문서 파생 — C3 |
| `untrustedFields` | title·fonts[] |

## 정지

질문이 쪽수·형식이면 여기서 멈춘다. `export-text` 전문은 X10.

## 표본

픽스처 `fixtures/envelopes/info.json` 은 생성 장 10의 실측 절단본이다.
