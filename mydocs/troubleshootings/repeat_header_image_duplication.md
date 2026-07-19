---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# 표 제목행 반복 시 이미지 중복 렌더링 문제

> 발견일: 2026-02-10
> 관련 타스크: 35 (표 안의 이미지 처리 문제 해결)
> 수정 파일: `src/renderer/layout.rs`, `src/model/table.rs`, `src/parser/control.rs`, `src/renderer/height_measurer.rs`, `src/renderer/pagination.rs`

## 증상

`samples/20250130-hongbo.hwp` 문서에서:
- 4행 1열 표가 페이지 3~4에 걸쳐 분할됨
- 페이지 4에 이미지가 2개 표시됨 (정상: 1개)
- 페이지 3의 제목행(row=0) 이미지가 페이지 4에도 중복 렌더링

## 원인

### 표 구조

```
P29 Table: rows=4 cols=1, repeat_header=true
  Cell0: row=0  pic(bin=6)  is_header=false  ← 이미지만 있는 행
  Cell1: row=1  (텍스트)    is_header=false
  Cell2: row=2  pic(bin=1)  is_header=false  ← 본문 이미지
  Cell3: row=3  (텍스트)    is_header=false
```

### 핵심: 제목 셀(is_header) 속성

HWP에서 표 제목행 반복은 두 가지 조건으로 동작한다:

1. **표 레벨**: `repeat_header=true` (표 속성 bit 1)
2. **셀 레벨**: `is_header=true` (셀의 제목 셀 체크박스)

**표의 `repeat_header`가 true여도, 행0 셀 중 `is_header=true`인 셀이 없으면 반복하지 않는다.**

### HWP 5.0 스펙에 제목 셀 속성 미기재 문제

HWP 5.0 스펙 문서(표 67: 문단 리스트 헤더, 표 82: 셀 속성)에는 제목 셀 속성이 정의되어 있지 않다.

그러나 실제 바이너리에서는 LIST_HEADER의 확장 속성으로 존재하며, HWPML 3.0 스펙(`hwp_spec_3.0_hwpml.md`)에는 `Header` 속성으로 문서화되어 있다:

```
| 속성 | Header | 제목 셀인지 여부 | true | false | false |
```

### 비트 위치 발견 과정

hwplib(Java HWP 라이브러리)의 `ListHeaderPropertyForCell` 클래스에서 비트 매핑을 확인:

| Bit | 속성 | HWP UI |
|-----|------|--------|
| 0~2 | 텍스트 방향 | 세로쓰기(E) |
| 3~4 | 줄바꿈 방식 | 한 줄로 입력(S) |
| 5~6 | 세로 정렬 | 세로 정렬 |
| 16 | 안 여백 지정 | 안 대백 지정(M) |
| 17 | 셀 보호 | 셀 보호(P) |
| **18** | **제목 셀** | **제목 셀(G)** |
| 19 | 양식모드 편집 가능 | 양식 모드에서 편집 가능(F) |

### 우리 파서와 hwplib의 필드 레이아웃 차이

LIST_HEADER 레코드의 첫 8바이트 해석이 다르다:

```
Byte 0-1: [n_paragraphs (u16)]      / [paraCount 하위 2B]
Byte 2-3: [list_attr 하위 2B]       / [paraCount 상위 2B]
Byte 4-5: [list_attr 상위 2B]       / [property 하위 2B]
Byte 6-7: [list_header_width_ref]   / [property 상위 2B]
```

**hwplib의 property bit 18 = 우리 `list_header_width_ref`의 bit 2**

```rust
// list_header_width_ref (bytes 6-7)에는 셀 확장 속성이 포함됨
// hwplib ListHeaderPropertyForCell 기준:
//   bit 0 (=property bit 16): 안 여백 지정
//   bit 1 (=property bit 17): 셀 보호
//   bit 2 (=property bit 18): 제목 셀
//   bit 3 (=property bit 19): 양식모드 편집 가능
cell.is_header = (cell.list_header_width_ref & 0x04) != 0;
```

### webhwp(한컴 공식 뷰어) 참조

webhwp에서는 헤더 반복 전에 모든 셀의 콘텐츠 타입을 검증한다:
- 모든 셀이 TEXT 타입이면 반복 허용
- 하나라도 비텍스트(이미지/도형)가 있으면 반복 비활성화

## 해결

### 1. Cell 모델에 `is_header` 필드 추가

```rust
// src/model/table.rs
pub struct Cell {
    // ...
    pub is_header: bool,  // 제목 셀 여부 (list_attr bit 18)
}
```

### 2. 파서에서 `is_header` 읽기

```rust
// src/parser/control.rs - parse_cell()
cell.is_header = (cell.list_header_width_ref & 0x04) != 0;
```

### 3. 레이아웃에서 `is_header` 기반 반복 판단

```rust
// src/renderer/layout.rs - layout_partial_table()

// 행0에 is_header 셀이 있을 때만 제목행 반복
let render_header = is_continuation && table.repeat_header && start_row > 0
    && table.cells.iter()
        .filter(|c| c.row == 0)
        .any(|c| c.is_header);

// is_header가 아닌 반복 셀은 건너뜀
if is_repeated_header_cell && !cell.is_header {
    continue;
}

// is_header 반복 셀에서는 컨트롤(이미지/도형) 건너뜀, 텍스트만 반복
if !is_repeated_header_cell {
    for ctrl in &para.controls { /* Picture, Shape, Table 배치 */ }
}
```

### 4. Pagination에서도 `has_header_cells` 반영

```rust
// src/renderer/height_measurer.rs
pub struct MeasuredTable {
    pub has_header_cells: bool,  // 행0에 is_header 셀 존재 여부
}

// src/renderer/pagination.rs
let header_overhead = if is_continuation && mt.repeat_header
    && mt.has_header_cells && row_count > 1 { ... };
```

## 결과

- 페이지 3: 이미지 1개 (Cell0의 이미지)
- 페이지 4: 이미지 1개 (Cell2의 이미지, 반복 헤더 없음)
- 415 테스트 통과
