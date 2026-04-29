# Task #468 구현계획서

## 목표

`build_single_column` 의 merge 그룹별로 첫/마지막 paragraph 의 sequential 인접 bf_id 를 검사하여 cross-column 박스 연속 시 partial_start/partial_end 플래그 보정.

## 데이터 흐름

### 현재 상태

`para_border_ranges` 의 튜플 구조 (Stage 5 이후):

```rust
(bf_id, x, y_start, w, y_end, top_inset, bottom_inset, is_partial_start, is_partial_end)
```

`paragraph_layout.rs:2529-2531` 에서 push 시점에 partial 플래그 결정:
```rust
let is_partial_start = start_line > 0;
let is_partial_end = end < composed.lines.len();
```

이는 **PartialParagraph** (단일 paragraph 내부 split) 만 검출.

### 추가 정보 필요

paragraph index (para_index) 가 튜플에 없어 merge 후 sequential 인접 paragraph bf_id 비교 불가.

## 변경 항목

### 1. `para_border_ranges` 튜플 확장

`para_index` 추가:

```rust
// before:
(bf_id, x, y_start, w, y_end, top_inset, bottom_inset, is_partial_start, is_partial_end)
// after:
(bf_id, x, y_start, w, y_end, top_inset, bottom_inset, is_partial_start, is_partial_end, para_index)
```

영향:
- `src/renderer/layout.rs:217-220` (선언)
- `src/renderer/layout/paragraph_layout.rs:2529-2531` (push)
- `src/renderer/layout.rs:1606-1750` (소비/merge)

### 2. `paragraph_layout.rs` push 시 para_index 포함

```rust
self.para_border_ranges.borrow_mut().push(
    (para_border_fill_id, col_area.x + box_margin_left, bg_y_start,
     col_area.width - box_margin_left - box_margin_right, y,
     top_inset, bottom_inset, is_partial_start, is_partial_end,
     para_index)  // ← 추가
);
```

### 3. `build_single_column` merge 후 partial 플래그 보정

merge 후 각 그룹의 첫/마지막 paragraph index 를 보유. sequential 인접 bf_id 검사:

```rust
// merge 그룹 구성: (bf_id, x, y_start, w, y_end, top_inset, bottom_inset,
//                  is_partial_start, is_partial_end, first_para_idx, last_para_idx)
let mut groups: Vec<(...)> = Vec::new();
for &(bf_id, x, y_start, w, y_end, top_inset, bottom_inset,
      is_partial_start, is_partial_end, para_idx) in ranges.iter() {
    if let Some(last) = groups.last_mut() {
        // ... (기존 merge 로직)
        if same_visual && (y_start - last.4) < 30.0 {
            last.4 = y_end;
            last.6 = bottom_inset;
            last.8 = is_partial_end;
            last.10 = para_idx;  // ← last_para_idx 갱신
            continue;
        }
    }
    groups.push((bf_id, x, y_start, w, y_end, top_inset, bottom_inset,
                 is_partial_start, is_partial_end, para_idx, para_idx));
}

// Cross-column partial 보정: sequential 인접 paragraph bf_id 검사
for g in groups.iter_mut() {
    let bf_id = g.0;
    let first_pi = g.9;
    let last_pi = g.10;
    
    // partial_start 보정: paragraph[first_pi-1] 가 같은 bf_id 인가?
    if first_pi > 0 && !g.7 {
        let prev_bf = composed.get(first_pi - 1)
            .and_then(|c| styles.para_styles.get(c.para_style_id as usize))
            .map(|s| s.border_fill_id)
            .unwrap_or(0);
        if prev_bf == bf_id {
            g.7 = true;  // partial_start
        }
    }
    
    // partial_end 보정: paragraph[last_pi+1] 가 같은 bf_id 인가?
    if !g.8 {
        let next_bf = composed.get(last_pi + 1)
            .and_then(|c| styles.para_styles.get(c.para_style_id as usize))
            .map(|s| s.border_fill_id)
            .unwrap_or(0);
        if next_bf == bf_id {
            g.8 = true;  // partial_end
        }
    }
}
```

### 4. group destructure 업데이트

`for (gi, (bf_id, x, y_start, w, y_end, top_inset, bottom_inset, is_partial_start, is_partial_end))` 패턴을 모두 `..., first_pi, last_pi)` 로 확장 (또는 `_, _` 로 무시).

## 단계

1. **튜플 정의·push 확장** (`paragraph_layout.rs`, `layout.rs:217-220`)
2. **merge 그룹 구조 확장 + partial 보정 로직 추가** (`layout.rs:1627+`)
3. **소비 코드 destructure 업데이트** (`layout.rs:1664+`)
4. **빌드 + 회귀 검증**

## 검증

- exam_kor.hwp 6p 좌측 단 박스 하단 stroke 미렌더 ✓ (목표)
- exam_kor.hwp 6p 우측 단 박스 상단 stroke 미렌더 (col 0 ← col 1 연속)
- exam_kor.hwp 7p 좌측 단 박스 (페이지 6 → 7 연속) 상단 stroke 미렌더
- exam_kor.hwp 14p (#463 Stage 2 정합 — 단일 박스 닫힘 유지)
- cargo test 1069 + svg_snapshot 6 통과
- 5종 샘플 (2010-01-06, biz_plan, 21언어, exam_eng, 2022국립국어원) 정상

## 위험·완화

| 위험 | 완화 |
|------|------|
| 별개 박스인데 같은 bf_id 가 sequential → 잘못 partial 마킹 | merge 로직이 이미 그룹 단위 분리 (다른 bf_id 가 끼거나 y-gap > 30px) → 그룹 경계가 박스 경계 보장 |
| composed[N+1] 의 para_style_id 가 다른 paragraph 의 bf_id 와 우연 일치 | bf_id 0 (border 없음) 은 무시하므로 false-positive 최소 |
| 영향 범위 (튜플 9-필드 → 10-필드) | 컴파일러가 destructure 누락 검출, 빌드 실패 시 즉시 발견 |

## 참조

- 수행계획서: `mydocs/plans/task_m100_468.md`
- 관련 Stage 5 (Task #463) 의 partial 플래그 도입: `working/task_m100_463_stage5.md`
