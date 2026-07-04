# PR #1783 리뷰 구현 메모

## Stage 1. 상태 확인

완료.

- PR: https://github.com/edwardkim/rhwp/pull/1783
- reviewer assign: `@jangster77`
- 실제 커밋: `0a0dd5db7bbd944cd464e5d0ffef13d76b8e5d59`

## Stage 2. 로컬 적용

완료.

```bash
git cherry-pick 0a0dd5db7bbd944cd464e5d0ffef13d76b8e5d59
```

충돌 없음.

## Stage 3. 코드 검토

완료.

- `PageAreas::from_page_def_for_page` 는 용지 크기 0 에만 A4 폴백.
- 여백 차감은 `saturating_sub`로 변경.
- 본문 영역이 소멸하면 용지 5% 기본 여백으로 복구.
- `PageLayoutInfo`도 0-size PDF/SVG 방지를 위해 같은 A4 폴백을 적용.

## Stage 4. 검증

완료.

- `git diff --check upstream/devel..HEAD`
- `cargo fmt --check`
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`

## Stage 5. 판단

merge 후보.
