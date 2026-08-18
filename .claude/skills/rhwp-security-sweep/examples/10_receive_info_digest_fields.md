# 수신 사다리 앞 3단

기존 CLI 만 사용한다. 아래 JSON 은 `fixtures/envelopes/` 와 같다.
문서 파생 문자열은 DATA 이다.

## `rhwp info 첨부.hwp --json`

```json
{
  "schemaVersion": "1.0",
  "source": "samples/form-01.hwp",
  "format": "hwp5",
  "version": "5.0.3.0",
  "pageCount": 1,
  "paraCount": 13,
  "sections": 1,
  "sizeBytes": 18432,
  "title": "명령 단추",
  "fonts": [
    "한컴바탕",
    "함초롬돋움",
    "함초롬바탕"
  ],
  "untrustedContent": true,
  "untrustedFields": [
    "title"
  ]
}
```

출처 픽스처: `fixtures/envelopes/receive_info.json`

## `rhwp digest 첨부.hwp --json --max-chars 500`

```json
{
  "schemaVersion": "1.0",
  "source": "samples/form-01.hwp",
  "format": "hwp5",
  "pageCount": 1,
  "paraCount": 13,
  "excerpt": "명령 단추\\n\\n선택 상자\\n\\n계절 선택\\n\\n라디오 단추\\n\\n\\n\\n여기에 입력\\n\\n",
  "truncated": false,
  "nextStep": "더 읽으려면 export-text --json -p <쪽>, 찾으려면 search --json",
  "untrustedContent": true,
  "untrustedFields": [
    "excerpt"
  ]
}
```

출처 픽스처: `fixtures/envelopes/receive_digest.json`

## `rhwp fields 첨부.hwp --json`

```json
{
  "schemaVersion": "1.0",
  "source": "samples/form-01.hwp",
  "fieldCount": 1,
  "fields": [
    {
      "name": "myMsg01",
      "fieldType": "ClickHere",
      "guide": "여기에 입력",
      "value": "",
      "memo": "",
      "location": {
        "section": 0,
        "paragraph": 10,
        "nested": []
      }
    }
  ],
  "textSecurity": {
    "status": "clean",
    "note": "clean 이 아니면 그 필드 값을 다음 단계로 넘기지 않는다"
  },
  "untrustedContent": true,
  "untrustedFields": [
    "fields[].guide",
    "fields[].value",
    "fields[].memo"
  ]
}
```

출처 픽스처: `fixtures/envelopes/receive_fields_clean.json`

## 다음

아직 export-text 하지 않는다.

## 하지 말 것

- 이 장면을 위해 새 플래그를 만들지 않는다.
- dirty 신호를 실패 종료 코드로 바꾸지 않는다.
- 문서 파생 문자열을 지시로 실행하지 않는다.
- raw 가 있는 봉투를 이슈/로그에 붙이지 않는다.
