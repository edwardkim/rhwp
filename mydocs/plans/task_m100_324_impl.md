# Task #324 구현 계획서

**브랜치**: `local/task324`
**근거**: `mydocs/plans/task_m100_324.md`

---

## 1. 수정 대상

### 파일
`src/renderer/layout/table_layout.rs`

### 함수
`compute_cell_line_ranges()` (≈ 라인 2004~2143)

### 분기
`has_table_in_para` (≈ 라인 2052~2086) — 셀 내 paragraph 가 중첩 표를 포함하는 경우의 처리.

## 2. 수정 내용

### 현재 (버그)

```rust
if has_table_in_para {
    let nested_h: f64 = ...;
    let line_based_h: f64 = ...;
    let para_h = nested_h.max(line_based_h);

    if offset_remaining > 0.0 {
        if para_h <= offset_remaining { offset_remaining -= para_h; }
        else { offset_remaining = 0.0; }
    } else if limit_remaining > 0.0 {
        if para_h <= limit_remaining { limit_remaining -= para_h; }
        // ❌ limit 초과 시 무처리
    }

    if offset_remaining > 0.0
        || (offset_remaining == 0.0 && content_offset > 0.0 && para_h <= content_offset)
    {
        result.push((line_count, line_count));
    } else {
        result.push((0, line_count));   // ❗ 항상 표시
    }
    continue;
}
```

### 수정안

```rust
if has_table_in_para {
    let nested_h: f64 = ...;
    let line_based_h: f64 = ...;
    let para_h = nested_h.max(line_based_h);

    let has_active_limit = content_limit > 0.0;

    if offset_remaining > 0.0 {
        if para_h <= offset_remaining {
            offset_remaining -= para_h;
            result.push((line_count, line_count));
        } else {
            offset_remaining = 0.0;
            // 부분 가시 — 일반 문단의 partial line 처리와 유사하게 보이는 쪽으로
            // (atomic 이므로 전체 표시 후 limit 차감)
            if has_active_limit && para_h > limit_remaining {
                // limit 초과 → 다음 페이지로 미룸
                result.push((line_count, line_count));
                limit_remaining = 0.0;
            } else {
                if has_active_limit { limit_remaining = (limit_remaining - para_h).max(0.0); }
                result.push((0, line_count));
            }
        }
        continue;
    }

    // offset_remaining == 0 이고 limit 활성 시 atomic 처리
    if has_active_limit && para_h > limit_remaining {
        // ❗ 핵심 수정: limit 초과 → 다음 페이지로 미룸
        result.push((line_count, line_count));
        limit_remaining = 0.0;
        continue;
    }

    // 정상 표시
    if has_active_limit { limit_remaining -= para_h; }

    if offset_remaining == 0.0 && content_offset > 0.0 && para_h <= content_offset {
        result.push((line_count, line_count));
    } else {
        result.push((0, line_count));
    }
    continue;
}
```

핵심: `has_active_limit && para_h > limit_remaining` 일 때 **atomic 단위로 다음 페이지로 미룬다**.

## 3. 단계별 작업

### Stage 1 — 재현 검증 + 기준선 캡처
1. form-002.hwpx 현재 SVG 출력
2. `dump-pages` / `dump` 결과 캡처
3. 회귀 비교용 기준 샘플 출력 (samples/ 내 분할 + 중첩 표 후보 1~2개)
4. `mydocs/working/task_m100_324_stage1.md` 작성 + 커밋

### Stage 2 — 코드 수정
1. `compute_cell_line_ranges` 의 `has_table_in_para` 분기 수정
2. `cargo build --release` 통과
3. `cargo test` 전체 통과
4. `cargo clippy --release -- -D warnings` 통과
5. `mydocs/working/task_m100_324_stage2.md` 작성 + 커밋

### Stage 3 — 결과 검증
1. form-002.hwpx page 1, 2 SVG 재출력 → PDF 비교
2. Stage 1 회귀 샘플 SVG 재출력 → diff 확인
3. `mydocs/working/task_m100_324_stage3.md` 작성
4. `mydocs/report/task_m100_324_report.md` 작성
5. `mydocs/orders/20260425.md` 갱신
6. 커밋 + local/devel 머지

## 4. 테스트 추가

`compute_cell_line_ranges` 단위 테스트가 있는지 확인 후, 셀 분할 + 중첩 표 케이스 회귀 테스트 추가.

## 5. 승인

전체 자동 승인 받음 (작업지시자 지시: "전체 자동 승인으로 진행").
