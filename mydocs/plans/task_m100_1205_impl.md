# 구현계획서 — Task M100-1205: 문단 borderFill NONE side 렌더링 정정

- 이슈: [#1205](https://github.com/edwardkim/rhwp/issues/1205)
- 브랜치: `issue-1205-para-border-none-side`
- 작성일: 2026-06-03
- 수행계획서: `mydocs/plans/task_m100_1205.md`
- 기준 커밋: `5137c07f`

## 설계 요약

#1205의 핵심은 HWPX 파서가 아니라 문단 border 렌더링 경로다.

현재 `src/parser/hwpx/header.rs`는 `leftBorder`, `rightBorder`, `topBorder`, `bottomBorder`를 side별로 파싱하고, `type="NONE"`을 `BorderLineType::None`으로 매핑한다. 그러나 `src/renderer/layout.rs`의 문단 border group 렌더링은 top border 기준의 단일 stroke를 만들고, 일반 경로에서는 `RectangleNode` stroke로 4면을 한 번에 그린다. partial/cross-column 경로도 `LineNode`로 분해하긴 하지만 좌우 수직선을 항상 만든다.

따라서 구현은 다음 원칙을 따른다.

```text
1. borderFill의 side 순서 [left, right, top, bottom]을 유지한다.
2. 4면이 모두 visible이고 동일 stroke인 경우에만 기존 Rectangle stroke 경로를 유지한다.
3. 일부 side가 NONE이거나 서로 다른 stroke이면 fill과 stroke를 분리한다.
4. stroke는 visible side만 LineNode로 생성한다.
5. partial_start/partial_end의 top/bottom skip 의미는 유지한다.
```

## Stage 1 — RED 테스트와 기존 경로 계측

목표: left/right `NONE`, top/bottom `SOLID` 문단 border가 수직선을 만들지 않아야 한다는 회귀 조건을 먼저 고정한다.

변경 후보:

- `src/renderer/layout/integration_tests.rs`
  - 기존 문단 border SVG 검사 패턴을 재사용한다.
  - synthetic 문서 구성보다 기존 layout test fixture를 우선 검토한다.
  - fixture가 과하면 렌더러 내부 테스트로 `ResolvedBorderStyle`과 문단 border group 생성 결과를 직접 검증하는 helper를 둔다.

테스트 방향:

```text
1. left/right NONE + top/bottom SOLID border style을 가진 문단 border를 렌더링한다.
2. SVG 또는 RenderNode children에서 수직 LineNode/4면 stroke RectangleNode가 없는지 단언한다.
3. 같은 영역에 top/bottom 가로선은 존재해야 한다.
```

RED 기준:

- 현재 코드에서는 일반 경로가 `RectangleNode` stroke를 만들거나, partial 경로가 좌우 수직선을 만들어 실패해야 한다.

보고서:

- `mydocs/working/task_m100_1205_stage1.md`

## Stage 2 — 문단 border side별 stroke 생성 구현

목표: 문단 border group 렌더링에서 side별 `BorderLineType::None`을 존중한다.

수정 후보:

- `src/renderer/layout.rs`
  - 문단 border group 렌더링 블록에서 `border_style.borders`를 side별로 조회한다.
  - 기존 `stroke_color/stroke_width`는 top-only 대표값으로만 쓰지 않는다.
  - fill 렌더링은 `RectangleNode` fill-only로 분리할 수 있게 한다.
  - stroke 렌더링은 다음 조건으로 분기한다.

```text
if 4면 모두 visible이고 line_type/width/color가 동일하고 partial skip이 없다:
    기존 RectangleNode stroke 경로 유지
else:
    fill-only RectangleNode 필요 시 생성
    visible side만 LineNode 생성
```

세부 구현:

- helper 후보:

```rust
fn is_visible_border(border: &BorderLine) -> bool
fn same_border_stroke(a: &BorderLine, b: &BorderLine) -> bool
fn push_para_border_line(...)
```

- `BorderLineType::None`이면 width/color가 있어도 렌더링하지 않는다.
- top/bottom line의 width/color는 각 side의 `BorderLine`에서 가져온다.
- left/right line도 각 side가 visible일 때만 생성한다.
- `is_partial_start`이면 top line은 skip한다.
- `is_partial_end`이면 bottom line은 skip한다.

기존 `src/renderer/layout/border_rendering.rs::create_border_line_nodes()`는 이중선/삼중선까지 지원하므로 재사용을 검토한다. 단, 문단 border insertion order와 bounding box가 기존 `col_node.children.insert(0, ...)` 계약을 유지해야 하므로 직접 생성 helper가 더 안전하면 우선 최소 구현을 택한다.

보고서:

- `mydocs/working/task_m100_1205_stage2.md`

## Stage 3 — 회귀 테스트 확장

목표: #1205 직접 조합과 기존 4면 문단 border를 함께 보호한다.

검증 항목:

```text
1. left/right NONE + top/bottom SOLID:
   - 수직선 없음
   - stroke 있는 4면 rect 없음
   - 상하 가로선 존재

2. left/right/top/bottom SOLID:
   - 기존처럼 4면 border가 표현됨
   - 가능하면 RectangleNode 유지 또는 4개 LineNode 허용 중 하나를 명시

3. partial/cross-column:
   - 기존 #469, #471 테스트 통과
   - NONE side가 있으면 좌우 수직선 강제 생성 없음
```

기존 회귀 테스트:

```text
cargo test --lib test_469_partial_start_box_does_not_cross_col_top
cargo test --lib test_471_cross_column_box_no_bottom_line_in_col0
```

위 함수들은 `src/renderer/layout/integration_tests.rs`가 `layout.rs` 내부 모듈로 포함되므로 `cargo test --lib`에서 실행된다.

보고서:

- `mydocs/working/task_m100_1205_stage3.md`

## Stage 4 — 실제 샘플 검증과 최종 정리

목표: 자동 검증을 통과시키고, 재현 샘플이 있으면 PDF 기준 시각 정합을 확인한다.

자동 검증:

```text
cargo fmt --all --check
cargo test --lib task_1205
cargo test --lib test_469_partial_start_box_does_not_cross_col_top
cargo test --lib test_471_cross_column_box_no_bottom_line_in_col0
cargo test --tests
```

실제 샘플이 있는 경우:

```text
cargo run -- export-svg "/Users/melee/Downloads/[2027] 온새미로 1 본교재.hwpx" -o output/poc/task1205_para_border -p 9
```

확인 항목:

```text
1. 10쪽 하단 문단 박스가 PDF처럼 가로선 중심으로 보인다.
2. 좌우 수직선이 사라진다.
3. #1196, #1197, #1201 관련 레이아웃을 건드리지 않는다.
```

문서:

- `mydocs/working/task_m100_1205_stage4.md`
- `mydocs/report/task_m100_1205_report.md`
- `mydocs/orders/20260603.md` 상태 갱신

## 제외 범위

- HWPX `borderFill` parser의 side 매핑 전면 변경.
- 표/셀 border renderer 변경.
- 문단 border spacing, ignoreMargin, connect 의미 변경.
- 특정 샘플 파일명, 문단 번호, `borderFill id=23`에만 맞춘 분기.
- 이슈 #1196, #1197, #1201의 동작 보정.

## 리스크와 가드

| 리스크 | 가드 |
|------|------|
| 기존 4면 문단 border가 LineNode 분해로 SVG 구조 회귀 | 4면 동일 visible stroke는 기존 Rectangle stroke 유지 |
| partial/cross-column border가 다시 헤더/꼬리말 영역을 침범 | #469, #471 회귀 테스트 유지 |
| 이중선/삼중선 문단 border 표현 누락 | `create_border_line_nodes()` 재사용 가능성 검토 |
| fill-only Rectangle 순서가 바뀌어 텍스트를 덮음 | 기존 `col_node.children.insert(0, ...)` 삽입 순서 유지 |
| 재현 샘플 부재로 시각 판정 미완 | synthetic 자동 테스트로 논리 결함 고정, 샘플 확보 시 별도 검증 기록 |

## 작업지시자 승인 요청

본 구현계획이 승인되면 Stage 1부터 소스 수정을 시작한다.
첫 수정 범위는 `src/renderer/layout/integration_tests.rs`의 RED 테스트 추가로 제한한다.

> 본 문서는 구현계획서이다. 소스 수정은 작업지시자 승인 후 진행한다.
