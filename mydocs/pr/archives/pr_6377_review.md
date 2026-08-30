---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6377
---

# PR #6377 review - PDF 원장 밖의 한글 쪽수 assertion도 sweep에 포함한다

## 검토 판단

**수용 권고.** CI gate를 추가하지 않는 독립 측정 도구이며, PDF 원장 밖 test assertion으로
조판 회귀를 보완한다. renderer 동작 변경이 없으므로 visual fixture 대상이 아니다.

## 근거

- 원 PR: https://github.com/edwardkim/rhwp/pull/6377
- 작성자 / reviewer: `kevin9327` / `jangster77` review request 등록
- source head: `5897f3a8e3be3ae83a924bc6008a9750652fd72e`
- 통합 sweep 실행: 578문서(pdf 553, test 25), 564 일치(97.6%), 불일치 14, skip 1.
- `--base` 재실행으로 개선 0 / 회귀 0을 확인했고, merge 후 원 PR에 해당 측정 결과를 남긴다.
