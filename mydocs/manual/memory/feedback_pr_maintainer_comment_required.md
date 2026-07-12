---
name: feedback_pr_maintainer_comment_required
description: PR 머지/close 후 PR 자체에 메인테이너 코멘트(처리 결과+검증 요약+감사)를 반드시 등록
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 2560b31a-9f1c-4764-bbf1-7ba5fc27c7ce
---

PR 처리(merge/close)를 마친 뒤 **PR 자체에 메인테이너 코멘트를 반드시 등록**한다. `gh pr merge --admin`이
GitHub에 "Merged" 상태를 자동 표시하지만, 처리 근거·검증 요약·감사 메시지는 별도이며 생략하면 안 된다.

**Why**: 2026-06-29 세션에서 #1635/#1485/#1641/#1640 4건을 admin merge 했으나 PR 코멘트를 생략(이슈에는
달았지만 PR에는 없음). 작업지시자가 "PR 처리후 메인테이너의 커멘트 추가하기를 생략하네요" 지적. 외부
컨트리뷰터 PR은 머지 결과와 감사를 PR에 남기는 것이 예우이고, 처리 투명성의 일부다.

**How to apply**:
- 코멘트 내용 = ① 처리 결과(admin merge / cherry-pick 통합 / close + 사유) ② 핵심 검증 요약(CI·회귀·시각
  판정·보안 테스트 중 해당분) ③ 감사. 필요 시 연결 이슈/후속 사항(예: dist 재빌드, 분리 이슈) 명시.
- 톤은 차분·사실 중심([[feedback_pr_comment_tone]]), 첫 PR이면 환영([[feedback_first_pr_courtesy]]).
- cherry-pick 통합 close 시엔 "squash 통합/devel 반영 sha" 안내 포함.
- 이슈 close 코멘트와 **별개** — PR 코멘트, 이슈 코멘트 둘 다 챙긴다.
- 보고서 아카이브·devel push 와 함께 PR 처리 마무리 체크리스트에 포함.

관련: [[feedback_pr_comment_tone]] [[feedback_first_pr_courtesy]] [[feedback_commit_reports_in_branch]]
