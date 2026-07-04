# PR #1783 리뷰 — Task #1583 PageDef 0/손상 방어 폴백

## PR 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1783 |
| 작성자 | @planet6897 |
| base / head | `devel` / `planet6897:pr/devel-1583` |
| 관련 이슈 | #1583 |
| reviewer assign | @jangster77 요청 완료 |
| 적용 방식 | 비시리즈·샘플 미포함 PR 누적 cherry-pick |

## 변경 범위

- `src/model/page.rs`
- `src/renderer/page_layout.rs`
- `mydocs/plans/task_m100_1583.md`

손상/미설정 PageDef 의 용지 크기 0 또는 여백 과대 입력에서 렌더링 본문 영역이 0 이 되는 문제를 A4 및 5%
기본 여백 폴백으로 방어한다. IR 자체는 바꾸지 않는 렌더링측 방어다.

## 검토 결과

정상 PageDef 경로는 기존 값을 그대로 쓰고, 용지 크기 0 또는 본문 영역 소멸 조건에서만 폴백한다.
`saturating_sub`로 debug underflow 위험도 같이 제거한다. 페이지 px 계산도 동일한 A4 폴백을 적용해 0-size
SVG/PDF 실패를 막는다.

## 검증

- 누적 cherry-pick 충돌 없음
- `git diff --check upstream/devel..HEAD`: 통과
- `cargo fmt --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과

## 결론

입력 방어 범위가 좁고 회귀 테스트가 포함되어 있다. merge 후보로 판단한다.
