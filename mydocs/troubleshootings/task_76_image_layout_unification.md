---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# 트러블슈팅: 이미지 배치 로직 불일치로 인한 회귀 반복

## 문제 상황

이미지/도형 배치 수정 시 한 곳을 고치면 다른 곳이 회귀하는 문제가 반복 발생.

## 근본 원인

`layout.rs`에서 Picture와 Shape의 좌표 계산이 **두 함수에서 서로 다른 로직으로 중복 구현**되어 있었다.

### 발견된 불일치 3곳

| 축/기준 | `layout_body_picture()` | `layout_shape()` | HWP 스펙 정답 |
|---------|------------------------|-------------------|--------------|
| **VertRelTo::Page** | `col_area.y + offset` | `offset` (기준점 없음!) | `body_area.y + offset` |
| **HorzRelTo::Para** | `container.x + offset` | `col_area.x + offset` | `container.x + offset` |
| **인라인 정렬** | `container.x/width` 기준 | `col_area.x/width` 기준 | `container.x/width` |

추가로 `calculate_shape_reserved_height()`도 `VertRelTo::Page`에서 기준점 없이 offset만 사용.

### 문제 메커니즘

1. `layout_body_picture()`에서 특정 기준(예: VertRelTo::Page)을 수정
2. `layout_shape()`는 별도 코드이므로 수정이 반영되지 않음
3. Shape 기반 이미지(그룹 묶기 등)에서 회귀 발생
4. `layout_shape()`를 수정하면 이번엔 Picture 쪽과 불일치

## 해결

### `compute_object_position()` 통합 함수 추출

```rust
fn compute_object_position(
    &self,
    common: &CommonObjAttr,
    obj_width: f64,
    container: &LayoutRect,
    col_area: &LayoutRect,
    body_area: &LayoutRect,
    para_y: f64,
    alignment: Alignment,
) -> (f64, f64)
```

통합 규칙 (HWP 스펙 기준):

```
가로(X):
  treat_as_char → 문단 정렬(container 기준)
  Paper  → 0 + offset
  Page   → body_area.x + offset
  Column → col_area.x + offset
  Para   → container.x + offset

세로(Y):
  treat_as_char → para_y
  Paper  → 0 + offset
  Page   → body_area.y + offset
  Para   → para_y + offset
```

### 리팩토링 대상

1. `layout_body_picture()` — 통합 함수 호출로 교체
2. `layout_shape()` — 통합 함수 호출로 교체
3. `calculate_shape_reserved_height()` — `body_area` 파라미터 추가, 통합 규칙 적용

### Paper 바이패스 조건 정밀화

```rust
// 수정 전: OR (한 축이라도 Paper이면 body clip 바이패스)
let is_paper_based = vert == Paper || horz == Paper;

// 수정 후: AND (양축 모두 Paper인 경우만 바이패스)
let is_paper_based = vert == Paper && horz == Paper;
```

## 검증

- 491개 Rust 테스트 통과 (기존 488 + 신규 회귀 테스트 3)
- hwp-multi-001.hwp: 2페이지 그룹 이미지 3장 정상
- hwp-3.0-HWPML.hwp: 1페이지 배경 이미지 정상
- hwp-img-001.hwp: 독립 이미지 4장 정상
- 웹 브라우저 검증 완료

## 교훈

1. **좌표 계산은 단일 함수로 통합**: 동일한 계산을 여러 곳에 분산하면 불일치가 필연적으로 발생한다.
2. **회귀 테스트 필수**: 이미지 배치 관련 변경 시 모든 샘플 파일의 이미지 위치를 자동 검증해야 한다.
3. **HWP 스펙 기준 통일**: VertRelTo::Page는 "쪽 영역 기준"이므로 `body_area.y`가 정답. 함수마다 다른 해석을 하면 안 된다.
