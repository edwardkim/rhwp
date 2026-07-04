# PR #1840 리뷰 — #1773 원 인코딩 보존 설계 기록

## 메타

| 항목 | 내용 |
|------|------|
| PR | https://github.com/edwardkim/rhwp/pull/1840 |
| 작성자 | @planet6897 |
| base / head | `devel` / `docs/task1773-design` |
| 작성 시점 참고 head | `29e0823cbf2ae31e7052450623c80ebbfcc8be84` |
| 작성 시점 참고 상태 | `MERGEABLE`, 문서-only fast-pass |
| reviewer assign | @jangster77 지정 완료 |

## 변경 범위

- `mydocs/tech/task_m100_1773_record_only_encoding.md` 를 추가한다.
- #1773 원 인코딩 보존 구현은 HWP3 재판별로 철회하고, 계약/설계 기록을 보존한다.
- 코드 변경은 없다.

## 로컬 검증

- 체리픽 커밋: `29e0823cbf2a` -> `015c6ce1e`
- 충돌: 없음
- 문서 변경이지만 누적 검증 브랜치에서 `git diff --check`, release-test integration, Clippy 통과를 확인했다.

## 판단

조사/설계 기록 보존 PR 이며 구현 철회를 명확히 문서화한다. #1834 와 함께 #1773 의 진단/설계 상태를 정리하지만,
근본 이슈를 닫는 PR 로 보지는 않는다.

## 결론

merge 후보. #1773 close 여부는 merge 후에도 별도 판단한다.
