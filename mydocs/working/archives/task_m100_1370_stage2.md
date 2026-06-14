# Task #1370 Stage 2 단계별 완료 보고서 — break-높이 디커플(snap-off), 6/13 해결

- **이슈**: #1370 (M100) / 브랜치 `local/task1370`
- **단계**: Stage 2 — 레버2(게이트 한정 compact override) 1차 적용
- **결과**: A3 회귀 **13 → 7** (6 해결), 1082 5/5 유지, 전 A3 suite 무회귀, B 무회귀

## 1. 정밀진단 → 정정된 근본 원인

Stage 1 의 "게이트 임계" 가설을 정정. 실제 근본은:

- A3 는 매 미주 para 누적 후 `st.current_height` 를 **정확 전-단 렌더 sim**
  (`simulate_endnote_column_bottom_y`)으로 **스냅**(typeset.rs:3637)했다.
- 이 exact 스냅이 rewind/빈 미주 para 를 hancom 보다 **단당 ~80px 높게** 누적 → 단이 일찍 가득 차
  경계 para(pi=786 등)를 split 못 하고 통째로 다음 쪽/단으로 밀어냄 → 하류 cascade(제목 밀림·
  overflow·분배 어긋남).
- 실증: 2022_nov p16 단1 A3=1013.4px(overflow) vs hancom 932.3px(pi=786 split 여유).

> 렌더러는 A3·B 동일. 차이는 **pagination 이 보는 높이**뿐. 1082(overflow≤60px)는 compact·exact
> 양쪽 green 이므로 compact 환원이 1082 를 깨지 않음(메모리 `tech_endnote_overflow_nonmonotonic_gate`
> 의 비단조 경고가 이 케이스에선 완화 — B 가 이미 13+1082 동시 green).

## 2. 적용 (레버2 — 게이트 한정 compact override)

`typeset.rs:3637` 의 A2sim 스냅을 `ssot_level >= A2` → `ssot_level == A2` 로 게이트.
**A3 에서 break-결정 높이 누적을 exact 스냅 OFF → compact(acc) 로 환원.**

핵심: `a2_overflow_with_para`(2729)는 여전히 exact `simulate_endnote_column_bottom_y` 를 직접
호출하므로 **overflow 안전선은 exact, 단 fill 은 compact** 로 깔끔히 분리된다(레버2 정합).

```rust
// [Task #1370 Stage 2 실험] A3 한정: exact 스냅이 rewind/빈 para 를 hancom 보다 ~80px/단 높게
// 누적해 경계 split 을 막고 13건 cascade 유발. A3 에서 스냅 OFF → break-결정 높이를 compact(acc)로 환원.
if ssot_level == EnSsotLevel::A2 {
    ...
    st.current_height = sim_bottom;
}
```

## 3. 검증 (전 exam 동시 + 전체 suite)

| 측정 | 결과 |
|------|------|
| A3 `issue_1139` | 13 failed → **7 failed** (6 해결) |
| A3 `issue_1082` | **5/5 pass** (무회귀) |
| A3 전체 cargo test | 총 실패 **7건** = issue_1139 잔여뿐 (다른 파일 무회귀) |
| B `issue_1139` / `issue_1082` | 72/72 · 5/5 (무회귀; B 는 A2 미만이라 스냅 원래 OFF) |

**해결 6건**: #1 page17_q30, #3 split_titles, #4 question29_full_para, #5 internal_rewind_formula_tail,
#8 page21_q23_left_tail, #9 page22_23_tail. (2022_sep/2022_nov/2024_between20 계열)

**잔여 7건**: #2, #6, #7, #10, #11, #12, #13. **대부분 `2023_sep`**(samples/3-09월_교육_통합_2023.hwp:
#2,6,10,11,12,13) + #7(2024_between20 page18). snap-off 로 **값 불변**(#11 q23 title y=799.44 동일)
→ 별개 root 의 cascade. Stage 3 대상.

## 4. 계획 대비 구조 정정

Stage 1 의 a-priori 그룹(A/B/C)은 실증 root 구조로 대체: **(i) snap-inflation cascade(6, 해결)** vs
**(ii) 2023_sep 잔여 cascade(7)**. 후속 Stage 는 (ii) 를 다룬다.

## 5. 다음 단계

Stage 3 — `2023_sep` 잔여 cascade 진단·해소. snap-off 와 무관한 별도 root(예: 2023_sep 특유의
TAC/그림 미주 흐름 또는 다른 게이트). 전 exam 동시 green 게이트 유지.

## 6. 승인 요청

Stage 2(6/13 해결, 무회귀) 검토 및 커밋 승인. 승인 시 Stage 3(2023_sep 잔여 7건) 착수.
