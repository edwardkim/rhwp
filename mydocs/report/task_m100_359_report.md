# Task #359 최종 결과 보고서 — 페이지네이션 fit 판정 vs Layout y 진행 정합

## 이슈

[#359](https://github.com/edwardkim/rhwp/issues/359) — `samples/k-water-rfp.hwp` 페이지 1 (정확히는 section 1 의 페이지 1, 글로벌 page 3) 의 LAYOUT_OVERFLOW (260~288px). pagination 단계의 fit 산정 (used=915.5px) 과 layout 단계의 실제 y 진행 (마지막 항목 y=1316.8) 사이 311px 드리프트.

## 결론

**드리프트 origin**: typeset 의 fit 산정에서 각 항목의 누적에 `height_for_fit` (trailing line_spacing 제외) 사용. N items 누적 시 N × trailing_ls 만큼 누적이 적게 되어 LAYOUT_OVERFLOW 발생.

**수정**: 누적은 `total_height` (full) 로, fit 판정은 `height_for_fit` 으로 분리. 아울러 단독 항목 페이지 발생을 차단하는 vpos-reset 가드 추가.

## 수정 내용

### 파일: `src/renderer/typeset.rs`

#### 1. fit 누적 분리 (Option A)

```rust
// fit 판정: height_for_fit (trailing_ls 제외)
// 누적: total_height (full)
if st.current_height + fmt.height_for_fit <= available {
    st.current_items.push(PageItem::FullParagraph { para_index: para_idx });
    st.current_height += fmt.total_height;  // changed from height_for_fit
    return;
}
```

#### 2. 단독 항목 페이지 차단 가드

```rust
let next_will_vpos_reset = if !st.current_items.is_empty() && para_idx + 1 < paragraphs.len() {
    let next_para = &paragraphs[para_idx + 1];
    let next_force_break = next_para.column_type == ColumnBreakType::Page
        || next_para.column_type == ColumnBreakType::Section;
    if next_force_break { false }
    else {
        let next_first_vpos = next_para.line_segs.first().map(|s| s.vertical_pos);
        let curr_last_vpos = para.line_segs.last().map(|s| s.vertical_pos);
        matches!((next_first_vpos, curr_last_vpos), (Some(nv), Some(cl)) if nv == 0 && cl > 5000)
    }
} else { false };

if next_will_vpos_reset {
    if para.text.is_empty() { continue; }
    else { st.skip_safety_margin_once = true; }
}
```

#### 3. TypesetState.skip_safety_margin_once 필드

가드와 typeset_paragraph 간의 1회성 신호 전달.

## 검증

### 자동 회귀

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | **1008 passed, 0 failed** |
| `cargo test --test svg_snapshot` | 6/6 통과 |
| `cargo test --test issue_301` | 1/1 통과 |
| `cargo clippy --lib -- -D warnings` | 통과 |
| `cargo check --target wasm32-unknown-unknown --lib` | 통과 |

### 7 핵심 샘플 + 추가 회귀

| 샘플 | 페이지 (수정 전→후) | LAYOUT_OVERFLOW (전→후) |
|------|-----|-----|
| form-01 | 1 → 1 | 0 → 0 |
| aift | 74 → 77 | 30 → **3** |
| KTX | 27 → 27 | 2 → **1** |
| **k-water-rfp** | **26 → 28** | **73 → 0** |
| exam_eng | 8 → 11 | 0 → 0 |
| **kps-ai** | **88 → 81** | **60 → 4** |
| hwp-multi-001 | 10 → 10 | 0 → 0 |

### 시각 판정 (작업지시자)

- **k-water-rfp p3**: LAYOUT_OVERFLOW 영역 정상화 (260~288px overflow → 0)
- **kps-ai p9-11**: 단독 빈 문단 페이지 (pi=162) 흡수, 페이지 흐름 정상화
- **kps-ai p35-36**: 단독 텍스트 페이지 (pi=317) 페이지 34 끝에 fit, 흐름 정상화
- **hwp-multi-001 p2**: 그룹 이미지 정상 표시 (가드 정밀화 후)

## 진단 과정 요약

### Stage 1 — 드리프트 origin 정량화
- `RHWP_TYPESET_DRIFT=1` 진단 활용
- page=0 (section-local) vs 글로벌 페이지 인덱스 모순 해결
- pagination used=915.5 vs hwp_used≈1226.7 = drift -311.2px 정량화

### Stage 2 — 가설 검증
- 가설 5개 중 **A (height_for_fit 누적이 trailing_ls 누락)** 확정
- 누적 36 items × 평균 ~9px ≈ 311px 와 정확히 일치

### Stage 3 — 코드 수정 + 자동 검증
- Option A 적용 → k-water-rfp 73→0 LAYOUT_OVERFLOW
- 시각 판정에서 kps-ai 단독 페이지 발견
- vpos-reset 가드 추가 (빈 문단 skip + 안전마진 1회 끄기)
- hwp-multi-001 회귀 발견 → force_page_break 제외 정밀화
- 1008 lib + 6 svg_snapshot 통과

### Stage 4 — WASM 빌드 + 시각 판정
- Docker WASM 빌드 (1m 20s)
- 작업지시자 시각 판정 통과

## 산출물

- 코드: `src/renderer/typeset.rs`
- 문서: 본 보고서 + Stage 1~3 보고서 + 수행/구현 계획서
- 트러블슈팅: `mydocs/troubleshootings/typeset_fit_accumulation_drift.md`
- WASM: `pkg/rhwp_bg.wasm` (4.1 MB)

## 후속 과제

- **kps-ai 잔존 LAYOUT_OVERFLOW 4건**: 다른 origin 으로 추정. 필요 시 별도 task
- **layout.rs 코드 중복 점검** (작업지시자 결정 C, Stage 1 추가 점검 항목): 본 task 범위 외, 별도 task 후보
- **Issue #345** (exam_eng 페이지 회귀): exam_eng 8→11 페이지 변경 — 한컴 의도와 일치 여부 추가 시각 확인 필요

## 관련

- 이슈: [#359](https://github.com/edwardkim/rhwp/issues/359)
- 관련: [#345](https://github.com/edwardkim/rhwp/issues/345) (exam_eng 페이지 회귀)
- 관련 PR: #343 (Task #321~#332 통합), #351 (Task #347 좌표 정합)
