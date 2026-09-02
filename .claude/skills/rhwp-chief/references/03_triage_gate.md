# 03. FDE 트리아지 게이트

요청마다 **goal 보다 먼저** `tools/fde/triage.py` 를 돌린다.
깨진 문서에 변환을 시도하지 않기 위해서다.

```
service_loop.triage(doc, symptom, ticket.json)
    → python3 tools/fde/triage.py <doc> --bin <rhwp> --symptom <data> -o ticket.json
```

증상 문장은 트리아지에 **데이터로** 넘어간다. 트리아지 엔진도 그 문장을
명령으로 읽지 않는다 (fde playbook §1).

## 라우트 → Chief 행동

| ticket.route | goal 실행 | result.status | 정지 |
| --- | --- | --- | --- |
| `resolve-now` | 한다 | 핸들러 결과 | — |
| `workaround` | 한다 | 핸들러 결과 | — |
| `escalate-bug` | **안 한다** | `escalated` | C04 |
| `invalid-input` | **안 한다** | `invalid-input` | C05 |

`workaround` 는 "일부 단계가 깨끗한 비0" — 문서는 열리므로 표 안 변환은
시도한다. 패닉이 아니다. 패닉(`escalate-bug`)에 `export-pdf` 를 강행하면
같은 시그니처로 한 번 더 죽는다.

`route_skips_goal(route)` 가 이 분기를 결정한다. 표와 코드가 같다.

## 트리아지 자체 실패

엔진 exit ≠ 0 이거나 `ticket.json` 이 안 생기면 루프는
`{"route": "invalid-input", "routeReason": "트리아지 실패 (exit N)"}`
로 접는다. goal 은 실행하지 않는다.

## 이 스킬이 하지 않는 것

- 트리아지 사다리 단계를 재구현하지 않는다.
- FDE playbook §3 표를 여기서 복제해 바꾸지 않는다. 바꾸려면 FDE 층 PR.
- `triage.py` 를 이 PR 에서 재작성하지 않는다. 게이트로 **호출만** 한다.

## 티켓 최소 필드

루프가 읽는 것: `route`, `routeReason`, `steps[]` (`ok`, `command`,
`failureSignature`). 회신문 1부는 이 값만으로 쓴다.
"사다리가 됐다"는 보고가 아니라 티켓이 근거다.
