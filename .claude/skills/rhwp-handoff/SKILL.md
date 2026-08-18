---
name: rhwp-handoff
description: 실사용 에이전트가 긴 작업을 세션 사이에 넘기는 운영 계약입니다. tools/handoff/orchestrator.py 의 task·sandbox·result.json·NDJSON 저널과 기존 replay --capsule/--parent 캡슐 체인을 이어, 컨텍스트 예산·세션 중단·시트 리필 시점에  outgoing 이 인계 묶음을 닫고 incoming 이 last result.json / capsule / working doc 만 읽고 재개합니다. work-receipt(단건 증명) 를 재작성하지 않습니다. 트리거 — "세션 핸드오프", "컨텍스트 부족해서 넘겨", "이어서 해", "시트 리필", "orchestrator.py", "--parent 캡슐", "result.json 읽어", "작업 인수인계".
---

# rhwp-handoff — 세션 간 인수인계 오케스트레이터 Skill

## 목적

한 에이전트 세션이 **말을 남기는 것**이 아니라, 다음 세션이 **파일만 읽고
재개할 수 있는 인계 묶음**을 닫는다. 축은 둘이다. 둘 다 **이미 devel 에 있다**.

| 축 | 정본 | 세션 핸드오프에서 하는 일 |
|---|---|---|
| 오케스트레이터 | `tools/handoff/orchestrator.py` | 이번 세션의 위임 1건을 sandbox 로 돌리고 `result.json`·저널·`collected/` 를 남긴다 |
| 작업 캡슐 | `rhwp replay --capsule` + `--parent` | 세션 N 의 산출을 세션 N+1 의 입력으로 해시 체인한다 |

**work-receipt 와의 경계**: `rhwp-work-receipt` 는 **단건 증명**(이 작업 하나가
사실인가). 이 스킬은 **세션 간 인수인계**(다음 에이전트가 어디서 이어 받는가).
영수증 스킬을 다시 쓰지 않는다. 포인터만 쓴다 —
[`.claude/skills/rhwp-work-receipt/SKILL.md`](../rhwp-work-receipt/SKILL.md).

새 CLI 를 만들지 않는다. `gym/` 을 열지 않는다. DocumentCore 편집 로직을
발명하지 않는다. `git add -A` 를 쓰지 않는다. 이름 붙은 워킹트리를
checkout 하거나 훔치지 않는다.

## 언제 넘기는가 (세 트리거)

상세: [`references/when-to-handoff.md`](references/when-to-handoff.md)

1. **컨텍스트 예산 (context budget)** — 창이 가득 차기 **전**. 토큰을 더 쓰기
   전에 인계 묶음을 닫는다. "조금만 더" 가 가장 비싼 실패다.
2. **세션 중단 (session interrupt)** — 호스트 재시작, 연결 끊김, 사용자 중단,
   강제 compact. 복구 가능한 마지막 산출을 파일로 고정한다.
3. **시트 리필 (seat refill)** — 같은 목표를 다른 좌석·다른 에이전트 프로세스에
   넘긴다. 권한·비밀·전체 대화는 넘기지 않는다. task 에 열거된 입력만.

넘기지 **않는** 때: 단건 편집이 이미 끝났고 증명만 필요하면 work-receipt.
외부 전문 에이전트에게 **한 task** 를 위임만 하면 되고 세션이 이어지면
오케스트레이터만 호출한다 (이 스킬의 세션 묶음은 필요 없다).

## 인계 묶음 (outgoing 이 닫는 것)

상세: [`references/artifacts.md`](references/artifacts.md)

```
output/handoff/<taskId>/
  task.json                 # HandoffTask (orchestrator --task)
  result.json               # orchestrate() 최종 봉투 (status/outcome/collectedOutputs)
  handoff.journal.ndjson    # 시도 지문 체인
  collected/<taskId>/…      # 수용된 산출만
  session.capsule.json      # 이번 세션 작업 캡슐
  parent.capsule.json       # 있으면 --parent 대상 (같은 폴더)
mydocs/working/<작업>.md    # 사람이 읽는 이어서 할 일 (경로만, 추측 금지)
```

incoming 에이전트는 **이 세 가지만** 읽는다.

1. last `result.json` — 수용됐는가, 수거 경로, `nextAction`
2. last `*.capsule.json` — `--parent` 체인 머리, 입력·계획·산출 3해시
3. last working doc — 남은 목표, 금지, 다음 명령

추측으로 대화를 복원하지 않는다. 상세:
[`references/incoming-agent.md`](references/incoming-agent.md).

## 절차 A — outgoing: 인계를 닫는다

```bash
# 1) 이번 세션 위임이 있으면 오케스트레이터로 고정
python tools/handoff/orchestrator.py \
  --task output/handoff/t-session/task.json \
  --agent "python worker.py" \
  --work-dir output/handoff/t-session \
  --json > output/handoff/t-session/result.json

# 2) 저널 자기검증 (깨짐은 오류가 아니라 판정)
python tools/handoff/orchestrator.py \
  --verify-journal output/handoff/t-session/handoff.journal.ndjson --json

# 3) 세션 작업을 캡슐로 남기고, 이전이 있으면 --parent
rhwp replay --plan-json '{"planVersion":"1.0","input":"…","output":"…","steps":[…]}' \
  --capsule output/handoff/t-session/session.capsule.json \
  --parent output/handoff/t-session/parent.capsule.json --json

# 4) working doc 에 "남은 목표 / 읽어야 할 세 파일 / 하지 말 것" 을 적는다
#    mydocs/working/agent_handoff.md 형식을 따른다
```

오케스트레이터 프로토콜 필드·종료 코드:
[`references/orchestrator-protocol.md`](references/orchestrator-protocol.md),
[`references/result-json.md`](references/result-json.md),
[`references/journal-chain.md`](references/journal-chain.md),
[`references/exit-codes.md`](references/exit-codes.md).

`--parent` 경로·불변·같은 파일 거절은 work-receipt 정본을 따른다.
이 스킬은 세션 체인으로만 쓴다:
[`references/capsule-parent-chain.md`](references/capsule-parent-chain.md),
[`references/work-receipt-boundary.md`](references/work-receipt-boundary.md).

## 절차 B — incoming: 세 파일을 읽고 재개한다

```
result.json.outcome == accepted 이고 nextAction.action == consume?
├─ 예 → collectedOutputs[].path 와 캡슐 receipt.outputSha256 을 대조
│        working doc 의 다음 명령만 실행
└─ 아니오
   ├─ nextAction.action == selfExecute → 위임을 접고 자체 실행 (새 위임 금지)
   ├─ missing capsule → 예외: 추측 재개 금지
   ├─ parent hash mismatch → 예외: 체인 재발급 전까지 후속 --parent 금지
   ├─ dirty named worktree → 예외: 그 트리를 checkout/reset 하지 말고 새 isolation
   └─ disk full → 예외: 산출을 더 쓰지 말고 인계를 미완으로 표시
```

예외 정본: [`references/exception-index.md`](references/exception-index.md),
[`references/exception-missing-capsule.md`](references/exception-missing-capsule.md),
[`references/exception-parent-hash.md`](references/exception-parent-hash.md),
[`references/exception-dirty-worktree.md`](references/exception-dirty-worktree.md),
[`references/exception-disk-full.md`](references/exception-disk-full.md).

## 절대 금지 (이 스킬이 닫는 운영 사고)

| 금지 | 이유 | 정본 |
|---|---|---|
| DocumentCore 편집 로직 발명 | 세션이 바뀌었다고 코어를 고치지 않는다. 기존 `rhwp edit`/`run` 만 | [`references/no-documentcore.md`](references/no-documentcore.md) |
| `git add -A` | 인계 산출·샌드박스·임시 저널이 스테이징된다 | [`references/staging-named-files.md`](references/staging-named-files.md) |
| 이름 붙은 워킹트리 checkout/훔치기 | `C:\Users\swsz9\rhwp`, `rhwp-desk*`, `rhwp-handoff`, `rhwp-scaffold-final`, `rhwp-doc-repro` 포함 | [`references/isolation-worktree.md`](references/isolation-worktree.md) |
| 새 CLI / `handoff` 명령을 만들지 않는다 | 오케스트레이터는 Python 도구다. 바이너리 명령을 추가하지 않는다 | 이 문서 · [`fixtures/catalog.json`](fixtures/catalog.json) |
| work-receipt 스킬 재작성 | 단건 증명의 정본은 그 스킬이다 | [`references/work-receipt-boundary.md`](references/work-receipt-boundary.md) |
| `gym/` 경로 | 이 스킬은 실사용 인수인계. 체육관 과제가 아니다 | 이 문서 |

## 판정은 데이터다

오케스트레이터 종료 코드는 DATP 상위 1자리: **0 수용 / 1 런타임 / 2 사용법 /
3 판정(인계) / 4 정책·boundary**. 3 과 4 는 도구 고장이 아니다. `outcome`·
`nextAction`·`findings[]` 를 읽고 분기한다.

`rhwp replay` 의 exit 3 도 같다 — 재현 실패는 판정. 상세는 work-receipt.

## 권장 흐름 (요청별)

- "컨텍스트가 바닥이다 / 창이 가득 찼다" → 절차 A, 트리거 `context_budget`
- "세션이 끊겼다 / 이어서 해" → 절차 B 먼저 (세 파일). 없으면 예외
- "시트 리필 / 다른 에이전트에게 넘겨" → 절차 A 후 후임에게 폴더 경로만
- "이 위임 결과가 믿을 만한가" → `orchestrator.py --verify-journal` + 캡슐 `--parent`
- "단건이 사실인가" → **이 스킬이 아님**. work-receipt 로 보낸다

판단 트리: [`references/decision-tree.md`](references/decision-tree.md).
레시피 색인: [`references/recipe-index.md`](references/recipe-index.md).
함정: [`references/pitfalls.md`](references/pitfalls.md).
봉투 필드: [`references/envelope-field-catalog.md`](references/envelope-field-catalog.md).
레퍼런스 목록: [`references/README.md`](references/README.md).
워킹 문서 형식: [`references/working-doc-handoff.md`](references/working-doc-handoff.md).

워크스루는 [`examples/README.md`](examples/README.md).
픽스처 카탈로그는 [`fixtures/catalog.json`](fixtures/catalog.json).
