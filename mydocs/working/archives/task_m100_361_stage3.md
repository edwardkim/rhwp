# Task #361 Stage 3 — 코드 수정 + 자동 회귀

## 수정 내용

`src/renderer/typeset.rs` 의 `finalize_pages` 함수.

### 변경 1: prev_page_last_para 추가 + page_last_para 위치 이동

```rust
let mut current_header: Option<HeaderFooterRef> = None;
let mut current_footer: Option<HeaderFooterRef> = None;
let mut page_num: u32 = 1;
// [Task #361] 이전 페이지의 마지막 문단 추적
let mut prev_page_last_para: Option<usize> = None;

for page in pages.iter_mut() {
    // 이 페이지에 속하는 첫/끝 문단 인덱스
    let page_last_para = page.column_contents.iter()
        .flat_map(|col| col.items.iter())
        .map(|item| match item {
            PageItem::FullParagraph { para_index } => *para_index,
            PageItem::PartialParagraph { para_index, .. } => *para_index,
            PageItem::Table { para_index, .. } => *para_index,
            PageItem::PartialTable { para_index, .. } => *para_index,
            PageItem::Shape { para_index, .. } => *para_index,
        })
        .max();
```

### 변경 2: NewNumber 적용 조건 수정

변경 전:
```rust
if let Some(fp) = first_para {
    for &(nn_pi, nn_num) in new_page_numbers {
        if nn_pi <= fp {                      // ← 매 페이지마다 재설정
            page_num = nn_num as u32;
        }
    }
}
```

변경 후:
```rust
for &(nn_pi, nn_num) in new_page_numbers {
    let after_prev = prev_page_last_para.map_or(true, |prev| nn_pi > prev);
    let in_current = page_last_para.map_or(false, |last| nn_pi <= last);
    if after_prev && in_current {
        page_num = nn_num as u32;
    }
}
```

### 변경 3: page_num += 1 직전에 prev_page_last_para 갱신

```rust
prev_page_last_para = page_last_para.or(prev_page_last_para);
page_num += 1;
```

### 변경 4: PartialTable 직후 fit 안전마진 비활성화 (시각 판정으로 발견된 추가 회귀 수정)

`typeset_paragraph` 의 안전마진 적용:
```rust
let prev_is_partial_table = matches!(
    st.current_items.last(),
    Some(PageItem::PartialTable { .. })
);
let safety = if st.skip_safety_margin_once {
    st.skip_safety_margin_once = false;
    0.0
} else if prev_is_partial_table {
    0.0
} else {
    LAYOUT_DRIFT_SAFETY_PX
};
```

**이유**:
- k-water-rfp p15: PartialTable rows=15..32 (used=894.8px) 직후 pi=181 (h=16.0) fit 검사
  - cur_h + height_for_fit = 894.8 + 16.0 = 910.8
  - avail = 915.5 - 10.0 (안전마진) = 905.5
  - 910.8 > 905.5 → fit 실패하여 다음 페이지로 밀림 (page 16 의 pi=190 표가 page 17 로 연쇄 회귀)
- PartialTable 의 cur_h 는 row 단위로 정확히 누적되어 안전마진이 과함
- 안전마진 비활성화 시 910.8 < 915.5 → 정상 fit

수정 효과: k-water-rfp 28 → **27 페이지** (v0.7.3 와 동일).

## 자동 회귀 결과

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | **1008 passed, 0 failed** |
| `cargo test --test svg_snapshot` | **6/6 통과** |
| `cargo test --test issue_301` | 1/1 통과 |
| `cargo clippy --lib -- -D warnings` | 통과 |
| `cargo check --target wasm32-unknown-unknown --lib` | 통과 |

## page_num 갱신 검증

### k-water-rfp (section=1 부분)
| 페이지 | 수정 전 | 수정 후 | v0.7.3 |
|---|---|---|---|
| p3 | 1 | 1 | 1 |
| p4 | 1 | 2 | 2 |
| p5 | 1 | 3 | 3 |
| ... | 1 | ... | ... |
| p28 | 1 | 26 | (v0.7.3 는 27 페이지 — split 차이) |

### kps-ai (section=0)
| 페이지 | 수정 전 | 수정 후 | v0.7.3 |
|---|---|---|---|
| p1 | 1 | 1 | 1 |
| p2 | 1 | 2 | 2 |
| p3 | 1 | 1 (NewNumber) | 1 |
| p4 | 1 | 1 | 1 |
| p5~p11 | 1 | 2~8 | 2~8 |
| p12 | 1 | 9 | 9 |

→ **v0.7.3 와 동일 패턴**으로 정상 갱신.

## 샘플 회귀 (LAYOUT_OVERFLOW + 페이지 수)

| 샘플 | 페이지 (변경 1~3 후 → 변경 4 후) | LAYOUT_OVERFLOW (전→후) |
|------|-----|-----|
| form-01 | 1 → 1 | 0 → 0 |
| aift | 77 → 77 | 3 → 3 |
| KTX | 27 → 27 | 1 → 1 |
| **k-water-rfp** | 28 → **27** | **0 → 0** |
| exam_eng | 11 → 11 | 0 → 0 |
| **kps-ai** | 81 → 81 | **4 → 4** |
| hwp-multi-001 | 10 → 10 | 0 → 0 |

→ k-water-rfp 28 → 27 페이지 (v0.7.3 와 동일), 다른 샘플 무변화 + LAYOUT_OVERFLOW 무변화.

## 다음 단계 (Stage 4)

1. WASM Docker 빌드
2. 작업지시자 시각 판정 — k-water-rfp / kps-ai 의 머리말꼬리말 페이지 번호 정상 표시 확인
3. 최종 보고서 + 트러블슈팅 + orders 갱신
4. 타스크 브랜치 커밋 + local/devel merge (작업지시자 승인 후)
