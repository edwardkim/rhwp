# PR #1959 리뷰

## 메타

| 항목 | 내용 |
| --- | --- |
| PR | #1959 |
| 작성자 | planet6897 |
| 제목 | Task #1955/#1956: wrap 밴드 오매칭·쪽나누기 무시 + 글뒤로 표 후행 문단 귀속 수정 |
| base | devel |
| 문서 작성 시점 참고값 | mergeable: MERGEABLE, mergeStateStatus: CLEAN |
| 변경 규모 | 4 files, +188/-12 |

## 변경 범위

- explicit page break 이후 wrap band를 해제해 새 쪽 문단이 이전 anchor 옆 흐름에 흡수되지 않도록 한다.
- anchor `LINE_SEG`가 단 전체 폭이면 wrap band arming을 억제한다.
- 글뒤로/글앞으로 표의 후행 빈 문단 귀속을 anchor 첫 fragment 기준으로 보정한다.
- `verify_pi_page_vs_hangul.py`에 `--list`와 BOM 내성을 추가한다.
- `RHWP_TABLE_DRIFT` 환경변수 진단 출력에 available/base/footnote/zone 정보를 추가한다.

## visual sweep 판정

- `src/renderer/**`, pagination, page count, wrap band에 영향을 주므로 시각 검증 대상이다.
- PR 본문은 한글 2022 오라클 기반 pi-page 매트릭스 3건이 MATCH로 전환되었다고 제시한다.
- GitHub Actions의 Render Diff `Canvas visual diff`가 성공했고, 로컬 `cargo test --profile release-test --tests`에 포함된 `svg_snapshot`도 통과했다.
- 별도 기준 PDF가 PR diff에 추가된 형태는 아니므로, 이번 리뷰에서는 PR 본문의 한글 오라클 검증표와 Render Diff CI를 동등 시각 검증 근거로 기록한다.

## 로컬 검증

누적 검토 브랜치 `review/planet-1940-1960`에서 오래된 순서로 cherry-pick했다.

- `git diff --check upstream/devel...HEAD`: 통과
- `cargo fmt --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --lib`: 통과, 2123 passed / 0 failed / 6 ignored
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과, `svg_snapshot` 포함
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과
- GitHub Actions: Build & Test, CodeQL, Render Diff `Canvas visual diff` 성공

## 검토 메모

- 마지막 `debug:` 커밋은 `RHWP_TABLE_DRIFT` 환경변수가 있을 때만 동작하는 기존 진단 출력에 필드를 추가한다. 무조건 출력 회귀는 아니다.
- PR 본문에 악화 1건이 적혀 있으나, 종전 wrap 오흡수가 우연히 한글 쪽번호와 일치하던 케이스가 #1921 캘리브레이션 축으로 노출된 것으로 설명되어 있고 본 PR 범위 밖으로 분리되어 있다.

## 결론

merge 후보로 판단한다. 렌더 영향이 있으므로 merge 전 최신 Render Diff 성공 상태를 다시 확인한다.
