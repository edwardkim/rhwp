---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6359
---

# PR #6359 review - 개체 host 뒤 저장 vpos의 쪽 상단 의미를 보존한다

## 검토 판단

**수용 권고.** 문서 명령 경로에서 저장된 vpos reset을 쪽 상단 좌표로 보존하는 좁은 수정이며,
새 렌더 출력 또는 fixture 추가가 없다. `issue_2158_hwpx_vpos_reset_preserve` 2/2와 통합 전체
회귀가 통과했다.

## 라우팅과 코멘트

- 원 PR: https://github.com/edwardkim/rhwp/pull/6359
- 작성자 / reviewer: `kevin9327` / `jangster77` review request 등록
- source head: `5727a2c449db010d8593a3d4e5916c7fb3a72a9e`
- 변경은 `src/document_core/commands/document.rs`와 구현 기록으로 한정된다.
- merge 후 원 PR에는 focused 2/2 및 전체 release-test 통과를 수용 근거로 남긴다.
