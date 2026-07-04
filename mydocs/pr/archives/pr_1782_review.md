# PR #1782 리뷰 — Task #1773 TextRun ±1 판별 조사

## PR 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1782 |
| 작성자 | @planet6897 |
| base / head | `devel` / `planet6897:pr/devel-1773` |
| 관련 이슈 | #1773 |
| reviewer assign | @jangster77 요청 완료 |
| 적용 방식 | 비시리즈·샘플 미포함 PR 누적 cherry-pick |

## 변경 범위

- `mydocs/plans/task_m100_1773.md`

소스 변경 없이 TextRun ±1 차이를 레거시 무-컨트롤문자 구역정의 직렬화 정규화 기인으로 판별한다.

## 검토 결과

부분 조사 문서이며, 즉시 코드 수정으로 확대하지 않고 원인 계통을 남긴다. 문서-only PR 로 fast-pass 대상이다.

## 검증

- 누적 cherry-pick 충돌 없음
- `git diff --check upstream/devel..HEAD`: 통과
- `cargo fmt --check`: 통과
- 전체 cargo 테스트/clippy는 누적 브랜치에서 통과

## 결론

문서-only 조사 PR로 merge 가능하다.
