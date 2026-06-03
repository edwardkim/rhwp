# Stage 2 재진단 (v2) — Task #1062: 미주 누적 메커니즘 + 수정 설계

- 브랜치: `local/task1062` (VPOS_DEBUG 측정, 소스 무변경)

## 측정 — 미주 구간 VPOS_CORR (pi ≥ 468)

- 미주는 `base=0`(또는 page_base)로 vpos→y 매핑. `y_in` 이 vpos_end 누적에 따라 단조 증가.
- **base 리셋은 페이지마다 일어남** (pi 510/545/584… 에서 y_in 1276→108 등 새 페이지 상단 복귀).
- 그러나 **리셋 직전 y_in 이 col_bottom(1092)을 넘김** — 최대 y_in=1912(820px 초과).

## 원인 — 미주 누적이 trailing_ls 과소 계상 (위치 정정)

`typeset.rs:1440-1452` 미주 페이지네이션:
```rust
if st.current_height + fmt.height_for_fit > available && !st.current_items.is_empty() {
    st.advance_column_or_new_page();
}
st.current_items.push(PageItem::FullParagraph { para_index: en_para_idx });
st.current_height += if st.col_count > 1 { fmt.height_for_fit } else { fmt.total_height };
```

- 다단(시험지)에서 미주 누적 = `height_for_fit` (= total − trailing_ls).
- 렌더러는 미주를 vpos 로 배치 → 문단당 vpos 전진 = lh + ls (trailing_ls **포함**, `prev_ls=452`=6px).
- 즉 typeset 이 미주당 **6px 과소 계상** → 페이지당 미주 과밀 배치 → 렌더러가 다음 페이지 base
  리셋 전에 col_bottom 초과.
- **검산**: 페이지당 ~30 미주 × 6px ≈ 180px ≈ 관측 overshoot(184px). 일치.

→ 최초 Stage 1~3 의 trailing_ls 가설은 **메커니즘은 옳고 위치만 틀렸다**(본문 누적 1834 ❌ →
미주 누적 1448 ⭕). 본문 변경이 무효였던 이유와 정합.

## 수정 설계 (Stage 3 안)

미주 누적/판정을 렌더러 vpos 전진과 통일 (미주 루프 한정):

- **누적**(1452): `height_for_fit`/`total_height` → **미주 문단 vpos 전진** = `en_para.line_segs`
  기반 (last.vpos + last.lh + last.ls − first.vpos). 미주 끝위치 추적(1425-1432)에서 이미 계산
  하는 값과 동일 → 재사용 가능.
- **fit 판정**(1440): 마지막 항목 trailing_ls 제외 의미 유지(현행 height_for_fit 유지 검토) —
  단, 누적과 일관되도록 vpos 전진 − trailing_ls 로 정합.
- 스코프: **미주 루프(1395-1453) 한정**. 본문/표 누적 불변.

## 안전성

- 변경이 미주 누적에만 적용 → 본문/표/그림 페이지네이션 불변.
- 미주 vpos 전진은 렌더러와 정확히 일치 → 페이지당 미주 수가 렌더러 실측과 맞아 overshoot 해소.
- 단일 줄 미주: vpos 전진 = lh+ls (현 height_for_fit보다 +6px) → 페이지당 미주 약간 감소 →
  쪽수 증가(우리 21→PDF 23 방향). 다중 줄 미주: vpos 실측이라 over/under 없음.

## 비회귀 확인 대상

- `endnote-01`(미주), `footnote-01`(각주) 골든.
- 미주 없는 다단 문서(exam_eng 등) — 미주 루프 미진입이라 불변 예상.

## 다음 (Stage 3)

미주 루프(1448 누적 + 1440 판정)를 vpos 전진 기반으로 수정 → 대상 4종 overflow/쪽수 PDF 정합,
비회귀 전수, 골든 회귀 0, 전 251 샘플 합계 점검.
