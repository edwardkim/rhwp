# Task #524 최종 보고서 — exam_science p2 우측 단 공백 + 쪽나누기 컬럼 인식

## 결론

**해결**. 비-TAC Square wrap 그림(`Picture` / `Shape::Picture`)의 `wrap_around_pic_bottom_px` 산출 시 anchor 기준점이 본 문단 BOTTOM 으로 잘못 사용되어 후속 문단 wrap_zone 종료 시 그림 높이만큼 페이지네이션 누적이 inflation 되던 결함 정정. exam_science.hwp 페이지 수 6 → 4, 문제 8 위치 페이지 3 → 페이지 2 (PDF 정합).

가설 A (`[쪽나누기]` 컬럼 미인식) 는 별도 수정 불필요 — Stage 1 진단대로 가설 B 정정으로 자연 해소.

## 본질

### 진단 정정 (Stage 1 가설 vs 실제 코드 경로)

Stage 1 진단의 의심 코드 (`src/renderer/pagination/engine.rs:1083` `Control::Picture` 분기)는 **dead code**.

- `src/document_core/queries/rendering.rs:890`: `RHWP_USE_PAGINATOR` env 가드. 기본 `false`.
- 실제 페이지네이션 = `TypesetEngine` (`src/renderer/typeset.rs`). `engine.rs::Paginator` 는 fallback.
- 본 문서의 그림은 `Control::Shape(ShapeObject::Picture(...))` 형태. `Control::Picture` 는 hwp5 파서에서 생성되지 않음.

### 실제 버그

`src/renderer/typeset.rs:670-682` — 비-TAC Square wrap 그림 감지 시 `wrap_around_pic_bottom_px` 산출.

```rust
// (기존)
let body_y = if matches!(cm.vert_rel_to, VertRelTo::Para) {
    st.current_height + v_off_px   // ← typeset_paragraph 호출 후 = 문단 BOTTOM
} else { ... };
st.wrap_around_pic_bottom_px = body_y + pic_h_px;
```

순서:
1. `typeset_paragraph` 호출 → 본 문단 `height_for_fit` 가산 → `current_height` = 문단 BOTTOM.
2. 직후 Square wrap 그림 감지 분기에서 `body_y = current_height` 사용.
3. 그림 anchor (`vert_align=Top`, `vert_offset=0`, `vert_rel_to=Para`) 는 본 문단 TOP 에 정렬되어야 함.
4. `wrap_around_pic_bottom_px = (문단 BOTTOM) + pic_h` → 본래 (문단 TOP) + pic_h 보다 (문단 height) 만큼 큼.
5. 다음 문단 wrap_zone 종료 시 `current_height = max(current_height, wrap_around_pic_bottom_px)` → 그림 높이만큼 누적 inflation.

### 정정

```rust
// typeset_section 루프, typeset_paragraph 호출 직전:
let para_top_y_for_pic_anchor = st.current_height;
if !has_table {
    let formatted = self.format_paragraph(...);
    self.typeset_paragraph(...);
} else { ... }

// ... wrap_around_pic_bottom_px 산출부:
let body_y = if matches!(cm.vert_rel_to, VertRelTo::Para) {
    para_top_y_for_pic_anchor + v_off_px   // ← 문단 TOP 사용
} else { ... };
```

부수 정정: `src/renderer/pagination/engine.rs:1083` (fallback 경로) 도 동일 시멘틱으로 정합 — Square wrap 시 lines_height 차감, TopAndBottom 시 전체 가산.

## 검증

### 단위 테스트

```
cargo test --lib
test result: ok. 1111 passed; 0 failed; 1 ignored
```

### SVG 회귀 검증 (`scripts/svg_regression_diff.sh`)

```
2010-01-06: total=6  same=6  diff=0
aift:       total=77 same=77 diff=0
exam_eng:   total=8  same=8  diff=0
exam_kor:   total=20 same=20 diff=0
exam_math:  total=20 same=20 diff=0
exam_science: total=6 same=0 diff=6  (의도된 정정 — 페이지 수 6→4)
synam-001:  total=35 same=35 diff=0
TOTAL: pages=172 same=166 diff=6
```

6/7 샘플 byte-identical. exam_science 6 페이지 diff = 페이지 수 변화 (6→4) 에 의한 의도된 packing 개선.

### 시각 판정 (작업지시자)

- 페이지 1 우측 단 끝 (보기/답지 안착) ✅
- 페이지 2 단 0 (문제 7 + 문제 8) PDF 정합 ✅
- 페이지 2 단 1 (문제 10 시작) PDF 정합 ✅
- 페이지 3, 4 PDF 정합 ✅

## 측정값 (exam_science.hwp)

| 페이지 | 변경 전 단 0 | 변경 전 단 1 | 변경 후 단 0 | 변경 후 단 1 |
|--------|--------------|--------------|--------------|--------------|
| 1 | 31, 1122.6 px | 46, 1112.6 px | 31, 1122.6 px | **48, 1108.9 px** (보기/답지 안착) |
| 2 | 2, 132.7 px | (없음) | **37, 1133.6 px** | **45, 1119.0 px** |
| 3 | 35, 1121.4 px | 2, 135.4 px | 25, 1064.4 px | 35, 1031.1 px |
| 4 | 44, 1193.8 px | 1, 21.5 px | 40, 1046.7 px | 32, 922.7 px |
| 5 | 25, 1064.4 px | 35, 1031.1 px | (없음) | (없음) |
| 6 | 40, 1046.7 px | 32, 922.7 px | (없음) | (없음) |

## 코드 변경

| 파일 | 변경 |
|------|------|
| `src/renderer/typeset.rs` | `para_top_y_for_pic_anchor` 변수 도입 + Square wrap `body_y` 산출부 정정 |
| `src/renderer/pagination/engine.rs` | `Control::Picture` 분기 (fallback) Square/TopAndBottom 분기 정합 |

총 변경: +28 / -8 lines.

## 단계별 진행

| Stage | 내용 | 커밋 |
|-------|------|------|
| 0 | 이슈 #524, #525 등록 + 수행 계획서 + 브랜치 분기 | `0b2391b` |
| 1 | 진단 — 누적 +59 px inflation 확정, 가설 B 우선순위 | `7aaa050` |
| 2-impl | 구현 계획서 (4단계) | `beab660` |
| 2-A | Square wrap 그림 anchor 위치 정정 + 회귀 검증 | `00e152f`, `9c249e1` |
| 2-D | 최종 보고서 + merge + close | (본 커밋) |

## 잔여

- **#525**: exam_science 8번 문제 인라인 화학식 글자 겹침 (Shape advance 폭) — 별도 task. 본 task 와 독립적.
- 다른 페이지 단 1 의 `diff=-XX px` (예: p3 단 1 -132.7 px 등) — 알고리즘 used 가 vpos 기반 hwp_used 보다 작음. 본 회귀 게이트 통과 (다른 샘플 byte-identical) 이고 시각 판정 PDF 정합으로 별도 본질 추정. 추후 별도 조사 가치.
