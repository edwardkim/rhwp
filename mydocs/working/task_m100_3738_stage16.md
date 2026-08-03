---
kind: investigation
status: completed
canonical: mydocs/manual/bug_hunting_playbook.md
last_verified: 2026-08-02
---

# Task #3738 Stage 16 — p31 두 줄 각주 fragment와 본문 reset 경계 복원

## 출발 상태

Stage 15 commit `9caaf2aa1`은 native HWP 표 24의 내부 저장 `vpos` reset tail을 분할해 그림 51/caption을
rhwp p77에 복원했다. 최신 native 출력은 221쪽이고 기준 한컴 PDF는 215쪽이다. 따라서 이후 판정은 같은
번호만 대조하지 않고, 제목·표/그림 caption·footnote 번호로 semantic page owner를 먼저 맞춘다.

사용자 화면에서 다음 잔여가 재관측됐다. 이 단계에서는 p31만 처리했고, 나머지는 다음
단계로 남긴다.

- p31: 문단 하단과 각주 overlap
- p37: 그림 두 개가 세 개로 보이는 중복 paint 또는 fragment duplication
- p43·p54: 본문과 각주 overlap
- p66: 표와 각주 overlap
- p76: 표 24가 기준의 다섯 줄 대신 네 줄로 보인다는 관측 — Stage 15 native 출력에서 row 4 reset 전 세 줄과
  p77 tail을 복원했으므로, 사용자 UI revision·viewport와 semantic row owner를 다시 대조한다.
- p77–p79: 그림 51의 단독 이월과 빈 표 페이지 관측 — Stage 15 native 출력에서는 해소됐으므로 최신 revision의
  owner를 재확인한다.
- p83: FullParagraph overflow 후보
- p87·p90·p99–p100: 기준 PDF와 semantic 흐름 차이
- p80: 표 셀 안 본문·URL·각주가 서로 겹치는 다층 paint. 표/각주 reservation과 cell fragment paint가
  동시에 어긋난 독립 후보로 기록한다.

## p31 분석

원본 HWP 문단 421은 `LINE_SEG` 다섯 줄을 `vpos=62214, 64214, 66214, 68214, 0`으로 저장한다. 각주 30의
marker character는 `223`으로 reset 전 네 번째 본문 줄(`text_start=183..247`) 안에 있다. renderer는 각주 30의
두 composed line을 하나의 원자 각주로 p31에 전부 등록했고, 그 결과 본문 마지막 줄 bottom `1006.0px`보다
위인 `992.3px`부터 FootnoteArea가 시작해 본문·separator·각주가 겹쳤다.

기준 PDF는 p31에 각주 30의 첫 줄(`Aktuelle Entwicklungen … und`)만 두고, p32 하단에는 번호와 separator 없이
연속 tail(`„incentives” für Transplantationszentren`)만 둔다. 따라서 일반 footnote owner 변경이나 전역
`vpos=0` 강제 split은 정답이 아니다.

## 구현

`FootnoteRef`에 compose된 평탄 line 범위와 separator/번호 여부를 담는 `FootnoteFragment`를 추가했다. 단,
fragment 생성은 다음을 **모두** 만족하는 native HWP5 첫 각주로 제한했다.

1. 각주가 문단 하나·compose 두 줄뿐이다.
2. 인접 `LINE_SEG`의 다음 줄이 `vpos=0` reset이다.
3. reset 전 본문 줄의 visible bottom이 full-note top을 실제로 넘는다. 단순 trailing line-spacing 침범은 제외한다.
4. 각주 marker가 그 reset 전 본문 줄의 `text_start..next.text_start` 안에 있다.

첫 fragment는 완료된 p31에 소급 등록해 separator와 번호를 그리고, 두 번째 fragment는 p32에 separator·번호 없이
등록한다. estimate와 paint 모두 동일 flat-line 범위를 사용하므로 reservation과 실제 높이가 다시 갈라지지 않는다.

## 검증과 판정

- `cargo fmt --check` 통과.
- `CARGO_TARGET_DIR=target/review-planet6897-20260802 CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_3738_rowbreak_table_footnote_fragment -- --nocapture` — **6 passed**.
- 같은 target으로 `cargo build --profile release-test --bin rhwp` 통과.
- native HWP/한컴 2020 PDF의 p31–p32, 144 DPI selected visual sweep은 `requested=completed=[31,32]`,
  `run_state=complete`, structural flag 0건이다. p31 render tree는 body bottom `1006.0px`, separator `1019.0px`로
  13.0px 간격을 보이며, p32에는 continuation tail만 남는다.

상세 raster/overlay 판정은 [Stage 16 visual sweep](task_m100_3738_stage16_visual_sweep.md)에 남겼다. 이 확인은 p31–p32
선택 범위에 한정한다. native HWP 221쪽/기준 PDF 215쪽의 전체 pagination 불일치는 해소되지 않았다.

## 다음 단계

p37 그림 중복, p43·p54·p66 footnote collision, p76–p79 재확인, p80 table-cell 다층 overlap, p83·p87·p90·p99–p100
semantic drift는 아직 잔여다. 이 단계 커밋 뒤 새 Stage 문서에서 p80 표 cell의 paint/reservation과 p37 그림 owner를
서로 섞지 않고 다시 분석한다.
