# #6368 — 표 행 컷 기본 용량 비교의 +0.5px 경계 관용 부재

`advance_row_cut_inner` / `advance_row_block_cut`의 **기본 용량 컷**만 경계 관용이 0이라,
부동소수 합산 끝자리(0.07px대) 초과만으로 마지막 줄이 다음 쪽으로 이월되어 한글과 줄
소유 쪽이 어긋났다. 같은 함수의 이웃 특례(atomic 진입·trailing trim)와
`advance_row_block_cut_with_row_offsets`의 흡수 판정은 전부 `+ 0.5`를 쓴다.

## 수정

- `advance_row_cut_inner` 기본 컷: `h + u.height > avail_height` →
  `> avail_height + ROW_CUT_CAPACITY_FP_EPSILON_PX` (**0.1px**)
- `advance_row_block_cut` 기본 컷: 동일 (+0.1px)
- 근인 추적용 `RHWP_DIAG_6368` 추가 — 컷 로그(≤2.0px)와 흡수 로그(관용 안 near-miss)

### 관용 폭을 0.5 → 0.1px 로 좁힌 경위 (CI 실패 수리)

최초 제출본은 이웃 특례(atomic 진입·trailing trim)와 같은 +0.5px 를 그대로 썼다.
그 폭은 부동소수 끝자리만이 아니라 **실제 경계 초과**까지 삼켜 CI 확정 게이트 2개를 깼다:

| 실패 게이트 | 흡수된 초과분 | 증상 |
|---|---|---|
| `text_overlap_baseline::text_overlaps_do_not_grow` | `table_giant_cell_overfill.hwpx` 0.1867px | 글자 겹침 18 → 19건 (확정 결함 래칫) |
| `issue2439_row_orphan_guard_uses_padded_visible_fragment_height` | 픽스처 0.4px | remarks 셋째 줄이 현재 쪽으로 흡수돼 고아 가드 계약 위반 |

동기 사례의 실측 초과분은 hwpctl_API_v2.4 **0.0267px**, 80168_regulatory **0.0133px**
(RHWP_DIAG_6368 흡수 로그) — 잡음대(≤0.03px)와 실초과(≥0.19px) 사이 0.1px 로 좁혀
동기 흡수 2건은 그대로 유지하고 회귀 2건은 다시 컷한다. 0.1px 적용 후 재실측:
두 동기 문서의 흡수 이벤트가 +0.5 때와 동일(각 1건, 같은 컷 지점)이므로 위 fidelity
측정은 그대로 유효하고, giant_cell 겹침은 18건(baseline)으로 복귀, issue2439 통과.

## 측정 (fidelity_compare --text-only --layout-ledger, 한컴 2022 정답지)

| 문서 | 총쪽수(전/후) | 쪽 경계 표류(전→후) | 해소 경계 | 신규 표류 |
|---|---|---|---|---|
| hwpctl_API_v2.4.hwp (105쪽) | 105 = 105 | 9 → 8 | p12→13 | **0** |
| 80168_regulatory_analysis.hwp (157쪽) | 157 = 157 | 23 → 21 | p121→122, p122→123 | **0** |

해소 3경계 모두 "rhwp가 한글보다 늦게(다음 쪽으로) 실은" 이월형이며, set-diff로 두 문서에서
**새로 생긴 표류 0건**을 확인했다.

## 블라스트 반경 (한 줄 epsilon)

- 핀 회귀: `issue_3931`(5) · `issue_3930`(3) · `issue_5828` · `issue_6307` · `overflow_cell_baseline` — **11/11 통과**
- 편람(최종).hwp **384=384**, .hwpx **382=382** (render-tree 쪽수 전/후 동일)
- fmt / clippy 통과

## 증적 (BEFORE devel | AFTER 이 PR | 한컴 2022 PDF)

| 파일 | 내용 |
|---|---|
| `hwpctl_API_v2.4_p12_before_after_ref.png` | Example 코드 상자 마지막 줄 `tbset.SetItem("Cols", 5);` — BEFORE 소실(이월), AFTER 한컴 일치 |
| `hwpctl_API_v2.4_p13_before_after_ref.png` | p12에서 넘어온 고아 코드줄이 AFTER에서 사라짐 |
| `80168_regulatory_analysis_p121_before_after_ref.png` | 표 9번 항목 마지막 줄("토지를 확보하는…우선") — BEFORE 이월, AFTER 한컴 일치 |
| `80168_regulatory_analysis_p122_before_after_ref.png` | p121 이월분 연쇄 해소 |
| `80168_regulatory_analysis_p123_before_after_ref.png` | p122→123 경계 정렬 복원 |

## 남은 것 (별개 후속)

같은 문서의 잔여 표류(api 8건·reg 21건)는 near-miss 계측(RHWP_DIAG_6368) 결과 이
두 컷 지점을 지나지 않는다 — 다른 층(고정 하드브레이크·상위 페이지 예산)의 별개 근인으로,
이 PR 범위 밖.
