# PR #2147 검토 실행 기록

## 대상

- PR: `edwardkim/rhwp#2147`
- 제목: `Issue #2145: 개요번호 ^n/^N 레벨 경로 자동코드 구현 — 리터럴 '^N' 출력 수정`
- 원 commit: `841d3ab0c7d8fb02bef574c8834590988f9785d7`
- 로컬 체리픽 commit: `9a32c90d4`

## 실행

```bash
gh pr edit 2147 --repo edwardkim/rhwp --add-reviewer jangster77
git fetch upstream pull/2147/head:local/pr2147
git cherry-pick -x 841d3ab0c7d8fb02bef574c8834590988f9785d7
CARGO_INCREMENTAL=0 cargo test --profile release-test --lib expand_numbering_format -- --nocapture
```

## 후속 메모

재현 문서/PDF가 없으므로 단위 테스트 중심으로 검토했다.
