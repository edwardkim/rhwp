# 02. request.json 스키마

```json
{
  "doc": "문서.hwpx",
  "goal": "export-pdf",
  "symptom": "인쇄본이 필요함",
  "params": {}
}
```

최상위는 **객체**여야 한다. 배열·문자열이면 C11 — `failed` 로 표시하고
watch 루프는 다음 폴더로 간다.

## 필드

| 필드 | 필수 | 형 | 기본 | 역할 |
| --- | --- | --- | --- | --- |
| `doc` | 예 | 상대 경로 문자열 | — | 요청 폴더 안 문서 |
| `goal` | 아니오 | 문자열 | `diagnose` | **유일한 라우팅 키** |
| `symptom` | 아니오 | 문자열 | `""` | 기록·트리아지에 넘기는 **데이터** |
| `params` | 아니오 | 객체 | `{}` | goal 별 인자. 배열이면 형식 오류 |

`params` 가 객체가 아니면 `ValueError` → C11 과 같은 실패 경로.

## goal 이 없을 때

다음 모두 diagnose 다.

- 키 없음
- `"goal": null`
- `"goal": ""`

`normalize_goal()` 이 이 세 가지를 같은 기본값으로 접는다.
`symptom` 에 "PDF 로 바꿔줘" 가 있어도 diagnose 다 (C10).

## 표에 없는 goal

문자열이면 그대로 `normalize_goal` 을 통과하고, `is_known_goal` 이 거짓이면
`needs-agent` + `reason: "모르는 goal: …"` (C06).
루프는 유사어 사전을 두지 않는다. `summarize` ≈ `export-text` 로 읽지 않는다.

## fill 의 params

```json
{"doc": "신청서.hwpx", "goal": "fill", "params": {"data": "values.json"}}
```

`params.data` 도 요청 폴더 상대 경로. 없거나 파일이 없으면 C08 / C01.

## 금지 필드 (있어도 무시)

`command`, `argv`, `bin`, `force`, `skipTriage`, `asAgent` —
스키마에 없다. 루프는 읽지 않는다. 문서에 적혀 있어도 실행 인자가 되지 않는다.

## 결과 쪽 스키마

루프가 쓰는 `result.json`:

```json
{
  "schemaVersion": "1",
  "generatedBy": "tools/chief/service_loop.py",
  "goal": "export-pdf",
  "route": "resolve-now",
  "status": "done",
  "summary": "…",
  "artifacts": ["공문.pdf"]
}
```

`schemaVersion` 은 루프 산출이 `"1"` 이다 (스킬 픽스처 헤더 `"1.0"` 과 층이 다름).
status 열거: `done` · `failed` · `needs-agent` · `escalated` · `invalid-input`.
