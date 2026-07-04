# PR #1482 self-merge 실행 계획

- PR: https://github.com/edwardkim/rhwp/pull/1482
- 작성일: 2026-06-23
- 경로: collaborator self-merge 후보
- 문서 작성 시점 참고 head: `8f1716a9` (본 review 문서/최종 보정 커밋 전)

## 1. 목적

PR #1482는 #1481 표 줄/칸 입력·지우기 회귀와 표 생성 직후 탈출 회귀를 보정하는 collaborator self-merge 후보 PR이다.

기존 PR head에는 이전 PR review 문서 전용 커밋이 중간에 포함되어 있었으므로, `mydocs/manual/pr_review_workflow.md` 8장에 따라 review 문서를 최종 PR head에 다시 포함하기 위해 해당 커밋을 히스토리에서 제거하고 `upstream/devel` 기준으로 rebase했다.

## 2. 커밋 목록

문서 작성 시점 로컬 커밋:

| 순서 | SHA | 제목 |
|------|-----|------|
| 1 | `554dce58` | `task 1481: 표 줄칸 편집 회귀 보정` |
| 2 | `be3702d6` | `task 1481: 줄칸 메뉴와 행 높이 회귀 보정` |
| 3 | `d3ce087f` | `task 1481: macOS 줄칸 단축키 보정` |
| 4 | `1a706395` | `task 1481: 줄칸 추가 단축키 통일` |
| 5 | `ee80a390` | `task 1481: 표 resize 높이 회귀 보정` |
| 6 | `8f1716a9` | `task 1481: 표 생성 탈출 회귀 보정` |

최종 문서/보정 커밋 예정:

- `mydocs/pr/archives/pr_1482_review.md`
- `mydocs/pr/archives/pr_1482_review_impl.md`
- `mydocs/orders/20260622.md`
- `mydocs/working/task_m100_1481_stage6.md`
- Stage 6 보정 소스와 회귀 테스트
- clippy `needless_return` 보정

## 3. 진행 단계

### Stage A - 히스토리 재작성

1. 작업분 stash 보관
2. 이전 PR review 문서 전용 커밋 제외 rebase
3. `upstream/devel` 기준 rebase 확인
4. stash 재적용
5. 이전 PR review 문서 전용 커밋이 현재 브랜치 히스토리에서 제거되었는지 확인

### Stage B - 전체 로컬 검증

다음 검증을 수행했다.

- `cargo build --release`
- `cargo test --release --lib`
- `cargo test --profile release-test --tests`
- `cargo fmt --check`
- `git diff --check`
- `cargo test --test svg_snapshot`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --doc`
- `cd rhwp-studio && npx tsc --noEmit`
- `cd rhwp-studio && npm test`
- `wasm-pack build --target web --out-dir pkg`

### Stage C - review 문서 재작성

1. `mydocs/pr/archives/pr_1482_review.md` 작성
2. `mydocs/pr/archives/pr_1482_review_impl.md` 작성
3. `mydocs/orders/20260622.md`에서 이전 PR review 문서 전용 커밋 기록 제거
4. 오늘할일에 최종 로컬 검증과 PR head 재작성 계획 기록

### Stage D - 기존 PR head 갱신

기존 PR #1482 head는 `upstream/task_m100_1481`이다. 히스토리에서 이전 PR review 문서 전용 커밋을 제거했으므로 push는 다음 형태로 수행한다.

```bash
git push --force-with-lease upstream HEAD:task_m100_1481
```

push 후 PR 본문을 최신 검증 결과와 Stage 6 범위에 맞춰 갱신한다.

### Stage E - GitHub Actions 및 merge

push 후 다음을 최신 PR head 기준으로 확인한다.

- GitHub Actions 전체 통과
- review 문서 2건과 오늘할일 문서가 PR diff에 포함됨
- 이전 PR review 문서 전용 커밋이 PR commit list에서 제거됨
- merge 가능한 상태
- 작업지시자 승인

조건 충족 후 `mydocs/manual/pr_review_workflow.md` 7장에 따라 merge와 후속 처리를 진행한다.

## 4. merge 후 후속 처리

- PR #1482 merge 결과 확인
- 이슈 #1481 close 여부 확인
- 필요 시 이슈 close + 코멘트
- PR에 merge 완료 및 검증 결과 코멘트
- `upstream/devel` 동기화
- `upstream/task_m100_1481` 원격 작업 브랜치 삭제
- 로컬 작업 브랜치와 임시 백업 브랜치 정리
