# Task M100 #2136 최종 결과보고서

**이슈**: #2136 — PI TAIL_PUSH/PULL 136건 (10k r12 최대 PI 계열)
**브랜치**: `local/task2136` (base: local/survey10k-r12) / **기간**: 2026-07-10

## 1. 요약

136건을 서브버킷으로 분해(stage1 보고서 §1)한 결과 단일 원인이 아닌 5개 이상 계열의
혼합으로 확인. 이번 태스크는 **명확한 결함 1건을 표적 수정**하고 잔여를 재분류 인계:

- 수정: `native_near_top_reset` 상한 2000→2500HU — sb=2500HU 문단의 저장 새쪽-리셋이
  500HU 차로 배제되어 측정 fit 과적(148753276: used 942 > body 933.6px)되던 결함.
- 효과: PUSH+TEXT 3건 MATCH 전환. 광역 359 재검 REGRESSED 0·SAME_OK 251, 게이트 전건.

## 2. 재분류 인계 (이슈 코멘트 게시)

| 잔여 계열 | n | 다음 행동 |
|---|---|---|
| 누적 근하단 텍스트 (PUSH+TEXT 잔여 주력) | ~30 | rhwp=저장 일치, 한글 재실측 이탈 — **시각 확증 선행** (visual sweep 교차) 후 방향 결정 |
| PULL+EMPTY (캐럿 오탐성) | 21 | 오라클 판정 개선: rp=hp+1 방향 빈 문단도 CARET 분류 + 시각 표본 확증 |
| TABLE_ANCHOR+LS | 42 | 캐럿 한계(#1757-(1)) 혼재 — 판별 로직 확장 검토 |
| PULL+TEXT / TITLE_BEFORE_TABLE / NOLS | 16/12/9 | 소계열, 후속 |

## 3. 산출물

- 소스: typeset.rs 1건(주석 포함). 검증 TSV·스캐너: `output/poc/survey10k_r12_0709/`
  (tailpush_scan/empty/pushtext39_after/pushtext_numbers).
- PR: upstream/devel 기준 단독 cherry-pick 제출.

## 5. 추기 (2026-07-10, TABLE_ANCHOR 재검 편입)

신판정 오라클(표 앵커 캐럿 2형)로 TABLE_ANCHOR 44건 재검: CARET 분리 13 / 해소 1 /
**잔존 30건 실차이 풀 편입** (PUSH+CTRL 15 / PULL+CTRL 14 / PUSH+TEXT 1). 시각 확증
혼재(PUSH 대표 worst 77.9% vs PULL 대표 91.3%) - 규칙 확장 보류, 실차이로 취급.
갱신 잔존 풀 약 103건. 산출: tableanchor44_recheck.tsv / tableanchor_residual30.tsv.

## 6. 추기 2 (2026-07-10, PUSH+TEXT 시각 확증)

36건 시각 스윕: **26건 시각 정합**(worst>=85%, 누적근하단 캐럿/재실측 무영향 확증) /
실차이 코어 6건(76.8~84.4%) / PAGEDIFF 4건(PDF 아티팩트 확인 필요). 잔존 실차이 풀
~103 -> 약 73건. 산출: pushtext36_vis.tsv.

## 7. 추기 3 (2026-07-10, 실차이 코어 6건 분해)

5/6건에서 시각 worst 페이지 = 불일치 경계(진성 실차이). 대표 156659503: rhwp는 저장
lineseg(첫 줄 p1 하단 fit)에 충실, 한글 재실측이 저장보다 커서 통째 p2 - **한글
재실측-저장 이탈(폰트 메트릭 정합, #2110-(a) 축)**로 귀속, 장기 과제 표본으로 보존.
#2136 잔여는 TABLE_ANCHOR 편입 30 + PULL+TEXT 16 + 소계열로 재분류 종결.
