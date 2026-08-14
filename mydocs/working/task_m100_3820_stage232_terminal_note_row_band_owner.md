# Stage 232: RowBreak 마지막 주석 행의 body band 소유권

## 목적

전체 integration 회귀에서 발견된 `21217935_simsa_jipyo.hwp`의 9쪽 과다 조판을
한글 COM 기준 8쪽으로 복원한다.

## 분리 결과

- 최신 `upstream/devel`의 `issue_2097_band_fill`은 8쪽 기준으로 통과한다.
- Stage 230 기준에서도 같은 샘플은 9쪽이므로 Stage 231의 native host-line frame
  변경과는 독립된 누적 회귀다.
- 문제 표 `pi=3`의 마지막 행은 1×4 full-width 주석 cell 하나이며, text/control이
  없는 spacer가 아니라 저장 LineSeg 하나를 가진 실제 주석이다.
- 마지막 continuation에서 현재 구현은 `consumed=910.5px`, remaining=`41.9px`,
  declared row=`53.7px`로 행을 분할 불가 이월해 53.7px tail page를 만들었다.
- upstream은 같은 행의 실제 content cut `16.0px`가 remaining band 안에 전부 들어감을
  확인하고 마지막 fragment의 row height를 body 끝까지로 정해 8쪽을 유지한다.

## 구현

- native/HWPX 구분이나 고정 px tolerance 없이 RowBreak 마지막 행의 구조를 판정한다.
- 마지막 행이 full-width 단일 cell, 단일 비합성 저장 line, visible text, control 없음,
  rowspan 없음일 때만 대상이다.
- `advance_row_cut`이 실제 text cut을 remaining band 안에 전부 수용하면, 선언 높이로
  overflow를 허용하지 않고 정확한 remaining band를 `end_row_height_override`로 쓴다.
- 주석 내용이 남은 band에 안 들어가거나 일반 multi-cell/rowspan/nested/strict frame이면
  기존 scanner 경로를 유지한다.

## 검증 범위

- `issue_2097_band_fill`: 6개 한글 COM page-count pin, 특히 21217935 8쪽.
- `issue_2020`, `issue_1921_59043_pagination_pin`.
- #3820 집중 게이트: `issue_2006_1790387_prep_pagination_pin`,
  `issue_3820_rowbreak_rowspan_band`, `issue_3930_hwpx_hwp_save_layout`, `issue_1733`.
- 집중 게이트 뒤 전체 `--lib`과 `--tests`를 다시 수행한다.
