---
kind: reference
status: active
canonical: mydocs/tech/parser_architecture.md
last_verified: 2026-07-17
---

# HWP ↔ HWPX IR 차이점 정리

HWPX 파싱 시 HWP 바이너리와 동일한 IR(중간 표현)을 생성하기 위해 주의해야 할 차이점을 기록한다.
향후 HWPX 읽기/쓰기, HWP→HWPX 변환 구현 시 참조한다.

## 1. 표(Table) CommonObjAttr

### HWP 바이너리
- CTRL_HEADER의 `ctrl_data` 전체가 CommonObjAttr 구조 (Shape/GSO와 동일)
- `table.common`: treat_as_char, text_wrap, vert_rel_to, horz_rel_to, width, height 등 자동 파싱
- `table.attr`: `table.common.attr`에서 동기화
- `table.raw_ctrl_data`: 원본 바이너리 보존 (라운드트립용)

### HWPX XML
- `<tbl>` 요소 속성: `textWrap`, `rowCnt`, `colCnt` 등
- `<pos>` 자식 요소: `treatAsChar`, `vertRelTo`, `horzRelTo`, `vertOffset`, `horzOffset` 등
- `<sz>` 자식 요소: `width`, `height`
- `<outMargin>` 자식 요소: `left`, `right`, `top`, `bottom`
- `raw_ctrl_data`는 비어있음 → **HWPX 판별 기준으로 활용 가능**

### 주의사항
- `table.attr` 비트 연산 대신 `table.common` 필드를 사용해야 HWPX에서도 동작
- Task 278에서 pagination/engine.rs 변환 완료
- Task 286에서 나머지 renderer 전체 변환 (약 40곳)

## 2. 셀 apply_inner_margin

### HWP 바이너리
- LIST_HEADER의 `list_attr bit 16`: "안 여백 지정"
- `cell.apply_inner_margin = (list_attr >> 16) & 0x01 != 0`
- false이면 셀 패딩 무시, 표 기본 패딩 사용

### HWPX XML
- `<tc>` 요소의 `hasMargin` 속성: `true`/`false`
- OWPML 스키마: `<xs:attribute name="hasMargin" type="xs:boolean" default="false"/>`

### 주의사항
- 기본값이 `false` → HWPX에서 `hasMargin` 미지정 시 표 기본 패딩 사용

## 3. LINE_SEG (lineSegArray)

### HWP 바이너리
- PARA_LINE_SEG 레코드로 저장
- `vpos`: 구역 시작 기준 절대 위치 (사전 계산됨)
- `line_height`: 표/그림 등 인라인 컨트롤 높이 포함
- pagination/layout이 vpos 기반으로 정밀 배치 가능

### HWPX XML
- `<linesegarray>` 요소 (일부 HWPX에서 누락 가능)
- `vpos`: **항상 0** (사전 계산되지 않음)
- `line_height`: 텍스트 폰트 높이만 반영, **인라인 표/그림 높이 미포함**
- 일부 HWPX 파일은 lineSegArray 자체가 없음 → `reflow_line_segs()` 필요

### 주의사항
- `vpos=0`이므로 vpos 기반 배치 로직이 동작하지 않음
- layout 엔진에서 `vpos==0 && para_idx > 0` 조건으로 vpos 보정 스킵하는 코드 존재
- 향후: HWPX 로드 시 vpos를 계산하여 채워넣는 방안 검토 필요

## 4. 어울림(Square) 판정

### HWP 바이너리
- `(table.attr >> 21) & 0x07 == 0` → 어울림 (text_wrap=Square)
- 어울림 표는 후속 문단이 표 옆에 배치됨

### HWPX XML
- `table.attr=0` → 비트 연산 시 **모든 표가 어울림으로 잘못 판정!**
- `table.common.text_wrap`을 사용해야 정확

### 영향
- 어울림으로 잘못 판정되면 후속 텍스트 문단이 높이를 차지하지 않음 → 겹침
- Task 286에서 발견·수정

## 5. raw_ctrl_data 활용

### HWP 바이너리
- `table.raw_ctrl_data`: CTRL_HEADER 원본 바이너리 (42+ 바이트)
- 위치/크기/여백 등 추출에 사용
- `get_table_vertical_offset()` 등에서 참조

### HWPX XML
- `table.raw_ctrl_data`: **빈 Vec** (바이너리 원본 없음)
- 위치/크기 정보는 `table.common` 필드에서 직접 참조
- **HWPX 판별**: `raw_ctrl_data.is_empty()` → HWPX 파일의 표

## 6. 향후 추가 조사 필요 항목

| 항목 | HWP | HWPX | 상태 |
|------|-----|------|------|
| Shape CommonObjAttr | CTRL_HEADER 파싱 | `<shapeObject>` 요소 | 부분 구현 |
| 각주/미주 | CTRL_FOOTNOTE | `<footNote>` | HWPX 미구현 |
| 머리말/꼬리말 | CTRL_HEADER/FOOTER | `<headerFooter>` | HWPX 미구현 |
| 그리기 객체 | control/shape.rs | `<rect>`, `<line>` 등 | HWPX 미구현 |
| 필드/하이퍼링크 | CTRL 태그 | `<ctrl>` | HWPX 미구현 |
| HWPX→HWP 변환 | - | - | 미구현 |
| HWP→HWPX 변환 | - | - | 미구현 |
