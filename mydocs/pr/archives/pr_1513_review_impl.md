# PR #1513 처리 계획 — Task #1498 v2 다운로드 기록 게이트웨이 후속 회귀

- PR: <https://github.com/edwardkim/rhwp/pull/1513>
- 작성일: 2026-06-24
- 기준 브랜치: `local/devel`
- 처리 결과: merge 완료 (`f54756c405d3f43f2c0bcbbc487da0f1e68fe52a`)

## 1. 현재 확인 상태

| 항목 | 값 |
|---|---|
| PR head | `c22f680181f51c8e80994891d1ed3349cedd404c` |
| 커밋 | `Task #1498 v2: guard stale download observer items` |
| base | `devel` |
| 변경 규모 | 8 files, +384 / -4 |
| GitHub 상태 참고값 | `MERGEABLE` / `CLEAN` |
| CI 참고값 | Build & Test 성공, CodeQL 성공 |

이 값들은 문서 작성 시점 참고값이다. merge 전 최신 상태를 다시 확인한다.

## 2. 처리 단계

### Stage 1 — 사전 검토

- PR #1513 메타/패치/댓글 확인.
- 관련 이슈 #1498 상태 확인.
- 기존 #1498 계획서/보고서와 확장 다운로드 관찰자 관련 문서 검색.
- 로컬 검증:
  - `node --test rhwp-chrome/sw/download-interceptor.test.mjs`
  - `node --check rhwp-chrome/sw/download-interceptor.js`
  - `node --check rhwp-firefox/sw/download-interceptor.js`
  - `git diff --check devel..local/pr1513`

상태: 완료.

### Stage 2 — 작업지시자 승인 대기

- 리뷰 문서 근거로 merge 후보 판단 공유.
- 작업지시자가 승인하면 merge 전 최신 GitHub Actions와 mergeability를 재확인한다.

상태: 완료.

### Stage 3 — Merge 수행

승인 후 수행:

```bash
gh pr merge 1513 --repo edwardkim/rhwp --merge --admin
```

실행 전 조건:

- PR head 최신 커밋 기준 CI 성공.
- `mergeStateStatus`가 `CLEAN` 또는 작업지시자가 허용한 상태.
- 작업지시자 명시 승인.

상태: 완료. merge commit은 `f54756c405d3f43f2c0bcbbc487da0f1e68fe52a`.

### Stage 4 — 후속 확인

- `devel` / `origin/devel` / `local/devel` 동기화.
- 관련 이슈 #1498은 이미 closed 상태이므로, 별도 auto-close 기대 대신 상태 유지 확인.
- PR에 필요한 경우 감사/처리 코멘트 작성.
- 오늘할일에 merge 결과와 후속 배포 판단 사항 기록.

상태:

- `local/devel`은 `origin/devel`의 merge commit `f54756c4`로 fast-forward 완료.
- 이슈 #1498은 기존 closed 상태 확인.
- 오늘할일 merge 결과 기록 완료.

## 3. 보류 조건

- GitHub Actions가 최신 head에서 실패.
- merge 상태가 `DIRTY` 또는 예상 밖으로 변경.
- 작업지시자 환경의 0.2.7 재현 케이스에서 여전히 과거 다운로드 탭이 열린다는 피드백.
- `startTime` fallback 정책을 더 엄격하게 바꿔야 한다는 판단.

## 4. 현재 권고

사전 검토 기준으로 merge 후보로 판단했고, 작업지시자 승인과 최신 CI 재확인 뒤 merge 완료했다.
