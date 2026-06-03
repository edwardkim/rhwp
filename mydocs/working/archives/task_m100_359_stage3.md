# Task #359 Stage 3 — 코드 수정 + 자동 검증

## 수정 내용

`src/renderer/typeset.rs` 3개 영역 변경.

### 1. Option A — fit 누적을 total_height 로 분리 (line 612-624)

```rust
// fit 판정은 height_for_fit (trailing_ls 제외) 으로,
// 누적은 total_height (full) 로 분리.
if st.current_height + fmt.height_for_fit <= available {
    st.current_items.push(PageItem::FullParagraph { para_index: para_idx });
    st.current_height += fmt.total_height;  // (was: height_for_fit)
    return;
}
```

이유: 각 항목별 trailing_ls 가 누적에서 빠지면 N items 누적 시 N × trailing_ls 만큼 drift. k-water-rfp p3 case 36 items × 평균 ~9px = ~311px LAYOUT_OVERFLOW.

### 2. 단독 항목 페이지 차단 가드 (line 387-415)

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
    if para.text.is_empty() { continue; }  // 빈 문단 skip
    else { st.skip_safety_margin_once = true; }  // 일반 텍스트는 안전마진 1회 끄기
}
```

발동 조건:
- 다음 pi 의 first_vpos=0 (vpos-reset)
- 현재 pi 의 last_vpos>5000 (~1.76mm 이상)
- 다음 pi 가 force_page_break 가 아님 (정상 쪽나누기 신호 제외)
- 현재 페이지에 항목 존재

처리:
- **빈 문단 (text.is_empty())**: skip (단독 빈페이지 차단)
- **일반 텍스트**: fit 안전마진 (10px) 1회 비활성화 (단독 텍스트 페이지 차단)

### 3. TypesetState 에 skip_safety_margin_once 필드 추가 (line 122-124)

가드와 typeset_paragraph 사이의 1회성 신호 전달용.

## 진단 케이스

### k-water-rfp.hwp (본 task 의 시작 케이스)
- 페이지 수: 26 → 28 (정상 분할)
- LAYOUT_OVERFLOW: 73 → 0
- p3 dump diff: -311.2px → +4.0px

### kps-ai.hwp (시각 판정 발견 케이스)
- 페이지 10 단독 빈 문단 (pi=162) → 다음 페이지로 흡수
- 페이지 35 단독 텍스트 (pi=317) → 페이지 34 끝에 fit
- 페이지 수: 88 → 81
- LAYOUT_OVERFLOW: 60 → 4

### hwp-multi-001.hwp (회귀 테스트로 발견)
- 가드 초기 버전이 force_page_break 케이스도 발동시켜 페이지 2 이미지 노드 사라짐
- 가드 정밀화 (next_force_break 제외) 로 회귀 차단

## 자동 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | **1008 passed, 0 failed** |
| `cargo test --test svg_snapshot` | **6/6 통과** |
| `cargo test --test issue_301` | **1/1 통과** |
| `cargo clippy --lib -- -D warnings` | 통과 |
| `cargo check --target wasm32-unknown-unknown --lib` | 통과 |

## 7 핵심 샘플 + 추가 샘플 회귀

| 샘플 | 페이지 (수정 전→후) | LAYOUT_OVERFLOW (전→후) |
|------|-----|-----|
| form-01 | 1 → 1 | 0 → 0 |
| aift | 74 → 77 | 30 → **3** |
| KTX | 27 → 27 | 2 → **1** |
| k-water-rfp | 26 → **28** | 73 → **0** |
| exam_eng | 8 → 11 | 0 → 0 |
| kps-ai | 81 → 81 | 60 → **4** |
| hwp-multi-001 | 10 → 10 | 0 → 0 |

모든 샘플에서 LAYOUT_OVERFLOW 유지 또는 감소.

## WASM 빌드

```
docker compose --env-file .env.docker run --rm wasm
[INFO]: :-) Done in 1m 20s
[INFO]: :-) Your wasm pkg is ready to publish at /app/pkg.

pkg/rhwp_bg.wasm: 4.1 MB
pkg/rhwp.js: 227 KB
```

작업지시자 시각 판정으로 k-water-rfp p3, kps-ai p9-11, p35-36 정상화 확인 완료.

## 다음 단계 (Stage 4)

1. 최종 보고서 (`mydocs/report/task_m100_359_report.md`)
2. 트러블슈팅 등록 (`mydocs/troubleshootings/task359_pagination_layout_drift.md`)
3. 오늘할일 갱신 (`mydocs/orders/20260426.md`)
4. 타스크 브랜치 커밋 + local/devel merge
