# Task #534 v2 보강 Stage — LINE_SEG.column_start 정합

**작성일**: 2026-05-02
**이슈**: [#534](https://github.com/edwardkim/rhwp/issues/534) (재오픈)
**브랜치**: `local/task534_v2`
**선행**: v1 fix (commit `4abee04`, inner_pad 정합) — PDF 와 시각 결함 잔존

## 1. 결론

> v1 fix (inner_pad) 후 PDF 시각 비교로 **picture 가 [A]/[B] 박스 위에 겹쳐 표시되는 결함 잔존** 발견. HWP IR `LINE_SEG.column_start` 가 Square wrap 인라인 표 영역 + outer_margin 이후 텍스트 시작 위치를 인코딩 (cs=2855 HU = 38.07 px for pi=50). layout_shape_item 가 cs 미반영 → picture 가 표 영역 위에 겹침. cs 가 effective_margin_left 보다 크면 cs 우선 (max 패턴) 으로 정정. exam_kor p18 pi=50/56 image x: 604.72 → **620.12** ✓ PDF 시각 정합.

## 2. PDF vs SVG 시각 비교 (v1 fix 후)

PDF 시각: [A] 박스 → 우측에 가로로 긴 picture (가로 일렬 배치).

v1 fix 후 SVG: picture (width 368 px) 가 column 시작 영역 (604.72) 부터 시작 → [A] 박스 (609.04) 와 같은 영역 → 시각적으로 [A] 박스가 picture 위에 겹쳐 표시.

## 3. 본질 정확 식별

### 3-1. HWP IR LINE_SEG 측정

```bash
target/release/rhwp dump samples/exam_kor.hwp -s 2 -p 50
```

| pi | LINE_SEG | column_start (cs) | segment_width (sw) |
|----|----------|------------------|---------------------|
| 46 (단독 그림) | ls[0] | 850 HU = 11.33 px | 30044 HU = 400.59 px |
| **50 (Square 표 + 그림)** | ls[0] | **2855 HU = 38.07 px** | 28039 HU = 373.85 px |
| **56 (Square 표 + 그림)** | ls[0] | **2855 HU = 38.07 px** | 28039 HU = 373.85 px |

→ HWP IR 가 정확한 column_start 를 인코딩. pi=50/56 의 cs=2855 HU 는 표 영역 (23 px) + outer_margin_right (11.33 px) + 추가 margin 합산 위치. 이 값을 사용하면 picture 가 표 옆 영역에 정확히 위치.

### 3-2. 기대 picture x

- pi=50: col_left (582.05) + cs (38.07) = **620.12 px** ✓ (PDF 정합)
- pi=46/54 (단독 그림): cs=11.33 → effective_margin_left=72.00 (inner_pad+indent 가 더 큼) → max() 로 기존 위치 유지

## 4. 변경

### 4-1. 코드 (`src/renderer/layout.rs::layout_shape_item`)

```rust
// [Task #534 v2] LINE_SEG.column_start 는 Square wrap 인라인 표/그림이
// 좌측에 floating 시 표 영역 이후 텍스트 시작 위치를 HWP IR 가 인코딩.
// cs 가 effective_margin_left 보다 크면 cs 우선.
let line_seg_cs_px = para.line_segs.first()
    .map(|s| hwpunit_to_px(s.column_start, self.dpi))
    .unwrap_or(0.0);
if line_seg_cs_px > effective_margin_left {
    effective_margin_left = line_seg_cs_px;
}
```

### 4-2. 변경량

| 영역 | 추가 | 삭제 | 수정 |
|------|------|------|------|
| `src/renderer/layout.rs` | 12 | 0 | 1 (let → let mut) |

## 5. 검증

### 5-1. 시각 검증

| 항목 | v1 fix | v2 fix |
|------|--------|--------|
| pi=50 picture x | 604.72 | **620.12** ✓ |
| pi=56 picture x | 604.72 | **620.12** ✓ |
| pi=46/54 picture x | 654.05 | 654.05 (변경 없음, max 로 cs 우선 안 됨) |
| PDF 시각 정합 | ✗ ([A] 위 겹침) | ✓ ([A] 우측 정상 배치) |

### 5-2. 단위/통합 테스트

| 게이트 | 결과 |
|--------|------|
| `cargo test --lib` | **1116 passed** |
| `cargo test --test svg_snapshot` | 6/6 |
| `cargo test --test issue_418` | 1/1 |
| `cargo test --test issue_501` | 1/1 |

### 5-3. 광범위 회귀 (v1 fix vs v2 fix)

```bash
scripts/svg_regression_diff.sh build 4b8e75de 9dfc56ac ...
```

| 샘플 | total | same | diff |
|------|-------|------|------|
| 2010-01-06 / aift / exam_eng / exam_math / exam_math_no / exam_science / synam-001 | 172 | 172 | 0 |
| **exam_kor** | 20 | 18 | **2** (p18, p19) |
| **합계** | **192** | **190** | **2** |

→ 변경 2 페이지 모두 동일 본질 (Square wrap 표 + TAC 그림) 의 추가 정정.

## 6. 회귀 차단 가드

| 가드 | 보호 영역 |
|------|----------|
| `cs > effective_margin_left` (max 패턴) | 단독 그림 (cs <= margin) 영역 변경 없음 |
| HWP IR cs 신뢰 | parser 인코딩값 직접 사용 — 휴리스틱 없음 |
| `tac=true` 분기 내부 | non-TAC Picture 영역 분리 |

## 7. 본질 학습

### 7-1. HWP IR LINE_SEG.column_start 룰 정합

`paragraph_layout.rs::layout_composed_paragraph` (line 874-882) 의 `[Task #489] effective_col_x` 도 `LINE_SEG.column_start` 를 사용하지만, 조건이 `has_picture_shape_square_wrap` (Picture/Shape with Square wrap) 으로 한정 → Square wrap **표** 케이스 미커버. 본 fix 는 layout_shape_item 의 TAC Picture 영역에 cs 사용 패턴 도입.

### 7-2. Stage 1 가설 다중 영역 학습

v1 fix 작성 시 Stage 1 추가 조사 (eprintln) 로 emit code path 식별 → inner_pad 누락 본질 확정. 그러나 그 이후 PDF 시각 비교 누락으로 잔존 결함 미인식. **PDF 시각 검증을 작업지시자 시각 판정 외에도 Stage 4 회귀 직후 자동화 영역으로 도입 권고**.

### 7-3. 메모리 정합

- `feedback_pdf_not_authoritative` — PDF 는 보조 ref. 그러나 본 케이스처럼 좌표/위치 결함은 PDF 시각 비교가 필수. 작업지시자 시각 판정 = 최종 ref.
- `feedback_essential_fix_regression_risk` — v1 fix 광범위 회귀 0 이었으나 본 결함 정정에 부족 → 본질 정확 식별의 중요성. cs 룰 적용으로 단일 룰 정정 (분기 없음).
- `feedback_rule_not_heuristic` — HWP IR LINE_SEG.column_start 는 한컴 인코딩 룰. 직접 사용으로 정확.

## 8. 다음 단계

작업지시자 시각 판정 통과 시 close 흐름:
- local/task534_v2 → local/devel merge
- local/devel → devel merge + push
- issue #534 close

## 9. 승인 게이트

- [x] PDF 시각 비교로 v1 fix 잔존 결함 식별
- [x] HWP IR LINE_SEG.column_start 룰 정확 식별
- [x] cs 기반 max() 정정 (+12 라인)
- [x] 단위 1116 + 광범위 회귀 192 페이지 / 2 변경 (의도)
- [x] 시각 정합 확인 (PDF 와 동일 [A] 우측 배치)
