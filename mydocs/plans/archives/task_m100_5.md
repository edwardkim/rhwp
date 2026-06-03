# Task #5: cellzoneList 셀 영역 배경 미지원 — 수행계획서

## 목표

표의 `cellzoneList`에 정의된 셀 영역 배경(이미지/그라데이션/단색)을 파싱하고 렌더링한다.

## 현상

- `samples/tac-img-02.hwpx` 15페이지, `s0:pi=169` (1x2 표)
- cellzone (row 0, col 0~1)에 borderFillIDRef=18 (이미지 배경) 정의
- 현재 파서/렌더러에서 cellzoneList를 지원하지 않음
- 셀[0] 흰색 텍스트가 배경 없이 보이지 않음

## HWPX 구조

```xml
<cellzoneList>
  <cellzone startRowAddr="0" startColAddr="0"
            endRowAddr="0" endColAddr="1"
            borderFillIDRef="18" />
</cellzoneList>
```

borderFill id=18: imgBrush (image1)

## 구현 단계

### 1단계: 모델 + 파서

- `Table` 모델에 `cell_zones: Vec<CellZone>` 필드 추가
- `CellZone` 구조체 정의: start_row, start_col, end_row, end_col, border_fill_id
- HWPX 파서(`section.rs`)에서 `cellzoneList > cellzone` 파싱
- HWP 바이너리 파서에서도 대응 (있는 경우)

### 2단계: 렌더러

- `table_layout.rs`에서 셀 렌더링 시 해당 셀이 cellzone에 포함되는지 확인
- cellzone의 border_fill 배경을 셀 배경보다 먼저(또는 대신) 렌더링
- 이미지 채우기(image_fill) 렌더링 지원 확인

### 3단계: 검증

- `tac-img-02.hwpx` 15페이지 SVG 확인
- `cargo test` 전체 통과
- 67페이지 전체 내보내기 회귀 없음

## 영향 범위

- `src/model/table.rs` — CellZone 모델
- `src/parser/hwpx/section.rs` — cellzoneList 파싱
- `src/renderer/layout/table_layout.rs` — cellzone 배경 렌더링

## 검증 기준

- pi=169 셀 영역에 이미지 배경이 렌더링됨
- `cargo test` 전체 통과
