# PR #5565 검토 기록

- PR: <https://github.com/edwardkim/rhwp/pull/5565>
- 작성자: `planet6897`
- 관련 이슈: #5555
- base / 원 head: `devel` / `3b496cd066de604c01f3b8bb13832615a9588a88`, `a1b1e8278920a14915000d6b07b8b2ed521d2123`
- 누적 적용: `453393787`, `7ba9e1a69` (`review/planet6897-20260819`)
- 공통 기록: [planet6897_20260819_integration_review.md](planet6897_20260819_integration_review.md)

## 변경 검토

HWP3 Latin-1 Supplement `0x00A0..=0x00FF`를 유니코드 코드값 그대로 통과시켜
`ü`, `ö`, `ä`, `ß`의 조용한 삭제를 막는다. 후속 commit은 새 source unit test를 남기지 않고
`tests/cases/issue_5555_hwp3_latin1_supplement.rs`로 옮겨 현재 테스트 수 래칫을 준수한다.

## 판정

데이터 손실을 항등 매핑으로 복구하고 범위 밖 문자의 기존 계약도 회귀로 고정했으므로 누적 수용을
권고한다. 공통 Rust CI가 최종 조건이다.
