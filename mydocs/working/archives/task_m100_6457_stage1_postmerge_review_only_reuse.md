---
kind: working
status: complete
issue: 6457
last_verified: 2026-08-30
---

# #6457 병합 후 review-only worker 재사용

## 관측

PR #6456은 `mydocs/` 3개 파일만 변경했고 PR preflight에서 Proptest와 Adapter worker가 모두
`skipped`였다. 그러나 squash merge `7592e9c99dab3e359c41a86a1b5e1608afc861a5`의 `devel` push는
PR payload가 없어서 두 일반 preflight가 각각 `non-pull-request:push`와
`full-non-devel-pr:push`로 Full worker를 실행했다.

- Proptest post-merge run: 33300506177, worker 성공, 약 3분
- Adapter post-merge run: 33300506178, worker 성공, 약 3분

## 구현 경계

`trusted-postmerge-ci-reuse`에서 다음 조건을 모두 만족한 직접 review-only PR만 재사용한다.

1. 기존의 same-repository PR, 유일한 merge mapping, merge tree=head tree, enforcement surface 불변
   검사를 통과한다.
2. PR 전체 파일과 linear commit history가 review-only 허용 범위다.
3. 정확한 PR workflow run이 성공했고, 해당 preflight는 success이며 해당 heavy worker는 `skipped`다.

이 증거가 없으면 기존처럼 Full로 닫는다. code candidate 또는 trailing review-only 경로의 기존 판정은
변경하지 않는다.

## 검증 계획

- direct review-only post-merge 재사용과 worker-skip 증거 누락 fail-closed Node 단위 테스트
- CI impact, policy, trusted reuse Node 계약 묶음
- Python workflow 계약 전체와 `actionlint`
