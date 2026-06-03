# Task #546 구현 계획서 — exam_science.hwp 페이지네이션 회귀 정정

## 1. 결함 요약 (Stage 1 결과)

| 항목 | 값 |
|------|-----|
| 회귀 origin | `82e41ba` (Task #460 보완5: Square wrap 그림 아래 텍스트 y위치 보정 — layout + typeset) |
| 변경 영역 | `src/renderer/layout.rs` (+58) + `src/renderer/typeset.rs` (+36) — 총 +94 LOC |
| 영향 파일 | exam_science.hwp 페이지 2 (총 페이지 4 → 6, p2 본문 37 → 2 items) |
| 영향 paragraph | pi=21 (Square wrap 그림 39.7×36.1mm) 이후 paragraph 들의 페이지 강제 분리 |

## 2. 결함 본질 분석

### 2.1 exam_science.hwp 의 Square wrap 영역

section 0 에 Square wrap (어울림) 그림 3개:

| pi | 그림 크기 | 페이지 | 비고 |
|----|---------|--------|------|
| 21 | 39.7×36.1mm | 1 | **첫 Square wrap, 회귀 직접 원인 후보** |
| 37 | 46.9×32.6mm | (2) | 회귀 후 영향 |
| 60 | 38.0×40.9mm | (3-4) | 회귀 후 영향 |

3 그림 모두 IR 속성: `vert=Para(0)`, `horz=Column(0)`, `tac=false`, `wrap=Square (어울림)`.

### 2.2 `82e41ba` 의 typeset.rs 변경 (회귀 origin)

```rust
// typeset.rs:644-679 (변경분)
let body_y = if matches!(cm.vert_rel_to, VertRelTo::Para) {
    st.current_height + v_off_px       // ← Para-relative
} else {
    let body_top_px = st.layout.body_area.y;
    (v_off_px - body_top_px).max(0.0)   // ← Paper/Page-relative
};
st.wrap_around_pic_bottom_px = body_y + pic_h_px;

// ... wrap zone 종료 시 (typeset.rs:506-510):
if st.wrap_around_pic_bottom_px > 0.0 {
    st.current_height = st.current_height.max(st.wrap_around_pic_bottom_px);
    st.wrap_around_pic_bottom_px = 0.0;
}
```

### 2.3 결함 가설

exam_science p1 의 Square wrap 그림 (pi=21) 의 `wrap_around_pic_bottom_px` 계산:

- pi=21 의 `current_height` ≈ 페이지 1 내 누적 px (그림 시작 시점)
- `body_y = current_height + v_off_px` = current_height (v_off_px=0)
- `wrap_around_pic_bottom_px = body_y + pic_h_px` = current_height + 36.1mm 의 px

wrap zone 종료 시 `current_height = max(current_height, wrap_around_pic_bottom_px)` 로 보정:

- 그림 자체가 페이지 1 끝 가까이 위치하면 보정값이 페이지 1 의 col_bottom (1215.1 px) 을 초과
- typeset 단의 `current_height` 가 페이지 경계 처리 없이 단조 증가 → 후속 paragraph 가 모두 새 페이지로 강제 이동

**즉**: `wrap_around_pic_bottom_px` 의 계산이 페이지 경계 (column 끝) 를 인식 못 하여, 그림이 다음 페이지 영역까지 "확장" 된 것처럼 처리됨. 그림 자체는 단일 페이지 내에 표시되지만 `current_height` 보정으로 후속 paragraph 들이 강제로 다음 페이지로 밀림.

### 2.4 layout.rs 변경 (현재 영향 0)

```rust
// layout.rs:1389-1393
wrap_pic_bottom_y = if matches!(cm.vert_rel_to, crate::model::shape::VertRelTo::Para) {
    0.0  // ← Para-relative: 미적용
} else {
    v_off_px + pic_h_px
};
```

→ Para-relative 일 때 `wrap_pic_bottom_y = 0.0` 이라 layout.rs 의 `y_offset` 보정은 적용 안 됨. **문제는 typeset.rs 만**.

## 3. 정정 정책 옵션

### 옵션 A — 단일 commit revert (`82e41ba` 전체)

**처리**: `git revert 82e41ba` 또는 동일 영역 코드를 직접 제거.

**장점**:
- 간단. 회귀 0 보장
- 변경 범위 작음 (94 LOC 제거)

**단점**:
- Task #460 보완5 의 의도 (HWP3 의 Square wrap 그림 아래 텍스트 y위치 정합) 손실
- 작업지시자가 HWP3 fixture 검증한 정합 결과 회귀 가능 → 다른 fixture (HWP3 Square wrap) 의 시각 결함 재발 가능

### 옵션 B — 부분 revert (typeset.rs 만)

**처리**: typeset.rs 의 +36 LOC 만 제거 (`wrap_around_pic_bottom_px` 영역). layout.rs 의 +58 LOC 는 보존 (현재 영향 0 이라 무해).

**장점**:
- exam_science 회귀 해소 (typeset.rs 가 회귀 직접 origin)
- layout.rs 변경 보존 (Paper/Page-relative 의 의도된 보정 유지, 다만 currently 사용 안 됨)

**단점**:
- Task #460 보완5 의 의도 (typeset 단 Square wrap 그림 하단 보정) 손실
- HWP3 fixture 의 Square wrap 그림 아래 텍스트 y위치 정합이 다시 결함 가능

### 옵션 C — 페이지 경계 인식 추가 (정합 정정)

**처리**: typeset.rs 의 wrap zone 종료 시 `wrap_around_pic_bottom_px` 보정에 **페이지 경계 (col_bottom) 검사** 추가:

```rust
if st.wrap_around_pic_bottom_px > 0.0 {
    let col_bottom = st.layout.body_area.y + st.layout.body_area.h;
    let bounded = st.wrap_around_pic_bottom_px.min(col_bottom);
    st.current_height = st.current_height.max(bounded);
    st.wrap_around_pic_bottom_px = 0.0;
}
```

또는 그림이 현재 페이지 영역을 벗어나면 보정 자체를 skip:

```rust
if st.wrap_around_pic_bottom_px > 0.0 && st.wrap_around_pic_bottom_px <= col_bottom {
    st.current_height = st.current_height.max(st.wrap_around_pic_bottom_px);
}
st.wrap_around_pic_bottom_px = 0.0;
```

**장점**:
- exam_science 회귀 해소 + Task #460 보완5 의 의도 보존
- 페이지 내 Square wrap 그림 정합 + 페이지 경계 안전성 확보

**단점**:
- 구현 복잡도 증가 (페이지 경계 정밀 검사)
- 광범위 회귀 검증 필수 (다른 fixture 의 Square wrap 영역 영향 점검)

### 옵션 D — exam_science 전용 분기 (case-specific)

**처리**: exam_science.hwp 만의 본질 식별 후 case-specific 가드 추가.

**장점**: 다른 fixture 영향 0

**단점**: case-specific 정정의 일반성 부족 (메모리 `feedback_hancom_compat_specific_over_general` 도 있지만 본 case 는 정합한 일반화 가능)

## 4. 권장 정정 정책

**옵션 C (페이지 경계 인식 추가)** 권장.

**근거**:
1. Task #460 보완5 의 의도 (HWP3 Square wrap 그림 아래 텍스트 y위치 정합) 보존
2. exam_science 의 페이지 경계 부작용 정밀 정정
3. 다른 fixture 의 Square wrap 처리도 더 안전 (페이지 경계 안전성 확보)
4. 코드 변경 범위 작음 (typeset.rs 의 wrap zone 종료 영역에 페이지 경계 검사 추가, ~5-10 LOC)

대안 — 옵션 C 가 광범위 회귀 발생 시 옵션 A (전체 revert) 로 fallback.

## 5. 변경 영역 (옵션 C 적용 시)

| 파일 | 영역 | 변경 |
|------|------|------|
| `src/renderer/typeset.rs` | wrap zone 종료 처리 (line 506-510) | 페이지 경계 검사 추가 (~5-10 LOC) |
| `tests/issue_546.rs` (신규) | 회귀 테스트 | exam_science.hwp p2 의 페이지 수 (4) + 단 0 items (37) 검증 |

## 6. 단계 분리

### Stage 3 — 정정 적용 (옵션 C)

1. typeset.rs 의 wrap zone 종료 영역에 페이지 경계 검사 추가
2. 직접 검증: `rhwp dump-pages samples/exam_science.hwp -p 1` → 4 페이지 + p2 단 0 items=37 / used=1133.6 px 복원
3. 단위 테스트 추가 (TDD): `tests/issue_546.rs::issue_546_exam_science_p2_pagination`

**산출물**: `mydocs/working/task_m100_546_stage3.md`

### Stage 4 — 회귀 검증

1. `cargo test --lib` (1113+ 통과)
2. `cargo test --test issue_546` (신규 1건 통과)
3. `cargo test --test issue_530/505/418/501` (회귀 0)
4. `cargo test --test svg_snapshot` (6/6 통과)
5. `cargo clippy --lib -- -D warnings` (0 건)
6. **광범위 fixture sweep** — Square wrap 사용 fixture 들의 페이지 수 정합 점검:
   - exam_kor / exam_eng / exam_math / exam_science (PR #506 + Task #460 의 정합 fixture)
   - synam-001 / 복학원서 / 2010-01-06 (회귀 가능성 점검)

**산출물**: `mydocs/working/task_m100_546_stage4.md`

### Stage 5 — 시각 판정 + 최종 보고

1. WASM 빌드 + studio 동기화
2. 작업지시자 시각 판정 (rhwp 자체 해석 권위):
   - **exam_science.hwp p2 정상 출력** (4 페이지 / 본문 37 items)
   - 한컴 2010/2020 의 동일 fixture 와 비교
   - HWP3 Square wrap fixture (Task #460 의 정합 결과) 회귀 0 확인
3. 시각 판정 통과 후:
   - 최종 보고서 (`mydocs/report/task_m100_546_report.md`)
   - orders 갱신
   - local/task546 → local/devel merge → devel push → 이슈 close

**산출물**: `mydocs/report/task_m100_546_report.md`

## 7. 회귀 테스트 (`tests/issue_546.rs`)

```rust
//! Issue #546: exam_science.hwp 2페이지 페이지네이션 회귀 (PR #506 origin)
//!
//! 본질: typeset.rs 의 wrap_around_pic_bottom_px 보정이 페이지 경계 미인식 →
//! Square wrap 그림이 페이지 끝 가까이 있을 때 후속 paragraph 가 강제 페이지 분리.

use std::fs;
use std::path::Path;

#[test]
fn issue_546_exam_science_p2_pagination() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/exam_science.hwp");
    let bytes = fs::read(&hwp_path).expect("read exam_science.hwp");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse");

    // v0.7.9 기준: 4 페이지 + p2 본문 37 items
    assert_eq!(doc.page_count(), 4, "exam_science.hwp 는 4 페이지여야 함 (회귀 시 6)");
    
    // p2 의 본문 paragraph 수 (정확한 검증 메트릭은 Stage 3 에서 확정)
    // ...
}
```

## 8. 위험 영역

| 위험 | 가능성 | 회피책 |
|------|--------|--------|
| 옵션 C 의 페이지 경계 검사가 다른 fixture 회귀 야기 | 🟧 중간 | Stage 4 광범위 fixture sweep + svg_snapshot 6/6 |
| Task #460 의 HWP3 fixture 정합 손실 (옵션 C 적용 후 HWP3 Square wrap 그림 아래 텍스트 다시 결함) | 🟧 중간 | Stage 5 시각 판정 시 HWP3 Square wrap fixture 직접 점검 |
| `wrap_around_pic_bottom_px` 의 `body_y` 계산 본질 자체가 결함일 가능성 (current_height 가 누적 절대 px 이라 페이지 경계 무시) | 🟧 중간 | Stage 3 정정 후 검증으로 식별 |

## 9. 메모리 정합

- `feedback_image_renderer_paths_separate` — typeset.rs 의 정정이 layout 단계 (renderer-신경 안 씀) 의 본질, renderer 별 분기 영향 없음
- `feedback_hancom_compat_specific_over_general` — case-specific 옵션 C (페이지 경계 검사) 가 일반화 정정 보다 안전
- `feedback_visual_regression_grows` — Stage 5 시각 판정 게이트 필수
- `feedback_pdf_not_authoritative` / `reference_authoritative_hancom` — 한컴 2010/2020 직접 판정으로 정답지 확정

## 10. 다음 단계

작업지시자 본 구현 계획서 승인 후 **Stage 3** (옵션 C 정정 적용) 진행. 옵션 C 광범위 회귀 발생 시 옵션 A (전체 revert) fallback.
