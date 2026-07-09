# 최종 결과보고 — #1921/#2004 트랙 A: 부동 표 셀 이미지 스택 콘텐츠 페이지네이션

브랜치 `local/task1921-float-table-pagination` (base: local/devel 통합 = origin/devel R11 + 열린 PR 6건).
수행계획: [task_m100_1921_float_table.md](../plans/task_m100_1921_float_table.md) 트랙 A ·
구현계획: [task_m100_1921_float_table_impl.md](../plans/task_m100_1921_float_table_impl.md).
선행 RCA: [floating_object_family_rca.md](../tech/floating_object_family_rca.md) (PR #2090).

## 1. 문제

`156714340`(rhwp 4쪽 vs 한글 8쪽): pi=42가 1×1 RowBreak **부동 표**(wrap=자리차지, tac=false)이고
셀 안에 전면급 Square 부동 이미지 5장이 varying offset(0/−3360/−2940…)으로 스택. 부동 이미지는
flow 미예약이라 셀 측정이 저장 높이(871.9px)에 머물고, `typeset.rs` fit 게이트를 통과해 **원자
배치** — 분할 스캔 미진입, 이미지 5장이 한 쪽에 겹침. 선행 실증 3종(cell reserve/place 캐스케이드,
재분류 셀재귀 단독, 셀문단 분할 단독) 전부 무효 — blocker는 셀 measurement 붕괴에 있었다.

## 2. 수정 (rendering.rs 단독, +88/−8, 정규화 전용 — 원본 IR 무손상)

3차 실증 무효 원인(분할된 빈-텍스트 inline 이미지 문단이 셀 composition에서 placeholder 1줄로
붕괴)을 해소하는 **결합 수정**:

1. **스택 판정 겹침-band 완화** — `para_is_floating_image_stack`의 동일-offset 요구를
   "offset spread ≤ min 이미지 높이"(서로 크게 겹침)로 완화. 세로로 벌어진 정상 배치는 제외.
2. **셀 재귀 재분류 N분할** — `reclassify_cell_floating_stacks` 신설: 표 셀 내 스택 문단을
   이미지 1장짜리 inline(tac=true) 문단 N개로 분할 (`compute_render_normalized`에서 적용).
3. **합성 line_seg** — 각 분할 문단에 `line_height=이미지 높이(HWPU)` line_seg 부여 → 셀 측정
   `text_height`가 스택 총높이를 자연 반영(`corrected_line_height`는 raw≥font size면 원값 유지).

측정이 참값(871.9→4310.6px)을 보고하자 **분할 스캔·컷·렌더는 무수정**으로 기존 RowBreak 분할
SSOT(#993/#1022/#1025)가 정합 동작: fragment 5개(각 843~870px), 컷 경계 [1]~[4] = 이미지 문단
단위, 쪽당 상이한 이미지 1장씩 렌더.

## 3. 검증

| 항목 | 결과 |
|---|---|
| 156714340.hwp | **4 → 8쪽 = 한글 8** (fragment당 이미지 1장, 겹침 소거) |
| 156714340.hwpx (쌍둥이) | **8쪽** 동반 수정 |
| 렌더 | p4~p8 각 `<image>` 1개, 5장 전부 상이(MD5), 한글 권위 PDF 구조 일치 |
| 콘텐츠 무손실 | export-text 전후 2,460자 동일 |
| **A/B 10,000 전수** | **changed=1**(=156714340.hwpx, 목표문서), err=0 — band 완화 누출 0 |
| A/B 2,500 | changed=1 동일 |
| 전체 테스트 | **2948 / 0** (219 스위트) |
| 핀 | 59043=42 · 1790387=141 · 86712(issue_1891) green · 1430000=3 유지 |

산출물: `output/poc/task1921_floattable/` (A/B 바이너리·tsv·SVG·한글 PDF 래스터).

## 4. 잔여 (본 타스크 스코프 외)

- **#1921/59043 +5** (42 vs 37): 2단 배치 밀도(부동 표 흐름 패킹) 축 — 별도 과제.
- 콘텐츠 보유 일반 부동 표(이미지 스택 아닌 텍스트 셀)의 콘텐츠 재측정·분할은 미착수 —
  본 수정은 전면 이미지 스택 셀(판정 술어 발동)에 한정된 bounded 경로.
