# Stage 2a 보고서 — Task #485 Bug-1 정정 (out-of-order 제거)

**작성일**: 2026-05-07
**브랜치**: `local/task485`
**대상**: `src/renderer/layout/table_layout.rs` `compute_cell_line_ranges`

---

## 1. 변경 요약

`compute_cell_line_ranges` 의 inner break 가 outer 루프를 빠져나오지 않아 후속 단락이 같은 `cum` 으로 재평가되어, 셀의 마지막 단락(line_spacing 제외로 `line_h` 작아짐)이 abs_limit 안에 fit 하여 **시각 순서 역전 + 본문 경계 클립**을 일으키던 결함 정정.

`limit_reached` 플래그를 도입하여 abs_limit 도달 시 outer 루프의 후속 단락 모두 미렌더 처리.

## 2. 변경 내용 (diff 요점)

### `src/renderer/layout/table_layout.rs:2214 부근`

```rust
let abs_limit = if has_limit { content_offset + content_limit } else { 0.0 };

// [Task #485 Bug-1] abs_limit 도달 후 렌더 차단 플래그.
let mut limit_reached = false;

let total_paras = composed_paras.len();
for (pi, (comp, para)) in composed_paras.iter().zip(cell.paragraphs.iter()).enumerate() {
    ...
    let line_count = comp.lines.len();

    // [Task #485 Bug-1] 한도 초과 후 후속 단락은 강제 미렌더 (시각 순서 보존).
    if limit_reached {
        let visible_count = if line_count == 0 { 0 } else { line_count };
        result.push((visible_count, visible_count));
        continue;
    }
    ...
}
```

### atomic 분기 (line_count==0 || has_table_in_para)

```rust
if was_on_prev || exceeds_limit {
    result.push((visible_count, visible_count));
    // [Task #485 Bug-1] limit 초과 단락 발생 시 후속 단락 차단.
    if exceeds_limit {
        limit_reached = true;
    }
} else {
    result.push((0, visible_count));
}
```

### 일반 분기 (라인 단위 누적)

```rust
if has_limit && line_end_pos > abs_limit {
    // [Task #431] abs_limit 와 비교 (단위 정합)
    // [Task #485 Bug-1] outer 루프도 차단 — 후속 단락의 작은 line_h slip 방지.
    limit_reached = true;
    break;
}
```

## 3. 검증 결과

### 3.1 trace 비교 (RHWP_T485_TRACE)

#### 페이지 15 (이전 vs 적용 후)

| 위치 | 이전 | 적용 후 |
|------|------|---------|
| pi=60 li=1 (마지막 OK) | OK (cum=2229.333, gap=+16.640) | 동일 |
| pi=61..83 첫 줄 | BREAK (gap=-6.827) | pi=61 BREAK 후 outer 루프 차단 |
| **pi=84 li=0 (cell-last, 14.667)** | **NEAR 통과 (gap=+1.973) — slip** | **차단 (limit_reached=true) — slip 제거** |

#### 페이지 20

- 이전: pi=169 (cell-last, line_h=13.333) slip 통과
- 적용 후: pi=68 BREAK 후 outer 차단, pi=169 미렌더

#### 페이지 21

- pi=108 NEAR (gap=+0.947) — 변동 없음 (slip 자체가 없었던 페이지)
- pi=109 BREAK 후 차단 — 의도대로

### 3.2 시각 검증 (export-svg)

- **페이지 15**: 클립 해소 ✓ (마지막 줄 본문 경계와 충분한 여유)
- **페이지 20**: 클립 해소 ✓
- **페이지 21**: **클립 잔존** — pi=108 의 NEAR 케이스 (gap=0.947) 는 Bug-2 (boundary epsilon) 영역으로 Stage 2b 에서 처리 예정

### 3.3 cargo test 결과

```
test result: ok. 1125 passed; 0 failed; 2 ignored
test result: ok. 6 passed (svg_snapshot)
test result: ok. 14, 25, 9, 8, ... (전체)
```

전 테스트 통과 — Task #431/#362/#398/#474 회귀 없음.

## 4. 잔여 작업

- **Stage 2b**: 페이지 21 의 boundary epsilon 정정 — break 조건에 epsilon 마진 추가
- **Stage 3**: 회귀 검증 (kps-ai.hwp, synam-001 외 페이지 시각 검증, PDF 대조)

## 5. 위험 / 부작용

### 5.1 atomic 분기의 `was_on_prev` 분기 처리

`was_on_prev` (이전 페이지에서 렌더링됨) 케이스는 `limit_reached` 트리거 안 함 — 적절. atomic 단락이 offset 영역에 있으면 이는 정상 시멘틱 (이전 페이지 콘텐츠).

### 5.2 atomic `bigger_than_page` 케이스

`bigger_than_page` (한 페이지보다 큰 nested table) 는 `exceeds_limit=false` 로 처리되어 `limit_reached` 트리거 안 함 — 적절. 다음 페이지로의 분할이 별도 메커니즘으로 동작.

### 5.3 후보 A (typeset 측 split_end_limit 정정) 진입 불요

본 Stage 2a 로 cell 마지막 단락 slip 의 본질이 layout 측에서 차단되므로, typeset 의 `split_end_limit = avail_content` 산정 자체는 변경 불요. 후보 A 보류 결정 유효.

## 6. 산출물 / 커밋

- 변경 파일: `src/renderer/layout/table_layout.rs` (3 hunk: 변수 도입 + atomic 분기 + 일반 분기)
- 커밋 예정: `Task #485 Stage 2a: out-of-order 정정 (limit_reached 플래그)`

## 7. 작업지시자 승인 요청

1. Bug-1 정정 효과 확인 (p15·p20 클립 해소) 동의?
2. Stage 2b (Bug-2 — p21 epsilon 정정) 진행 동의?
3. epsilon 권장값 — 고정 2.0px 또는 line_h × 0.1 — 선호?

승인 후 Stage 2b 진행.
