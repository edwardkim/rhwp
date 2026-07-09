# 최종 결과보고 — #1921 RowBreak 블록컷 sliver 흡수 (59043 48→42)

브랜치 `fix/1921-rowbreak-sliver-absorb` (base: devel). 수행계획: [task_m100_1921_float_table.md](../plans/task_m100_1921_float_table.md) 트랙 B.

## 1. 근본원인 (RHWP_TABLE_DRIFT 계측 확정)
`59043_규제영향분석서.hwp`(rhwp 48 vs 한글 37, +11)의 pi=160(3×3 rowspan 블록, 법령 원문 셀 104·77문단)에서 **컷 진동**: fragment 소비가 `946px → 22px → 946px → 22px` 교대. 컷 좌표로 확증 — 셀 유닛 전진 `+46, +1, +46, +1`.

**기전**: `advance_row_block_cut_with_row_offsets`(블록컷 walk)가 **예산**(page budget)에 걸려 정지한 지점 **직후 수십 px 뒤에 저장 hard-break**(한글이 실제로 페이지를 넘긴 지점)가 있으면, 다음 fragment가 그 극소 잔여만 담고 hard-break에 걸려 **22px sliver가 페이지 하나를 통째로 소비**. 한글은 자기 그리드에서 그 잔여를 앞 페이지에 담았음(저장 break 위치가 증거).

## 2. 수정
`absorb_tail_before_stored_hard_break` helper 추가: 예산 정지 유닛부터 다음 hard-break 유닛까지 잔여가 **48px 이내**면 흡수(fragment가 저장 break 정확히에서 끝남). **`advance_row_block_cut_with_row_offsets` 경로에만 적용** — 이 walk는 hard-break가 무조건 fragment 경계라 흡수가 정합적. `src/renderer/layout/table_layout.rs`.

**스코프 교훈**: 처음에 형제 walk(`advance_row_cut`/`advance_row_block_cut`)에도 적용했으나 **86712 공식PDF 핀 회귀(65→66)** — 그 walk들은 `relaxed_hard_break`(hard-break 조건부 무시) 의미론이라 다음 break로의 흡수가 비정상 경계를 강제. with_row_offsets 한정으로 좁혀 회귀 해소(#1926 반례-정제 방법론).

## 3. 검증
| 항목 | 결과 |
|---|---|
| 59043 | **48 → 42** (한글 37, +11→+5). pi=160 컷 `+47,+47,+47` 정상 보폭 |
| 콘텐츠 무손실 | export-text 대조 −24자 = 감소 6쪽의 꼬리말 쪽번호(4자×6)뿐, 본문 동일 |
| 86712 공식PDF 핀 | **65 유지** (issue_1891 ok) |
| 전체 테스트 | **2946 / 0** |
| A/B 2,500 | **변화 0** (발동조건 극희소) |
| 픽스처 | 156714340=4·1790387=141 불변 |

## 4. 잔여
59043 +5(42 vs 37)는 sliver 아닌 배치 밀도(2단 패킹) 축 — 트랙 A(부동 표 콘텐츠 미측정, 156714340)와 함께 후속.
