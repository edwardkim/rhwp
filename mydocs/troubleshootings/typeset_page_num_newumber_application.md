# TypesetEngine page_num NewNumber 적용 + PartialTable fit 안전마진 — 트러블슈팅 (Task #361)

## 증상

### 1. page_num 갱신 결함
TypesetEngine 이 default pagination 으로 사용된 후 (Task #313 이후), section 내 모든 페이지가 `page_num=1` 로 표시. 머리말꼬리말의 페이지 번호 / PageNumberPos 컨트롤 / 홀짝 페이지 처리 모두 영향.

### 2. PartialTable 직후 fit 회귀 (시각 판정으로 발견)
PartialTable (rows 분할 표) 직후 작은 텍스트가 잔여 공간 부족 (5.3px) 으로 다음 페이지로 밀려 표 배치 연쇄 회귀.

## 원인

### 1. NewNumber 적용 조건의 시멘틱 결함

`src/renderer/typeset.rs::finalize_pages` 의 결함 코드:
```rust
if let Some(fp) = first_para {
    for &(nn_pi, nn_num) in new_page_numbers {
        if nn_pi <= fp { page_num = nn_num as u32; }
    }
}
```

문제: `nn_pi <= fp` 만 체크하므로 NewNumber 가 이전 페이지에 이미 적용됐어도 **매 페이지마다 page_num 을 nn_num 으로 강제 재설정**. 직전의 `page_num += 1` 이 무용지물.

### 2. PartialTable 직후 fit 안전마진 (10px) 의 과도함

`LAYOUT_DRIFT_SAFETY_PX = 10.0` 안전마진은 일반 문단의 typeset/layout drift 보정용. 그러나 PartialTable 의 cur_h 는 **row 단위로 정확히 누적** 되어 안전마진이 과함. 직후 작은 텍스트가 잔여 5px 부족으로 fit 실패하여 다음 페이지로 밀림 → 표 배치 연쇄 회귀.

## 해결

### 1. Paginator 의 시멘틱 그대로 이식

```rust
let mut prev_page_last_para: Option<usize> = None;

for page in pages.iter_mut() {
    let page_last_para = ...;  // 이 페이지의 마지막 문단

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

조건:
- `after_prev`: NewNumber 가 이전 페이지에 이미 적용되지 않음
- `in_current`: NewNumber 가 이 페이지 안에 있음
- → 한 페이지에서 한 번만 적용 → 후속 페이지는 +1 정상 증가

### 2. PartialTable 직후 fit 안전마진 비활성화

```rust
let prev_is_partial_table = matches!(
    st.current_items.last(),
    Some(PageItem::PartialTable { .. })
);
let safety = if st.skip_safety_margin_once { ... }
    else if prev_is_partial_table { 0.0 }
    else { LAYOUT_DRIFT_SAFETY_PX };
```

PartialTable 의 cur_h 는 정확하므로 안전마진 불필요.

## 재발 방지 체크리스트

### 새 엔진 도입 / pagination 로직 변경 시
- [ ] Paginator 의 finalize_pages 와 동일 시멘틱 유지 (특히 NewNumber, HF apply, PageHide)
- [ ] page_num 갱신 회귀 검증: kps-ai p1~p11 (1, 2, 1, 1, 2~8) 와 일치
- [ ] section=1 의 첫 페이지 page_num=1 부터 정상 갱신

### typeset 안전마진 변경 시
- [ ] PartialTable 등 누적이 정확한 항목 직후 fit 시 안전마진 적용 여부 점검
- [ ] k-water-rfp 의 페이지 분할 회귀 비교

## 진단 도구

- `dump-pages` 의 `page_num=` 출력으로 갱신 정상 여부 확인
- `RHWP_TYPESET_DRIFT=1` 로 fit 검사 시 cur_h / avail / height_for_fit trace
- `RHWP_USE_PAGINATOR=1` 로 옛 Paginator fallback (회귀 비교용)

## 관련 task

- Task #361 (본 task)
- Task #313 (TypesetEngine default 전환 — 회귀 도입)
- Task #359 (typeset fit drift + 단독 항목 페이지 차단)
- Task #340 (typeset 경로 정합)
