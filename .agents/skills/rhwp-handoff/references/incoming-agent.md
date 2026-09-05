# incoming 에이전트가 읽는 것

후임은 대화를 복원하지 않는다. 아래 세 파일만 읽는다. 순서는 고정이다.

1. last `result.json`
2. last `*.capsule.json` (머리가 `session.capsule.json`)
3. last working doc (`mydocs/working/…`)

## 1. result.json

[`result-json.md`](result-json.md) 대로 `outcome`·`nextAction`·`collectedOutputs`
를 읽는다. 파일이 없으면 오케스트레이터 위임이 없었던 세션일 수 있다 —
그때는 2번과 3번만으로 재개할지 working doc 이 명시해야 한다. 명시가 없으면
예외.

## 2. capsule

`rhwp replay --capsule` 로 발급된 `workCapsule` 이다. 세션 스킬은 스키마를
바꾸지 않는다.

읽는 키:

- `kind` == `workCapsule`
- `receipt.inputSha256` / `planSha256` / `outputSha256`
- `parent` 가 있으면 `parent.capsule` (캡슐 파일 기준 상대) 과 `parent.sha256`
- `planText` 와 `plan` 이 같은 객체인지

`--parent` 가 가리키는 파일을 열어 해시가 같은지 확인한다. 다르면
[`exception-parent-hash.md`](exception-parent-hash.md). 파일이 없으면
[`exception-missing-capsule.md`](exception-missing-capsule.md).

단건 재현이 필요하면 work-receipt 절차 1 로 보낸다. 이 스킬에서
`rhwp audit`/`lineage` 를 다시 설명하지 않는다.

## 3. working doc

[`working-doc-handoff.md`](working-doc-handoff.md) 칸을 채운 문서.
최소:

- 이슈/목표 한 줄
- 트리거 (`context_budget` / `session_interrupt` / `seat_refill`)
- `taskId`
- 세 파일의 경로
- 남은 명령 (기존 CLI 만)
- 하지 말 것 (DocumentCore, `git add -A`, named worktree)

칸이 비어 있으면 추측하지 않는다.

## 재개 루프

```
세 파일을 읽는다
→ result.outcome 분기
→ 캡슐 parent 해시 확인
→ working doc 의 다음 명령 1개만 실행
→ 그 명령의 봉투를 새 세션의 시작 산출로 삼는다
→ 다시 컨텍스트 예산이 보이면 절차 A 로 인계를 닫는다
```

한 턴에 새 탐색과 새 위임을 같이 하지 않는다. incoming 의 첫 턴은 **읽기**다.

## 읽지 않는 것

- 호스트 대화 요약
- `sandbox_*`
- 이름 붙은 워킹트리의 미커밋 파일
- 다른 스킬의 SKILL.md 를 고쳐서 기억을 이식
- gym pack 점수

## 픽스처

`fixtures/incoming/read-order.json` 이 세 경로와 순서를 고정한다.
`fixtures/incoming/first-turn.json` 이 "첫 턴은 읽기" 를 고정한다.
