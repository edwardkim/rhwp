# Stage 3 분류 보고서 — #1046: 잔여 12건(in-scope) overflow 분류

- 타스크: #1046 (M100), 브랜치 `local/task1046`
- 단계: 측정 통일(B) Stage 3 — 잔여 overflow 분류 (소스 수정 전, 진단만)
- 작성일: 2026-05-21
- baseline: Stage 2 후 총 16 = in-scope 12 + page-larger 2(pi=323, pi=567).
  (Stage 2가 in-scope 14→12, pi=242/256 해소.)

## 분류 (RHWP_TABLE_DRIFT TABLE_SPLIT_AVAIL/RESULT + RHWP_TYPESET_DRIFT)

### Class A — 첫 fragment 강제배치 (Stage 2 계열, fits=false) · 1건
- **pi=290** (sec0, 8.7px): `cur_h=913.3 page_avail=23.8 avail_for_rows=23.8`,
  행0=38.6 > avail → `fits=FALSE` 인데 배치. Stage 2 가드 발동 직전(`remaining_on_page
  23.8` vs `min_content 23.76`, 0.04px 차)에서 `first_row_force_splittable` 예외가
  표를 붙잡아 분할 불가 행을 통째 강제. host_before=4.0, vert_off=0.0.
- 원인: 가드의 force-split 추정(pad+min(line_h,20)≈20)이 실제 비분할 행과 어긋남.
  #874(1×1 거대 셀 force-split)와의 간섭 주의 필요.

### Class B — 통째 표 측정/host_spacing 드리프트 (fits=true, 렌더러 초과) · 4건
- pi=266(7.2), pi=308(11.6), sec1 354(8.3), 357(10.0).
- 예 pi=308: `cur_h=548.9 host_before=4.0 page_avail=388.3`, eff_h=382.5 → `fits=true`
  (548.9+4.0+382.5=935.4<941.1)인데 렌더러는 표를 ~11.6px 더 그려 초과.
- TABLE_DRIFT상 eff_h=mt_sum이나 렌더러 실측이 7~12px 큼 → 통째 tac 표의 행높이
  (resolve_row_heights) 또는 host_spacing 적용이 페이지네이터 측정과 어긋남(추가 조사 필요).

### Class C — 연속 PartialTable 소드리프트 (fits=true, ~2-3px) · 2건
- pi=218(2.2), pi=600(2.7): 두 fragment 모두 `fits=true`인데 렌더러가 2-3px 초과.
  잔류 누적 드리프트(≈1.9px 문단 누적/반올림 계열). 가장 작고 난해.

### Class D — 문단 분할 줄단위 드리프트 · 5건
- pi=361(4.3, PartialPara), 429(10.7, FullPara), 781(15.8, FullPara),
  sec1 268(12.3, PartialPara), 406(3.1, FullPara).
- TYPESET_DRIFT_PI: 페이지네이터 fmt_total이 vpos_h보다 **과다**(diff +6.9~+24.3,
  trailing_ls #359 정책)인데도 분할 문단 마지막 줄이 본문 초과 → 표 경로 무관,
  partial-paragraph 줄단위 배치(렌더러 line baseline/lh) 드리프트.

## 권고 (Stage 3 수정 타깃 후보)

- **A (pi=290)**: Stage 2 직접 연장. 가드의 force-split 추정을 실제 분할가능성과 정합
  (비분할 행이면 whole-row 기준으로 이월). #874 회귀 가드 필수. 효과 1건/8.7px.
- **B (4건)**: 통째 tac 표 height 드리프트 근원(resolve_row_heights vs cut/eff,
  host_spacing) 추가 진단 후 정합. 효과 4건/7~12px. 중간 난도.
- **C (2건)**: 누적 소드리프트. 효과 작고(2-3px) 난해 — 후순위.
- **D (5건)**: 문단 분할 줄단위 — 별도 서브시스템. 표 작업과 분리 권장.

A는 Stage 2와 동일 메커니즘이라 가장 자연스러운 다음 타깃. B는 표 통일(B 목표)에 부합.
C/D는 효과/리스크상 후순위 또는 별도 이슈.
