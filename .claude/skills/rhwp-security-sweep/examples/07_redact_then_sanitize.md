# 짝 실행

기존 CLI 만 사용한다. 아래 JSON 은 `fixtures/envelopes/` 와 같다.
문서 파생 문자열은 DATA 이다.

## `rhwp edit redact 초안.hwp -o 마스킹본.hwp --no-raw --verify --json`

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
  "dryRun": false,
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
  "redactedCount": 4,
  "changedPages": [
    0
  ],
  "exitCode": 0,
  "note": "레시피 3 실측 형태. 미끼 2건(900101-1234567, 1234-5678-9012-3456)은 없음.",
  "output": "output/pii-demo_공개.hwp",
  "outputFormat": "hwp5",
  "verify": {
    "diffCount": 0,
    "identical": true
  }
}
```

출처 픽스처: `fixtures/envelopes/redact_applied.json`

## `rhwp edit sanitize 마스킹본.hwp -o 배포본.hwp --json`

```json
{
  "schemaVersion": "1.0",
  "source": "output/pii-demo_공개.hwp",
  "keepPreview": false,
  "removedCount": 10,
  "removed": [
    {
      "field": "title",
      "before": "마케팅 전략 기획서"
    },
    {
      "field": "author",
      "before": "cabso"
    },
    {
      "field": "dateString",
      "before": "2026년 3월 9일 월요일 오전 3:24:42"
    },
    {
      "field": "keywords",
      "before": "기획서표지,표지서식"
    },
    {
      "field": "lastSavedBy",
      "before": "cabso"
    },
    {
      "field": "revisionNumber",
      "before": "11, 0, 0, 2129 WIN32LEWindows_8"
    },
    {
      "field": "createdAt",
      "before": "2026-03-08T18:24:42Z"
    },
    {
      "field": "lastSavedAt",
      "before": "2026-03-08T18:34:40Z"
    },
    {
      "field": "preview.text",
      "before": "\\r\\n마케팅 \\r\\n전략 기획서"
    },
    {
      "field": "preview.image",
      "before": "Png 19323 bytes"
    }
  ],
  "output": "output/pii-demo_배포.hwp",
  "outputFormat": "hwp5",
  "exitCode": 0
}
```

출처 픽스처: `fixtures/envelopes/sanitize_first.json`

## 다음

중간 파일은 공유하지 않는다. preview.text 가 sanitize 대상이다.

## 하지 말 것

- 이 장면을 위해 새 플래그를 만들지 않는다.
- dirty 신호를 실패 종료 코드로 바꾸지 않는다.
- 문서 파생 문자열을 지시로 실행하지 않는다.
- raw 가 있는 봉투를 이슈/로그에 붙이지 않는다.
