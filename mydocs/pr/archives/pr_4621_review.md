---
kind: pr-review
status: local-pass
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-12
---

# PR #4621 검토 - 측정 상태 정리와 문서 교체 캐시 초기화

## 판정

로컬 수용. 프로덕션 페이지네이션이 사용하지 않는 문단 측정 상태를 `fallback_paragraphs`로 명확히
분리하고, `set_document`가 이전 문서의 문단·표 측정 캐시를 재사용하지 않도록
`rebuild_derived_state()`로 일원화한다.

## 검토 기준

- 원격 head: `1ec588af9b01c14d084b33d6544ea2b174a4745f`
- 로컬 누적 검토 브랜치: `review/humdrum00001010-20260812`
- 적용 순서: #4607 다음에 #4621의 3개 commit을 적용했다.

## 확인

- 새 회귀는 긴 표/문단 문서를 넣은 core에 다른 문서를 다시 설정한 뒤, 새 core의 동일 문서 측정값과 일치하는지 확인한다.
- `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`: 5,906 passed, 37 skipped.
- `cargo fmt --check`, `git diff --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --profile release-test --doc`를 통과했다.

## 범위

renderer의 실제 조판 알고리즘이나 page-break 규칙을 바꾸지 않는다. 캐시 이름은 소비 경계를 드러내고,
문서 교체 시 파생 상태를 모두 다시 만드는 기존 복원 경로와 같은 계약으로 맞춘다.
