# Task #1733 Stage 1 — saved-vpos tail split 완화

## 현재 상태

- 기준 PDF: `pdf/text_footnote_tail_overpagination-2024.pdf` = 242쪽
- 현재 `devel` 기준:
  - `samples/task1725/text_footnote_tail_overpagination.hwpx` = 250쪽
  - `samples/task1725/text_footnote_tail_overpagination.hwp` = 249쪽
- #1736 부분 완화 커밋(`4eba6367f`)은 245/244쪽까지 줄였지만, 이후 wrap-around/표 재개 로직이 정밀화되면서 현재 `devel`에서는 다시 250/249쪽이다.

## 관찰

현재 near-empty 후보는 다음 유형으로 나뉜다.

- 짧은 `PartialParagraph` tail 고립: HWPX 59, 73, 181, 218, 240쪽
- 빈 문단만 남은 페이지: HWPX 26, 157쪽
- 큰 TopAndBottom 표 단독/호스트 문단 페이지: HWPX 199쪽

이번 스테이지에서는 저장 `LINE_SEG`/vpos 증거가 있는 본문 tail 고립과, 같은 저장 흐름에서 확인되는 하단 빈 문단 bridge만 다룬다.
큰 표 단독/호스트 문단 페이지는 별도 조건이 필요하므로 이번 변경 범위에서 제외한다.

## 변경

- `src/renderer/typeset.rs`
  - split 루프에서 누적 높이상 overflow 이지만, 남은 줄의 저장 `LINE_SEG` vpos가 모두 현재 본문 하단 안에 있고 중간 vpos reset 이 없는 경우 tail split 을 허용한다.
  - 문단 전체/표/다단에는 적용하지 않고, partial paragraph split 지점에서만 적용한다.
  - 동일 조건으로 마지막 overflow 재검사에서 다시 다음 페이지로 밀리지 않게 한다.
  - 단일 줄 tail 도 저장 좌표가 본문 하단 안에 있고 다음 흐름이 새 쪽임을 가리키면 동일한 drift 허용값을 사용한다.
  - 하단 빈 문단 run 뒤 본문이 vpos-reset 되거나, 하단 제목 뒤 reset 되는 경우에 한해 빈 문단을 0-높이로 흡수한다.
  - 합성 회귀 테스트 `multiline_saved_vpos_tail_does_not_split_to_near_empty_page`, `page_bottom_empty_run_before_vpos_reset_does_not_create_blank_page` 추가.

## Stage 1 중간 결과

- 72px 허용값 기준: HWPX 250→247, HWP 249→246
- 남은 후보 중 HWPX 72쪽 `pi=1505` 마지막 한 줄과 108쪽 단일 줄은 저장 vpos가 본문 하단 안에 있으므로, 128px로 drift 허용값을 보정해 재측정한다.
- 단일 줄 tail 의 body 절대 vpos 보강은 제목 줄까지 전면 적용하면 241쪽으로 과소 조판된다. PDF 대조상 `진수 및 회수...` 목록 continuation 한 줄만 실제 over-pagination 후보이므로, `.`, `-`, `·`, `•`로 시작하는 목록 continuation tail 에 한정한다.
- 하단 빈 문단이 2개 이상 이어진 뒤 본문이 vpos-reset 되거나, 하단 빈 문단 뒤 하단 제목 한 줄이 있고 그 다음 본문이 reset 되는 경우를 0-높이 흡수 대상으로 추가한다.

## 검증 예정

```bash
cargo test --lib multiline_saved_vpos_tail_does_not_split_to_near_empty_page
cargo build --bin rhwp
./target/debug/rhwp dump-pages samples/task1725/text_footnote_tail_overpagination.hwpx | rg -c '^=== 페이지'
./target/debug/rhwp dump-pages samples/task1725/text_footnote_tail_overpagination.hwp | rg -c '^=== 페이지'
```

## 최종 검증 결과

```bash
cargo test --lib page_bottom_empty_run_before_vpos_reset_does_not_create_blank_page
cargo test --lib multiline_saved_vpos_tail_does_not_split_to_near_empty_page
cargo build --bin rhwp
./target/debug/rhwp dump-pages samples/task1725/text_footnote_tail_overpagination.hwpx | rg -c '^=== 페이지'
# 242
./target/debug/rhwp dump-pages samples/task1725/text_footnote_tail_overpagination.hwp | rg -c '^=== 페이지'
# 242
pdfinfo pdf/text_footnote_tail_overpagination-2024.pdf | rg '^Pages:'
# Pages:           242
```

추가 회귀 테스트:

```bash
cargo test --test issue_1733
```
