---
name: 이슈 close 시 정정 commit 이 devel 에 머지됐는지 확인 필수
description: 이슈를 close 하기 전에 정정 commit 이 devel / main 에 실제 머지됐는지 git branch --contains 로 검증. close 만 하면 commit 이 임시 브랜치에만 남아 회귀 재발 위험
type: feedback
originSessionId: 263bacf0-af73-4169-8b90-2cc13f939a31
---
이슈를 close 하기 전에 **정정 commit 이 devel (또는 main) 에 실제 머지됐는지** 반드시 확인.

**Why:** Task #376 (2026-04-27) 은 commit `45419a2` 에 정정 코드가 있었고 이슈는 close 됐지만, 실제 commit 이 devel 에 머지되지 않고 `pr-360-head` 임시 브랜치에만 존재했음. 결과적으로 같은 결함 (hwpspec.hwp 20 페이지 빈 문단 + TAC Picture 이중 emit) 이 그대로 남아 Task #418 에서 재발견 → 재정정 필요.

**How to apply:** 이슈를 close 하기 전 다음 검증 절차:

```bash
# 정정 commit 이 어느 분기에 포함되었는지 확인
git branch --contains <commit-hash>

# devel / main 에 commit 이 들어있는지 확인
git merge-base --is-ancestor <commit-hash> devel && echo "devel 에 머지됨" || echo "devel 미머지"
```

`devel 미머지` 결과면 이슈 close 보류 + local/devel 머지 + push 후 close.

특히 외부 PR cherry-pick 처리 시 임시 브랜치 (`pr-XXX-head` 등) 에서 작업 후 local/devel 머지 단계가 누락되면 commit 이 사라진 것처럼 보이지만 실은 임시 브랜치에 남음. cherry-pick 직후 즉시 머지 + push 절차 준수.
