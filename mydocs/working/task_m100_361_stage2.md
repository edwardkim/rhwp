# Task #361 Stage 2 — 수정 방안 + 영향 분석

## Task 2.1 — page_num 수정안 확정

### 변경 전 (`src/renderer/typeset.rs:1689-1707`)

```rust
let mut page_num: u32 = 1;

for page in pages.iter_mut() {
    let first_para = page.column_contents.first()
        .and_then(|col| col.items.first())
        .map(|item| ...);

    if let Some(fp) = first_para {
        for &(nn_pi, nn_num) in new_page_numbers {
            if nn_pi <= fp {                  // ← 결함: 매 페이지마다 재설정
                page_num = nn_num as u32;
            }
        }
    }
    // ...
    page.page_number = page_num;
    page_num += 1;
}
```

### 변경 후 (Paginator 시멘틱 이식)

```rust
let mut page_num: u32 = 1;
let mut prev_page_last_para: Option<usize> = None;  // ← 추가

for page in pages.iter_mut() {
    let first_para = page.column_contents.first()
        .and_then(|col| col.items.first())
        .map(|item| ...);

    // page_last_para 계산을 NewNumber 검사 전으로 이동
    let page_last_para = page.column_contents.iter()
        .flat_map(|col| col.items.iter())
        .map(|item| ...)
        .max();

    // NewNumber 적용 — 한 페이지에서 한 번만
    for &(nn_pi, nn_num) in new_page_numbers {
        let after_prev = prev_page_last_para.map_or(true, |prev| nn_pi > prev);
        let in_current = page_last_para.map_or(false, |last| nn_pi <= last);
        if after_prev && in_current {
            page_num = nn_num as u32;
        }
    }

    // 머리말/꼬리말 갱신 (변경 없음)
    if let Some(last_pi) = page_last_para {
        for (hf_pi, hf_ref, is_header, apply) in hf_entries {
            // ... 기존 그대로
        }
    }

    page.page_number = page_num;
    // ...

    prev_page_last_para = page_last_para;  // ← 추가
    page_num += 1;
}
```

### 핵심 변경 항목

1. **`prev_page_last_para: Option<usize>` 추가** — 이전 페이지의 마지막 문단 추적
2. **`page_last_para` 계산 위치 이동** — NewNumber 검사 전으로 (현재는 머리말꼬리말 검사 안에만 있음, 두 곳에서 사용하도록 위치 변경)
3. **NewNumber 적용 조건 강화**:
   - `nn_pi > prev_page_last_para` (이전 페이지에서 이미 적용 안됨)
   - `nn_pi <= page_last_para` (이 페이지 안에 있음)
4. **`first_para` 사용 제거** — Paginator 의 시멘틱은 first_para 가 아니라 prev_page_last_para 기반

### Paginator 와 등가성 검증

Paginator 의 코드 (`src/renderer/pagination/engine.rs:1850-1856`):
```rust
for (para_idx, new_num) in new_page_numbers {
    if *para_idx > prev_page_last_para || i == 0 {
        if *para_idx <= page_last_para {
            page_num_counter = *new_num as u32;
        }
    }
}
```

Paginator 의 `prev_page_last_para` 는 i64 (-1 초기값) 이고 `i == 0` 는 첫 페이지의 가드.
TypesetEngine 의 `Option<usize>` 의 `map_or(true, ...)` 는 첫 페이지에서 None → true (Paginator 의 `i == 0` 분기와 동일 효과).

→ **시멘틱 등가성 확보**.

## Task 2.2 — page_number 사용처 영향 분석

### grep 결과 (`grep -rn "page_number\b" src/`)

핵심 사용처:
| 위치 | 용도 |
|------|------|
| `src/renderer/layout.rs:495,511,539,582,591,750,757,871` | 머리말꼬리말 필드 치환, 홀짝 페이지 처리, 페이지 번호 표시 |
| `src/renderer/layout.rs:1059-1088 build_page_number` | 페이지 번호 컨트롤 (PageNumberPos) 의 텍스트 생성 |
| `src/renderer/layout/shape_layout.rs:1293`, `table_layout.rs:1175` | shape/table 내부의 page-number 필드 |
| `src/document_core/queries/rendering.rs:767-1020` | section 간 page_number carry (다중 section 페이지 번호 연속화) |
| `src/document_core/queries/rendering.rs:788, 914, 1020` | `is_odd_page` 판정 (홀짝 적용) |
| `src/renderer/pagination.rs:37, 277` | `PageContent.page_number` 정의 + copy_converged_pages |
| `src/renderer/typeset.rs:267, 1740` | 초기화 + 갱신 |

### 회귀 영향

본 수정으로 page_number 가 정상 1, 2, 3, ... 으로 갱신되면:
1. **머리말꼬리말 페이지 번호 표시 정상화** (k-water-rfp / kps-ai 등에서 페이지 번호가 1, 2, 3, ... 표시)
2. **홀짝 페이지 처리 정상화** (Even/Odd 머리말꼬리말 적용)
3. **section 간 carry 정상화** — 1 section 끝 page_num 이 정상이면 2 section 시작 page_num 도 carry 정상
4. **PageNumberPos 컨트롤 정상화** — 페이지 번호 표시 위치 컨트롤
5. **PageHide 변경 없음** (page_num 과 무관)

### 회귀 테스트 항목

- `cargo test --lib` (1008+) — 머리말꼬리말 / 페이지 번호 관련 테스트 통과
- `cargo test --test svg_snapshot` (6/6) — golden SVG 의 페이지 번호 표시 영역
- 7 핵심 샘플 + form-002 + k-water-rfp + kps-ai page_num 회귀 비교
- LAYOUT_OVERFLOW 회귀 0 (Task #359 효과 유지)

## Task 2.3 — Stage 3 의 회귀 검증 항목

### page_num 갱신 검증

| 샘플 | 페이지 | 기대값 (v0.7.3) | 현재 (main) | 수정 후 |
|---|---|---|---|---|
| kps-ai | p1~p11 | 1, 2, 3, ..., 8 | 모두 1 | 1, 2, 3, ..., 8 |
| k-water-rfp | p3~p10 (sec 1) | 1, 2, 3, ..., 8 | 모두 1 | 1, 2, 3, ..., 8 |
| k-water-rfp | p1~p2 (sec 0) | 1, 2 | 1, 2 (정상) | 1, 2 |

### LAYOUT_OVERFLOW 회귀 (Task #359 효과 유지)

| 샘플 | 현재 | 수정 후 (기대) |
|---|---|---|
| k-water-rfp | 0 | 0 |
| kps-ai | 4 | 4 |
| aift | 3 | 3 |
| KTX | 1 | 1 |
| exam_eng | 0 | 0 |
| hwp-multi-001 | 0 | 0 |

## 다음 단계 (Stage 3)

1. 위 수정안 적용
2. `cargo build --release`
3. 자동 회귀 (cargo test --lib, svg_snapshot, issue_301, clippy, wasm32)
4. dump-pages 로 page_num 갱신 확인 (kps-ai, k-water-rfp)
5. LAYOUT_OVERFLOW 회귀 0 확인
6. Stage 3 보고서 작성 → 승인 요청
