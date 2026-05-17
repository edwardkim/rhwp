# Task #959 — 최종 보고서

- 이슈: [#959](https://github.com/edwardkim/rhwp/issues/959)
- 마일스톤: M100 / v1.0.0
- 브랜치: `local/task959`
- 기간: 2026-05-17 (1일)

## 1. 작업 범위

원 issue #952 의 Issue 3 (HWP5 시험지 page 1 우측 단 문9 vertical 처짐) 해결.

## 2. Root cause

### 2.1 증상 데이터

시험지 (3-11월) page 1 우측 단:
| 문제 | rhwp y (before) | 한컴 (PDF) |
|------|----------------|-----------|
| 문6 | 103 | ~ |
| 문7 | 319 | ~ |
| 문8 | 562 | ~ |
| **문9** | **1061 ⚠️** | ~810 |

문8 → 문9 간격 499px (다른 문제 간격 ~250px 의 2배 처짐).

### 2.2 RHWP_DEBUG_TAC_CURSOR 추적

```
Shape pi=69 ci=0 y_in=709.4 y_out=983.4 dy=274.0 ⚠️
... pi=70~72 (빈 줄) ...
FullPara pi=73 y_in=1043.5 ... (문9 line)
```

→ Shape pi=69 ci=0 (non-TAC TopAndBottom picture) 가 column 에 274px reservation.

### 2.3 Picture 위치 분석

pi=69 picture (dump):
- 크기: 16786×20400 HU (59.2×72.0mm, **height 272px**)
- horz_rel_to = Column (단), 가로 offset 79.5mm (300px), 정렬 Center
- vert_rel_to = Para (문단), 세로 offset 0.5mm (150 HU)
- wrap = 위아래 (TopAndBottom), tac=false

`compute_object_position` 계산:
```
x = col_area.x + (col_area.width - pic_width) / 2 + h_offset
  = 399 + (360 - 224) / 2 + 300
  = 767
```

- pic_emit_x = 767 (col_area right = 759.7 외부!)
- Picture 가 column 우측 outside 에 emit

### 2.4 한컴 PDF 정합

`pdf/3-11월_실전_통합_2022.pdf` page 1: 우측 단에 picture **표시 안 됨**. 한컴 viewer 는 column flow 에 reservation 하지 않음.

## 3. Fix

`src/renderer/layout.rs:3500-3556`:

```rust
let saved_y_offset = y_offset;
result_y = self.layout_body_picture(...);
// [Task #959] horz_rel_to=Column 의 picture 가 col_area 우측을 초과하는 위치에
// emit 되면 한컴 viewer 는 column flow 에 reservation 하지 않음.
if matches!(pic.common.horz_rel_to, HorzRelTo::Column) {
    let pic_width_px = hwpunit_to_px(pic.common.width as i32, self.dpi);
    let h_offset_px = hwpunit_to_px(pic.common.horizontal_offset as i32, self.dpi);
    let pic_emit_x = match pic.common.horz_align {
        HorzAlign::Left | HorzAlign::Inside => col_area.x + h_offset_px,
        HorzAlign::Center => col_area.x + (col_area.width - pic_width_px) / 2.0 + h_offset_px,
        HorzAlign::Right | HorzAlign::Outside => col_area.x + col_area.width - pic_width_px - h_offset_px,
    };
    if pic_emit_x >= col_area.x + col_area.width {
        result_y = saved_y_offset;
    }
}
```

+ `RHWP_DEBUG_TAC_CURSOR` 환경변수 영구화 (paragraph item 별 y_offset 추적 도구).

## 4. 검증

### 4.1 cargo test
- `cargo test --release --lib`: **1288 passed, 0 failed, 2 ignored**
- golden SVG diff: 0 regression

### 4.2 단위 검증 (시험지 page 1)
- pi=69 ci=0 dy: 274 → 18 (line advance만) ✓
- pi=73 (문9) y_in: 1043 → 787 ✓
- 문9 시각 위치: ~y 805 (한컴 ~y 810 정합) ✓

### 4.3 회귀 검증
- 시험지 4종 (3-09월 2022/2023, 3-10월 2022, 3-11월 2022): 정상
- exam_kor page 18 (Square wrap picture, Task #722 영역): 정상
- exam_math/eng/hwp3-sample11: 정상
- column 내부 picture: 영향 없음 (Fix 조건 미진입)

## 5. 영향 평가

| 영역 | 영향 |
|------|------|
| horz_rel_to=Column + col 외부 emit picture | result_y advance 제거 (회귀 fix) |
| horz_rel_to=Column + col 내부 emit picture | 영향 없음 (Fix 조건 미진입) |
| horz_rel_to=Paper/Page picture | 영향 없음 (is_paper_based 분기) |
| horz_rel_to=Para picture | 영향 없음 (Column 검사 false) |
| TAC picture | 영향 없음 (별도 분기) |

## 6. 관련 작업

- 원 issue #952 + PR #956 — Issue 1 (페이지 외곽선) 해결
- PR #958 — Issue 2 (sample16 page 18 본문 누락) 해결
- 본 PR — Issue 3 (시험지 문9 vertical) 해결
- 원 #952 의 3 issue 모두 해결 → close 가능

## 7. 후속

- 원 issue #952 close (3 issue 모두 해결)
- 작업지시자가 PR 머지 + #952 close
- 신규 issue [#960](https://github.com/edwardkim/rhwp/issues/960) — 시험지 page 2 문14 multi-line equation (cases formula) off-by-one line 매핑 overlap. **Fix C 와 무관 pre-existing 결함**. 본 task 의 page 2 시각 검증 중 발견 (TAC_CURSOR 추적: cases 가 ls[2] 에, h(x)=lim 가 ls[0] 에 잘못 emit). 별도 task 분리.
