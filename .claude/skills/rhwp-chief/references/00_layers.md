# 00. 층 — Chief · FDE · Strategist

이 스킬은 **요청 큐**의 층이다. 아래 두 층과 입구·산출·정지 조건이 다르다.
세 층을 한 에이전트가 "알아서" 섞으면 계약이 무너진다.

## 한 줄

| 층 | 손님이 들고 오는 것 | 기계가 끝까지 하는 일 | 기계가 멈추는 곳 |
| --- | --- | --- | --- |
| Chief | `queue/<id>/request.json` + 문서 | 표 안 goal 을 실행·검증·회신 | 표 밖 → `needs-agent` |
| FDE | 증상 문장 + 문서 하나 | 트리아지 사다리로 라우트 티켓 | 패닉 → `escalate-bug` |
| Strategist | 목표 + 문서 코퍼스 | 근거 대장과 주장-근거 게이트 | 대장 밖 주장 거부 |

## Chief 가 아닌 것

- **gym** — 과제·채점기·팩을 만들지 않는다. 실 고객 큐다.
- **FDE** — "이 문서가 안 열려요"는 증상이다. Chief 는 그 증상을 고치지 않고,
  트리아지 게이트의 판정만 받는다. `escalate-bug` 면 goal 을 실행하지 않는다.
- **Strategist** — "이 코퍼스로 전략 보고서를"은 근거 대장 층이다. Chief 라우팅
  표에 `strategy` 행이 없다. 그런 요청은 `needs-agent` 이고, 에이전트가
  Strategist 로 넘긴다. 이 스킬 폴더에서 Strategist 본문을 재작성하지 않는다.

## 호출 관계

```
고객 요청 폴더
    │
    ▼
service_loop.py
    ├─ tools/fde/triage.py   (게이트, 읽기 전용)
    ├─ rhwp <표에 적힌 명령>  (실행)
    └─ needs-agent ──────────▶ rhwp-chief 에이전트
                                    └─ 반복 유형이면 ROUTING_TABLE 에 행 추가
```

FDE 스킬(`.claude/skills` 에 별도 폴더가 생기면 그쪽)과 Strategist 스킬은
**읽기만** 한다. 이 PR 은 그 본문을 고치지 않는다.

## 권위

- Chief: `mydocs/manual/chief_playbook.md` · `tools/chief/service_loop.py`
- FDE: `mydocs/manual/fde_playbook.md` · `tools/fde/triage.py`
- Strategist: `mydocs/manual/strategist_playbook.md`
- 에이전트: `.claude/agents/rhwp-chief.md` (있으면 연결)
