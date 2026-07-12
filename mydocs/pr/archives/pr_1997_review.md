# PR #1997 리뷰 - render-diff 노드 인덱스 매칭 허위 변위 차단

## 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1997 |
| 제목 | Issue #1993: render-diff 노드 인덱스 매칭 허위 변위 원천 차단 |
| 작성자 | planet6897 |
| base | `devel` |
| head | `fix/1993-renderdiff-position-match` |
| 문서 작성 시점 head SHA | `b78e6facdf8dc171023f2203863ed8a195626c10` |
| 실제 변경 commit | `4e33872c7aad5bb2df7101efcefd657d53d74797` |
| 체리픽 commit | `7ebb22152` |
| 규모 | 1 file, +184 / -0 |
| 변경 파일 | `src/diagnostics/render_geom_diff.rs` |
| mergeable | 문서 작성 시점 참고값: `MERGEABLE` |
| CI | GitHub Actions CI/CodeQL 통과, WASM Build skipped |

## 관련 이슈

- https://github.com/edwardkim/rhwp/issues/1993
- `render-diff`가 계층 LCS/인덱스 기반으로 렌더 트리 노드를 매칭하면서, 위치가 같은 노드의 순서만 달라져도 잘못 짝지어 큰 변위 `OVER`를 보고하던 문제다.
- 이슈 본문 예시는 픽셀 비교가 동일한데도 `3066815`, `3066817` 등에서 별지 370px 오탐이 발생하는 사례를 제시한다.

## 변경 범위

- `TextLine` 자식 `TextRun`들의 x 구간을 병합 인터벌로 요약하는 `line_run_coverage`를 추가했다.
- `TextRun` 개별 노드는 제외하고 구조 노드의 타입/bbox 및 `TextLine` run coverage를 위치 서명 멀티셋으로 평탄화하는 `flatten_pos_set`을 추가했다.
- `diff_page` 초기에 두 렌더 트리의 위치 서명 멀티셋이 같으면 zero-diff로 단락해 순서 차이에 따른 허위 변위를 제거한다.
- 회귀 테스트 3건을 추가했다.
  - 같은 줄의 run 재분할은 zero-diff
  - 형제 줄 순서만 바뀐 경우 zero-diff
  - 실제 run coverage 변화는 구조 불일치로 검출

## 시각 검증 필요 여부

별도 visual sweep은 수행하지 않았다.

이 PR은 `src/renderer/**` 출력 경로나 PDF/SVG 렌더 결과를 바꾸는 PR이 아니라, 이미 생성된 render tree를 비교하는 `render-diff` 진단 로직을 바꾸는 PR이다. 따라서 검증 기준은 기준 PDF 대조가 아니라 `render_geom_diff`의 분류 동작과 기존 baseline/통합 테스트 통과 여부다.

## 코드 검토

차단 이슈는 발견하지 못했다.

- `flatten_pos_set`은 노드 순서가 아니라 타입과 bbox 양자화 값으로 비교하므로, 이슈 #1993의 순서 변경 오탐을 직접 겨냥한다.
- 개별 `TextRun`을 서명에서 제외하되 `TextLine`에 run coverage를 넣어 run 경계 재분할 노이즈와 실제 coverage 변화를 구분한다.
- `real_run_coverage_change_is_not_masked` 테스트가 사전 단락이 실제 폭/coverage 변화까지 숨기지 않는다는 최소 회귀 guard 역할을 한다.
- 기존 `render_geom_diff` 자체가 텍스트 내용/스타일의 픽셀 동일성을 판정하는 도구는 아니므로, 이번 zero-diff 단락도 geometry 진단 범위 안에서 해석해야 한다. 텍스트 내용, 스타일, z-order의 실제 픽셀 회귀는 별도 visual/pixel 검증 계층에서 잡아야 한다.

## 로컬 검증

검토 시작 전 `/Users/tsjang/rhwp/target` 하위 항목을 삭제한 뒤 순차 실행했다.

```bash
gh pr edit 1997 --repo edwardkim/rhwp --add-reviewer jangster77
git fetch upstream devel pull/1997/head:refs/remotes/upstream/pr/1997
git switch -c pr1997-review upstream/devel
git cherry-pick 4e33872c7aad5bb2df7101efcefd657d53d74797
git diff --check
cargo fmt --check
env CARGO_INCREMENTAL=0 cargo test --lib diagnostics::render_geom_diff
env CARGO_INCREMENTAL=0 cargo test --test hwpx_roundtrip_baseline
env CARGO_INCREMENTAL=0 cargo test --test visual_roundtrip_baseline
env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings
env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests
```

결과:

- reviewer assign: `jangster77`
- 실제 변경 commit 체리픽: 충돌 없음
- PR head의 `b78e6fac`는 `Merge branch 'devel' into fix/1993-renderdiff-position-match` merge commit이라 로컬 기능 검토 체리픽에서는 제외했다.
- `git diff --check`: 통과
- `cargo fmt --check`: 통과
- `cargo test --lib diagnostics::render_geom_diff`: 16 passed
- `cargo test --test hwpx_roundtrip_baseline`: 4 passed
- `cargo test --test visual_roundtrip_baseline`: 3 passed
- `cargo clippy --all-targets -- -D warnings`: 통과
- `cargo test --profile release-test --tests`: 통과, `svg_snapshot` 포함
- GitHub Actions: CI/CodeQL 통과, WASM Build skipped

## 후속 처리 주의

PR 본문은 `## 배경 (#1993)` 형태로 이슈를 참조하지만 `Closes #1993` closing keyword는 없다. merge 후 GitHub auto-close가 동작하지 않을 수 있으므로, `mydocs/manual/pr_review_workflow.md`에 따라 #1993 상태 확인 및 후속 코멘트/close 처리가 필요하다.

## 결론

PR 내용과 로컬/GitHub 검증 기준으로 blocker는 없다. 최종 권고: merge 후보.
