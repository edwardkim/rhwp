# Task m100-2956: lineShape headStyle/tailStyle 하드코딩 수정

## 이슈

edwardkim/rhwp#2956 — HWP5(바이너리) 문서를 HWPX 로 저장(라운드트립)할 때, 그리기 개체
(직선/사각형/OLE 등)의 화살표 시작/끝 모양(headStyle/tailStyle)이 항상 `NORMAL` 로 고정
방출되어 원본 문서에 설정된 화살표 모양(화살/오목화살/다이아몬드/원/사각 등)이 소실되는 문제.

## 원인

- `src/parser/control/shape.rs:577` 에서 `border.attr = r.read_u32().unwrap_or(0);` 로
  HWP5 SHAPE_COMPONENT 레코드의 `ShapeBorderLine.attr` 32비트 전체를 그대로 읽어들인다. 이
  attr 안에는 화살표 시작 모양(bit 10~15), 끝 모양(bit 16~21), 채움 여부(bit 30/31)가 이미
  포함돼 있다. `src/renderer/layout/utils.rs::arrow_type_from_hwp` 가 렌더링 시 동일 비트를
  실제로 해석해서 사용 중임을 확인했다.
- 그런데 `src/serializer/hwpx/shape.rs::write_line_shape()` 는 파싱된 `bl.attr` 값을 무시하고
  `("headStyle", "NORMAL")`, `("tailStyle", "NORMAL")` 을 하드코딩 방출하고 있었다. 함수 상단
  주석에도 "headStyle/tailStyle/alpha 는 파서 미적재 → NORMAL/0 고정 방출" 이라고 적혀 있었으나,
  실제로는 파서가 이미 값을 읽어들이고 있어 잘못된 전제였다.
- 이는 이 저장소에서 반복적으로 발견/수정돼 온 "파서는 값을 읽는데 직렬화 시 하드코딩되어
  버려지는" 패턴(dropCapStyle, outline/shadow 토글, groupLevel, reverse, lock, numberingType,
  tab leader, pageBorderFill, textDirection 등)과 동일 유형이다.

## 수정 내용

`src/serializer/hwpx/shape.rs`:
- `write_line_shape()` 에서 `bl.attr` 의 bit 10~15(head 모양), bit 16~21(tail 모양) 을
  OWPML Core 스키마의 `ArrowType` 값(NORMAL/ARROW/SPEAR/CONCAVE_ARROW/
  EMPTY_·FILLED_DIAMOND/CIRCLE/BOX) 으로 매핑하는 `arrow_style_str()` 헬퍼를 추가하고,
  `headStyle`/`tailStyle` 속성값을 이 매핑 결과로 방출하도록 변경했다.
  채움 여부는 기존에 이미 계산되던 `headfill`/`tailfill` 판별용 bool(bit 30/31)을
  그대로 재사용해 일관성을 유지했다.
- `alpha` 는 여전히 파서가 값을 읽어들이지 않으므로(별도 이슈 대상) 이번 수정 범위에서
  제외하고 "0" 고정 방출을 유지했다.

## 테스트 (Red → Green)

`task2956_line_shape_arrow_style_preserved`:
- `attr = (4u32 << 16) | (1 << 30)` (tail 화살표 모양 = 4/다이아몬드, tail 채움 bit 설정)
  으로 `ShapeBorderLine` 을 만들고 `write_line_shape()` 를 호출한 뒤, 방출된 XML 에
  `tailStyle="FILLED_DIAMOND"` 가 포함되는지 검증한다.
- 수정 전 코드는 `("tailStyle", "NORMAL")` 하드코딩이라 위 조건이 항상 실패(RED)했고,
  수정 후에는 attr 값이 올바르게 매핑되어 통과(GREEN)한다.

## 검증

```
cargo test --lib task2956_line_shape_arrow_style_preserved
```
→ `test result: ok. 1 passed; 0 failed`

```
cargo check --lib
```
→ 정상 통과 (경고 없음, 신규 에러 없음)

`rustfmt --edition 2021 src/serializer/hwpx/shape.rs` 적용 완료.

## 변경 파일

- `src/serializer/hwpx/shape.rs` (fix + test)
- `mydocs/report/task_m100_2956_report.md` (본 문서)
