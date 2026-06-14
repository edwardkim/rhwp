# PR #1371 리뷰 - 미주 높이 모델 측정 SSOT A3 opt-in

- PR: https://github.com/edwardkim/rhwp/pull/1371
- 제목: Task #1363: 미주 높이 모델 측정 SSOT - scratch 전-단 순차 렌더 (A3 opt-in)
- 작성일: 2026-06-12
- 작성자: `planet6897`
- 관련 이슈: #1363
- base: `devel`
- head: `planet6897:task1363`
- 상태: merged, draft 아님
- merge commit: `0f0c7319cd88ecd2b78136df1907f1d541ed180c`

## 1. 요약 판단

GitHub 상태와 로컬 빌드/테스트 기준으로 merge 가능한 상태까지 회복됐다.

메인터너 코멘트에서 지적한 `git diff --check` trailing whitespace 1건은
`36a474ed`에서 직접 정리했고, 최신 head 기준 `git diff --check`도 통과했다.
GitHub Actions와 로컬 사전 검증도 모두 통과했다.

PR #1374가 이 PR 위에 쌓인 스택 PR이므로, #1374를 안전하게 처리하려면 #1371을 먼저
처리하는 순서가 맞다.

## 2. PR 정보

| 항목 | 값 |
|---|---|
| 상태 | merged |
| draft | false |
| base | `devel` |
| head | `planet6897:task1363` |
| head SHA | `36a474ed6479f66a02bdd59a2605f24027874115` |
| mergeable | `MERGEABLE` |
| merge state | `CLEAN` |
| merge commit | `0f0c7319cd88ecd2b78136df1907f1d541ed180c` |
| 변경량 | 23 files, +8865 / -15 |
| 연결 이슈 | #1363 |

커밋:

- `251b4c5a` - Task #1363: 미주 높이 모델 측정 SSOT - scratch 전-단 순차 렌더 (A3 opt-in)
- `670b6a29` - Merge branch 'devel' into task1363
- `36a474ed` - Task #1363: trailing whitespace 정리

## 3. 메인터너 코멘트 확인

메인터너 코멘트: https://github.com/edwardkim/rhwp/pull/1371#issuecomment-4677470193

요청 사항과 현재 확인 결과:

| 요청 | 현재 상태 |
|---|---|
| 최신 `devel` 기준으로 rebase/update | `670b6a29`에서 `devel` merge 반영, GitHub `CLEAN` |
| `src/renderer/typeset.rs` 충돌 해소 | GitHub `MERGEABLE`, merge-tree 충돌 0 |
| `EnSsotLevel` 기본값 `B` 유지 | diff상 `_ => EnSsotLevel::B`, `A3`는 env opt-in |
| archive 문서 재도입 주의 | active `mydocs/plans/report/working` 문서 추가가 남아 있어 merge 후 archive 정리 필요 |
| trailing whitespace 제거 | `36a474ed`에서 정리, `git diff --check` 통과 |
| A3 기본 승격 금지 | PR 본문/코드 모두 `RHWP_EN_SSOT=A3` opt-in 유지 |

inline review thread는 없다.

## 4. 변경 범위

### 4.1 `src/renderer/layout.rs`

`LayoutEngine::measure_endnote_column_bottom` 경로를 추가해 미주 단 전체 items를 scratch
`build_single_column`으로 렌더하고, 실제 렌더와 같은 단 bottom을 측정한다.

목적:

- per-para 고립 측정의 누적 오차를 줄인다.
- 미주 단 전체 흐름을 렌더러와 같은 경로로 측정한다.
- `RHWP_EN_SSOT=A3` opt-in에서 sim==render 확인 경로를 마련한다.

### 4.2 `src/renderer/typeset.rs`

`EnSsotLevel::A3`를 추가하고 `RHWP_EN_SSOT=A3`일 때만 scratch 전-단 측정을 사용한다.

확인한 방향:

- 기본값은 `B`로 유지된다.
- `A3`는 opt-in이며 기본 렌더 경로를 바꾸지 않는다.
- `issue_1082` 계열 overflow 확인용으로 A3 경로를 분리한다.

### 4.3 문서

Task #1363 계획서, 단계 문서, 보고서, 측정 TSV가 추가된다.

주의:

- 현재 PR diff에는 active `mydocs/plans`, `mydocs/report`, `mydocs/working` 문서가 포함된다.
- 일부 archive 경로 문서도 함께 포함되어 있다.
- merge 후 `mydocs/pr` 리뷰 문서와 함께 archive 정리 대상이다.

## 5. 검증 결과

### 5.1 GitHub Actions

최신 head `36a474ed` 기준 전체 통과.

| 체크 | 결과 |
|---|---|
| Build & Test | pass, 14m 58s |
| Analyze (javascript-typescript) | pass |
| Analyze (python) | pass |
| Analyze (rust) | pass |
| Canvas visual diff | pass |
| CodeQL | pass |
| WASM Build | skipped |

### 5.2 로컬 사전 검증

검증 worktree: `/tmp/rhwp-pr1371-fix`

| 항목 | 결과 |
|---|---|
| `git merge upstream/devel --no-commit --no-ff` | `Already up to date`, 충돌 없음 |
| `git diff --check upstream/devel...HEAD` | 통과 |
| `cargo build --lib` | 통과 |
| `cargo test --lib` | 통과, 1724 passed / 0 failed / 6 ignored |
| `cargo clippy -- -D warnings` | 통과 |
| `cargo test --doc` | 통과, 0 passed / 0 failed / 1 ignored |
| `cargo test --test svg_snapshot` | 통과, 8 passed / 0 failed |

## 6. 리스크

| 리스크 | 평가 |
|---|---|
| 기본 렌더 회귀 | 낮음. A3는 env opt-in |
| A3 경로 품질 | 중간. PR 본문에서도 13건 hancom 배치 재보정은 후속 #1370 범위로 분리 |
| 문서 정리 | 중간. active 작업 문서와 archive 문서가 함께 들어오므로 merge 후 정리 필요 |
| whitespace | 해소됨. `36a474ed`와 `git diff --check`로 확인 |
| #1374 스택 | 중요. #1374가 #1371 위에 있으므로 #1371 선처리가 필요 |

## 7. 최종 권고

merge 완료.

후속 처리:

1. #1371은 `0f0c7319`로 merge됐다.
2. #1363은 closed 상태를 확인했다.
3. `local/devel`은 `upstream/devel` 기준으로 rebase해 #1396 누락 문서 커밋을 보존했다.
4. PR #1371 리뷰 문서는 `mydocs/pr/archives/`로 이동했다.
5. 닫힌 PR #1396에 들어갔던 PR #1395 처리 문서도 같은 devel 문서 처리 흐름에 포함했다.
6. 이후 #1374를 다시 검토한다.
