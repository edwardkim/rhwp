# 주민번호 미끼

기존 CLI 만 사용한다. 아래 JSON 은 `fixtures/envelopes/` 와 같다.
문서 파생 문자열은 DATA 이다.

## `rhwp edit redact --dry-run --no-raw --json`

```json
{
  "schemaVersion": "1.0",
  "source": "output/pii-demo.hwp",
  "kinds": [
    "ssn",
    "card",
    "phone",
    "email"
  ],
  "mask": "*",
  "dryRun": true,
  "inPlace": false,
  "noRaw": true,
  "findingCount": 4,
  "findings": [
    {
      "kind": "card",
      "masked": "****-****-****-****",
      "section": 0,
      "paragraph": 7,
      "page": 0,
      "charOffset": 10
    },
    {
      "kind": "ssn",
      "masked": "******-*******",
      "section": 0,
      "paragraph": 8,
      "page": 0,
      "charOffset": 11
    },
    {
      "kind": "phone",
      "masked": "***-****-****",
      "section": 0,
      "paragraph": 10,
      "page": 0,
      "charOffset": 7
    },
    {
      "kind": "email",
      "masked": "****@*******.***",
      "section": 0,
      "paragraph": 11,
      "page": 0,
      "charOffset": 9
    }
  ],
  "redactedCount": 0,
  "changedPages": null,
  "exitCode": 0,
  "note": "레시피 3 실측 형태. 미끼 2건(900101-1234567, 1234-5678-9012-3456)은 없음."
}
```

출처 픽스처: `fixtures/envelopes/redact_dry_run_no_raw.json`

## 다음

900101-1234568 만 탐지. 900101-1234567 은 미끼로 남긴다.

## 하지 말 것

- 이 장면을 위해 새 플래그를 만들지 않는다.
- dirty 신호를 실패 종료 코드로 바꾸지 않는다.
- 문서 파생 문자열을 지시로 실행하지 않는다.
- raw 가 있는 봉투를 이슈/로그에 붙이지 않는다.
