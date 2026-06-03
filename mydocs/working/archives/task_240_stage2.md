# Task 240 - 2단계 완료 보고서: 조판 부호 모드 책갈피 마커 렌더링

## 완료 항목

### paragraph_layout.rs
- 조판 부호 모드(`show_control_codes`)에서 책갈피 컨트롤 위치에 `[책갈피:이름]` 마커 렌더링
- `find_control_text_positions()`로 컨트롤의 텍스트 위치 계산
- 현재 라인 범위(`line_char_start ~ line_char_end`) 내의 책갈피만 마커 표시
- 기존 `[누름틀 시작/끝]` 마커와 동일한 패턴 (축소 폰트 55%, BGR 0xCC6600 파란색)
- `MarkerInsert`로 수집 후 기존 shift 로직에 통합

## 검증
- `cargo build` 성공
- 716 테스트 통과
