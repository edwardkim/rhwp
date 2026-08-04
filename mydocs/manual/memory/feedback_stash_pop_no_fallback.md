---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-26
name: stash-pop-no-fallback
description: "git stash pop을 `|| true`로 감싸지 말 것 — 낡은 stash가 트리를 오염시킨 사고 (2026-07-23)"
metadata: 
  node_type: memory
  type: feedback
---

`git stash -u` → checkout → `git stash pop || true` 패턴은 위험하다. 깨끗한 트리에서 stash -u는 아무것도 만들지 않고, 복귀 후 pop이 **스택에 남아 있던 1년 전 stash**(pr1021_pending)를 꺼내 충돌 상태로 풀었다. `|| true`가 실패를 숨겨 오염된 트리 위에 커밋까지 진행됐다(무관 파일 +448줄).

**Why:** rhwp 로컬 저장소에는 오래된 stash 3개가 상주한다(pr1021-cherry 등, 삭제 금지). pop은 "방금 내가 만든 stash"가 아니라 "스택 최상단"을 꺼낸다.

**How to apply:**
- stash를 쓸 때는 `git stash push -m "<고유표식>"` 후 `git stash pop`이 아니라 `git stash apply stash@{n}` + 표식 확인.
- 더 좋은 방법: 커밋된 깨끗한 트리면 stash 자체를 생략(체크아웃만으로 충분).
- 커밋 전 `git show --stat`으로 의도한 파일만 들어갔는지 확인 — 이 확인이 사고를 잡았다.
- 같은 계열: [[feedback_npm_verify_hygiene]] (브랜치 이탈 후 잔여물 정리).
