# Task #962 — 최종 보고서

- 이슈: [#962](https://github.com/edwardkim/rhwp/issues/962)
- 마일스톤: M100 / v1.0.0
- 브랜치: `local/task962`
- 기간: 2026-05-17 (1일)

## 1. 작업 범위

원 issue #952 의 5번째 결함 — page 2 문14 <보기> textbox (TAC InFrontOfText 사각형 + 내부 글상자) 내부 inline 수식 중복 emit 으로 인한 시각 scramble 해결.

## 2. Root cause

### 2.1 증상

`samples/3-11월_실전_통합_2022.hwp` page 2 문14 <보기> textbox 내부 content scramble:
- 한컴: ㄱ. h(1)=3, ㄴ. 함수 h(x)는..., ㄷ. 함수 g(x)가...
- rhwp: 수식이 textbox 좌측 edge + gap 위치에 **각각 emit** → 시각 overlap

### 2.2 데이터

pi=118 의 [사각형] (InFrontOfText TAC) 내부 글상자에 3 paragraphs + 6 inline equations.

SVG 실측:
- 보기 box 영역 (y 440-540, x 400-760) 에 **12 equations** (= 6 expected × 2 duplicates)
- Set 1 (gap 위치, 정상): paragraph_layout 가 inline TAC 처리 시 emit
- Set 2 (textbox 좌측 edge x=406, 누적): shape_layout 두번째 loop 가 중복 emit

### 2.3 Code path

**Path 1 (정상)**: `shape_layout.rs:1440` 의 `layout_composed_paragraph` 호출 → paragraph_layout 의 inline TAC (`paragraph_layout.rs:2078+`) 이 gap 위치에 equation emit.

**Path 2 (duplicate)**: `shape_layout.rs:1609-1644` (두번째 loop) 의 Equation branch 가 같은 paragraph 의 controls 를 다시 iterate, equation 을 `inline_x` 에 emit.

→ 두 paths 가 같은 equation 을 emit 하여 시각 duplicate.

## 3. Fix

`src/renderer/layout/shape_layout.rs:1609-1675`:

```rust
Control::Equation(eq) => {
    let eq_w = hwpunit_to_px(eq.common.width as i32, self.dpi);
    let eq_h = hwpunit_to_px(eq.common.height as i32, self.dpi);
    // [Task #962] 글상자 내부 paragraph 의 inline equation 은 paragraph_layout 가
    // layout_composed_paragraph 경로에서 정확한 gap 위치 (text 사이) 에 emit 한다.
    // 본 두번째 loop 는 paragraph_layout 미지원 이전의 legacy fallback.
    // paragraph_layout 가 이미 inline_shape_position 으로 등록한 경우 중복 emit 차단.
    let equiv_cell_ctx = CellContext {
        parent_para_index: para_index,
        path: { /* parent_cell_path + textbox entry */ },
    };
    if tree.get_inline_shape_position(
        section_index, pi, ctrl_idx_in_para, Some(&equiv_cell_ctx)
    ).is_some() {
        // paragraph_layout 가 이미 emit — inline_x 만 advance
        inline_x += eq_w;
    } else {
        // legacy fallback
        // ... 기존 emit 분기 유지
    }
}
```

## 4. 검증

### 4.1 cargo test
- `cargo test --release --lib`: **1288 passed, 0 failed, 2 ignored**

### 4.2 단위 검증 (시험지 page 2)
- 보기 textbox equations: 12 → 6 ✓ (duplicates 제거)
- 시각: ㄱ. h(1)=3, ㄴ. 함수 h(x)는..., ㄷ. 함수 g(x)가... ✓ 한컴 PDF 정합
- 문12, 문14, 문15 의 textbox content 모두 정상

### 4.3 회귀 검증
- LAYOUT_OVERFLOW count: **325 → 325 (회귀 0)**
- 시험지, exam_kor/math/eng, sample14: 시각 회귀 0

## 5. 영향 평가

| 영역 | 영향 |
|------|------|
| textbox 내부 inline Equation (paragraph_layout 등록 case) | duplicate 제거 (회귀 fix) |
| textbox 내부 inline Equation (legacy fallback, paragraph_layout 미emit) | 영향 없음 (else 분기 유지) |
| textbox 내부 Shape/Picture/Table | 본 fix 미대상 (Stage 4 검증 시 미발견) |
| textbox 외부 standalone equation | 영향 없음 (별도 path) |

## 6. 관련 작업

- 원 issue #952 + PR #956 (Issue 1 외곽선)
- PR #958 (Issue 2 sample16 page 18)
- PR #961 (Issue 3 시험지 page 1 문9 vertical)
- PR #963 (Issue 4 cases formula off-by-one)
- 본 PR (Issue 5 보기 textbox duplicate equation)

## 7. 후속

- 원 issue #952 close (5 issue 모두 해결)
- 작업지시자가 5 PR 머지

## 8. 평가

본 task 가 archive/task936 의 9 시도 + 5 revert 패턴 우려 가장 큰 영역 (textbox + inline 수식 + InFrontOfText) 이었으나, **회귀 0 으로 안전 fix 완료**. 명확한 root cause 식별 + 최소 침습 fix (단일 분기 조건 추가) 가 효과적.
