# Task #297 구현 계획서 (v2)

> **v1 폐기 사유**: 단계 3 진입 후 원인 재분석 결과, 문제는 바탕쪽 표가 아니라 본문 pi=22의 `VertRelTo::Page` 표 배치였음. v1은 바탕쪽 bottom-anchoring 가설이었으나 실제 영향 없음을 확인하고 되돌림. v2로 교체.

## 수정 개요

`compute_table_y_position`에서 `VertRelTo::Page`와 `VertRelTo::Paper`가 동일하게 `(0.0, page_h_approx)`를 기준으로 사용하여 **Page와 Paper가 구분되지 않는 것**이 근본 원인.

HWP 스펙 의미:
- **Paper**: 용지 전체 (절대 좌표)
- **Page**: 쪽 본문 영역 (body area, 여백/머리말/꼬리말 제외)
- **Para**: 문단
- **Column**: 단

현재 Page가 Paper처럼 처리되어, `VertAlign::Bottom` + `v_offset=141 HU` 바디 표가 용지 바닥에서 계산 → PDF 대비 ~147 px 아래로 밀림.

## 수정 지점

**파일**: `src/renderer/layout/table_layout.rs`  
**위치**: `compute_table_y_position` L986-990

```rust
let (ref_y, ref_h) = match vert_rel_to {
    crate::model::shape::VertRelTo::Page => (col_area.y, col_area.height),  // 본문 영역
    crate::model::shape::VertRelTo::Para => (anchor_y, col_area.height - (anchor_y - col_area.y).max(0.0)),
    crate::model::shape::VertRelTo::Paper => (0.0, page_h_approx),
};
```

변경: `VertRelTo::Page` 기준점을 `(0.0, page_h_approx)` → `(col_area.y, col_area.height)`로 변경.

## 계산 검증

**본문 pi=22 표** (`vert=Page/141, valign=Bottom, size=419.5×134.7px`):

| 경우 | 공식 | 결과 |
|------|------|------|
| 현재 (Page=Paper) | `0 + 1508.1 − 134.7 − 1.9` | **1371.5 px** (SVG 실측 일치) |
| 수정 (Page=body) | `147.4 + 1213.3 − 134.7 − 1.9` | **1224.1 px** (PDF 1226±2 px 일치 ✓) |

## 회귀 영향

### 바탕쪽(master page) 표
바탕쪽 문맥에서 `col_area = paper_area`(`x=0, y=0, width=paper_w, height=paper_h`)이므로:
- 수정 후: `VertRelTo::Page` → `(0.0, paper_h)` = 현재 `page_h_approx`와 동일 (paper_h_approx는 col_area 기반 추정치라 미세 차이 가능)

**스캔 결과**: 바탕쪽 `VertRelTo::Page` 표는 exam_eng/exam_kor/exam_science에서 5건. 이들의 col_area.y=0이므로 기존과 동일 결과 → **회귀 없음**.

단, `page_h_approx = col_area.y*2 + col_area.height`에서 `col_area.y=0`일 때 page_h_approx = col_area.height = paper_h. 수정 후 `col_area.height`도 paper_h (바탕쪽 col_area는 paper_area 전체). 일치.

### 본문 `VertRelTo::Page` 표
- exam_math.hwp pi=22 ("* 확인 사항") — 목표 수정
- 다른 샘플 스캔 결과: 본문에 `vert=Page`인 표는 없음 (scan 시 모두 master page 경로였음)

### 본문 `VertRelTo::Paper`, `VertRelTo::Para` 표
- 수정 없음 → 영향 없음

## 단계 3 세부 (수정 + 단위 검증)

**3-1. 코드 수정**
- `compute_table_y_position:986-990`에서 `VertRelTo::Page` 분기 수정
- 주석 추가: "Page는 본문영역 기준, Paper는 용지 전체 기준"

**3-2. 단위 검증 (exam_math.hwp 12쪽)**
- `cargo build --release`
- `rhwp export-svg samples/exam_math.hwp -p 11 -o /tmp/t297v2/`
- SVG에서 pi=22 표의 `BoundingBox` 또는 "* 확인 사항" 텍스트 y좌표 확인
- 목표: 현재 1385.47 → 1237±5 px (PDF 기준)

**3-3. 회귀 검증 (시각 비교)**
- `exam_math.hwp` 전체 페이지 (1~20쪽) 바탕쪽 박스 위치 변화 없음 확인
- `exam_eng.hwp`, `exam_kor.hwp`, `exam_science.hwp`, `exam_social.hwp`, `exam_math_8.hwp` — 바탕쪽 `vert=Page` 있는 파일 — 시각 회귀 확인
- `equation-lim.hwp`, `text-align-2.hwp` — 바탕쪽 없음 — 변화 없음 확인

## 단계 4 (회귀 테스트 + 최종 보고)

- `cargo test` 전체 통과
- 기타 주요 샘플 export-svg 시각 확인
- `mydocs/report/task_m100_297_report.md` 작성

## 수정 승인 요청

v2 계획으로 단계 3(수정+검증) 진행. 승인 후 곧바로 코드 수정 적용.
