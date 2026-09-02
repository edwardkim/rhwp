# Stage 235: Stacked host의 trailing anchor 범위

## 목적

source-frame refactor 뒤의 저장 tail helper가
`samples/issue2813/dangjik_dutylog.hwpx`를 2쪽에서 3쪽으로 과분할한 회귀를
복원하면서, #3820 HWPX direct frame과 #3930 Q5 page owner를 유지한다.

## 분리 결과

- 최신 `upstream/devel`의 `issue_2813_para_float_stack_anchor_line`은 2쪽으로
  통과한다. 실행형 이분 탐색은 중간 non-compiling commit 때문에 하나의 SHA를
  확정하지 못했지만, executable good `d726f638`과 bad `bafb23a` 사이의
  RowBreak source-frame refactor로 범위를 좁혔다.
- #2813 host 문단은 빈 문단이고 para-relative TopAndBottom 표 둘이 같은 anchor를
  공유한다. 유일한 저장 LineSeg는 두 표의 stack 뒤, 본문 안의 razor-fit 위치를
  가리킨다. 기존 `host_line_trails_float_stack`은 이 구조를 표 개수, 빈 host,
  저장 line bounds로 이미 검증한다.
- 현재 `saved_bounds_fit_at_flow_tail`은 저장 line이 current flow와 겹쳐야 한다.
  이 일반 의미를 float stack의 trailing anchor에 적용하면, `top=666.7px`인
  anchor가 stack 시작 flow `192.3px`와 겹치지 않아 false가 된다. 그러나 그 line은
  stack 뒤에 있어야 하며 `bottom=680.1px`가 body `680.3px` 안에 든다는 것이
  정답 source 계약이다.
- #3930 Q5의 단일 direct HWPX frame은 일반 current-flow tail helper를 계속 사용한다.
  이 Stage는 그 helper나 HWPX reset scanner 조건을 바꾸지 않는다.

## 구현

- `host_line_trails_float_stack`의 기존 구조 조건(복수 co-anchored float, 빈 host,
  stack 뒤의 단일 저장 line)을 유지한다.
- 이 구조에서만 일반 current-flow-overlap helper 대신 저장 line의 physical bottom이
  body 안에 드는지를 직접 판정한다.
- scanner의 HWPX reset/landscape gate는 불변이다. 문서 ID, page number, 새 고정 px
  allowance는 사용하지 않는다.

## 검증 범위

- `issue_2813_para_float_stack_anchor_line`: 2쪽, page 1 표 둘과 뒤따르는 anchor line.
- `issue_3930_hwpx_hwp_save_layout`, `issue_3820_rowbreak_rowspan_band`.
- Stage 232-234의 #2097, #1073 집중 gate와 전체 lib/integration suite.
