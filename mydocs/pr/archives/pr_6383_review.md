---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6383
---

# PR #6383 review - HWP3 사적 문자와 아래아 채움 초성을 보존한다

## 검토 판단

**수용 권고.** `decode_johab` 미매핑이 `'?'`로 바뀐 뒤 HWP3 parser가 문자를 버리던 경로를
samples 실측 매핑으로 닫는다. 저장/renderer 변경이 아니라 parser 문자 보존 수정이다.

## 근거와 코멘트

- 원 PR: https://github.com/edwardkim/rhwp/pull/6383
- 작성자 / reviewer: `chrisryugj` / `jangster77` review request 등록
- source head: `117d5d4eeb382675ea451f1425361c51d7ae5b6c`
- `issue_6380_hwp3_samples_symbol_map`: 7/7 통과.
- 최신 통합 head의 manifest, clippy, full release-test가 통과했다. merge 후 원 PR에는
  HWP3 sample 7건과 전수 회귀 결과를 수용 근거로 남긴다.
