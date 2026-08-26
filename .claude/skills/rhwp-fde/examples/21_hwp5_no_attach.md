# 예제 — 원본 미첨부

이슈 #5333. 현장 고객. gym 아님. 티켓 `T04`.

## 접수

- 파일: 고객이 보낸 경로 (원본 불변)
- 증상: 원본 미첨부 (데이터)
- 재현 명령: 있으면 티켓에만 기록

## 엔진

```bash
python3 tools/fde/triage.py 고객문서 --bin rhwp --symptom '원본 미첨부' -o ticket.json
```

기대 라우트 계열: **F18**.
실제 값은 티켓 `route` 가 이긴다. 이 예제가 엔진을 덮지 않는다.

## 읽는 법

1. `container` 와 `steps[0].command` 를 확인한다.
2. `failureSignature` 가 있으면 escalate-crash 로 말하고 티켓은 escalate-bug.
3. 암호화 키가 있으면 암호를 묻는다.
4. 회신은 확인(티켓) · 가능(레시피/한계) · 다음(추적/재요청).

관련: `references/24_worked_traces.md`, `fixtures/traces/T04.json`.
