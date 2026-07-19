---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# 그룹 자식 TextBox 세로 정렬 미적용 문제

## 날짜
2026-02-16

## 관련 타스크
타스크 96 (Container/Group 렌더링)

## 증상

- 독립 도형(글상자)에서는 세로 정렬(가운데)이 정상 동작
- 동일한 글상자를 화살표 이미지와 그룹핑하면 세로 정렬이 무시됨
- `samples/basic/tbox-center.hwp` (독립) → 세로 중앙 정렬 정상
- `samples/basic/tbox-center-02.hwp` (그룹) → 세로 정렬 미적용 (상단 정렬)

## 원인 분석

### HWP 구조 차이

**독립 도형** — 별도의 TextBox 파싱 경로를 통해 LIST_HEADER 데이터를 읽음:
```
CTRL_HEADER 'gso'
  SHAPE_COMPONENT
  SHAPE_COMPONENT_RECTANGLE
  LIST_HEADER  ← list_attr=0x00000020 (center), margins=(283,283,283,283)
    PARA_HEADER + PARA_TEXT + ...
```

**그룹 자식** — `parse_container_children()`에서 LIST_HEADER를 만나면 `continue`로 건너뜀:
```
CTRL_HEADER 'gso'
  SHAPE_COMPONENT (그룹 컨테이너)
  SHAPE_COMPONENT_CONTAINER
    SHAPE_COMPONENT (자식 사각형)
    SHAPE_COMPONENT_RECTANGLE
    LIST_HEADER  ← 데이터가 파싱되지 않고 건너뜀!
      PARA_HEADER + PARA_TEXT + ...
```

### 근본 원인

`parse_container_children()` 함수(control.rs)에서 자식 도형의 레코드를 순회할 때:

```rust
// 수정 전 코드
if record.tag_id == tags::HWPTAG_LIST_HEADER && !list_started {
    list_started = true;
    continue;  // ← LIST_HEADER 데이터를 읽지 않고 건너뜀!
}
```

LIST_HEADER 레코드의 data에는 다음 정보가 포함되어 있음:
- `para_count` (4바이트)
- `list_attr` (4바이트) — bit 5~6: 세로 정렬 (0=위, 1=가운데, 2=아래)
- `margin_left/right/top/bottom` (각 2바이트)
- `max_width` (4바이트)

이 데이터를 읽지 않아서 TextBox의 `list_attr`이 항상 0(위쪽 정렬), margins가 모두 0으로 설정됨.

## 해결 방법

### 1. LIST_HEADER 데이터 캡처 및 파싱

`parse_container_children()`에서 LIST_HEADER 레코드의 데이터를 보존:

```rust
let mut list_header_data: Option<&[u8]> = None;
// ...
if record.tag_id == tags::HWPTAG_LIST_HEADER && !list_started {
    list_started = true;
    list_header_data = Some(&record.data);  // 데이터 캡처
    continue;
}
```

문단 수집 완료 후 TextBox 생성 시 LIST_HEADER 데이터에서 속성 파싱:

```rust
if let Some(lh_data) = list_header_data {
    let mut lr = ByteReader::new(lh_data);
    let _para_count = lr.read_u32().unwrap_or(0);
    text_box.list_attr = lr.read_u32().unwrap_or(0);
    let v_align = ((text_box.list_attr >> 5) & 0x03) as u8;
    text_box.vertical_align = match v_align {
        1 => VerticalAlign::Center,
        2 => VerticalAlign::Bottom,
        _ => VerticalAlign::Top,
    };
    text_box.margin_left = lr.read_i16().unwrap_or(0);
    text_box.margin_right = lr.read_i16().unwrap_or(0);
    text_box.margin_top = lr.read_i16().unwrap_or(0);
    text_box.margin_bottom = lr.read_i16().unwrap_or(0);
    text_box.max_width = lr.read_u32().unwrap_or(0);
}
```

### 2. SHAPE_COMPONENT 인라인 텍스트 속성 (fallback)

`parse_shape_component_full()` 반환 타입을 4-tuple로 확장하여 채우기 데이터 이후의 인라인 텍스트 속성(표 92)도 반환. LIST_HEADER 레코드가 없는 경우의 fallback으로 사용.

## 수정 파일

| 파일 | 수정 내용 |
|------|----------|
| `src/parser/control.rs` | `parse_container_children()`: LIST_HEADER 데이터 캡처 및 파싱 |
| `src/parser/control.rs` | `parse_shape_component_full()`: 반환 타입 4-tuple 확장 |

## 검증 결과

| 파일 | 수정 전 | 수정 후 |
|------|--------|--------|
| tbox-center.svg (독립) | 세로 중앙 정렬 정상 | 세로 중앙 정렬 정상 |
| tbox-center-02.svg (그룹) | 세로 정렬 미적용 (상단) | 세로 중앙 정렬 정상 |
| KTX.hwp 범례 | 텍스트 상단 몰림 | 세로 중앙 정렬 적용 |

텍스트 Y 좌표 비교 (수정 후):
- 독립 도형: y=696.77, 723.43
- 그룹 자식: y=696.77, 723.43 (동일)

## 교훈

- 그룹 자식 도형은 독립 도형과 파싱 경로가 다르므로, 레코드 순회 시 모든 레코드의 데이터를 정확히 처리해야 함
- `continue`로 건너뛰는 레코드라도 그 data 필드에 중요한 속성이 포함될 수 있음
- 독립 도형에서 동작하는 기능이 그룹 자식에서 동작하지 않으면, 파싱 경로 차이를 의심할 것
