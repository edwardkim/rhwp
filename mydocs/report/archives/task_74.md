# 타스크 74: 개체 묶기(Group Shape) 파싱 및 렌더링 — 최종 결과 보고서

## 개요

HWP의 "개체 묶기"(그리기 개체 > 묶음) 기능을 구현하였다. 여러 개체(그림, 도형 등)를 하나의 그룹으로 묶어 함께 관리하는 기능으로, 기존에는 그룹 내 자식이 그림(Picture)인 경우를 처리하지 못해 이미지가 누락되거나 빈 사각형으로 렌더링되었다.

## 해결한 문제

### 1. 모델 확장 — ShapeObject에 Picture 변형 추가

- `ShapeObject` enum에 `Picture(Box<Picture>)` 변형 추가
- `GroupShape.children: Vec<ShapeObject>`에 그림 개체 포함 가능
- 수동 Default 구현으로 `render_sx/sy` 기본값 1.0 보장

### 2. 파서 — 그룹 자식 Picture 파싱

- `parse_container_children()`에서 `HWPTAG_SHAPE_COMPONENT_PICTURE` 태그 매칭 추가
- 구형 그룹 감지 수정: `child_records[1]`만 검사 → 전체 레코드에서 deeper-level SHAPE_COMPONENT 스캔
- `HorzRelTo` 비트 매핑 수정: HWP 스펙에서 0,1=page, 2=column, 3=para

### 3. 렌더링 변환 행렬 파싱 (핵심)

그룹 내 이미지의 `current_width/height`가 원본 이미지 크기를 나타내며, 실제 표시 크기는 SHAPE_COMPONENT의 **렌더링 변환 행렬**(아핀 변환)을 통해 결정됨을 발견하였다.

- `ShapeComponentAttr`에 `render_tx/ty/sx/sy` 필드 추가
- `parse_shape_component_full()`에서 아핀 행렬 합성 (Translation × Scale[0] × Rot[0] × ... ) 구현
- 그룹 자식 좌표: `render_tx/ty`로 위치, `current_size × render_sx/sy`로 크기 계산

| 자식 | 원본 크기 | 스케일 | 실효 크기 | 위치 |
|------|-----------|--------|-----------|------|
| 이미지 0 | 9480×3300 | (0.724, 0.724) | 6860×2387 | (0, 1133) |
| 이미지 1 (배너) | 53640×8340 | (0.518, 0.446) | 27778×3720 | (9360, 0) |
| 이미지 2 | 6082×2457 | (1.472, 1.287) | 8949×3162 | (38559, 474) |

### 4. HorzRelTo::Page 본문 영역 기준 수정

- `HorzRelTo::Page`에서 `x=0` (용지 왼쪽) → `body_area.x + offset` (본문 영역 기준)으로 수정
- `layout_shape()`, `layout_body_picture()`에 `body_area` 파라미터 추가

## 검증

- 488개 Rust 테스트 통과
- `samples/hwp-multi-001.hwp` 2페이지: 3개 그룹 이미지 정상 렌더링
- `samples/hwp-img-001.hwp`: 독립 그림 4개 정상 (회귀 없음)
- WASM 빌드 성공, 웹 브라우저에서 확인 완료

## 수정 파일 목록

| 파일 | 변경 내용 |
|------|-----------|
| `src/model/shape.rs` | ShapeObject에 Picture 변형 추가, ShapeComponentAttr에 render_tx/ty/sx/sy, 수동 Default |
| `src/parser/control.rs` | PICTURE 파싱, 렌더링 변환 행렬 합성, HorzRelTo 비트 수정, 구형 그룹 감지 수정 |
| `src/renderer/layout.rs` | 그룹 자식 렌더링 변환 적용, HorzRelTo::Page 본문 기준, body_area 파라미터 추가 |
| `src/serializer/control.rs` | ShapeObject::Picture 분기 추가 |
| `src/main.rs` | info 커맨드 그룹 자식 정보 출력 |
| `mydocs/plans/task_74_impl.md` | 구현 계획서 |
