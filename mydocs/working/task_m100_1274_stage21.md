# task 1274 stage21: PR 닫힘 후 전체 시각 재검증 잔여 overflow

## 배경

- stage20 변경분을 `task 1274: 미주 하단 잔여 overflow 공통 보정`으로 커밋한 뒤, 닫힌 PR #1277 본문 기준으로 수정 지점 전체를 다시 SVG/PDF PNG/비교 PNG로 생성했다.
- 6개 교육 통합 문서와 `issue_241.hwpx` 도장 host 케이스를 다시 확인했다.

## 재검증 산출물

- 전체 수정 지점 contact sheet:
  - `output/task1274/pr1277_modified_pages_contact.png`
- 6개 교육 통합 문서 전체 sweep:
  - `python3 scripts/task1274_visual_sweep.py --target all`
  - `output/task1274/summary.json`
- `issue_241.hwpx` 별도 비교:
  - `output/task1274/issue-241-pr1277/compare/compare_001.png`

## 통과한 항목

- `3-09월_교육_통합_2022.hwp`: SVG/PDF/compare 23/23/23, overflow 0건.
- `3-09월_교육_통합_2023.hwp`: SVG/PDF/compare 20/20/20, overflow 0건.
- `3-09월_교육_통합_2024-구분선아래20.hwp`: SVG/PDF/compare 23/23/23, overflow 0건.
- `3-10월_교육_통합_2022.hwp`: SVG/PDF/compare 18/18/18, overflow 0건.
- `3-11월_실전_통합_2022.hwp`: SVG/PDF/compare 21/21/21, overflow 0건.
- `samples/hwpx/issue_241.hwpx`: `issue_241` 단독 테스트 2개 통과, 비교 PNG 생성 완료.
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`: 51개 통과.

## 남은 문제

### `3-09월_교육_통합_2024-미주사이20.hwp` page 12/24

- 자동 로그:
  - `LAYOUT_OVERFLOW_DRAW: section=0 pi=662 line=0 y=1125.3 col_bottom=1092.3 overflow=33.1px`
  - `LAYOUT_OVERFLOW_DRAW: section=0 pi=662 line=1 y=1162.0 col_bottom=1092.3 overflow=69.7px`
  - `LAYOUT_OVERFLOW: page=11, sec=0, col=1, para=662, type=FullParagraph, first=false, y=1162.0, bottom=1092.3, overflow=69.7px`
- 비교 이미지:
  - `output/task1274/2024-09-between20/compare/compare_012.png`
- 실제 증상:
  - 오른쪽 단 하단의 `[알짜 풀이]` 다음 파란 줄이 page frame 아래로 내려간다.
  - PDF 기준은 같은 줄이 page frame 안에 남는다.
- 판단:
  - summary overflow가 오탐이 아니라 실제 시각 overflow다.
  - stage20의 `split_endnote_to_fit == 1` 차단이 page 13 `문15)` 잔여 overflow를 막았지만, 이 케이스에서는 page 12 하단의 실제 한 줄 fit까지 막아 남은 줄이 frame 아래로 내려간 것으로 보인다.

## 다음 수정 방향

- 한 줄 split을 전면 차단하지 않고, 실제 line advance와 frame 하단 여유가 충분한 경우에는 PDF처럼 한 줄을 page 12 하단에 남길 수 있게 조건을 좁힌다.
- 반대로 stage20에서 고친 page 13 `문15)`, page 21 `문26)`, `3-10월_교육_통합_2022.hwp` page 11/16은 회귀시키지 않는다.
- 수정 후 최소 확인:
  - `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - `python3 scripts/task1274_visual_sweep.py --target 2024-09-between20`
  - `python3 scripts/task1274_visual_sweep.py --target all`

## 상태

- stage21은 아직 소스 수정 전 분석 기록이다.
- 작업지시자 승인 후 `typeset.rs`의 single-line split 조건을 더 좁히는 방향으로 수정한다.
