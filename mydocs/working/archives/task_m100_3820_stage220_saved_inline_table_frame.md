# Stage 220: 저장 단일 inline 표의 physical frame 보존

## 목적

`issue2006/1790387_prep_final_report.hwpx`의 section 3 `pi=292`가 독립 tail page를
만드는 문제를, HWP 2020이 저장한 표 physical frame으로 해결한다.

## 원인

- 표는 `treat_as_char=true`이지만 `is_effective_tac_table()`에는 포함되지 않아 일반
  `typeset_block_table()` 경로를 탄다.
- 1×1 빈 host의 비합성 LineSeg는 `vpos=66944`, `line_height=2355`이며 table
  `common.height=2355`와 정확히 일치한다. 즉 이 LineSeg가 표의 물리 frame 전체다.
- 누적 cursor 919.3px은 source top보다 뒤에 있고, generic `table_total=43.1px`는
  저장 frame 31.4px보다 커서 표를 다음 page로 이월한다.

## 수정

block-table 경로에서 단일 inline table의 실제 source frame을 판별한다. 조건은 1×1,
빈 단일-control host, 표 자체 각주 없음, 정확히 하나의 비합성 LineSeg, LineSeg 높이와
선언 table 높이의 HU 단위 일치, source top보다 뒤에 있는 cursor, source bottom의 현재
body 내 포함이다. 모두 충족하면 cursor와 placement 시작을 source top으로 복원하고,
source frame 높이만큼 advance한다.

## 안전 경계

- source frame을 추정하는 허용값이나 고정 px 상수는 추가하지 않는다.
- 여러 LineSeg, text host, 다행/다열 표, 각주 표, source body 밖 frame은 일반 측정
  경로를 그대로 사용한다.
- 앞 Stage의 TAC entry 실험과 일회성 진단은 제거했다. 이 수정은 실제 1×1 block-table
  owner 경로에만 존재한다.

## 검증 계획

1. `issue_2006_1790387_prep_pagination_pin`에서 144쪽 tail이 제거되는지 확인한다.
2. page map을 최신 HWP 2020 MCP 140쪽 PDF와 비교해 남은 physical tail owner를 다음
   stage로 분리한다.

## 검증 결과

`cargo test --profile release-test --test issue_2006_1790387_prep_pagination_pin -- --nocapture`
결과는 `1 passed; 0 failed`다. rhwp 페이지 수는 144쪽에서 143쪽으로 복구됐다.
