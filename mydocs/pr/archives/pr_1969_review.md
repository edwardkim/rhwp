# PR #1969 리뷰 - 하단 고정 틀 저장 flow 이중 계산 보정

## 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1969 |
| 제목 | Issue #1920: 하단 고정 틀(vert=쪽·Bottom) 저장 flow 이중 계산 수정 (36373162 MATCH) |
| 작성자 | planet6897 |
| base | `devel` |
| head | `fix/1920-bottom-fixed-double-count` |
| 문서 작성 시점 head SHA | `b0a0477a58e8bf14a4d2cb42bff25be0daec7cc7` |
| 실제 변경 commit | `b0a0477a58e8bf14a4d2cb42bff25be0daec7cc7` |
| 체리픽 commit | `2cfac2f4e` |
| 규모 | 1 file, +24 / -3 |
| 변경 파일 | `src/renderer/typeset.rs` |
| mergeable | `MERGEABLE` |
| CI | GitHub Actions CI/CodeQL/Render Diff 통과 |
| 처리 방식 | #1968 뒤에 체리픽해 통합 PR로 처리 |

## 변경 범위

- 하단 고정 틀의 같은 쪽 편입 판정에서 `available`을 배타 영역 미차감 값으로 보고 `available - prospective_excl`로 판정하도록 수정했다.
- 하단 고정 틀이 본문 flow에서 롤백한 소비 높이를 `vpos_page_base` 또는 `vpos_lazy_base`에 반영해, 후속 문단의 저장 vpos 스냅과 배타 영역 차감이 이중 계산되지 않게 했다.
- 보정 근거는 하단 고정 틀의 `bottom_fixed` 판정, 저장 vpos base, block 높이, 문서 flow 좌표다.
- 특정 샘플명/페이지 번호로 분기하는 하드코딩은 없다.

## 시각 검증

PR 본문 기준 `36373162` 원본/PDF는 현재 체크아웃에 없어 직접 visual sweep하지 못했다. 대신 PR 본문에서 회귀 guard로 언급한 `36389312` 1쪽 유지 여부를 직접 검증했다.

| 항목 | 내용 |
|---|---|
| 샘플 | `samples/hwpx/opengov/36389312_결재문서본문_특정소방대상물 화재발생 알림(화재번호 2026-177).hwpx` |
| 기준 PDF | `pdf/36389312_결재문서본문_특정소방대상물 화재발생 알림(화재번호 2026-177)-2024.pdf` |
| dump-pages | `target/debug/rhwp dump-pages <sample>` 결과 1쪽 |
| 실행 | `python3 scripts/task1274_visual_sweep.py --key pr1968-1969-36389312 --hwp <sample> --pdf <pdf> --page 1 --out output/pr1968-1969-visual` |
| 결과 | SVG/PDF 1/1쪽, `flagged=0/1` |
| pixel match | `92.64419%` |
| 내용 픽셀 중심 자동 일치율 보조값 | `15.94925%` |
| 대표 asset | `mydocs/pr/assets/pr_1968_1969_36389312_review_177.png` |

사람 판정 메모: 기준 PDF와 rhwp 출력 모두 1쪽이며 큰 배치 회귀 후보는 없다. overlay의 차이는 주로 폰트/raster 차이다.

## 로컬 검증

검토 시작 전 `/Users/tsjang/rhwp/target` 하위 항목을 삭제한 뒤 순차 실행했다.

```bash
gh pr edit 1969 --repo edwardkim/rhwp --add-reviewer jangster77
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

하단 고정 틀의 배타 영역 계산과 저장 vpos base 보정 위치가 원인 설명과 맞다. 기존 하단 고정 틀/footnote/page-bottom 회귀 테스트, `36389312` visual sweep, 전체 release-test, clippy가 모두 통과했다.

최종 권고: #1968 뒤에 적용하는 통합 체리픽 PR로 merge 후보.

