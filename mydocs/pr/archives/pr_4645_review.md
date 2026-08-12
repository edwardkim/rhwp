---
kind: pr-review
status: local-pass
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-12
---

# PR #4645 검토 - SVG 글꼴 파일 후보 경계

## 판정

로컬 수용. 문서 유래 글꼴 후보는 계획 단계에서 단일 normal path component로 제한하고, 파일시스템
루프는 검증된 `FontFileName`만 검색 root에 결합한다. nested path와 traversal 후보는 임베드되지 않고,
정상 direct-child face의 기존 동작은 유지된다.

## 검토 기준

- 원격 head: `0540769cda3ad77159763c48d973299eba9a6ac7`
- 로컬 누적 검토 브랜치: `review/humdrum00001010-20260812`
- 적용 순서: #4637 다음에 #4645의 6개 commit을 적용했다.

## 확인

- `cargo test --profile release-test --test issue_4645_font_lookup_boundary`: 1 passed.
- 통합 전체 Rust 회귀: 5,906 passed, 37 skipped.
- `cargo fmt --check`, `git diff --check`, `cargo clippy --all-targets -- -D warnings`, doctest 통과.

## 범위

font alias, bold candidate 우선순위, `font_paths`의 검색 root 순서는 보존했다. 렌더러의 SVG font
embedding 경계만 검증했으며 parser와 다른 backend의 파일 탐색 정책은 바꾸지 않는다.
