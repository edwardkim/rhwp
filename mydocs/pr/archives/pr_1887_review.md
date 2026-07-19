# PR #1887 리뷰 — #1811 HWPX saved bounds RowBreak 조판 보정

- PR: #1887 `task 1811: HWPX saved bounds RowBreak 조판 보정`
- 작성자: @jangster77
- 기준 브랜치: `devel`
- 관련 이슈: #1811
- 문서 작성 시점 코드 head: `6c6037ca9d4961d4c6fdd08e6aede5f9ddfc3943`
- 규모: 12 files, +817/-83
- 검토 경로: collaborator self-merge 후보, 옵션 1
- 최종 merge 조건: PR head 최신 커밋 기준 GitHub Actions 통과 + 작업지시자 승인

## 변경 요약

HWPX `saved_bounds_cumulative_page_break` 샘플의 p5 tail/line drift 후속 검토를 처리한다.

핵심 변경:

- HWPX RowBreak 표의 합성 lineSeg 높이와 row cut 계산을 저장된 표/셀 속성에 맞게 보정
- 같은 문단 안의 visible RowBreak host text 와 nested table fragment 배치 순서 보정
- HWPX synthetic lineSeg 경로와 HWP/HWP3-origin missing lineSeg fallback 폭 계산 분리
- draft PR #1875 계열 미주 커밋이 섞였을 때 발생한 `issue_1139_inline_picture_duplicate` 회귀 분석 기록 보존
- renderer/layout/parser 보정은 특정 샘플명·페이지·임의 계수가 아니라 문서 속성 기반으로 해야 한다는 review 규칙 기록

## 범위 정정

처음 잘못 생성한 PR #1885에는 draft PR #1875 계열 미주 커밋이 함께 포함됐다. #1885는 close 했고,
CI/CodeQL run은 force-cancel 처리했으며 원격 브랜치도 삭제했다.

PR #1887은 `upstream/devel` 기준 clean 브랜치에서 #1811 RowBreak/saved bounds 커밋만 다시 구성했다.
Stage 6 문서는 버리지 않고 보존하되, 그 안의 미주 코드 수정은 #1811 PR 코드 범위에 포함하지 않았다.

## 렌더 영향 판정

렌더링/조판 변경 PR 이므로 visual sweep 대상이다.

사용한 문서 속성 근거:

- RowBreak 표 셀의 저장 높이와 row cut 단위
- 문단의 `line_segs`, synthetic lineSeg 여부, cell inner width
- 같은 셀 문단 내부의 `TextRun` line unit 과 `mixed_nested_fragment` unit 순서
- HWPX synthetic lineSeg 와 HWP/HWP3-origin missing lineSeg fallback 의 source/context 차이

특정 샘플명, 페이지 번호, PR/issue 번호, 임의 계수에 맞춘 분기는 merge 후보로 보지 않는다는 규칙에 따라,
이번 보정은 입력 문서에서 읽을 수 있는 구조와 lineSeg/control 속성에 근거하는지 확인했다.

## 로컬 검증

PR 생성 전 clean 브랜치 기준으로 다음 검증을 완료했다.

- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과
- `git diff --check upstream/devel...HEAD`: 통과

추가 focused 확인:

- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1749_saved_bounds_page_break -- --nocapture`: 통과, 3 passed
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1035_alignment -- --nocapture`: 통과, 4 passed
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1139_inline_picture_duplicate -- --nocapture`: 통과, 85 passed

## 시각 검증

`mydocs/manual/verification/visual_sweep_guide.md` 기준으로 `samples/task1749/saved_bounds_cumulative_page_break.hwpx` 와
`samples/task1749/saved_bounds_cumulative_page_break-2024.pdf` 를 비교했다.

### p4

- 임시 산출물: `output/issue1811-stage5-p4-visual/issue1811-page-break-p4/review/review_004.png`
- PR asset: `mydocs/pr/assets/pr_1887_issue1811_saved_bounds_review_p004.png`
- exported SVG/render_tree/PDF pages: 5/5/5
- requested page: 4
- visual sweep flagged pages: 0/1
- pixel match: 91.02025%
- visual_accuracy_proxy_percent: 9.63581%
- 사람 판정: p4에서 RowBreak host 본문이 nested table fragment보다 먼저 배치되고, 자동 후보는 남지 않았다.

### p5

- 임시 산출물: `output/issue1811-stage5-p5-visual/issue1811-page-break-p5/review/review_005.png`
- PR asset: `mydocs/pr/assets/pr_1887_issue1811_saved_bounds_review_p005.png`
- exported SVG/render_tree/PDF pages: 5/5/5
- requested page: 5
- visual sweep flagged pages: 0/1
- pixel match: 89.7506%
- visual_accuracy_proxy_percent: 5.10161%
- 사람 판정: #1811 본문에 기록된 p5 tail/line drift 후보는 자동 후보 0건으로 정리됐다.

## 주요 리스크

Blocking finding 없음.

주의점:

- `visual_accuracy_proxy_percent` 는 사람 판정 정확도가 아니라 내용 픽셀 중심 자동 일치율 보조값이다.
- 이번 merge 판단은 자동 후보 0건, 페이지 수 5쪽 유지, p4/p5 흐름의 사람 판정을 함께 본다.
- 옵션 1 문서/asset/오늘할일 커밋이 PR head 에 추가되면 GitHub Actions는 최신 head 기준으로 다시 확인한다.

## 리뷰 결과

Merge 후보.

HWPX saved bounds page-break 후속으로 남았던 p5 tail/line drift는 RowBreak host 순서, 합성 lineSeg row cut,
missing lineSeg fallback 폭 분리를 통해 해결 후보로 판단한다. Stage 6 문서는 #1875 계열 커밋이 섞였을 때의
회귀 분석 기록이므로 보존하되, #1887 코드 변경 범위에는 미주 보정 코드를 포함하지 않았다.

## merge 후 후속 처리 계획

- PR #1887 GitHub Actions 최신 head 통과 확인
- admin merge
- `upstream/devel` 동기화
- #1811 auto-close 여부 확인
- auto-close 되지 않으면 PR #1887 merge SHA와 대표 visual asset 링크를 남기고 #1811 수동 close
- 옵션 1 경로이므로 별도 문서-only PR 은 만들지 않는다.
- PR branch/worktree 정리
