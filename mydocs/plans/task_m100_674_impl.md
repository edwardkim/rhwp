# Task #674 구현 계획서

## 개요

Issue #674 "paragraph_layout 줄 위치 vs row_heights 정합 — line_segs 부재 paragraph 마지막 줄 시각 클립" 정정 구현. 3 단계 진행.

## Stage 1: 본질 진단 — 25.88px 오프셋 원인 식별

### 목표

`samples/계획서.hwp` 셀 [21] 의 `text_y_start = 379.37` 결정 경로 정확 식별. 25.88px 오프셋의 정확한 위치 (text_y_start 분기 또는 추가 보정) 진단.

### 진단 절차

1. **text_y_start 디버그 추가**
   - `src/renderer/layout/table_layout.rs:1374-1388` 에 환경변수 기반 디버그 출력
   - 셀 [21] (cell_idx=21, r=5,c=1) 의 분기 결과 + first_line_vpos + effective_valign + inner_height + total_content_height + mechanical_offset 출력

2. **분기 식별**
   - 어느 분기 (first_line_vpos / Top / Center / Bottom) 진입
   - mechanical_offset 또는 추가 offset 영역

3. **다른 셀 비교**
   - 정상 표시되는 셀 (예: 셀 [13]) 의 text_y_start 분기 비교
   - 차이 영역 식별

### 진단 산출물

`mydocs/working/task_m100_674_stage1.md` — 본질 진단 결과 + Stage 2 정정 방향 결정.

### 승인 요청

Stage 1 진단 결과 + Stage 2 정정 방향 승인 요청.

## Stage 2: 본질 정정

### 목표

Stage 1 진단 결과에 따라 `text_y_start` 결정 로직 또는 관련 영역 정정. paragraph layout 시작 위치 = `cell_y + pad_top` (Top 정렬, line_segs 부재 케이스).

### 정정 원칙

- **케이스 가드 명시**: line_segs 부재 paragraph 만 영향. 정상 인코딩된 paragraph 무영향.
- **회귀 위험 좁힘**: 다른 셀 / 다른 fixture 의 text_y_start 결정 로직에 영향 없음.
- **본질 룰**: 25.88px 오프셋의 본질 원인 정확히 정정 (휴리스틱 금지).

### 정정 위치 후보

`src/renderer/layout/table_layout.rs:1374-1388`:

```rust
let text_y_start = if !has_nested_table && first_line_vpos.filter(|&v| v > 0.0).is_some() {
    cell_y + pad_top + first_line_vpos.unwrap()
} else {
    match effective_valign {
        VerticalAlign::Top => cell_y + pad_top,
        VerticalAlign::Center => { ... }
        VerticalAlign::Bottom => { ... }
    }
};
```

Stage 1 진단에 따른 정정 후보:

#### A. first_line_vpos 분기 잘못 진입 시

- line_segs.is_empty() 인 paragraph 에서 first_line_vpos 가 다른 값 (예: 다른 paragraph 의 vpos) 으로 결정되는 경우
- 가드: `cell.paragraphs.first().filter(|p| !p.line_segs.is_empty()).and_then(...)`

#### B. effective_valign 이 Center 시

- 셀 [21] 의 vertical_align 이 Center 로 잘못 인식
- 또는 mechanical_offset 계산 오류 (inner_height vs total_content_height)

#### C. mechanical_offset 계산 오류

- total_content_height 가 inner_height 보다 작게 측정 → 비례 offset 발생
- recompose_for_cell_width 로 줄 수 변경 시 total_content_height 재계산 필요

### 검증 절차

1. **결정적 검증**
   - cargo test --lib --release 회귀 0 (1155+ passed)
   - svg_snapshot 6/6 + issue_546 1/1 + issue_554 12/12
   - cargo clippy --release 신규 경고 0
2. **시각 검증**
   - `samples/계획서.hwp` 셀 [21] 3 줄 / [52] 3 paragraph 정상 표시 (PNG 변환)
3. **광범위 sweep**
   - 187 fixture 페이지 수 차이 0 (Stage 3)

### 정정 산출물

- 영향 코드 변경 (`src/renderer/layout/table_layout.rs` 등)
- `mydocs/working/task_m100_674_stage2.md` 단계별 보고서

### 승인 요청

Stage 2 정정 결과 + 시각 판정 통과 → Stage 3 진행 승인.

## Stage 3: 광범위 회귀 sweep + 최종 검증

### 목표

광범위 페이지네이션 회귀 sweep + 시각 판정 게이트웨이 통과. `samples/계획서.hwp` 1 페이지 표 시각 결함 완전 해소.

### 검증 절차

1. **광범위 페이지네이션 회귀 sweep**
   - 187 fixture BEFORE/AFTER 페이지 수 차이 0
2. **결정적 검증 (release 모드)**
   - cargo test --lib --release 1155+ passed
   - cargo test --release 전체 GREEN
   - cargo clippy --release 신규 경고 0
3. **시각 판정 게이트웨이 (작업지시자)**
   - `samples/계획서.hwp` 1 페이지 시각 판정 (PNG)
   - 셀 [21] 3 줄 모두 표시 + 셀 [52] 3 paragraph 모두 표시 확인
   - 다른 셀 영역 회귀 0 확인

### 산출물

- `mydocs/report/task_m100_674_report.md` 최종 보고서
- `mydocs/orders/20260507.md` Task #674 상태 갱신

### 승인 요청

최종 보고서 + 시각 판정 게이트웨이 통과 후 작업지시자 승인 → fork push + PR 생성 영역.

## 회귀 위험 영역 좁힘 원칙

- **수정 영역 명시**: text_y_start 결정 로직 또는 관련 영역만
- **케이스 가드**: line_segs 부재 paragraph + 본 결함 발현 영역만 정정 영향
- **광범위 sweep 검증**: 187 fixture 페이지 수 차이 0
- **다른 영역 영향**: cell-clip 영역, paragraph_layout y 누적 등 무영향

## 의존성

- **선행 의존**: Task #671 + #672 정정 코드 (`local/task672` 브랜치) — 본 task 는 그 위에서 분기
- **후행 의존**: 없음 — `samples/계획서.hwp` 시각 결함 완전 해소

## 최종 결과 영역

본 task 완료 후:
- Issue #674 close (closes #674 키워드)
- `samples/계획서.hwp` 1 페이지 표 시각 결함 (Task #671/#672/#674) 완전 해소
- Task #671~#674 시리즈 완료
