# Task #361 최종 결과 보고서 — TypesetEngine page_num + PartialTable fit 안전마진 정정

## 이슈

[#361](https://github.com/edwardkim/rhwp/issues/361) — TypesetEngine 의 페이지 경계 처리 결함:
1. `page_num` 이 section 내 모든 페이지에서 1 (정상은 1, 2, 3, ...)
2. dump-pages 의 vpos / hwp_used 가 페이지마다 reset 안되고 누적

v0.7.3 (Paginator) 에서는 정상. `edddebd Task #313` 에서 TypesetEngine 을 default 로 전환한 시점에 도입.

## 결론

**핵심 결함 origin**: TypesetEngine 의 `finalize_pages` 에서 NewNumber 컨트롤 적용 조건이 `nn_pi <= fp` 만 체크 → NewNumber 가 이전 페이지에 적용됐어도 매 페이지마다 page_num 을 강제 재설정.

**부수 결함 (시각 판정 발견)**: PartialTable 직후 작은 텍스트 fit 시 안전마진 (10px) 이 과해 다음 페이지로 밀려 연쇄 회귀.

**수정**:
1. Paginator 시멘틱 그대로 이식 — `prev_page_last_para` 추적 + `nn_pi > prev_page_last_para && nn_pi <= page_last_para`
2. PartialTable 직후 fit 시 안전마진 비활성화 (PartialTable 의 cur_h 는 row 단위로 정확)

## 수정 내용

### 파일: `src/renderer/typeset.rs`

#### 1. `finalize_pages` 의 NewNumber 적용 조건 수정

**변경 전**:
```rust
if let Some(fp) = first_para {
    for &(nn_pi, nn_num) in new_page_numbers {
        if nn_pi <= fp { page_num = nn_num as u32; }
    }
}
```

**변경 후**:
```rust
let mut prev_page_last_para: Option<usize> = None;

for page in pages.iter_mut() {
    let page_last_para = page.column_contents.iter()
        .flat_map(|col| col.items.iter())
        .map(|item| match item { ... })
        .max();

    for &(nn_pi, nn_num) in new_page_numbers {
        let after_prev = prev_page_last_para.map_or(true, |prev| nn_pi > prev);
        let in_current = page_last_para.map_or(false, |last| nn_pi <= last);
        if after_prev && in_current {
            page_num = nn_num as u32;
        }
    }
    // ...
    prev_page_last_para = page_last_para.or(prev_page_last_para);
    page_num += 1;
}
```

Paginator (`src/renderer/pagination/engine.rs:1850-1856`) 와 동일 시멘틱.

#### 2. `typeset_paragraph` 의 PartialTable 직후 fit 안전마진 비활성화

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

## 검증

### 자동 회귀

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | **1008 passed, 0 failed** |
| `cargo test --test svg_snapshot` | 6/6 통과 |
| `cargo test --test issue_301` | 1/1 통과 |
| `cargo clippy --lib -- -D warnings` | 통과 |
| `cargo check --target wasm32-unknown-unknown --lib` | 통과 |

### page_num 갱신 검증

**k-water-rfp.hwp section=1**:
| 페이지 | 수정 전 | 수정 후 | v0.7.3 |
|---|---|---|---|
| p3~p27 | 모두 1 | 1, 2, 3, ..., 25 | 1, 2, 3, ..., 25 |

**kps-ai.hwp section=0** (NewNumber 컨트롤 두 번):
| 페이지 | 수정 전 | 수정 후 | v0.7.3 |
|---|---|---|---|
| p1, p2 | 1, 1 | 1, 2 | 1, 2 |
| p3 (NewNumber) | 1 | 1 | 1 |
| p4 | 1 | 1 | 1 |
| p5~p11 | 1 | 2~8 | 2~8 |

→ **v0.7.3 와 동일 패턴** 으로 정상 갱신.

### 7 핵심 샘플 + 추가 회귀

| 샘플 | 페이지 (수정 전→후) | LAYOUT_OVERFLOW (전→후) |
|------|-----|-----|
| form-01 | 1 → 1 | 0 → 0 |
| aift | 77 → 77 | 3 → 3 |
| KTX | 27 → 27 | 1 → 1 |
| **k-water-rfp** | 28 → **27** | **0 → 0** |
| exam_eng | 11 → 11 | 0 → 0 |
| **kps-ai** | 81 → 81 | 4 → 4 |
| hwp-multi-001 | 10 → 10 | 0 → 0 |

k-water-rfp 28 → 27 페이지: PartialTable 안전마진 수정으로 p15 의 pi=181 이 정상 fit 되어 p16/p17 의 표 배치가 v0.7.3 와 동일하게 정상화.

### 시각 판정 (작업지시자)

- **k-water-rfp p3~p27**: 머리말꼬리말 페이지 번호 정상 표시 (1, 2, 3, ...)
- **k-water-rfp p15-17**: PartialTable 직후 텍스트 정상 fit + pi=190 표가 page 16 에 정상 배치
- **kps-ai p1~p11**: 페이지 번호 정상 갱신
- 시각 판정 통과 (작업지시자 확인)

## 진단 과정 요약

### Stage 1 — 결함 origin 정량 분석
- finalize_pages 에 임시 디버그 출력 → section=0 정상, section=1 의 page_numbers=[1, 1, 1, ..., 1]
- 결함 위치 line 1701-1707 식별 (NewNumber 적용 조건 결함)
- vpos / hwp_used 누적은 HWP 원본 데이터 + dump 알고리즘 한계 (Paginator 도 동일) — 본 task 범위 외

### Stage 2 — 수정 방안 + 영향 분석
- Paginator 시멘틱 (`prev_page_last_para` 추적) 이식안 확정
- page_number 사용처 grep — 머리말꼬리말 / 홀짝 처리 / section carry / dump 모두 정상 page_num 받게 됨

### Stage 3 — 코드 수정 + 자동 회귀
- 수정 적용 → 1008 lib + svg_snapshot 6/6 통과
- LAYOUT_OVERFLOW 회귀 0 (Task #359 효과 유지)

### Stage 4 — WASM 빌드 + 시각 판정
- 1차 빌드 후 시각 판정에서 k-water-rfp p16/p17 표 배치 회귀 발견
- PartialTable 직후 fit 안전마진 비활성화 추가 → 회귀 해소
- 2차 WASM 빌드 + 시각 판정 통과

## 산출물

- 코드: `src/renderer/typeset.rs`
- 문서: 본 보고서 + Stage 1~3 보고서 + 수행/구현 계획서
- 트러블슈팅: `mydocs/troubleshootings/typeset_page_num_newumber_application.md`
- WASM: `pkg/rhwp_bg.wasm` (4.1 MB)

## 후속 과제 (별도 task 후보)

- **vpos / hwp_used 누적 표시** — `compute_hwp_used_height` 의 알고리즘 한계 (Paginator 도 동일). dump 도구 표시 결함, 별도 정리
- **콘텐츠 split 차이** — TypesetEngine 의 fit/split 로직이 Paginator 보다 한 페이지에 ~1 항목 더 욱여넣는 경향. v0.7.3 와 미세한 페이지 분할 차이의 origin
- **kps-ai 잔존 LAYOUT_OVERFLOW 4건** — 다른 origin 으로 추정

## 관련

- 이슈: [#361](https://github.com/edwardkim/rhwp/issues/361)
- 회귀 도입 commit: `edddebd Task #313: 2-3단계 - TypesetEngine default 전환`
- 관련 task: #313 (도입), #340 (typeset 정합), #321~#332 (typeset/layout drift), #359 (fit drift)
- 검증 기준: v0.7.3 (Paginator) 의 page_num + 페이지 분할 동작
