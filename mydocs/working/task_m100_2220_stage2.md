# Task M100 #2220 — 2·3단계 완료 보고: tac 호스트 줄 outer_margin 이중 계상

- 이슈: #2220 / 브랜치: `local/task2220` / 작성일: 2026-07-12

## 2단계 — 가산 지점 특정 (계측 분해)

임시 계측(커밋 제외)으로 단 항목 y_offset 전이를 분해했다:

- Table pi=6 항목이 단 상단 60.47에서 시작해 **409.05로 전진(+348.58px =
  26,144HU)** — 정확히 `다음 문단 저장 vpos 24440 + outer_margin 상하 1704`.
- 경로: ①표 시작 전에 om_top 선가산(60.47→71.83) ②`tac_seg_applied` 음수 ls
  분기(Task #9)가 `tac_table_y_before`(=om_top 포함 기점)에서 저장 lh+ls
  (24440HU)를 advance ③#521 처리가 om_bottom(852HU)을 후가산.
- 저장 host lh 24700 = 표 22996 + om 상하 1704 — **om이 이미 lh에 포함**되어
  ①+③이 이중 계상(+22.7px). 다음 문단은 `vpos_adjust`의 `vpos==0` 바이패스
  (:202, prev 문단 vpos=0)로 저장 vpos 스냅을 받지 못해 오차가 단 전체에 전파.
- 두-경로 확인: typeset 회계(used=686.1)는 정상 — layout 방출 측 단독 결함.

## 3단계 — 정정 ([layout.rs](../../src/renderer/layout.rs) tac_seg_applied 분기)

`stored_lh_covers_om` 증거 가드 신설: 음수 ls 분기에서 **저장 lh ≥ 표 선언높이
+ om 상하합**(±10HU)이고 om 합 > 0이면 —

1. advance 기점을 `para_y_for_table`(문단 줄 상단)로 — om_top 선가산분 제거.
2. #521 om_bottom 후가산 생략 — 저장 lh에 이미 포함.

증거 미충족(전형적 Fixed 줄간격 TAC 표: lh < 표높이, om=0)은 기존 경로 불변 —
Task #9/#521 캘리브레이션 보존. 하드코딩 없음(판정 근거는 저장 lh·표 높이·om
스펙 필드).

## 정량 효과 (p1 우측 단, 한컴 대조 — 측정 오프셋 ≈ +11)

| 항목 | 수정 전 | 수정 후 |
|------|--------|--------|
| 문단 0.7("▮▮기도나눔") baseline | 427.3 (+26.2) | **405.x (기대 405)** |
| 우측 단 델타 min/med/max | +26.7/+30.7/+41.0 | **+4.0/+12.7/+21.0** |
| 마지막 줄("최한나…함솔이") | ≈760 (하단 절단) | **+9.3 (실질 ~−2, 완전 포함)** |

## 표적 테스트 + 게이트

- `tests/issue_2220_tac_host_line_outer_margin.rs` 신설 — 수정 전 **FAILED
  (427.3)** 실증 / 수정 후 ok.
- fmt 통과 / clippy 0 / `--tests --no-fail-fast` **3,048 / 실패 0**
  (#2211 좌측 단·#1994·golden 8/8 전부 포함)
- OVR 5샘플 분리 폴더 회귀 0건

## 시각 판정 자산

`output/poc/issue2220/compare_right_bottom.png` — 우측 단 하단 3-way
(한컴/수정 전/수정 후). 마지막 줄 "최한나…함솔이" 완전 복원.
