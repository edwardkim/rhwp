# 판단 트리 — 요청 → 단계

사용자 말을 8단 중 어디에 올릴지 고른다. 항상 단 1 부터 비어 있는
가장 앞 단계를 닫는다.

```
요청
├─ "이슈 있어?" / "이미 PR 있어?"
│    → 단 1. issue-first, duplicate
├─ "왜 이렇게 바꿔야 해?" / "정본이 뭐야?"
│    → 단 2. analyze-canonical
├─ "브랜치 만들어" / "devel 에서"
│    → 단 3. fetch upstream/devel, isolation worktree
├─ "이 경로 이미 worktree 야"
│    → 예외. 훔치지 말고 새 이름
├─ "구현해" / "코드 짜"
│    → 단 4. 범위 확인. DocumentCore·새 CLI 아니면 진행
├─ "커밋해"
│    → staging-named-files. git add -A 거부
├─ "fmt" / "포맷" / "린트"
│    → 단 5 HARD GATE. cargo fmt --all -- --check
├─ "cargo fmt --check 돌렸어"
│    → 거절. 낡은 표기. 다시 --all -- --check
├─ "clippy" / "테스트"
│    → clippy-and-tests
├─ "레이아웃 바뀌었어" / "페이지 수"
│    → visual-evidence
├─ "영수증" / "캡슐"
│    → work-receipt-pointers (스킬 재작성 금지)
├─ "working 문서"
│    → working-doc
├─ "PR 올려"
│    → 단 5 통과 여부 확인 → korean-pr, 첫 칸 fmt
├─ "CI 안 떠" / "noci"
│    → exceptions noci vs FAILURE
├─ "CI 빨개"
│    → FAILURE. 로그. fmt/clippy/test 재실행
└─ "gym 과제로 해보자"
     → 거절. 이 스킬은 실기여
```

확정이 안 되면 [procedure-order.md](procedure-order.md) 의 빈 칸부터 닫는다.
