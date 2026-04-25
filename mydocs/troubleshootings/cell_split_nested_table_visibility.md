# 셀 분할 시 중첩 표 (atomic) 가시성 결정

## 발견일

2026-04-25 — Task #324 (PR #327, 외부 기여자 [@planet6897](https://github.com/planet6897))

## 증상

`samples/hwpx/form-002.hwpx` 의 외부 표 row 19 에 있는 1×1 내부 표 ("연구개발계획서 제출시…") 가:

- **page 1 하단에 잘못 표출** (PDF 기준 page 2 상단에 와야 함)
- (수정 시도 1차 후) page 1 에서 제거됐으나 page 2 에서도 누락
- (수정 시도 2차 후) page 2 에 표출되지만 SVG 클립에 가려져 일부만 보임

## 근본 원인

### 잔량(remaining) 기반 추적의 한계

기존 `compute_cell_line_ranges` (in `src/renderer/layout/table_layout.rs`):

```rust
let mut offset_remaining = content_offset;
let mut limit_remaining = if content_limit > 0.0 { content_limit } else { f64::MAX };

for para in cell.paragraphs {
    if offset_remaining > 0.0 { /* 잔량 차감 */ }
    else if limit_remaining > 0.0 { /* 잔량 차감 */ }
    // limit 초과 시 무처리 — 가시성 결정은 별도 분기
}
```

**문제**: 잔량이 0 도달하면 cumulative position 정보가 손실되어 atomic nested table 의 정확한 페이지 분할 결정이 불가능. 특히 `has_table_in_para` 분기에서:

```rust
if offset_remaining > 0.0 || (offset_remaining == 0.0 && content_offset > 0.0 && para_h <= content_offset) {
    result.push((line_count, line_count));   // 스킵
} else {
    result.push((0, line_count));            // ❗ limit 초과해도 항상 첫 페이지에 표시
}
```

`limit_remaining` 초과 케이스가 가시성 결정에서 누락 → **셀 분할 시 limit 초과 후 등장하는 중첩 표 문단이 무조건 첫 페이지에 표시**.

### content_y_accum 갱신 누락

`layout_partial_table` (in `src/renderer/layout/table_partial.rs`) 에서 offset 으로 완전 소비된 일반 문단 (`line_ranges=(n,n)`) 이 스킵될 때 `content_y_accum` 미갱신 → 후속 nested table 위치 판정 부정확 → split-end 페이지에서 "이미 이전 페이지에 렌더링됨" 으로 잘못 스킵.

### split-start row visible height 계산 오류

`has_nested_table` 셀 분기가 `calc_nested_split_rows` 에 raw `split_start_content_offset` (cell 전체 기준) 을 그대로 전달 → inner table 의 cell 내 위치 무시 → cell-clip-6 height 가 작게 계산되어 inner table 일부 클립.

## 해결

### v1 — `compute_cell_line_ranges` 누적위치 기반 재작성

잔량 추적 폐기, 셀 시작부터 누적 위치 (`cum`) 명시적 추적:

```rust
let has_offset = content_offset > 0.0;
let has_limit = content_limit > 0.0;
let mut cum: f64 = 0.0;

for para in cell.paragraphs {
    let para_start_pos = cum;
    let para_end_pos = cum + para_h;
    cum = para_end_pos;

    if line_count == 0 || has_table_in_para {
        // atomic — 한쪽 페이지에만 렌더링
        let was_on_prev = has_offset && para_end_pos <= content_offset;
        let exceeds_limit = has_limit && para_end_pos > content_limit;
        if was_on_prev || exceeds_limit {
            result.push((line_count, line_count));   // 스킵
        } else {
            result.push((0, line_count));            // 표시
        }
    } else {
        // 일반 문단: line_end_pos = cum + line_h 와 content_offset/limit 비교
        // ...
    }
}
```

**핵심**:
- `has_offset` / `has_limit` 플래그로 무한대 대신 명시적 활성화 여부 표시
- atomic 문단의 `was_on_prev` (이전 페이지 전체 포함) + `exceeds_limit` (다음 페이지로 미룸) 두 가지 명시적 가드
- offset 경계를 걸치면 현재 페이지 (continuation) 에서 렌더링

### v2 — `content_y_accum` 갱신 추가

`layout_partial_table` 의 `!has_nested_table` 분기에 `is_in_split_row` 일 때 `para_full_text_h` 계산식으로 `content_y_accum` 갱신:

```rust
if !has_nested_table && is_in_split_row {
    // line_ranges=(n,n) 인 문단도 누적 갱신
    content_y_accum += para_full_text_h;
}
```

### v3 — 통일 경로

`has_nested_table` 셀 분기 제거, 모든 셀에 대해 `compute_cell_line_ranges` + `calc_visible_content_height_from_ranges` 통일 경로 사용. inner table 의 cell 내 위치도 정확히 반영.

## 검증

- `cargo test --release`: 992 단위 + 71 통합 통과
- `cargo test --test svg_snapshot`: 6/6 passed (form_002_page_0 갱신, 의도)
- `cargo clippy --release -- -D warnings`: clean
- form-002 page 1 인너 표 제거 ✓
- form-002 page 2 인너 표 정상 표출 (3줄 모두) ✓
- 작업지시자 WASM 시각 검증 통과

## 교훈

### 1. 잔량 추적 vs 누적위치 추적

페이지 분할 결정 같은 "위치 의식적" 알고리즘에서 **잔량 추적은 정보 손실 위험**. 잔량이 0 도달하면 cumulative position 정보가 사라져 후속 atomic 단위 결정 시 부정확. **명시적 누적위치 (`cum`)** 추적이 정공법.

### 2. atomic 단위 가시성의 두 가드

`was_on_prev` (이전 페이지 전체 포함) + `exceeds_limit` (다음 페이지로 미룸) 두 가지 가드를 **명시적으로 분리** 해서 표현. 한쪽만 검사하면 한쪽 누락 케이스가 남는다.

### 3. 통일 경로 vs 특수 분기

`has_nested_table` 같은 특수 분기는 종종 다른 path 의 가드를 우회한다. **단일 경로로 통일** 하면 코드 단순화 + 일관성 확보. 특수 케이스는 통일 경로 안의 가드로 처리.

### 4. 시각 검증의 가치

3-단계 자체 보강 (v1 → v2 → v3) 은 자동 테스트 통과 후 시각 검증으로만 발견 가능한 결함을 식별. 자동 테스트 (svg_snapshot) 만으로는 잡히지 않는 시각적 클립 / 일부 누락 같은 문제는 **WASM 시각 검증** 으로 해결.

## 관련 자료

- 이슈: [#324](https://github.com/edwardkim/rhwp/issues/324)
- PR: [#327](https://github.com/edwardkim/rhwp/pull/327)
- 외부 기여: [@planet6897](https://github.com/planet6897) (Jaeook Ryu) — 분석·구현·자체 보강
- 후속 이슈 후보: #325 (cell.h vs 실제 콘텐츠 누적 높이 불일치, diff=-21.2px)
