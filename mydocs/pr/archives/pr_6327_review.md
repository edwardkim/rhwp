---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6327
---

# PR #6327 review - 한 글자 탭 문단의 block TAC 표 leading을 제외한다

## 검토 판단

**수용 권고.** 탭 한 글자만 있는 문단의 leading이 block TAC 표 위치에 중복 반영되지 않도록
경로를 한정한다. `issue_6167_leading_space_tac_table_own_line` focused test 1/1과 실제
2024 저장 fixture의 시각 비교가 같은 결론을 낸다.

## 라우팅과 증적

- 원 PR: https://github.com/edwardkim/rhwp/pull/6327
- 작성자 / reviewer: `t2c-lab` / `jangster77` review request 등록
- source head: `6e89cf1682ee39dc770d3ff26235ca7e180b5da2`
- fixture: `samples/issue6167/leading_space_tac_table.hwpx`
- `info --json`: `hancom-office-2024 13.0.0.1053`, 1쪽. 2024 bucket PDF
  `pdf/pr_6275/by_saved_version/pr6275_issue6167_leading_space_tac_table-2024.pdf`를 사용했다.
- visual sweep p1: 완료 1/1, 자동 flag 0, pixel match `95.86267%`.
  표 선과 입력 위치의 흐름 이탈은 보이지 않았다.
- 보관 asset: `mydocs/pr/assets/pr_6327_issue6167_{info,visual_sweep_summary}.json`,
  `mydocs/pr/assets/pr_6327_issue6167_p1_review.png`.

## 후속 코멘트

통합 PR merge 후 원 PR에는 2024 engine 선택 근거, focused 1/1, p1 flag 0과 대표 이미지를
merge SHA 고정 raw URL로 남긴다.
