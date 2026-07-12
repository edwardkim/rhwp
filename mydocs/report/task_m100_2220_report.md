# Task M100 #2220 최종 보고 — BehindText+tac 표 호스트 줄 outer_margin 이중 계상

- 이슈: #2220 (#2211 진단에서 분리) / 브랜치: `local/task2220`
- 기간: 2026-07-12 / 시각 판정: **통과** (한컴 2022 PDF 3-way)
- 재현: `samples/basic/issue1994_behindtext_table_20200830.hwp` p1 우측 단

## 결론

빈 문단이 호스트인 tac 표(글뒤로·RowBreak)에서 outer_margin 상하합(1704HU=
22.7px)이 흐름 전진에 이중 계상되어 우측 단 전체가 +29px 하강, 마지막 줄이
페이지 하단에서 절단되던 결함을 **저장 lh 증거 가드**로 정정. 우측 단 델타
+26.7~+41 → +4~+21, 마지막 줄("최한나…함솔이") 완전 복원.

## 원인 (계측 분해로 산식 단위 확정)

- 저장 host lh **24700 = 표 선언 22996 + om 상하 852×2** — 한컴은 호스트 줄이
  곧 표 박스 (om 포함).
- 우리 전진 = ①om_top 선가산 기점(+852) → ②`tac_seg_applied` 음수 ls 분기
  (Task #9)가 그 기점에서 저장 lh+ls(24440HU) advance → ③#521 om_bottom
  후가산(+852). 합계 26,144HU — 저장 흐름(24440) 대비 +1704HU 정확 일치.
- 다음 문단은 `vpos_adjust`의 vpos==0 바이패스(prev 문단 vpos=0)로 저장 vpos
  스냅 미적용 → 오차가 단 전체 전파. typeset 회계는 정상(두-경로 확인) —
  layout 방출 단독 결함.

## 정정 ([layout.rs](../../src/renderer/layout.rs), `06c2ace8`)

`stored_lh_covers_om`(저장 lh ≥ 표 높이 + om 상하합 ±10HU, om>0) 충족 시:
advance 기점을 문단 줄 상단(`para_y_for_table`)으로 + #521 om_bottom 후가산
생략. 증거 미충족(전형 Fixed 줄간격 TAC: lh<표높이, om=0)은 기존 경로 불변 —
Task #9/#521 캘리브레이션 보존. 하드코딩 없음.

## 게이트 + 검증

| 항목 | 결과 |
|------|------|
| fmt / clippy | 통과 / 0 |
| `--tests --no-fail-fast` | **3,048 / 실패 0** (#2211·#1994·golden 8/8 포함) |
| 표적 테스트 신설 `tests/issue_2220_tac_host_line_outer_margin.rs` | 수정 전 FAILED(427.3) → ok |
| OVR 5샘플 (분리 폴더) | 회귀 0건 |
| 시각 판정 | **통과** — `output/poc/issue2220/compare_right_bottom.png` |

## 주보 재현 파일 아크 결산 (4건 완결)

#2189(성명서 자간) → #2207(픽토그램 앵커) → #2211(중첩 표 행높이+가사) →
#2220(우측 단 om). p1/p2 시각 결함 전부 한컴 정합. 잔여: #2212(편집 bbox),
#2221(상위 float 표 pad), #2222(렌더 캐시), #2206(폰트 메트릭).

## 부수 기록

작업 중 `output/poc/task2004·task2019` 추적 파일 5개 삭제 상태 발견·복원
(원인 스크립트 미특정, 게이트 재발 없음 — 커밋 전 status 확인으로 감시).

## 산출물

- 커밋: `bd725335`(계획), `06c2ace8`(정정+테스트+2·3단계 보고)
- 문서: `working/task_m100_2220_stage2.md`, 본 보고서
