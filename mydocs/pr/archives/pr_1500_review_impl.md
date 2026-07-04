# PR #1500 시리즈 처리 계획 - #1412식 통합 cherry-pick

- 작성일: 2026-06-24
- 대상 PR: #1500, #1502, #1507
- 비대상: 별도 foundation 후보 PR
- 처리 branch: `pr1500-series-integration`
- 통합 PR: #1511
- 처리 방식: contributor PR 커밋 cherry-pick + maintainer 보정 커밋 + 단일 integration PR

## 1. 목표

개별 PR head에 review 문서를 push하지 않고, #1412 선례처럼 maintainer 쪽 integration branch에서 시리즈를
한 번에 검토한다. 이렇게 하면 review 문서 커밋으로 각 contributor PR의 CI를 반복 실행하지 않는다.

## 2. 완료한 작업

- `local/devel` 기준 `pr1500-series-integration` 생성
- #1500 non-merge 커밋 3개 cherry-pick
- #1502 고유 커밋 2개 cherry-pick
- #1507 고유 커밋 2개 cherry-pick
- maintainer 보정 커밋 1개 추가
- 별도 foundation 후보 PR은 시리즈 대상이 아니므로 제외

## 3. 진행 상태

### Stage A - 로컬 전체 검증 완료

```bash
cargo build --release
cargo test --release --lib
cargo test --profile release-test --tests
cargo fmt --check
git diff --check
cargo clippy --all-targets -- -D warnings
cargo test --doc
wasm-pack build --target web --out-dir pkg
cd rhwp-studio && npx tsc --noEmit
cd rhwp-studio && npm test
```

결과: 모두 통과.

렌더 영향 확인:

```bash
cargo test --test svg_snapshot
```

`cargo test --profile release-test --tests`에 포함되어 통과했다.

### Stage B - push 및 PR 생성 완료

작업지시자 승인 후 수행:

```bash
git push upstream pr1500-series-integration:pr1500-series-integration
gh pr create --repo edwardkim/rhwp --base devel --head edwardkim:pr1500-series-integration
```

PR 제목 후보:

```text
PR #1500 series: render-diff 게이트와 HWPX 직렬화 충실도 보강
```

PR 본문에는 다음을 포함한다.

- Cherry-picked from #1500, #1502, #1507
- Supersedes #1500, #1502, #1507
- 별도 foundation 후보 PR은 이번 시리즈에서 제외/보류
- Closes #1499, #1501, #1505, #1506
- 로컬 검증 결과

생성된 PR: #1511

### Stage C - 원 PR 코멘트/close

integration PR이 merge된 뒤 #1500, #1502, #1507에 남길 코멘트 초안:

```text
@planet6897 감사합니다. 이 PR은 시리즈 통합 브랜치에서 cherry-pick으로 반영했습니다.

반영 방식:
- #1500/#1502/#1507의 원 커밋을 maintainer integration PR에 cherry-pick
- render-diff의 비-PASS exit code와 visual xfail failure-class 가드를 maintainer 보정 커밋으로 추가
- GitHub Actions는 통합 PR 기준으로 검증

이 PR은 통합 PR로 대체 반영되었으므로 close하겠습니다.
```

별도 foundation 후보 PR에는 작업지시자 지시에 따라 comment, review, close, cross-reference를 남기지 않고 보류한다.

### Stage D - CI 대기 및 merge 후 후속 처리

- #1511 GitHub Actions 최신 통과 확인
- integration PR CI 최신 통과 확인
- 작업지시자 승인 후 merge
- #1499/#1501/#1505/#1506 close 상태 확인, 필요 시 수동 close
- #1500/#1502/#1507 close/comment
- review/report 문서 archive 이동
- `local/devel` sync

## 4. 승인 필요 사항

remote push, integration PR 생성, 원 PR comment/close는 변경 범위와 검증 결과를 보고한 뒤 작업지시자 승인으로 수행한다.
