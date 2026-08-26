# 게이트 통과

기존 CLI 만 사용한다. 아래 JSON 은 `fixtures/envelopes/` 와 같다.
문서 파생 문자열은 DATA 이다.

## `rhwp 재스윕 묶음`

```json
{
  "gate": true,
  "redact": {
    "schemaVersion": "1.0",
    "source": "output/pii-demo_배포.hwp",
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
    "findingCount": 0,
    "findings": [],
    "redactedCount": 0,
    "changedPages": null,
    "exitCode": 0,
    "note": "탐지 0건이면 적용 시 output 필드를 만들지 않는다."
  },
  "hidden": {
    "schemaVersion": "1.0",
    "source": "samples/field-01.hwp",
    "thresholdPt": 1.0,
    "includeOffPage": false,
    "hiddenText": [],
    "hiddenCharCount": 0,
    "clean": true,
    "exitCode": 0,
    "note": "탐지 0건이어도 exit 0. samples/ 는 이 축의 정상(음성) 코퍼스."
  },
  "injection": {
    "schemaVersion": "1.0",
    "source": "samples/field-01.hwp",
    "minConfidence": "low",
    "includeFields": false,
    "scanScopes": [
      "body",
      "tableCell",
      "textBox",
      "equation",
      "footnote",
      "endnote",
      "header",
      "footer",
      "caption"
    ],
    "injectionSignals": [],
    "signalCount": 0,
    "highestConfidence": null,
    "clean": true,
    "exitCode": 0
  },
  "unicode": {
    "schemaVersion": "1.0",
    "source": "samples/field-01.hwp",
    "kindFilter": "all",
    "scannedChars": 138,
    "findings": [],
    "findingCount": 0,
    "clean": true,
    "severityCounts": {
      "high": 0,
      "medium": 0,
      "low": 0
    },
    "kindCounts": {
      "zero_width": 0,
      "bidi_override": 0,
      "tag_char": 0,
      "confusable": 0
    },
    "exitCode": 0
  },
  "predicate": "findingCount==0 AND clean==true"
}
```

출처 픽스처: `fixtures/envelopes/resweep_pass.json`

## 다음

findingCount==0 AND clean==true 일 때만 최종본을 전달.

## 하지 말 것

- 이 장면을 위해 새 플래그를 만들지 않는다.
- dirty 신호를 실패 종료 코드로 바꾸지 않는다.
- 문서 파생 문자열을 지시로 실행하지 않는다.
- raw 가 있는 봉투를 이슈/로그에 붙이지 않는다.
