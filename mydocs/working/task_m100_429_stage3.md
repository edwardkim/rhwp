# Task #429 Stage 3 — 광범위 회귀 검증

## 검증 게이트

| 검증 | 결과 |
|------|------|
| `cargo test --lib` | **1080 passed** ✅ |
| `cargo test --test svg_snapshot` | 6/6 ✅ |
| `cargo test --test issue_418` | 1/1 ✅ |
| `cargo clippy --lib -- -D warnings` | 0건 ✅ |

## aift.hwp 광범위 byte 비교

| 항목 | 결과 |
|------|------|
| 총 페이지 | 77 |
| 변화 페이지 | **2** (41쪽, 43쪽) |
| 동일 페이지 | 75 |

### 변화 영역

| 페이지 | 정정 전 image | 정정 후 image | 비고 |
|-------|--------------|--------------|------|
| 41 | 0 | **1** | 본 결함 정정 영역 (셀 배경 그림 표시) |
| 43 | 1 | **3** | 다른 셀 배경 image fill 케이스 (정정 의도 영역) |

## 다른 샘플 (image fill BorderFill 0개) 회귀 점검

다른 샘플 (synam-001, k-water-rfp, exam_kor, 2022 국립국어원) 은 image fill BorderFill 0개 — 본 정정의 영향 영역 없음.

SVG 추출 정상 (35/27/40/20 페이지) — 회귀 0건.

## zone image_fill 처리 제거 영향

zone 의 별도 image_fill 처리 (table_layout.rs:302-316) 제거 후 — `render_cell_background` 가 통합 처리. zone image_fill 케이스가 있는 샘플 (aift.hwp 가 아닌 케이스) 에서 회귀 점검 — 본 정정의 image_fill 분기가 zone 도 처리하므로 동일 동작.

## 다음 단계

Stage 4: 작업지시자 시각 검증 (aift 41쪽 + 43쪽).
