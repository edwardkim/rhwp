---
kind: working
status: active
issue: 5339
handoffTrigger: session_interrupt
taskId: t-ord
---

# incoming 스냅샷

## 인계 머리
- result: output/handoff/t-ord/result.json
- capsule: output/handoff/t-ord/session.capsule.json
- parent: output/handoff/t-ord/parent.capsule.json
- journal: output/handoff/t-ord/handoff.journal.ndjson

## 남은 목표
부칙 별표 CSV 연도를 2026 으로 고친 뒤 같은 자리에 되돌린다.

## 다음 명령
rhwp csv-to-table samples/forms/ordinance_2024.hwp --csv output/handoff/t-ord/s04-tables-fixed.csv --table 0 --dry-run --json

## 하지 말 것
- DocumentCore 편집 로직 발명 금지
- git add -A 금지
- 이름 붙은 워킹트리 checkout 금지
