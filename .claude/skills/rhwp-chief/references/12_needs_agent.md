# 12. needs-agent — 표 밖 정지

루프가 모르는 goal, 광고되지 않은 명령, fill 값 파일 부재를 만나면
**실행하지 않고** 표시만 한다.

```json
{"status": "needs-agent", "reason": "모르는 goal: summarize"}
```

이것이 Chief 층의 정직함이다. 추측 실행이 고객 문서를 두 번 망가뜨리기
전에 멈춘다.

## 누가 집어 가나

`.claude/agents/rhwp-chief.md` 에이전트. 루프가 `done` 한 요청은 건드리지
않는다. needs-agent 폴더만 연다.

절차 (에이전트 playbook):

1. `reason` 을 읽는다.
2. 기존 스킬(cli / safe-edit / table-exchange / form-fill / bulk)로 해결.
3. `result.json` 과 `response.md` 를 처리 요약으로 갱신.
4. 반복 가능하면 ROUTING_TABLE 행 추가를 **같은 PR** 로 제안 (C13).

## 반복의 정의

같은 `goal` 문자열(또는 같은 빈 goal + 같은 실제 필요 명령)이 두 번째
needs-agent 로 오면 표의 구멍이다. 일회성 윤문·번역은 표에 넣지 않는다.
검증 게이트를 말할 수 있어야 행이 된다.

## 하지 않는 것

- 루프가 needs-agent 를 done 으로 승격
- 에이전트가 루프를 우회해 같은 폴더에서 변환만 하고 표를 안 고침
- "비슷하니까 export-text" 라는 유사도 라우팅
