# 요청 → 절차 판단 트리

```
사용자가 무엇을 말하나?
│
├─ "이 작업 증명해 / 영수증 / 감사 / 계보"
│    → 이 스킬이 아님. rhwp-work-receipt
│
├─ "컨텍스트가 바닥 / 창이 가득 / compact 예정"
│    → 트리거 context_budget → 절차 A (outgoing 닫기)
│
├─ "세션이 끊겼다 / 이어서 해 / 어디까지 했지"
│    → 트리거 session_interrupt → 절차 B (세 파일 읽기)
│    │    세 파일 없으면 예외 갈래
│
├─ "시트 리필 / 다른 에이전트에게 넘겨"
│    → 트리거 seat_refill → 절차 A 후 폴더 경로만 전달
│    │    이름 붙은 트리 경로를 주지 않는다
│
├─ "이 위임 결과가 믿기나 해"
│    → result.json + --verify-journal
│    │    단건 재현은 work-receipt replay verify
│
├─ "캡슐이 없다 / 부모가 안 맞는다 / 디스크가 가득 / 트리가 dirty"
│    → exception-index 네 갈래. 추측 재개 금지
│
├─ "코어를 고쳐서 이어서 하자"
│    → 거부. no-documentcore
│
└─ "폴더 전부 커밋해 / 저 워킹트리 써서 해"
     → 거부. staging-named-files / isolation-worktree
```

절차 A/B 는 SKILL.md. 레시피는 [`recipe-index.md`](recipe-index.md).
