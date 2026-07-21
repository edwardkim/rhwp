# PR: fix(hwpx): 공용 도형 경로 hp:sz 크기 기준·크기 보호 소실 (ellipse/arc/polygon/curve/chart)

## 개요

Issue #2726. #2697(표)과 #2712(rect/line/picture)에서 해결된 hp:sz의
widthRelTo/heightRelTo/protect IR 보존이 나머지 도형 타입(ellipse, arc, polygon, curve,
chart)에서 여전히 hardcoded "ABSOLUTE"/"0"으로 방출되는 문제를 수정한다.

## 분석

`src/serializer/hwpx/shape.rs`의 `write_sz()` 함수(978~992번째 줄)는 `CommonObjAttr`의
`width_criterion`, `height_criterion`, `size_protect` 필드를 무시하고 항상
`widthRelTo="ABSOLUTE"`, `heightRelTo="ABSOLUTE"`, `protect="0"`을 방출했다.

이 함수는 다음과 같은 모든 도형 타입에서 공통으로 호출된다:
- `<hp:rect>` — write_rect (103번째 줄)
- `<hp:line>` / `<hp:connectLine>` — write_line (250번째 줄)
- `<hp:container>` — write_container_close (321번째 줄)
- `<hp:ole>` — write_ole (390번째 줄)

rect(#2712)와 line(#2712)은 각각 write_sz를 호출하므로, write_sz 자체를 수정하면
모든 도형 타입이 한 번에 fix된다. ole는 chart/ole 개체를 포함한다.

## 변경 내용

**`src/serializer/hwpx/shape.rs`**

1. `SizeCriterion` import 추가
2. `size_criterion_width_str()` — `pub(crate)` 헬퍼 함수 (Paper→PAPER, Page→PAGE,
   Column→COLUMN, Para→PARA, Absolute→ABSOLUTE)
3. `size_criterion_height_str()` — `pub(crate)` 헬퍼 함수 (Paper→PAPER, Page→PAGE,
   Column/Para/Absolute→ABSOLUTE; 파서가 높이는 allow_column_para=false로 읽으므로
   Column/Para는 존재하지 않지만 fallback으로 ABSOLUTE)
4. `write_sz()` — hardcoded "ABSOLUTE"/"0" 대신 위 헬퍼 함수와
   `bool01(c.size_protect)` 사용

## 검증

- `cargo fmt --all -- --check` — 통과
- `cargo clippy --all-targets -- -D warnings` — 통과 (45.04s)

## 관련 PR

- #2697 — 표 hp:sz IR 보존
- #2712 — rect/line/picture hp:sz IR 보존
- #2733 — 본 PR
