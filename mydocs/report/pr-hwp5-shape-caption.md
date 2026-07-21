# PR: 그리기 도형·묶음·차트 캡션이 HWP5 직렬화에서 전량 소실

## 개요

HWP5 직렬화기(`serialize_shape_control`)에서 도형(ShapeObject)의 모든 variant에 대해
캡션(caption)을 방출하지 않던 문제를 수정.

## 분석

### 파서 상태

`src/parser/control/shape.rs`의 파서는 모든 GSO 도형의 캡션을 올바르게 읽고 있음:

- 기본 도형(Line, Rectangle, Ellipse, Arc, Polygon, Curve): `drawing.caption`에 저장
- 묶음(Group): `group.caption`에 저장  
- 차트(Chart): `chart.caption`에 저장
- OLE: `ole.caption`에 저장
- 그림(Picture): `picture.caption`에 저장 (별도 함수 `serialize_picture_control`에서 처리)

### 직렬화 문제

`src/serializer/control.rs`에서 `serialize_caption` 호출은 다음 두 곳뿐이었음:

1. 표(`serialize_table_control`, 492행)  
2. 그림(`serialize_picture_control`, 998행)

`serialize_shape_control`(1181행-)의 모든 arm에는 `serialize_caption` 호출이 전혀 없어,
도형에 첨부된 캡션이 HWP5 저장 시 전량 소실됨.

### 적용 범위 (9개 ShapeObject variant)

| Variant | 캡션 위치 | 직렬화 위치 (SHAPE_COMPONENT 이전) |
|---------|-----------|-----------------------------------|
| Line | `line.drawing.caption` | CTRL_HEADER + synthesized_ctrl_data 이후 |
| Rectangle | `rect.drawing.caption` | 동일 |
| Ellipse | `ellipse.drawing.caption` | 동일 |
| Polygon | `poly.drawing.caption` | 동일 |
| Arc | `arc.drawing.caption` | 동일 |
| Curve | `curve.drawing.caption` | 동일 |
| Group | `group.caption` (직접 필드) | 동일 |
| Chart | `chart.caption` (직접 필드) | CTRL_HEADER 이후 (synthesized_ctrl_data 없음) |
| Ole | `ole.caption` (직접 필드) | CTRL_HEADER 이후 (synthesized_ctrl_data 없음) |

### 배치 패턴

기존 그림(`serialize_picture_control`)의 캡션 배치와 동일하게:
1. CTRL_HEADER (`level`)
2. 캡션 (`level + 1`, LIST_HEADER + 문단)
3. SHAPE_COMPONENT (`level + 1`)
4. CTRL_DATA (`level + 2`, SHAPE_COMPONENT의 자식)
5. 텍스트 박스 및 도형별 레코드 (`level + 2`)

## 코드 변경

**파일**: `src/serializer/control.rs`
**추가된 호출**: 9개의 `serialize_caption` 호출 (각 ShapeObject arm당 1개)

각 호출 패턴:
```rust
// 캡션 (SHAPE_COMPONENT 앞, level+1)
if let Some(ref caption) = $shape.$caption_field {
    serialize_caption(caption, level + 1, records);
}
```

## 검증

- `cargo check` 통과
- `cargo fmt --all` 적용 완료
- 기존 파서-직렬화 라운드트립 검증: 파서가 읽은 캡션을 직렬화가 방출하므로
  라운드트립 보존 (기존 파서 변경 없음)

## PR 번호

PR #2737
