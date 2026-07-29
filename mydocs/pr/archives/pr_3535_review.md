# PR #3535 검토 — char_count 문단 종결자 규약 통일

- 검토일: 2026-07-29
- 작성자: [@planet6897](https://github.com/planet6897)
- PR: https://github.com/edwardkim/rhwp/pull/3535
- base / 원본 head: `devel` / `e910c1e5914313bde2badc130feaa0bed251e3d9`
- 규모: +153 / -7, 4 files (GitHub 조회 시점 기준)
- reviewer: `@jangster77` 배정 완료
- 관련 이슈: #3494

## 변경과 충돌 해소

HTML import/table import와 HWP3 parser의 `char_count`를 문단 종결자를 포함하는 UTF-16 규약으로
통일해 IR diff 928건의 잡음을 없앤다. 통합 적용은 `5eb0d447b`이다.

최신 devel에는 이미 #3510의 HWP3 `utf16_len + 1` 보정이 있었으므로 같은 코드를 중복 적용하지
않고 유지했다. PR의 HTML import 두 경로와 새 regression test를 적용하고, 해당 주석에는
`#3494, #3510`의 공통 규약을 명시했다. 이로써 선행 수정의 효과를 보존하면서 실제 충돌을
해소했다.

## 검증과 판정

- `issue_3494_char_count_convention`: 2 passed.
- 전체 통합 검증·최신 devel rebase·CI 진행 순서는
  [공통 구현 기록](pr_3503_review_impl.md)에 집계한다.
- IR/파서 계약 변경이며 paint/layout 변경이 없으므로 visual sweep은 필수 경로가 아니다.

**권고: 수용.** 통합 PR CI 성공 뒤 #3494를 close한다.
