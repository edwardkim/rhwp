# PR #1739 처리 계획 — #1666 PR CI release-test profile 전환

## 대상

- PR: #1739
- 작성자: @postmelee
- 관련 이슈: #1666, #1668
- 문서 작성 시점 PR head: `8614c95b4a46f5500ff46362af8cbe16ac9bcddd`
- 처리 판단: 수용 및 merge 완료
- merge commit: `1a7a8305d765830605a4ae8f9bbb99f61febb82c`
- mergedAt: 2026-07-01 13:58:02Z
- 후속 문서 처리: PR head merge 이후 별도 문서-only PR로 반영

## 커밋

1. `6afd312eb50c09656febb16d379a34b16f5167ef`
   - `Task #1666: use release-test profile for PR CI`
   - CI workflow profile 분기와 #1666 작업 문서 추가
2. `01992f53fbfeab3f1c955fce72ab0960b4fe5616`
   - `Merge branch 'devel' into task1666-ci-release-test-profile`
   - 최신 `devel` 동기화용 merge commit
3. `8614c95b4a46f5500ff46362af8cbe16ac9bcddd`
   - `Merge branch 'devel' into task1666-ci-release-test-profile`
   - 최신 `devel` 동기화용 merge commit

## Stage 1. PR 메타 확인

완료.

- base branch: `devel`
- author/head owner: @postmelee
- draft: false (merge 전 확인 시점 참고값)
- mergeable: `MERGEABLE` (merge 전 확인 시점 참고값)
- mergeStateStatus: `CLEAN` (merge 전 확인 시점 참고값)
- labels: `ci`, `performance`
- milestone: `v1.0.0`
- assignee: @postmelee
- review request: 없음
- 규모: 5 files, +598/-4

라우트 판단:

- 외부 contributor PR이 아니므로 collaborator-mediated 외부 PR 경로가 아니다.
- PR 작성자와 작업 준비자가 collaborator인 self-merge 후보 예외 경로로 처리한다.

## Stage 2. 변경 내용 검토

완료.

검토한 핵심 변경:

- `.github/workflows/ci.yml`
  - `Build`
  - `Native Skia tests`
  - `Run lib tests`
  - `Run integration tests`

각 step은 `GITHUB_EVENT_NAME == pull_request`일 때 `release-test`, 그 외 event에서 `release`를 사용한다.

보존 확인:

- `Build & Test` job 이름 유지
- cache restore/save 조건 유지
- `Cargo.toml` profile 정의 변경 없음
- 테스트/golden 변경 없음

## Stage 3. 로컬 정적 검증

완료.

- `git diff --check upstream/devel...HEAD`: 통과
- `actionlint .github/workflows/ci.yml`: 통과
- Ruby YAML parse: 통과
- `Cargo.toml`, `tests/**`, `tests/golden_svg/**` 변경 없음 확인

## Stage 4. 충돌 확인

완료.

- `git merge-base --is-ancestor upstream/devel HEAD`: 통과
- `git merge-tree --write-tree upstream/devel HEAD`: 충돌 없음

## Stage 5. GitHub Actions 확인

완료.

문서 작성 시점 PR head `8614c95b4a46f5500ff46362af8cbe16ac9bcddd` 기준:

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

`Build & Test` job에서 확인한 핵심 로그:

- `profile=release-test event=pull_request`: 4회
- `Finished release-test`: Build, Native Skia tests, Run lib tests, Run integration tests
- cache restore exact hit
- PR event 조건상 cache save skipped
- error/warning/cache failure 표식 없음

## Stage 6. review 문서 작성

완료.

작성 문서:

- `mydocs/pr/archives/pr_1739_review.md`
- `mydocs/pr/archives/pr_1739_review_impl.md`

주의:

- 코드, workflow, 테스트 파일은 이 단계에서 수정하지 않는다.
- PR #1739는 review 문서 push 직전 이미 merge되었으므로 원 PR head에 추가 push하지 않는다.
- `mydocs/manual/pr_review_workflow.md` 7.3에 따라 별도 후속 문서-only PR로 반영한다.

## Stage 7. 후속 문서 PR 계획

진행 계획:

1. 최신 `upstream/devel`에서 후속 문서 브랜치 생성
2. review 문서 2개만 포함하는 문서 전용 커밋 생성
3. 원격 브랜치 push
4. `devel` base 후속 문서 PR 생성
5. PR diff가 `mydocs/pr/archives/pr_1739_review.md`, `mydocs/pr/archives/pr_1739_review_impl.md` 2개로 제한됐는지 확인

fast-pass 판단:

- 변경 범위는 `mydocs/**` 문서만 해당한다.
- PR #1739의 코드 검증 대상 SHA `8614c95b4a46f5500ff46362af8cbe16ac9bcddd`는 relevant checks success/skipped
  상태였다.
- 후속 문서 PR에서 heavy job이 skipped될 수 있으므로 preflight 결과와 merge 가능 상태를 확인한다.

## Stage 8. 후속 문서 PR merge 전 대기 조건

후속 문서 PR merge는 별도 작업지시자 승인 후에만 진행한다.

후속 문서 PR merge 직전 재확인:

1. 후속 문서 PR 최신 head SHA
2. latest check rollup 또는 fast-pass 상태
3. `mergeable` / `mergeStateStatus`
4. review 문서 2개만 PR diff에 포함됐는지 확인
5. 작업지시자 merge 승인

## merge 후 필수 후속 처리 계획

`mydocs/manual/pr_review_workflow.md` 기준으로 처리한다.

1. 후속 문서 PR metadata에서 merge commit SHA와 merged timestamp 확인
2. #1666 state 확인
3. #1668 state 확인
4. #1666에 PR CI measurement와 release-test 전환 결과 기록 여부 확인
5. `devel` push run에서 trusted event release profile 검증 확인
6. #1666 close 여부 작업지시자 승인 요청
7. #1668은 tracking/RFC issue로 후속 sub-issue가 남아 있으므로 open 유지 여부 확인
8. 로컬 후속 문서 브랜치와 원격 브랜치 정리

## 후속 코멘트 요지

- PR에서 `Build & Test` 네 cargo step이 `release-test`로 실행됨을 확인
- `devel`/`main` push, tag, workflow_dispatch에서는 `release` profile 검증을 유지하는 구조임을 설명
- `Build & Test` job 이름과 required check 표면은 유지됨
- PR #1739 merge commit SHA를 함께 기록
- #1666 close 여부와 #1668 open 유지 여부를 명시
