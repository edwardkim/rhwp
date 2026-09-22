# Task #7280 Stage 54 — R4 각주·미주 책임 묶음

- 이전: [Stage53](task_m100_7280_stage53.md).
- 승인 범위: [구현계획 §4.1](../plans/task_m100_7280_impl.md#41-r3r5-책임-묶음-진행으로-전환)의 R4 전체.
- 시작 head: `2613b32bb`, 동작 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: 구현 완료 후 고정 head 검증 진행 중. 원격 push·PR·댓글은 수행하지 않는다.

## 책임과 호출 경계

`notes/footnotes`는 저장 줄 분할/소유 쪽 Query, 내용 측정, 본문 각주 등록을 분리한다.
본문 control 루프의 같은 위치에서 `register_body_footnote`를 호출하며, 기존 control-loop
`continue` 세 곳은 함수 `return`으로 치환한다. 완료 쪽/현재 쪽/다음 쪽 선택과 예약 순서는 같다.
표 fragment 큐는 R3의 `table/footnotes`와 `table/continuation`이 계속 소유한다.

`notes/reservation`은 TypesetState의 각주 예약 Query와 Command를 한 책임 아래 둔다.
projected 높이 조회는 `&self`, 실제 예약과 완료 쪽 갱신은 `&mut self`이며, 구분선·여백·
footer 공간 회수와 본문 예산 갱신을 기존 순서대로 수행한다. 전역 state 쓰기 제한은 R5다.

`notes/endnotes`의 구역 조정자는 원본 참조 순서로 prepare → paragraph → emit을 연결한다.
content/profile/types는 번호·원본 해석과 기존 정책 관측/미주별 carry를 소유한다.
format/measure는 동일 구성 컨텍스트 및 scratch 레이아웃을 쓰며 실제 페이지 상태를 바꾸지 않는다.
metrics의 호출자 로컬 accumulator 갱신과 진단 출력은 보존하므로 모든 함수를 순수 함수로
새로 주장하지 않는다.

미주 문단 루프의 initial-fit → rewind-tail → tail-fit → new-note-fit 계산은 각각
명시적 Input/Result로 분리했다. Input의 TypesetState 참조는 읽기 전용이다. 각 조회 사이의
배치·단 전환·offset/carry 갱신을 원래 위치에 유지해, 이후 조회가 변경된 상태를 읽는 순서도
보존한다. fit은 기존 수용 판정, emit은 확정된 분할/전체 배치를 담당한다.
기존 다수 플래그와 수치 정책은 유지하며 이번 분리를 정책 정당성 검증이나 결함 수정으로
보고하지 않는다. 큰 Query 입력을 더 작은 정책 모델로 바꾸는 설계는 동작 근거 없이 강행하지 않는다.

## 변경 전후 대조와 검증 계획

- `output/7280/r4-batch/structural-proof.json`: 이동한 함수 본문, 네 Query 계산,
  문단 Command 순서, 본문 각주 선택/예약 **98/98 일치**. formatter 차이와 함수 경계의
  continue→return·참조 전달만 정규화했다. 타입/호출 연결 검토 및 실행 검증을 대신하지 않는다.
- 첫 컴파일 및 lib Clippy 통과. 중간 import/가시성·Query 입력 타입 오류는 수정했다.
- 테스트 원본·기대값·baseline·ignore·IR·public API는 변경하지 않는다.
- 고정 제품 SHA에서 review worktree 파생 suite 준비, fmt·정책·native Clippy,
  누적 집중 및 전체 회귀, Native/fresh Docker WASM 대표 각주·미주 출력 비교를 수행한다.
- 정확한 head와 실행 결과·대표 PNG·남은 차이는 이 문서에 이어 기록한다.
