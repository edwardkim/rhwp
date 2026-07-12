# 최종 결과보고서 — Task M100 #2173: advance_row 계열 워크 통합 (2차 R2)

- 이슈: #2173 (#2131 서브) / 2026-07-10~11.

## 총량 판정 (v2.1, --diff 분해)

| 축 | 수치 |
|---|---|
| **라운드 기여** | table_layout.rs **−13** (기대 −15~25 하한 근접) — 통합 walk `advance_cells_cut` CC 18, 원함수 2개는 wrapper 화 |
| 유입 (동시간 원격 머지) | +25 (text_editing +8 / ooxml_chart +7 / composer +6 / rendering +4) |
| 전역 총합 | 11,699 → 11,722 (**+23**) — 라운드 순감을 유입이 상회 |

## 수행

- `advance_row_cut`(194) + `advance_row_block_cut`(152) → `advance_cells_cut` 1벌 +
  `RowCutPolicy` 4필드. **drift 정확 재현**: orphan rewind 호출 방식 차이(조건부 force=true
  vs 무조건 force=false — rewind(false)가 no-op 아님 확인)를 `orphan_force: Option<bool>` 로.
- 게이트 전수 통과: 테스트 3,035/0 · rowbreak 표적 5핀 10/10 · issue_1116 · OVR 5/5 · clippy 0.
- 차순위 실사(선평가 기록): shape_props 쌍 = 함수 내 반복 유형(위임 불가),
  diff_page_def CC 13(스캐너 ② 과대평가 — 후속 보정 소재).

## 사이클 관찰 (R1 −2, R2 −13)

기계적 유형 ①의 라운드 소득(−2~−13)이 **일일 유입(+25)에 못 미친다** — 총량 축의
구조적 해답은 유형 ④(소스분기 = Provenance/Profile) 착수라는 산술이 두 라운드로 실증됨.
사이클 지속 여부는 작업지시자 판단 사항.
