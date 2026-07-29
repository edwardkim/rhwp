# PR #3527 검토 — convert --verify 코퍼스 래칫

- 검토일: 2026-07-29
- 작성자: [@planet6897](https://github.com/planet6897)
- PR: https://github.com/edwardkim/rhwp/pull/3527
- base / 원본 head: `devel` / `257b8cea18219cbf8a0c8c8cf22f14e02832a763`
- 규모: +290 / -3, 3 files (GitHub 조회 시점 기준)
- reviewer: `@jangster77` 배정 완료
- 관련 이슈: #3505

## 변경과 검토

`convert --verify`를 corpus partition 래칫으로 만들고 현재 알려진 손실을 명시해, 해결된 항목의
재유입을 막는다. 기능 커밋은 `5d7f3473b`, 그 래칫 근거 문서는 `7f93ad3cf`으로 통합 적용했다.

`#3525`가 bookmark 실패를 실제로 해결했으므로 이에 해당하는 allow-list를 제거하는
`699d0fe32` 보정도 함께 필요했다. 이 변경은 기대 실패 수를 임의로 낮춘 것이 아니라, 원인 test가
통과하도록 고친 결과를 래칫 기준에 반영한 것이다.

## 검증과 판정

- `convert_verify_corpus_ratchet`: partition 4/4 passed (13.33s).
- 전체 통합 검증·최신 devel rebase·CI 진행 순서는
  [공통 구현 기록](pr_3503_review_impl.md)에 집계한다.
- CLI/검증 경로이며 layout 출력 변경이 없으므로 visual sweep은 적용 대상이 아니다.

**권고: 수용.** 통합 PR CI 성공 뒤 #3505를 close한다.
