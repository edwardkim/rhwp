# PR #1872 처리 계획 - #1667 Build & Test 기본 test step 정리

## 대상

- PR: #1872
- 작성자: @postmelee
- 관련 이슈: #1667, #1668
- 처리 경로: collaborator self-merge 후보 예외 경로
- 코드 검증 기준 SHA: `3f987ecf5e3aa35c1fc7bab4bca093e0b23f6c52`
- 처리 판단: 문서 후속 커밋 push 후 최신 PR head 기준 CI/fast-pass와 merge state를 재확인하고, 작업지시자
  승인 후 merge 여부를 결정한다.

## 커밋 구성

원 코드/작업 커밋:

| SHA | 제목 | 비고 |
|-----|------|------|
| `b381a557864c13602ec90e3a01dbdf91dcee85fb` | `Task #1667: reorder Build & Test steps for cache analysis` | Build & Test step 순서 재배치 및 측정 문서 |
| `c2713b5f14b9cd4fb6690fb24f559bb22e4920ae` | `Task #1667: remove duplicate lib test step` | `Run lib tests` 제거 및 `Run default-feature tests` 정리 |
| `672cb4ce7b4891a76051cc3b0d4ba5454c6ac73a` | `Merge remote-tracking branch 'upstream/devel' into task-1667-dirty-rhwp-analysis` | `mydocs/orders/20260704.md` 충돌 포함 최신화 |
| `3f987ecf5e3aa35c1fc7bab4bca093e0b23f6c52` | `Merge branch 'devel' into task-1667-dirty-rhwp-analysis` | 최종 코드 검증 기준 SHA |

후속 문서 커밋:

| 항목 | 내용 |
|------|------|
| 오늘할일 정리 | `mydocs/orders/20260704.md` 표 구조 정리 |
| review 문서 | `mydocs/pr/archives/pr_1872_review.md` |
| 처리 계획 | `mydocs/pr/archives/pr_1872_review_impl.md` |
| 변경 범위 | `mydocs/**` 문서만 변경 |

후속 문서 커밋은 이 문서를 포함하므로 문서 안에 자기 자신의 최종 SHA를 고정 기록하지 않는다. Push 후
`gh pr view 1872 --json headRefOid,statusCheckRollup,mergeable,mergeStateStatus`로 최신값을 확인한다.

## Stage 1. PR 메타 정렬

완료.

- labels: `ci`, `enhancement`
- milestone: `v1.0.0`
- assignee: @postmelee
- review request: 없음. PR 작성자와 현재 reviewer가 같아 self-review request는 적용하지 않는다.
- base: `devel`
- head: `postmelee:task-1667-dirty-rhwp-analysis`
- 문서 작성 시점 참고 상태: draft false, `MERGEABLE`, `CLEAN`

## Stage 2. 변경 내용 검토

완료.

검토한 핵심 변경:

- `.github/workflows/ci.yml`
  - `Run integration tests` 명령인 `cargo test --tests` 유지
  - step 이름을 `Run default-feature tests`로 보정
  - 별도 `Run lib tests` step 제거
  - `Native Skia tests` 유지

보존 확인:

- cache restore/save action, key, path 유지
- branch protection 표면인 `Build & Test` job 이름 유지
- `tests/**`, `tests/golden_svg/**`, sample 변경 없음
- third-party Rust cache action 도입 없음

## Stage 3. 로컬 검증

완료.

- `actionlint .github/workflows/ci.yml`: 통과
- YAML parse: 통과
- `git diff --check upstream/devel...HEAD`: 통과
- Cargo target 비교:
  - `cargo test --profile release-test --tests --no-run --message-format=json`: executable artifact 187개
  - `cargo test --profile release-test --lib --no-run --message-format=json`: executable artifact 1개
  - `--lib`에만 존재하는 고유 target 0개
- `git merge-tree --write-tree upstream/devel HEAD`: 충돌 없음

## Stage 4. GitHub Actions 확인

완료.

코드 검증 기준 SHA `3f987ecf5e3aa35c1fc7bab4bca093e0b23f6c52` 기준:

- CI preflight: success
- Build & Test: success
- Render Diff preflight: success
- Canvas visual diff: success
- CodeQL preflight: success
- Analyze (python): success
- Analyze (javascript-typescript): success
- Analyze (rust): success
- CodeQL: success
- WASM Build: skipped

Build & Test job log:

- `Run default-feature tests` 실행 확인
- `Run lib tests` 제거 확인
- `Run integration tests` 이름 제거 확인
- `running 2081 tests` 1회 확인
- `Native Skia tests` 48 tests 확인

## Stage 5. 후속 문서 커밋

진행 계획:

1. `mydocs/orders/20260704.md` 표 구조 정리
2. `mydocs/pr/archives/pr_1872_review.md` 작성
3. `mydocs/pr/archives/pr_1872_review_impl.md` 작성
4. staging 범위가 위 세 파일로 제한됐는지 확인
5. 문서 커밋 생성
6. PR head branch `postmelee:task-1667-dirty-rhwp-analysis`로 push
7. PR diff에 review 문서 2건과 오늘할일 정리가 포함됐는지 확인

## Stage 6. 문서 커밋 push 후 comment

문서 커밋 push 후 PR에 follow-up comment를 남긴다.

코멘트에는 다음을 포함한다.

- 오늘할일 표 구조 정리 사실
- `pr_1872_review.md`, `pr_1872_review_impl.md` 추가 사실
- follow-up 커밋은 `mydocs/**` 문서만 변경하며 workflow 코드는 추가 변경하지 않았다는 점
- 최신 merge 판단은 문서 커밋 push 후 PR head 기준 CI/fast-pass와 merge state 재확인 후 별도 승인 게이트로
  남긴다는 점

## Stage 7. merge 전 대기 조건

문서 커밋 push 후 즉시 merge하지 않는다. 다음 조건을 모두 확인한 뒤 작업지시자 승인으로 넘어간다.

- latest PR head 기준 GitHub Actions 또는 후속 기록 fast-pass 결과가 merge 가능 상태
- `mergeable` / `mergeStateStatus` 최신값 재확인
- PR diff에 review 문서와 오늘할일 정리가 포함됨
- 작업지시자 최종 merge 승인

## Stage 8. merge 후 후속 확인 계획

merge 후에는 다음을 확인한다.

- PR merge commit과 mergedAt
- #1667, #1668 상태
- #1667은 PR 본문이 `Refs` 형식이므로 auto-close되지 않는 것이 정상이다. 측정값 정리와 cache 전략 판단이
  남아 있으면 open 유지한다.
- #1668은 tracking issue이므로 후속 sub-issue 상태에 따라 open 유지 여부를 판단한다.
