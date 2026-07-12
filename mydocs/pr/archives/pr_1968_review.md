# PR #1968 리뷰 - tail-before-vpos-reset 표 각주 안전마진 완화

## 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1968 |
| 제목 | Task #1921: tail-before-vpos-reset 표의 각주 안전마진 완화 (75828 MATCH) |
| 작성자 | planet6897 |
| base | `devel` |
| head | `fix/1921-footnote-margin` |
| 문서 작성 시점 head SHA | `d0ddd4faffb500ddbe1e56157557c12e455e9c2c` |
| 실제 변경 commit | `750477549840af6d75c6f23d970739fe990594c4` |
| 체리픽 commit | `fd176b441` |
| 규모 | 1 file, +23 / -1 |
| 변경 파일 | `src/renderer/typeset.rs` |
| mergeable | `MERGEABLE` |
| CI | GitHub Actions CI/CodeQL/Render Diff 통과 |
| 처리 방식 | #1969와 순서대로 체리픽해 통합 PR로 처리 |

## 변경 범위

- 표 내 각주가 있는 문단에서 다음 문단의 저장 `LINE_SEG.vertical_pos`가 새 쪽 시작을 가리키는 경우에만 각주 안전마진을 완화한다.
- 보정 근거는 현재 문단과 다음 문단에서 읽은 non-synthetic `LineSeg.vertical_pos`, 표 각주 높이/개수, 기존 `footnote_safety_margin`이다.
- 특정 파일명이나 issue 번호로 분기하지 않는다. 다만 `vertical_pos <= 500`은 저장 vpos가 쪽 상단 재시작을 뜻하는지를 판정하는 휴리스틱이므로, 후속 회귀에서는 이 조건이 과도하게 넓지 않은지 계속 확인해야 한다.

## 시각 검증

PR 본문에서 기준으로 든 `75828` 원본/PDF는 현재 체크아웃에 없어서 직접 visual sweep을 수행하지 못했다. 대신 #1969와 통합한 상태에서 기존 하단 고정 틀 회귀 샘플 `36389312`에 대해 보조 visual sweep을 수행했다.

| 항목 | 내용 |
|---|---|
| 샘플 | `samples/hwpx/opengov/36389312_결재문서본문_특정소방대상물 화재발생 알림(화재번호 2026-177).hwpx` |
| 기준 PDF | `pdf/36389312_결재문서본문_특정소방대상물 화재발생 알림(화재번호 2026-177)-2024.pdf` |
| 실행 | `python3 scripts/task1274_visual_sweep.py --key pr1968-1969-36389312 --hwp <sample> --pdf <pdf> --page 1 --out output/pr1968-1969-visual` |
| 결과 | SVG/PDF 1/1쪽, `flagged=0/1` |
| pixel match | `92.64419%` |
| 내용 픽셀 중심 자동 일치율 보조값 | `15.94925%` |
| 대표 asset | `mydocs/pr/assets/pr_1968_1969_36389312_review_177.png` |

사람 판정 메모: 문서 구조와 페이지 수는 기준 PDF와 맞고, 큰 배치 회귀 후보는 없다. 자동 일치율 보조값은 폰트/raster 차이를 크게 반영하므로 merge blocker로 보지 않는다.

## 로컬 검증

검토 시작 전 `/Users/tsjang/rhwp/target` 하위 항목을 삭제한 뒤 순차 실행했다.

```bash
gh pr edit 1968 --repo edwardkim/rhwp --add-reviewer jangster77
git fetch upstream devel pull/1968/head:local/pr1968 pull/1969/head:local/pr1969
git switch -c task_m100_1968_1969_cherrypick upstream/devel
git cherry-pick 750477549840af6d75c6f23d970739fe990594c4
git cherry-pick b0a0477a58e8bf14a4d2cb42bff25be0daec7cc7
git diff --check upstream/devel...HEAD
cargo fmt --check
env CARGO_INCREMENTAL=0 cargo build
env CARGO_INCREMENTAL=0 cargo test --test issue_1658_page_bottom_fixed_exclusion
env CARGO_INCREMENTAL=0 cargo test --test issue_1611_footer_page_bottom_pagination
env CARGO_INCREMENTAL=0 cargo test --test issue_1624_footer_overpush_pagination
env CARGO_INCREMENTAL=0 cargo test --test issue_1858
env CARGO_INCREMENTAL=0 cargo test --test issue_1858_bottom_anchor_flush
env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests
env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings
```

결과:

- reviewer assign: `jangster77`
- #1968 -> #1969 순서 체리픽: 충돌 없음
- `git diff --check upstream/devel...HEAD`: 통과
- `cargo fmt --check`: 통과
- `cargo build`: 통과
- targeted tests: 모두 통과
- `cargo test --profile release-test --tests`: 통과
- `cargo clippy --all-targets -- -D warnings`: 통과

## 검토 결과

PR #1968 단독 기준 샘플은 로컬에서 직접 재현하지 못했지만, 변경은 문서의 저장 `LineSeg` 신호와 각주 높이 계산에 근거하며, #1969와 통합한 실제 merge 후보 상태에서 관련 하단/각주/쪽 하단 회귀 테스트와 전체 테스트가 통과했다.

최종 권고: #1969와 함께 통합 체리픽 PR로 merge 후보.

