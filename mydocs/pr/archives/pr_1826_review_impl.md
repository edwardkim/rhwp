# PR #1826 리뷰 구현 메모

## 대상

- PR: https://github.com/edwardkim/rhwp/pull/1826
- 제목: `Task #1811: 합성 LINE_SEG 증거 오인 차단 — HWPX saved_bounds p5 drift 원인 1 수정 + 원인 2 판별`
- 원본 커밋: `9d69668a670cd466a7e1484bbd5d74a7f5ba9a19`
- 로컬 체리픽 커밋: `6cfdc12d6`

## Stage 1. 메타 확인

완료.

- reviewer assign 완료.
- base 는 `devel`.
- 작성 시점 참고 상태는 `MERGEABLE`, GitHub Actions 통과.

## Stage 2. 변경 검토

완료.

- 합성 LINE_SEG 에 구현속성 태그를 부여하고, 작은 전방 vpos 보정 오인을 차단하는 흐름을 확인했다.
- 최신 PR 코멘트상 seoul_1006 회귀는 v2 에서 보정되었고, cherry-pick 대상도 최신 head 다.

## Stage 3. 시각 검증

완료.

- `scripts/task1274_visual_sweep.py` 로 p5 비교를 수행했다.
- `review_005.png` 는 잔존 후보를 보여주며, PR 설명의 원인 2 잔존과 일치한다.
- 대표 asset: `mydocs/pr/assets/pr_1826_visual_saved_bounds_p005.png`

## Stage 4. 후속 처리 메모

- merge 코멘트에서는 원인 1 수정과 원인 2 잔존을 분리해 설명한다.
- #1811 issue close 여부는 잔여 원인 2 추적 상태를 확인한 뒤 결정한다.
