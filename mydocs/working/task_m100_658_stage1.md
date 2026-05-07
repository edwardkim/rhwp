# Task M100 #658 단계 1 완료보고서

## 단계명

재현 계측 및 native rect 회귀 가드

## 작업 범위

첨부 영상의 드래그 선택 문제를 코드 레벨에서 재현할 수 있도록 `exam_social.hwp`의 대상 위치를 확인하고, native selection rect 반환값을 직접 관찰하는 진단 예제를 추가했다.

## 확인한 대상

- 샘플 문서: `samples/exam_social.hwp`
- 영상 기준 페이지: 2/4쪽
- dump 기준 위치: `section=1`
- 오른쪽 자료 박스:
  - 문제 문단: `pi=15`
  - 자료 1x1 표: `parent_para=16`, `control=0`, `cell=0`
  - 선택 대상 셀 문단: `cell_para=0..6`

`cargo run --bin rhwp -- dump-pages samples/exam_social.hwp -p 1` 결과, 페이지 2 오른쪽 단의 자료 박스는 `pi=16`의 1x1 표이며 `treat_as_char=true`, `wrap=TopAndBottom`로 배치되어 있었다.

## 추가한 진단 도구

- `examples/inspect_658_selection.rs`

진단 예제는 다음 값을 출력한다.

- `get_page_info(1)`의 페이지 폭 및 단 영역
- 본문 문제 문단 선택 rect
- 자료 박스 셀 내부 선택 rect
- rect별 `x`, `width`, `x + width`
- 페이지 폭 초과 rect 개수
- 영상 위치 부근 hit-test 결과

## 재현 결과

실행 명령:

```bash
cargo run --example inspect_658_selection
```

핵심 출력:

```text
page_info={"pageIndex":0,"width":1028.0,...,"columns":[{"x":70.7,"width":425.4},{"x":532.0,"width":425.4}]}

--- data table p16 c0 paragraph 0 ---
#02 p=1 x=952.4 y=239.0 w=255.5 h=12.7 right=1207.9 OVERFLOW
overflow_count=1

--- data table p16 c0 paragraphs 0..6 ---
#02 p=1 x=952.4 y=239.0 w=255.5 h=12.7 right=1207.9 OVERFLOW
#04 p=1 x=956.4 y=269.3 w=98.6 h=12.7 right=1055.0 OVERFLOW
#06 p=1 x=955.4 y=299.7 w=75.6 h=12.7 right=1031.0 OVERFLOW
#08 p=1 x=952.0 y=330.0 w=135.3 h=12.7 right=1087.3 OVERFLOW
#17 p=1 x=954.4 y=466.6 w=188.4 h=12.7 right=1142.8 OVERFLOW
overflow_count=5
```

페이지 폭은 `1028.0px`인데 셀 내부 선택 rect가 최대 `right=1207.9px`까지 확장된다. 이는 영상에서 파란 선택 하이라이트가 오른쪽 회색 영역까지 튀는 현상과 일치한다.

## 의심 지점

`src/document_core/queries/cursor_nav.rs::get_selection_rects_native()`에서 줄 경계 offset을 처리할 때 `find_cell_cursor()`가 다음 줄 시작 TextRun이 아니라 이전 줄 끝 TextRun을 먼저 반환하는 패턴이 확인된다.

관찰 근거:

- 같은 셀 문단 선택에서 연속 rect가 같은 `y=223.8`에 두 번 생성된다.
- 다음 줄 시작이어야 할 rect가 `x=957.1`처럼 이전 줄 끝 좌표에서 시작한다.
- 이후 `right_hit`와 조합되면서 폭이 실제 셀/페이지 경계보다 커진다.

따라서 단계 2에서는 다음 방향을 우선 검토한다.

- `CursorHit`에 TextRun bbox 또는 line 식별 정보를 추가해 `lh`와 `rh`가 같은 줄에서 온 값인지 검증한다.
- 줄 시작 offset이 이전 줄 끝 TextRun에 매칭되지 않도록 line range 기반 cursor 검색을 보강한다.
- 셀 내부 선택은 셀 bbox 또는 현재 TextRun/line bbox를 기준으로 rect를 클램프한다.
- 최종 rect는 페이지 폭을 넘지 않도록 방어 클램프한다.

## 검증

```bash
cargo check --example inspect_658_selection
```

결과: 통과

## 다음 단계 요청

단계 2에서는 native `get_selection_rects_native()`를 수정해 위 진단 예제의 `OVERFLOW` rect가 사라지도록 정합화한다.
