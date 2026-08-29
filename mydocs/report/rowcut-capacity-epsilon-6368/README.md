# #6368 — 표 행 컷 기본 용량 비교의 +0.5px 경계 관용 부재

`advance_row_cut_inner` / `advance_row_block_cut`의 **기본 용량 컷**만 경계 관용이 0이라,
부동소수 합산 끝자리(0.07px대) 초과만으로 마지막 줄이 다음 쪽으로 이월되어 한글과 줄
소유 쪽이 어긋났다. 같은 함수의 이웃 특례(atomic 진입·trailing trim)와
`advance_row_block_cut_with_row_offsets`의 흡수 판정은 전부 `+ 0.5`를 쓴다.

## 수정

- `advance_row_cut_inner` 기본 컷: `h + u.height > avail_height` → `> avail_height + 0.5`
- `advance_row_block_cut` 기본 컷: 동일 (+0.5)
- 근인 추적용 `RHWP_DIAG_6368`(near-miss 컷 로그, ≤2.0px) 추가

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
