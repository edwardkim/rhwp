# 작업 영수증 — 포인터만 (스킬을 다시 쓰지 않는다)

문서를 실제로 편집·생성한 기여는 영수증을 남기는 것이 권장이다.
제출 조건은 아니다 (`AGENTS.md` 작업 증빙 절).

이 문서는 **명령 이름과 스킬 경로만** 가리킨다. `rhwp-work-receipt` 의
3해시·audit·lineage 계약을 여기 복제하지 않는다.

## 포인터

| 단 | 명령 | 스킬 |
|----|------|------|
| 영수증 | `rhwp replay --capsule` (`--plan-json <계획> --capsule work.capsule.json --json`) | `.claude/skills/rhwp-work-receipt/` |
| 검증 | `rhwp replay --plan-json <계획> --expect-output-sha256 <64hex> --json` | 같은 스킬 `references/replay-attest.md` |
| 감사 | `rhwp audit <폴더> --json` | 같은 스킬 `references/audit-accounting.md` |
| 계보 | `rhwp lineage <머리캡슐> --json` | 같은 스킬 `references/lineage-chronicle.md` |

연속 작업은 `--parent 이전.capsule.json`.

## 이 스킬에서 하는 일

1. 문서 편집이 있었는지 판단한다.
2. 있었으면 위 명령을 실행하라고 안내한다.
3. 나온 JSON 봉투를 PR 본문에 붙이라고 안내한다.
4. `rhwp-work-receipt` 의 `SKILL.md` 를 **수정하지 않는다.**

## 이 스킬에서 하지 않는 일

- `receipt` / `prove` 같은 새 명령을 발명하지 않는다
- 캡슐 스키마를 여기 다시 정의하지 않는다
- gym pack 으로 영수증을 채점하지 않는다

예제: [14_work_receipt_capsule.md](../examples/14_work_receipt_capsule.md).
