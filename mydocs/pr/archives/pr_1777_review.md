# PR #1777 리뷰 — Task #1775 CFB 스트림 경로 구분자 테스트 Windows 정합

## PR 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1777 |
| 작성자 | @planet6897 |
| base / head | `devel` / `planet6897:pr/devel-1775` |
| 관련 이슈 | #1775 |
| reviewer assign | @jangster77 요청 완료 |
| 적용 방식 | 비시리즈·샘플 미포함 PR 누적 cherry-pick |

## 변경 범위

- `tests/issue_852_hwpx_to_hwp_contract_streams.rs`
- `mydocs/plans/task_m100_1775.md`

Windows 에서 CFB stream path 표시가 `\` 구분자를 사용할 수 있어, 테스트 비교용 문자열만 `/`로 정규화한다.
프로덕션 코드 변경은 없다.

## 검토 결과

테스트 수집 경로에서만 `replace('\\', "/")`를 적용하므로 macOS/Linux의 기존 `/` 경로에는 영향이 없다. CFB
저장/파싱 로직은 건드리지 않고, 테스트 기대값의 OS 차이만 제거한다.

## 검증

- 누적 cherry-pick 충돌 없음
- `git diff --check upstream/devel..HEAD`: 통과
- `cargo fmt --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과

## 결론

테스트 전용 정규화로 범위가 좁고 전체 검증도 통과했다. merge 후보로 판단한다.
