# Stage 236: Native RowBreak 첫 fragment frame 증거

## 목적

native HWP5 `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`에서
표 24 row 4 tail 뒤의 그림 51이 p77 각주 위에 남도록, #3820 첫 fragment frame
보정의 적용 범위를 실제 source fragment로 한정한다.

## 분리 결과

- 전체 integration에서 `issue_3738_rowbreak_table_footnote_fragment`의
  `native_hwp5_rowbreak_tail_keeps_figure_51_with_its_pdf_page`가 실패했다. p77은
  그림 51을 보유해야 하지만 현재는 그림 caption만 남고 그림은 p78으로 이월됐다.
- `upstream/devel` `c121f6185`와 `9f4d8c0ad`, `f2b7c7315`, `008549a8d`,
  `fed02cb03`, `20ffd3594`는 이 fixture 33건을 통과했다. 중간의
  `b537e6137`와 `f60497c4f`는 helper 인자 전환이 끝나지 않아 컴파일되지 않았다.
  첫 실행 가능한 실패 기준인 `bafb23a05`에서 native 첫 fragment paint slack이
  추가됐다.
- 그 slack은 `common.height`가 전체 표가 아니라 첫 physical fragment를 저장한
  경우에만 유효하다. 그림 51 fixture는 현재 page에 이미 `43.4px`의 기존 각주가
  예약돼 있다. upstream은 이 footnote-aware body bound에서 마지막 행을 `64.8px`로
  자른 뒤 남은 cell line을 다음 fragment에 소유시킨다. table-level slack이 whole
  row를 허용하면 이 cut이 사라져 그림 51의 owner가 p78으로 밀린다.

## 구현

- native HWP5 첫 fragment slack은 현재 physical page에 기존 footnote reservation이
  없을 때만 허용한다. existing footnote가 있으면 cell-unit scanner가 actual body
  boundary에서 partial row를 소유한다.
- HWPX source-frame 경로와 Stage 235 float stack trailing anchor는 변경하지 않는다.
- 문서 ID, 페이지 번호, 고정 px allowance를 사용하지 않는다.

## 검증 범위

- `issue_3738_rowbreak_table_footnote_fragment`: 그림 51의 p77 owner.
- `issue_2813_para_float_stack_anchor_line`: HWPX 2쪽 유지.
- `issue_3930_hwpx_hwp_save_layout`, `issue_3820_rowbreak_rowspan_band`.
- 전체 `lib`와 integration 회귀.
