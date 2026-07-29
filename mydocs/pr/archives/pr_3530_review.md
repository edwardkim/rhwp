# PR #3530 검토 — 표 캡션 경계를 직계 레벨로 판정

- 검토일: 2026-07-29
- 작성자: [@planet6897](https://github.com/planet6897)
- PR: https://github.com/edwardkim/rhwp/pull/3530
- base / 원본 head: `devel` / `7c81725b2b12623dd6604c693cc62b62aac7b704`
- 규모: +398 / -4, 5 files (GitHub 조회 시점 기준)
- reviewer: `@jangster77` 배정 완료
- 관련 이슈: #3528

## 변경과 적층 처리

중첩 표가 캡션 경계 판단을 가로채지 않도록 직접 자식 레벨에서만 경계를 판정한다. 원 PR은 #3527의
두 커밋 위에 적층돼 있으므로 이미 적용한 `5d7f3473b`/`7f93ad3cf`은 중복 cherry-pick하지 않고,
본질 증분만 `e25f3c376`으로 적용했다.

CLI 회귀 테스트의 runtime binary 탐색은 collaborator 보정 `639e6250d`으로 통합했다. 이는 archive
환경 호환만 바꾸며 nested-caption 기대값과 parser 조건은 원 PR 그대로다.

## 검증과 판정

- `issue_3528_nested_caption_boundary`: 1 passed.
- 전체 통합 검증·최신 devel rebase·CI 진행 순서는
  [공통 구현 기록](pr_3503_review_impl.md)에 집계한다.
- parser 경계 변경으로 layout visual 차이 자체를 수용 근거로 사용하지 않는다.

**권고: 수용.** 통합 PR CI 성공 뒤 #3528을 close한다.
