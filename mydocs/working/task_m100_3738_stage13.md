---
kind: investigation
status: completed
canonical: mydocs/manual/bug_hunting_playbook.md
last_verified: 2026-08-02
---

# Task #3738 Stage 13 — HWP p30 각주 소유권과 p68 그림 49 RowBreak near-fit 이월

## 출발점과 재확인 범위

Stage 12 commit `7c7681dcb` 뒤 같은 release-test 바이너리로 HWP p30, p66, p68–p70을 다시
144 DPI raster/render-tree 대조했다. 이 과정에서 Stage 12가 p30 본문 tail의 각주-area 충돌은
피했지만, 문단을 flush한 뒤 각주 29를 현재 page(p31)에 등록해 **각주 표식은 p30, 본문은 p31**로
갈리는 것을 확인했다. p68의 그림 49도 여전히 rhwp p69로 통째 이월됐고 PDF에서는 p68 각주 위에서
끝난다. p66은 이 release-test native raster에서는 본문·각주 ink가 겹치지 않았으나, 사용자 UI에서
다시 관측한 충돌은 Stage 14에서 WASM 산출물과 함께 별도 재현한다.

원본 HWP/HWPX·기준 PDF는 [증적 보관 목록](../../pdf/pr3740/README.md)에 보관한다. p68–p69
수정 전 PNG 여섯 장은 `git check-attr filter` 결과 `unspecified`(비-LFS)를 확인한 뒤
`mydocs/pr/assets/pr_3740_issue3738_stage13/`에 고정했다.

## 입력과 현재 layout의 정확한 경계

### p30 각주 29

- native HWP5 문단 407은 첫 footnote control을 포함하며 `vpos=0` reset으로 p30의 마지막 두 body
  line을 p31로 넘긴다. control 순회는 이 split 뒤에 실행되므로 기존 `add_footnote_height`는 p31에
  `FootnoteRef`와 `FootnoteArea`를 만들었다.
- `find_inline_control_target_page`가 이미 flush된 p30의 inline marker를 찾을 수 있으므로, 이 좁은
  first-footnote/reset 형상에서는 해당 completed page에 footnote height와 ref를 함께 소급 등록해야 한다.

### p68 그림 49

- 입력 문단 749는 native HWP5의 비글자처럼 `TopAndBottom` `RowBreak` 2×1 표다. 첫 행은
  그림 49(Picture `bin_id=49`), 둘째 행은 `그림 49. OPTN 생존 장기기증 원칙` caption이다.
  해당 anchor `LINE_SEG`는 `vpos=44000`, 다음 빈 문단 750은 `vpos=59968`이다.
- rhwp p68의 `FootnoteArea`는 `y=898.1px`, `h=141.2px`이고 p68 끝에는 표가 없다.
  p69에 배치된 표 749의 동일 geometry는 `x=94.5, y=669.8, w=370.7, h=218.0px`다. 그 위치로
  p68에 배치하면 table bottom `887.8px`로 각주 상단보다 `10.3px` 앞에서 끝난다.
- 그러나 RowBreak 최초 행 gate는 `remaining_on_page=198.3px`, picture 행
  `split_unit_h=200.9px`로 산정한다. 비분할 다행 표이고 fresh page에는 맞는다는
  `multirow_clean_defer=true`가 되어 table 전체를 p69로 미룬다. 실제 table+caption가
  이미 존재하는 각주 영역과는 겹치지 않는다는 물리 geometry와, 40px footnote safety buffer를
  포함한 일반 flow gate가 충돌한 것이다.

## 가설과 범위 제한

일반 다행 `RowBreak` 표의 clean-defer를 완화하면, 요구사항 표·rowspan 표의 의도된 fresh-page
이월을 깨뜨릴 수 있다. 다음 후보는 다음을 모두 만족하는 narrow HWP5 형상으로 한정한다.

1. 비글자처럼 `TopAndBottom` `RowBreak`이고 2×1이며 row/col span이 없다.
2. 첫 행에 Picture가 있고 둘째 행은 caption text만 가진 그림+caption 표다.
3. table 자체에는 footnote가 없고, 현재 페이지에 이미 예약된 footnote가 있다.
4. safety buffer를 제외한 실제 footnote top까지의 공간에는 전체 table geometry가 들어가지만,
   일반 first-row gate만 소폭 초과한다.

## 구현과 결과

1. `TypesetState::add_footnote_to_completed_page`를 추가하고, native HWP5의 첫 footnote/reset split에서만
   marker가 놓인 completed page에 `FootnoteRef`와 실제 footnote-area 높이를 등록했다. p30은
   `Dattani, Nikesh` 각주 29를 보유하고 p31에는 더 이상 해당 각주가 없다.
2. 그림 49는 실제 `FootnoteArea` 상단을 기준으로 table total을 대조했다. native HWP5의 비글자처럼
   `TopAndBottom` `RowBreak` 2×1, 첫 row Picture·둘째 row caption, span/표 각주 없음이라는 형상에만
   적용했다. 일반 clean-defer와 same-paragraph float-defer를 이 실제 경계에 맞춰 해제하고, continuation
   첫 fragment에도 같은 경계를 전달했다.
3. 수정 뒤 p68의 그림 49와 caption은 함께 p68에 있고 table bottom은 footnote top보다 10.3px 위에
   머문다. p69는 `나. 생존 장기기증 승인 절차`로 시작하며 p70에는 caption 고아가 없다.

다른 `RowBreak` 표, 일반 footnote margin, 또는 전체 215쪽 pagination에는 이 Stage의 예외를 적용하지
않는다. p58의 첫 footnote safety-margin 조기 이월, p77 그림 51, p83 overflow는 Stage 14 잔여로 분리한다.

## 검증과 증적

- `cargo fmt`
- `CARGO_TARGET_DIR=target/review-planet6897-20260802 CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_3738_rowbreak_table_footnote_fragment -- --nocapture` — 3 passed
- `CARGO_TARGET_DIR=target/review-planet6897-20260802 CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_3738_hwp_caption_cell_alignment` — 1 passed
- `CARGO_TARGET_DIR=target/review-planet6897-20260802 CARGO_INCREMENTAL=0 cargo build --profile release-test --bin rhwp` — passed
- HWP p30–p32, p66, p68–p70, 144 DPI native visual sweep — 7/7 완료, SVG/render tree는 223쪽 생성.

선택 raster와 정량 지표, 1건의 heuristic flag 해석은
[Stage 13 visual sweep](task_m100_3738_stage13_visual_sweep.md)에 기록했다. 수정 전 p68–p69 PNG 6장과
수정 후 p30/p66/p68–p70 PNG 15장은 `mydocs/pr/assets/pr_3740_issue3738_stage13/`에 보관했다. 모든 PNG는
추가 전 `git check-attr filter`로 `unspecified`(비-LFS)임을 확인했다.
