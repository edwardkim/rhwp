# PR #3525 검토 — HWP3 책갈피를 공통 Control로 보존

- 검토일: 2026-07-29
- 작성자: [@planet6897](https://github.com/planet6897)
- PR: https://github.com/edwardkim/rhwp/pull/3525
- base / 원본 head: `devel` / `a77e1712b3114232001f59bb0411d6be70535501`
- 규모: +129 / -7, 2 files (GitHub 조회 시점 기준)
- reviewer: `@jangster77` 배정 완료
- 관련 이슈: #3524

## 변경과 충돌 해소

HWP3 bookmark control을 임시 unknown field가 아니라 공통 `Control::Bookmark`로 만들어 저장
roundtrip에서 유실되지 않게 한다. 통합 적용 커밋은 `f83e8ee1a`이다.

최신 devel에는 bookmark 원시 부가 바이트를 `info_buf`로 넘기는 선행 변경이 있었다. 원 PR의
dispatcher 직접 삽입을 그대로 적용하면 control이 두 번 생길 수 있으므로, 충돌은 다음처럼 해소했다.

1. dispatcher의 `info_buf = bookmark_extra.to_vec()` 전달은 유지한다.
2. tail parser에서 control 하나만 `Control::Bookmark { name }`로 만든다.
3. unit expectation도 unknown field 문자열이 아닌 bookmark name으로 갱신한다.

해결 뒤 verify corpus에서 고쳐진 `hwp3-sample16.hwp` 실패 허용 항목을 제거한 collaborator 보정은
`699d0fe32`이다. 고쳐진 입력을 expected failure로 남기면 래칫이 역으로 실패하므로 필요한 갱신이다.

## fixture와 검증

- `samples/hwp3-sample11.hwp`: SHA-256
  `51b743b2823a2df9b6fac243f56aebecedbbd02e2a8baad58ffc2e5a4e695f20`.
- 한컴 2020 MCP로 생성한 기준 PDF:
  `pdf/hwp3-sample11-2020.pdf`, SHA-256
  `bb3a0d8c0d6be9dfde11875df23eedfb699a740c4d853260622987dcfe751721`, 151 pages.
  parser 구조 보존 PR이므로 이 PDF는 장기 재현 기준으로만 보관하며 visual 차이를 merge blocker로
  쓰지 않는다.
- `issue_hwp3_bookmark_native`: 2 passed; `convert_verify_corpus_ratchet`: 4 passed.
- 전체 통합 검증·최신 devel rebase·CI 진행 순서는
  [공통 구현 기록](pr_3503_review_impl.md)에 집계한다.

**권고: 수용.** 통합 PR CI 성공 뒤 #3524를 close한다.
