# 타스크 235 최종 완료 보고서

## 개요

TAC 블록 표의 공백 기반 수평 위치 오류 수정 및 pagination/layout 간 vpos base 불일치로 인한 페이지 오버플로우 감소.

## 수정 내용

### 1. TAC 블록 표 공백 x 오프셋 (layout.rs)

- **문제**: 편집자가 `·[표]··` 패턴으로 TAC 표의 수평 위치를 조정하는데, 블록 TAC 표 렌더링 시 선행 공백이 무시됨
- **수정**: 공백만 있는 호스트 문단의 경우, FFFC(표 컨트롤 문자) 이전 공백 폭을 `effective_margin`에 추가
- **결과**: 공백 기반 수평 위치 조정이 블록 TAC 표에도 반영

### 2. pagination vpos base 동기화 (engine.rs)

- **문제**: `paginate_text_lines`에서 페이지 분할 후 FP/PP를 배치할 때 `page_vpos_base`를 설정하지 않아, 이후 메인 루프에서 다른 문단의 vpos로 base가 설정됨. layout은 해당 페이지의 실제 첫 항목 vpos를 base로 사용하므로 불일치 발생
- **수정**: `paginate_text_lines` 내에서 FP 배치 시 `para.line_segs.first()`, PP 배치 시 `para.line_segs.get(cursor_line)`으로 `page_vpos_base` 설정
- **결과**: pagination과 layout의 vpos base가 동일한 문단 기준으로 설정되어 높이 계산 동기화

### 3. TAC 표 vpos snap 기준 변경 (layout.rs)

- `col_area.y` → `para_y_for_table` (문단 시작 y 좌표 기준)
- vertical_pos가 문단 시작으로부터의 오프셋이므로, 문단 시작을 기준으로 line_end 계산

### 4. ls/2 조건부 적용 (layout.rs)

- 같은 문단 내 다중 TAC 표 사이에서만 `line_spacing/2` 적용
- 마지막 TAC 표 이후에는 추가하지 않아 pagination과 일치

## 결과

| 항목 | 수정 전 | 수정 후 |
|------|---------|---------|
| LAYOUT_OVERFLOW | 12건 | 9건 |
| 최대 오버플로우 | 28.1px (page 6) | 22.3px (page 35) |
| 해소된 페이지 | - | 6, 7, 9, 74 |
| 페이지 수 | 78 | 78 (한컴과 동일) |
| cargo test | 716 pass | 716 pass |
| 회귀 (KTX, field-02, f11-01) | 0건 | 0건 |

## 잔여 9건 오버플로우

개별 표의 effective_height 차이(pagination 추정 < layout 실제 렌더 높이)가 원인. 대형 표(800~900px)에서 주로 발생하며 별도 태스크 대응 필요.

## 변경 파일

- `src/renderer/layout.rs` — TAC 블록 표 공백 x오프셋, vpos snap 기준 변경, ls/2 조건부 적용
- `src/renderer/pagination/engine.rs` — paginate_text_lines 내 page_vpos_base 설정

## 커밋

- `2b39380` TAC 블록 표 공백 x오프셋 및 pagination vpos base 동기화 (Task 235)
