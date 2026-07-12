# PR #2056 처리 계획 — 옵션 1 self-merge 후보

## 1. 처리 방향

PR #2056 은 collaborator self-merge 후보로 처리한다. 작업지시자가 옵션 1을 지시했으므로 review 문서, visual
asset, 오늘할일을 같은 PR head 에 포함한다.

## 2. 완료된 준비

| 항목 | 상태 |
|------|------|
| PR 생성 | 완료: https://github.com/edwardkim/rhwp/pull/2056 |
| base/head | `devel` ← `task/m100-2020-remaining-visual` |
| related issue | #2020, PR body 에 `Closes #2020` 포함 |
| visual evidence | `mydocs/pr/assets/pr_2056_issue2020_*` 파일명으로 review/overlay PNG, summary JSON, manifest 보존 |
| review 문서 | `mydocs/pr/archives/pr_2056_review.md`, `pr_2056_review_impl.md` |
| 오늘할일 | `mydocs/orders/20260708.md` 에 #2056 처리 기록 추가 |

## 3. 로컬 검증 완료

PR 생성 전 코드 기준으로 다음을 통과했다.

```bash
cargo build --release
cargo test --release --lib
CARGO_INCREMENTAL=0 cargo test --profile release-test --tests
cargo fmt --check
git diff --check
cargo clippy --all-targets -- -D warnings
cargo test --doc
wasm-pack build --target web --out-dir pkg
cd rhwp-studio && npx tsc --noEmit
cd rhwp-studio && npm test
```

대표 visual sweep 6개 대상도 모두 `flagged_page_count=0` 으로 통과했다.

## 4. 남은 단계

1. 본 review 문서/오늘할일 커밋을 PR head 에 push 한다.
2. GitHub Actions 가 최신 PR head 기준으로 통과하는지 확인한다.
3. 작업지시자 merge 승인을 받은 뒤 merge 한다.
4. merge commit 을 확인한다.
5. #2020 auto-close 여부를 확인한다. 지연 후에도 open 이면 수동 close 한다.
6. #2020 후속 코멘트에 다음을 기록한다.
   - PR #2056 및 merge commit
   - GitHub Actions 결과
   - 로컬 전체 검증 결과
   - visual sweep 증적 경로
   - 한컴 전용 폰트 부재에 따른 font fidelity 분리 판단
7. `upstream/devel` 동기화 후 로컬/원격 작업 브랜치를 정리한다.

## 5. fast-pass 판단

본 PR 의 본질 커밋에는 renderer/layout/test/sample/asset 변경이 있으므로 원 코드 PR 자체는 heavy CI 대상이다.
다만 PR 생성 뒤 추가하는 본 문서/오늘할일/asset 정리 커밋은 review evidence 보존 목적의 후속 커밋이므로,
내용상 9.3.1 의 후속 기록 fast-pass 성격을 갖는다. 최종 merge 판단은 최신 PR head 의 GitHub Actions 상태와
작업지시자 승인으로 확정한다.
