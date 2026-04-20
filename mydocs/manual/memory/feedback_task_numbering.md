---
name: 타스크 번호는 GitHub Issues로 채번
description: 수동 번호 할당 대신 gh issue create로 자동 채번. 중복 방지 및 협업 지원.
type: feedback
---

타스크 번호는 **GitHub Issues 자동 채번**을 사용한다. 수동 번호 할당 금지.

**Why:** 서로 다른 컨트리뷰터가 동일 번호를 할당하여 devel merge 시 충돌 발생 (Task 403/404 중복 사례).

**How to apply:**
1. 새 타스크 등록: `gh issue create --repo edwardkim/rhwp --title "제목" --body "설명"`
2. 반환된 Issue 번호(#1, #2, ...)를 타스크 번호로 사용
3. 브랜치: `local/task{issue번호}` (예: `local/task1`)
4. 커밋: `Task #1: 내용`
5. 완료 시: `gh issue close {번호}` 또는 커밋에 `closes #번호`
6. `mydocs/orders/`에서 `[#1](https://github.com/edwardkim/rhwp/issues/1)` 형식으로 참조
