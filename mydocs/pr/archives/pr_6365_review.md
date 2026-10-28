---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6365
---

# PR #6365 review - oracle page-count 원장의 오짝과 형식 태그를 보정한다

## 검토 판단

**수용 권고.** 원장 재생성기와 TSV의 불일치 원인을 오짝 PDF, 부분 수록, 확장자 태그로
명시하는 test/tooling 변경이다. 런타임 renderer 변경이 아니므로 별도 visual fixture는 요구되지 않는다.

## 근거

- 원 PR: https://github.com/edwardkim/rhwp/pull/6365
- 작성자 / reviewer: `kevin9327` / `jangster77` review request 등록
- source head: `3a9acd72a3fe09d6f963d6af9bb68991855bdee9`
- 통합 `oracle_page_count_baseline`: 16/16 통과.
- 전체 oracle sweep은 578문서(pdf 553, test 25) 중 564 일치(97.6%), 불일치 14,
  skip 1로 정상 종료했다.
- merge 후 원 PR에는 원장 gate와 전체 sweep 결과를 수용 근거로 남긴다.
