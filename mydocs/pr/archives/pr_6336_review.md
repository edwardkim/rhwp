---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-29 19:15 KST
pr: 6336
issue: 6335
author: lpaiu-cs
---

# PR #6336 review - source guard 주석 오염 차단

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6336
- 작성자: `lpaiu-cs`
- reviewer request: `jangster77` 등록 확인
- 원 PR head: `654ce91a7779522b91272ef6c8dc4704b529b50d`
- 통합 검토 브랜치: `review/lpaiu-cs-20260829`
- 최신 기준: `upstream/devel@cf366d2faad63a57fb663ce38b2e02d99b873e22`
- 적용 commit: `a71ceebc5`
- 원 PR 상태: non-draft, `CLEAN`, blocking/non-green checks 없음

## 검토 판단

**수용 권고.** 전문 소스 pin이 주석 속 과거 선언에 걸려 실제 선언 변경을 놓치는 계급을
`codeOnly()` helper로 줄인다. 문자열과 줄 구조를 보존하고 주석만 제거하는 방식이라 기존 소스
가드의 진단성과 문자열 pin 계약을 해치지 않는다.

## 증적과 검증

- `source-guard-support.test.ts`: 3 pass
- `command-history-snapshot.test.ts` + `source-guard-support.test.ts`: 13 pass
- Studio test run: 1242 pass, 1 skipped
- 공통 검증과 head 증적:
  `mydocs/pr/assets/pr_6331_6333_6336_6351_6356_validation_20260829.md`

## 코멘트 처리

merge 후 원 PR에는 주석 디코이 차단 helper와 고위험 pin 전환을 targeted Studio 검증으로 확인했고,
추가 메인터너 보정 없이 수용했다는 점을 남긴다.
