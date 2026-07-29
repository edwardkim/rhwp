# PR #3503 검토 — 미주 앞 실제 공백 보존

- 검토일: 2026-07-29
- 작성자: [@planet6897](https://github.com/planet6897)
- PR: https://github.com/edwardkim/rhwp/pull/3503
- base / 원본 head: `devel` / `85f813d9ae8231cd14e6f03866866ce693ae5b1f`
- 규모: +126 / -4, 2 files (GitHub 조회 시점 기준)
- reviewer: `@jangster77` 배정 완료
- 관련 이슈: #3495

## 변경과 검토

HWpx roundtrip 직렬화에서 미주 앞의 실제 공백을 자동번호 placeholder로 오인해 버리던 경로를
분리하고 재현 fixture를 추가한다. 원본 단일 커밋은 최신 `upstream/devel` 위 통합 검토 브랜치
`review/planet6897-20260729`에 `1f8b471e0`으로 적용했다.

`tests/issue_3495_endnote_space_eaten.rs`는 `CARGO_BIN_EXE_rhwp`가 없는 nextest archive에서도
실행되도록 runtime 탐색을 보완했다. 이는 contributor 원 변경과 구분되는 collaborator 보정
`639e6250d`이며, 기능 조건이나 기대값은 바꾸지 않는다.

## 검증과 판정

- 해당 회귀 테스트: 2 passed.
- 전체 통합 검증·최신 devel rebase·CI 진행 순서는
  [공통 구현 기록](pr_3503_review_impl.md)에 집계한다.
- serializer 구조 보존 변경이므로 별도 visual sweep은 merge 판단의 필수 조건이 아니다.

**권고: 수용.** 최신 통합 PR의 required CI가 모두 성공하면 squash merge하고 #3495를 close한다.
