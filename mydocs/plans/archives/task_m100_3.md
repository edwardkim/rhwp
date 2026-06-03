# Task #3: 셀 내 TAC 이미지 수직 배치 버그 수정 — 수행계획서

## 목표

표 셀 내부에 TAC(treat_as_char) 이미지가 여러 개 있을 때, LINE_SEG 줄 분배 정보를 기반으로 수직 배치가 올바르게 동작하도록 수정한다.

## 현상

- `samples/tac-img-02.hwpx` 14페이지, `s0:pi=165` (1x1 표)
- 셀 내 문단 1개에 TAC 이미지 3개, LINE_SEG 3줄
- 이미지가 수평으로만 나열되어 셀 너비를 초과 → 크롭 발생

## 원인

| 위치 | 문제 |
|------|------|
| `table_layout.rs:1213-1237` | TAC 이미지를 `inline_x += pic_w`로 수평 배치만 수행. LINE_SEG 줄 정보 미참조 |
| `paragraph_layout.rs:1671` | `cell_ctx.is_none()` 조건으로 셀 내부 TAC 이미지 스킵 (table_layout에 위임) |
| `table_layout.rs:1132-1148` | `total_inline_width`가 모든 TAC 너비를 합산 (줄별 분배 미고려) |

## 해결 전략

`table_layout.rs`의 셀 내 TAC 이미지 배치 루프에서 `composed.tac_controls`의 char position과 `composed.lines`의 `char_start`를 비교하여 이미지가 속한 줄을 판별하고, 줄이 바뀌면 `inline_x`를 리셋하고 `y` 좌표를 해당 LINE_SEG의 vpos 기준으로 이동한다.

## 구현 단계

### 1단계: 줄별 TAC 분배 로직 구현

- `table_layout.rs`의 TAC 이미지 배치 루프(1213~)를 수정
- 각 TAC 컨트롤의 `abs_pos`가 어느 `composed.lines[i].char_start` 범위에 속하는지 판별
- 줄이 바뀌면 `inline_x`를 줄 시작 x로 리셋, `y`를 LINE_SEG vpos 기준으로 이동
- `total_inline_width` 계산도 줄별 최대 너비로 변경

### 2단계: SVG 내보내기 검증 및 회귀 테스트

- `tac-img-02.hwpx` 14페이지 SVG로 이미지 3개 수직 배치 확인
- 기존 `cargo test` 전체 통과 확인
- 기존 샘플 파일의 TAC 이미지 렌더링 회귀 없는지 확인

### 3단계: dump 코드 정리

- 디버깅용으로 추가한 셀 내부 컨트롤 상세 출력 코드 정리 (유지 또는 제거 판단)

## 영향 범위

- `src/renderer/layout/table_layout.rs` — TAC 이미지 배치 로직 수정
- `src/main.rs` — dump 코드 정리 (선택)

## 검증 기준

- `tac-img-02.hwpx` 14페이지에서 이미지 3개가 셀 내부에 위→아래로 배치
- `cargo test` 전체 통과
- 기존 샘플의 TAC 렌더링 회귀 없음
