# Task #524 Stage 2-A — Square wrap 그림 anchor 위치 정정

## 1. 정정 내용

### 1.1 진단 정정 — 실제 코드 경로 확인

Stage 1 진단에서 의심 코드로 지목한 `src/renderer/pagination/engine.rs:1083` (`Control::Picture` 분기 height 가산)는 **dead code**였음. 사유:

- `src/document_core/queries/rendering.rs:890` 에 `RHWP_USE_PAGINATOR` env 가드.
- 기본 `false` → 실제 페이지네이션은 `TypesetEngine` 이 담당 (`src/renderer/typeset.rs`).
- engine.rs 의 `paginate_with_measured_opts` 는 fallback 경로.

또한 본 문서의 그림 (pi=21 ci=0 등) 은 `Control::Shape(ShapeObject::Picture(...))` 형태로 파싱됨. `Control::Picture` 는 hwp5 파서에서 생성되지 않으며, html_table_import 에서만 사용.

### 1.2 실제 버그 위치

`src/renderer/typeset.rs:670-682` — 비-TAC Square wrap 그림의 `wrap_around_pic_bottom_px` 산출.

**원인**:
1. `typeset_section` 루프에서 `typeset_paragraph` 호출 (line 603) → 본 문단의 `height_for_fit` 가 `current_height` 에 가산.
2. 그 직후 (line 638-685) 비-TAC Square wrap 그림 감지 분기에서 `body_y = st.current_height + v_off_px` 산출.
3. 이 시점 `current_height` 는 본 문단 BOTTOM 위치 (텍스트 전체 가산 후).
4. 그러나 그림 anchor (`vert_align=Top`, `vert_offset=0`, `vert_rel_to=Para`) 는 본 문단 TOP 에 정렬되어야 함.
5. `wrap_around_pic_bottom_px = (문단_bottom) + pic_h` 으로 잘못 산출.
6. 다음 문단 진입 시 wrap_zone 종료 보정 (line 510-513): `current_height = max(current_height, wrap_around_pic_bottom_px)` → 그림 높이만큼 추가 inflation.

### 1.3 정정

```rust
// typeset_section 루프 내 typeset_paragraph 호출 직전:
let para_top_y_for_pic_anchor = st.current_height;
if !has_table {
    let formatted = self.format_paragraph(...);
    self.typeset_paragraph(...);
} else { ... }

// ... wrap_around_pic_bottom_px 산출부:
let body_y = if matches!(cm.vert_rel_to, VertRelTo::Para) {
    para_top_y_for_pic_anchor + v_off_px   // ← 본 문단 TOP 사용
} else { ... };
```

부수: `src/renderer/pagination/engine.rs:1083` (fallback 경로) 도 동일 시멘틱으로 정합 — Square wrap 시 lines_height 차감, TopAndBottom 시 전체 가산.

## 2. 측정 결과

### 2.1 exam_science.hwp

**페이지 수**: 6 → **4** (-2 페이지)

| 페이지 | 변경 전 단 0 | 변경 전 단 1 | 변경 후 단 0 | 변경 후 단 1 |
|--------|--------------|--------------|--------------|--------------|
| 1 | 31 items, 1122.6 px | 46 items, 1112.6 px | 31 items, 1122.6 px | **48 items, 1108.9 px** (보기/답지 안착) |
| 2 | 2 items, 132.7 px | (없음) | **37 items, 1133.6 px** | **45 items, 1119.0 px** |
| 3 | 35 items, 1121.4 px | 2 items, 135.4 px | 25 items, 1064.4 px | 35 items, 1031.1 px |
| 4 | 44 items, 1193.8 px | 1 items, 21.5 px | 40 items, 1046.7 px | 32 items, 922.7 px |
| 5 | 25 items, 1064.4 px | 35 items, 1031.1 px | (없음) | (없음) |
| 6 | 40 items, 1046.7 px | 32 items, 922.7 px | (없음) | (없음) |

문제 8 위치 검증: 페이지 2 단 0 의 `pi=37` "8.-그림은 수소와 원소 로 구성된 분자 (가)(라)의 공유 전자쌍 수와…" — **PDF 정합 (페이지 2 좌측)**.

### 2.2 회귀 검증 (svg_regression_diff.sh)

```
2010-01-06: total=6  same=6  diff=0
aift:       total=77 same=77 diff=0
exam_eng:   total=8  same=8  diff=0
exam_kor:   total=20 same=20 diff=0
exam_math:  total=20 same=20 diff=0
exam_science: total=6 same=0 diff=6  (의도된 정정 — 페이지 수 변화 6→4)
synam-001:  total=35 same=35 diff=0
TOTAL: pages=172 same=166 diff=6
```

**6/7 샘플 byte-identical**. exam_science 의 6 페이지 diff 는 페이지 수 변화에 의한 의도된 정정 (4 페이지로 packing 개선).

### 2.3 단위 테스트

```
test result: ok. 1111 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
```

## 3. 시각 검증 자료

`output/svg/exam_science_001.svg` ~ `exam_science_004.svg` 생성. 페이지 2 (`exam_science_002.svg`) 좌측 단에 문제 8 stem 표시 확인 가능.

**작업지시자 시각 판정 요청 항목**:
- 페이지 1 우측 단 끝 (보기/답지 안착) 정합
- 페이지 2 단 0 (문제 7 + 문제 8) PDF 정합
- 페이지 2 단 1 (문제 10 시작) PDF 정합
- 페이지 3, 4 PDF 정합 (페이지 수 4페이지로 일치)

## 4. 위험·회귀

- ✅ exam_science 외 6 샘플 (172 페이지 - 6 = 166 페이지) byte-identical.
- ✅ 1111 단위 테스트 0 회귀.
- ⚠️ exam_science 페이지 수 6→4: PDF 정합 (작업지시자 시각 판정 필요).
- 적용 조건: 비-TAC + Square wrap + VertRelTo::Para + Picture/Shape::Picture. 이 조건의 다른 샘플은 회귀 검증 통과 (exam_eng, aift 등에 동일 패턴 사용 가능성 있으나 영향 0).

## 5. 다음 단계

Stage 2-B (회귀 검증) 사실상 본 단계와 동시 수행 완료. 다음:

- Stage 2-C: 작업지시자 시각 판정.
- Stage 2-D: 최종 보고서 + merge + close.

승인 요청: 작업지시자 시각 판정 요청. 예상 결과 정합 시 Stage 2-D 진입.
