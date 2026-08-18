# work-receipt 와의 경계

두 스킬은 같은 `replay`/`--capsule`/`--parent` 를 보지만 **질문이 다르다**.

| | rhwp-work-receipt | rhwp-handoff |
|---|---|---|
| 질문 | 이 작업 하나가 사실인가 | 다음 세션이 어디서 이어 받는가 |
| 단위 | 단건 영수증·폴더 감사·계보 | 세션 인계 묶음 |
| 도구 | `rhwp replay` / `audit` / `lineage` | `tools/handoff/orchestrator.py` + 위 명령 **포인터** |
| 산출 | 3해시 영수증, 재현율, `brokenAt` | `result.json` + 세션 캡슐 + working doc |
| 트리거 | "증명해 / 감사 / 계보" | 컨텍스트 예산·세션 중단·시트 리필 |
| 재작성 | 이 스킬이 정본 | **다시 쓰지 않는다. 복제하지 않는다** |

## 이 스킬이 복사하지 않는 것

- attest / `--expect-output-sha256` verify 절차의 재서술
- `reproducedRate = reproduced/total` 공식
- `parentOk` · `lineageOk` · `reproduced` · `brokenAt` 의 재정의
- audit 비재귀 `*.capsule.json` 규칙의 재시험
- work-receipt 의 examples/ 와 fixtures/

필요할 때 한 줄로 보낸다:

> 단건 증명은 `.claude/skills/rhwp-work-receipt/SKILL.md`.

## 이 스킬만의 것

- 언제 넘기는가 (세 트리거)
- 오케스트레이터 `result.json` / 저널을 세션 머리에 두는 법
- incoming 이 세 파일을 읽는 순서
- 예외 네 갈래 (캡슐 부재, 부모 해시, dirty named worktree, disk full)
- DocumentCore / `git add -A` / named worktree 금지 (인계 맥락)

## 겹쳐 보이면

같은 `--parent` 를 써도 된다. 세션 캡슐은 그냥 `workCapsule` 이다.
이 스킬의 픽스처 캡슐은 **세션 시나리오 이름**을 쓰고, work-receipt 픽스처
파일명을 재사용하지 않는다.

두 스킬의 SKILL.md 를 한 커밋에서 동시에 고치지 않는다. 이 PR 은
`rhwp-work-receipt` 를 읽기만 한다.
