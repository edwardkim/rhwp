---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-26
name: docs-only-ci-exempt
description: "devel push 시 뜨는 \"Required status check Build & Test is expected\"는 문서 전용 변경에서는 무시한다"
metadata: 
  node_type: memory
  type: feedback
---

`devel`에 push하면 브랜치 보호 규칙 때문에 다음 메시지가 나올 수 있다.

```
remote: - Required status check "Build & Test" is expected.
```

**문서 전용 변경(마크다운만 수정)은 이 상태 체크의 예외다.** push는 이미 통과한
것이며, CI 완료를 기다리거나 결과를 확인할 필요가 없다.

**Why:** 2026-07-26 메모리 덤프 현행화 push에서 이 메시지를 보고 CI 확인 여부를
질의하자 작업지시자가 "문서 전용은 예외"로 확정. 빌드에 영향이 없는 변경까지
CI 확인 절차를 붙이면 불필요한 왕복이 생긴다.

**How to apply:**
- 변경 파일이 `mydocs/**` 등 문서뿐이면 이 메시지를 보고도 CI 확인을 제안하지 않는다.
- 소스·설정·워크플로 파일이 섞였으면 예외가 아니다. 이때는
  [[feedback_pr_ci_before_pr]]와 [[feedback_push_full_test_required]]를 따른다.
- 진행중 CI의 수치를 성능 근거로 보고하지 않는 규칙은 별개다
  ([[feedback_no_metrics_from_inprogress_ci]]).

**같이 볼 것:** `devel`은 dependabot이 수시로 커밋을 밀어 넣어 push가 경합으로
거부되기 쉽다. push 직전 `git fetch` → rebase → 즉시 push가 안전하다
([[feedback_stash_pop_no_fallback]]의 트리 위생과 같은 계열).
