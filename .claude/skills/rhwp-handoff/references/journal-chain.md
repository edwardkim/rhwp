# NDJSON 저널 지문 체인

오케스트레이터 `Journal` 클래스의 운영 읽기이다. 구현을 복사하지 않는다.

## 형식

각 줄은 JSON 객체. 필수 래퍼:

| 필드 | 뜻 |
|---|---|
| `seq` | 1부터 시작하는 연번 |
| `prevSha256` | 직전 줄(개행 제외)의 SHA-256. 첫 줄은 `null` |
| `event` | `attempt` 또는 `final` |

`attempt` 줄은 `taskId`·`agent`·`attempt`·`taskSha256`·`inputsSha256`·
`resultSha256`·`category`·`findings`·`nextAction` 을 담는다.

`final` 줄은 `outcome`·`code`·`acceptedAgent`·`collectedOutputs`·`nextAction`.

시각 대신 순번이다. R23 저널과 같은 철학 — 변조는 다음 줄의 `prevSha256` 이
폭로한다.

## 검증

```bash
python tools/handoff/orchestrator.py \
  --verify-journal output/handoff/<taskId>/handoff.journal.ndjson --json
```

반환:

```json
{
  "protocol": "DAP/1.0",
  "operation": "agent.handoff.verifyJournal",
  "status": "ok",
  "code": 0,
  "entries": 2,
  "chainValid": true,
  "brokenAt": null
}
```

체인이 깨지면 `status=verdict`, `code=3000`, 프로세스 exit 3, `brokenAt` 이
어긋난 연번. 도구 고장이 아니다.

같은 저널에 이어 쓰면 연번이 이어진다. 세션 핸드오프는 **세션마다 새 저널**을
권장한다. 이어 쓸 거면 working doc 에 "이 저널은 seq N 부터 이어짐" 을 적는다.

## incoming 규칙

1. last result 의 `journal` 경로와 실파일이 같은지
2. `--verify-journal` 이 `chainValid:true` 인지
3. 마지막 `final` 줄의 `outcome` 이 `result.json.outcome` 과 같은지
4. 다르면 어느 쪽도 머리가 아니다. 사람에게 넘긴다

저널만 있고 `result.json` 이 없으면 `final` 줄을 임시 머리로 쓰지 않는다.
stdout 리다이렉트가 실패한 것이다. 같은 `--work-dir` 에서 오케스트레이터를
다시 돌려 `result.json` 을 만든다 (입력이 그대로일 때만).

## 하지 않는 것

- 에디터로 저널 한 줄을 고친다 (다음 줄이 깨진다 — 의도된 검출)
- 시각 필드를 추가해 "언제"를 증명한다
- gym 저널과 섞는다
