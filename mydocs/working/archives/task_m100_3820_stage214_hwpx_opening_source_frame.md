# Stage 214 - Direct HWPX opening source-frame ownership

## 목적

`2025 행정업무운영 편람(최종).hwpx`의 section 10, paragraph 4에 있는 `1x1`
`RowBreak` 표가 첫 fragment에서 source frame 끝보다 두 줄 앞에서 capacity cut되는
문제를 source 좌표 계약으로 해결한다.

## 진단 근거

- Stage213 trace에서 HWP와 HWPX 모두 `CellUnit` 95개, 첫/둘째 stored reset
  index `29`/`62`가 동일했다.
- HWP 원본은 첫 source frame의 `[0, 29)`를 소비해 3개 fragment로 끝난다.
  HWPX는 일반 capacity cut이 `[0, 27)`에서 멈춰 짧은 tail fragment를 만들고
  전체 출력이 384쪽이 된다.
- 차이는 텍스트 측정이나 테이블 선언 높이가 아니라 `typeset`의 source-frame tail
  적용 범위였다. 기존에는 continuation 또는 terminal response에서만
  `stored_frame_cut_for_row()` 결과를 선택했다.

## 수정

direct HWPX의 opening RowBreak fragment도 다음 조건을 모두 만족할 때만 기존의
`stored_frame_cut_for_row()` 결과를 선택한다.

- `RowBreak`, 비-TAC, 단일 가시 source cell, 실제 stored vpos rewind
- 첫 fragment이며 continuation이 아님
- 텍스트가 편집 후 reflow되지 않음

이 조건은 파일명, 페이지 번호, 표 높이, 문단 수 또는 고정 pixel allowance를 쓰지
않는다. source가 기록한 정확한 CellUnit frame 끝만 선택하며, writer-local reset이나
재조판된 텍스트에는 일반 capacity cut을 유지한다.

## 검증 결과

`target/release-test/deps/issue_3930_hwpx_hwp_save_layout-ba610b42a8e8d816 --nocapture`
실행 결과는 `2 passed; 0 failed`다.

- `issue_3930_preserves_page_count_and_inherited_even_master_page`가 HWPX와 HWP의
  383쪽 page count와 p283 Q5 source owner를 함께 검증했다.
- `issue_3820_hwpx_behind_text_stamp_placeholders_keep_common_y_and_offsets`도 함께
  통과해 direct HWPX frame 선택이 behind-text stamp 좌표 계약을 바꾸지 않았음을
  확인했다.

다음 Stage에서 이 수정이 다른 fixture에 미치는 회귀와 HWP 2020 MCP PDF 래스터
대조를 수행한다.
