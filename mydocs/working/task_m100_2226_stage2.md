# Task M100 #2226 — 2·3단계 완료 보고: 셀 내 겹침 배치 다중 그림 flow 회계

- 이슈: #2226 / 브랜치: `local/task2226` / 작성일: 2026-07-12

## 2단계 — 계측 귀속 (임시 계측, 제거 완료)

| 팽창 | 실체 |
|------|------|
| r0 +62.4px | 셀[2] 측정 `measure_non_inline_controls_height`가 TopAndBottom 그림 2개 flow를 **직렬 합산**(112.8px) — 저장 ls 사다리(줄 vpos 51.3 = 그림 아래)가 이미 배치를 반영 |
| r1 +60.7px | rowspan 셀[0] wrap bottom이 **para_top(밀린 줄 vpos 134.5) + 그림 bottom(134.5) 이중 계상**(269px) → 2-b가 마지막 행에 deficit 배분 |

추가 발견: 팽창 해소 후에도 붓글씨 그림이 소실 — 그림 앵커(#577 공식)가 밀려난
줄 vpos를 기점으로 잡아 축소된 셀 클립 밖으로 이탈 (제3 결함 축).

## 3단계 — 정정 3건 + 캘리브 가드 (수렴 3회 실측)

1. **wrap bottom para 기점** ([table_layout.rs](../../src/renderer/layout/table_layout.rs)
   / [height_measurer.rs](../../src/renderer/height_measurer.rs) 대칭): "개체가
   문단 시작~줄 상단을 채운"(prev_extent + obj_bottom ≤ first_vpos) 문단은
   사다리 기반 문단 시작 사용 — TopAndBottom 한정 1차 가드가 Square 로고를
   놓쳐 기하 판정으로 일반화.
2. **측정 저장 흐름 신뢰**: ladder가 개체 밀림을 흡수한 증거(텍스트-빈 문단 +
   첫 vpos>0 + 비인라인 개체 보유)가 있는 셀만 content = stored extent.
   반례 캘리브 2건을 게이트로 확정: KTX TOC(개체 없음 — additive가 한컴 쪽,
   1차 가드에서 golden 3종 실패) / #1282 쪽영역제한 ON(텍스트 문단 vpos 0 —
   한컴이 그림만큼 행 성장, 2차 가드에서 실패) → 증거 조건 3중 가드로 수렴.
3. **그림 앵커 원점**: 밀림-빈문단(트림 빈 텍스트 + vpos>0)의 비인라인 그림
   앵커 = 문단 시작 (top_and_bottom/overlay/Square 공통) — 붓글씨(향린교회)가
   한컴과 동일 위치(셀 상단)로 복원.

하드코딩 없음 — 판정 근거는 저장 LINE_SEG/텍스트 유무/wrap·voff 스펙 필드.

## 정량 효과 (p2 로고 표)

| 항목 | 전 | 후 | 한컴 |
|------|-----|-----|------|
| r0 / r1 | 131.3 / 141.5px | **69.7 / 80.8px** | 68.9 / 80.8 (선언) |
| 표 하단 | 865.3 (페이지 밖) | **743.0** | ≈743 |
| 주소 블록 3줄 | 소실 | **복원** | 표시 |
| 향린교회 붓글씨 | 위치 어긋남 | **셀 상단 정합** | 셀 상단 |

## 표적 테스트 + 게이트

- `tests/issue_2226_cell_flow_pictures_overlap.rs` 신설 — 수정 전 **FAILED
  (272.7px)** 실증 / 수정 후 ok (행높이 + 하단 + 주소 3종 방출 검증).
- fmt 통과 / clippy 0 / `--tests --no-fail-fast` **3,049 / 실패 0**
  (golden 8/8 무변동, #1282/#1486/#1748/#1858/KTX/exam_kor/복학원서/주보 아크)
- OVR 5샘플 분리 폴더 0건 / 추적 파일 삭제 재발 없음

## 시각 판정 자산

`output/poc/issue2226/compare_footer_3way.png` — 로고 표 3-way (한컴/전/후).
