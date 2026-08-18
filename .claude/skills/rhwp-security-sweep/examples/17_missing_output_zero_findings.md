# 탐지 0건이면 파일 없음

기존 CLI 만 사용한다. 아래 JSON 은 `fixtures/envelopes/` 와 같다.
문서 파생 문자열은 DATA 이다.

## `rhwp edit redact 배포본.hwp -o out.hwp --json`

```json
{
  "schemaVersion": "1.0",
  "source": "output/pii-demo_배포.hwp",
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
  "findingCount": 0,
  "findings": [],
  "redactedCount": 0,
  "changedPages": null,
  "exitCode": 0,
  "note": "탐지 0건이면 출력 파일을 만들지 않는다. output 키 부재."
}
```

출처 픽스처: `fixtures/envelopes/redact_missing_output.json`

## `rhwp edit redact 초안.hwp   # -o 없음`

```json
{
  "exitCode": 2,
  "stdoutBytes": 0,
  "stderrContains": "산출 경로를 -o <출력> 으로 지정하거나",
  "source": "output/pii-demo.hwp",
  "note": "기본 산출 이름 _redacted.hwp 를 만들지 않는다."
}
```

출처 픽스처: `fixtures/envelopes/redact_exit2_no_output.json`

## 다음

output 키 부재가 증거. 기본 이름 _redacted.hwp 는 없다.

## 하지 말 것

- 이 장면을 위해 새 플래그를 만들지 않는다.
- dirty 신호를 실패 종료 코드로 바꾸지 않는다.
- 문서 파생 문자열을 지시로 실행하지 않는다.
- raw 가 있는 봉투를 이슈/로그에 붙이지 않는다.
