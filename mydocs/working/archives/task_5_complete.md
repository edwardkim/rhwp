# Task #5 — 완료보고서

## cellzoneList 셀 영역 배경 지원 ✅

### 수정 파일

- `src/parser/hwpx/section.rs` — HWPX cellzoneList > cellzone 파싱
- `src/parser/hwpx/header.rs` — imgBrush mode="TOTAL" → FitToSize 매핑 추가
- `src/parser/control.rs` — HWP 바이너리 표 레코드에서 zones 파싱
- `src/renderer/layout/table_layout.rs` — cellzone 배경(이미지/단색/그라데이션) 렌더링
- `src/main.rs` — dump에 zone/border_fill 상세 출력 추가

### 변경 내용

1. **HWPX 파서**: `cellzoneList > cellzone` XML 요소를 `Table.zones`에 파싱
2. **HWP 바이너리 파서**: HWPTAG_TABLE 레코드의 border_fill_id 이후 zones 데이터 파싱 (필드 순서: start_row, start_col, end_row, end_col, bf_id)
3. **imgBrush mode**: `"TOTAL"` → `FitToSize` 매핑 추가 (기존에 누락)
4. **렌더러**: 표 배경 렌더링 후, 셀 레이아웃 전에 cellzone 전체 영역에 배경 1회 렌더링

### 검증 결과

- `tac-img-02.hwpx` 15페이지: cellzone 이미지 배경 정상 렌더링 (SVG + 웹 캔버스)
- `tac-img-02.hwp` 15페이지: HWP 바이너리에서도 동일하게 렌더링
- `cargo test`: 777 passed, 0 failed
- 67페이지(HWPX) / 66페이지(HWP) 전체 내보내기 정상
