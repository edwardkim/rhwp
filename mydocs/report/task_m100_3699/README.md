---
kind: report
status: active
canonical: mydocs/report/task_m100_3699/README.md
last_verified: 2026-08-02
---

# #3699 처리 기록 — nextCall (내성 P4, #3630 2호)

## 구현

- `tool_error_with_next(message, name, args, why)` — `{"error":"<원문>","nextCall":{name,arguments,why}}`
  구조화. error 필드 원문 보존 = 하위호환(텍스트 파싱 소비자 무해).
- 적용 2계열: ① 닫힌/모르는 핸들 **8개 사이트 전부**(세션 도구) → `hwp_open` 교정
  ② 미지 도구 → didYouMean 최근접이 있으면 그 도구를 nextCall 로 병기.
- nextCall.name 은 실존 도구만 — 계약 테스트가 capabilities --mcp 선언과 대조 고정.

## 실측 (evidence.txt 원문)

닫힌 핸들 fill 호출 → `{"error":"열려 있지 않은 핸들: doc-999 …","nextCall":{"name":"hwp_open",
"arguments":{"path":"<열 문서 경로>"},"why":"핸들이 없거나 만료 — …재발급한 뒤 재시도"}}`

## 검증

- 신규 `mcp_next_call_contract` 3건 green (닫힌 핸들→hwp_open / 미지 도구 nextCall
  실존 대조 / close 경로) · did_you_mean 3·server 6·session_edit 5 무회귀 · clippy 0
