# Task #3 — 단계별 완료보고서

## 1단계: 줄별 TAC 분배 로직 구현 ✅

### 수정 파일

- `src/renderer/layout/table_layout.rs` — TAC 이미지 배치 로직 수정

### 변경 내용

1. **`total_inline_width` 계산을 줄별 최대 너비로 변경**
   - 기존: 모든 TAC 너비 합산 (한 줄로 간주)
   - 변경: 줄별 너비 합산 벡터(`tac_line_widths`) 구축, 줄별 최대 너비로 정렬 계산

2. **LINE_SEG 기반 줄 판별 및 수직 배치**
   - 빈 문단(runs 없음): TAC 순번으로 LINE_SEG에 1:1 매핑 (`tac_seq_index`)
   - 텍스트 있는 문단: char position으로 줄 판별 (기존 로직 유지)
   - 줄이 바뀌면 `inline_x` 리셋, `tac_img_y`를 LINE_SEG vpos 기준으로 이동

## 2단계: SVG 내보내기 검증 및 회귀 테스트 ✅

- `tac-img-02.hwpx` 14페이지: 이미지 3개 수직 배치 확인
- WASM 빌드 + 웹 캔버스 검증 완료
- `cargo test`: 777 passed, 0 failed
- 67페이지 전체 내보내기: 에러/패닉 없음

## 3단계: dump 코드 정리 ✅

- 셀 내부 컨트롤 상세 출력 코드(`src/main.rs`) **유지** (디버깅 유용성)

## 발견된 별도 이슈

- [#4](https://github.com/edwardkim/rhwp/issues/4): 비-TAC 그림(어울림 배치) 높이가 후속 요소 y에 미반영 (21페이지)
