# 최종 보고서 — Task M100 #2003: 1차 리팩토링 라운드 4 (B 블록 + GSO 디스패치)

- 이슈: #2003 (계획 #1883 v2, umbrella #1582, 선행 #1904·#1925·#2001) / 브랜치: `local/task2003`
- 기간: 2026-07-06 / 거버넌스: v2 전 조항 준수 / 작업지시자 확정 범위 ①+②

## 1. 결과 요약 (공식, 스냅샷 `2026-07-06-r4/`)

| 함수 | 이전 | 이후 |
|---|---|---|
| ① `layout_composed_paragraph` | CC 226 (전체 1위) · 3,088줄 | **CC 146 (−35%) · 2,093줄** |
| ② `parse_object_control_char` | CC 104 · 1,039줄 | **CC 37 · 289줄** |
| 전체 최대 CC | 226 | **179** (`typeset_section_endnotes`) |

**전체 1위 함수 4라운드 연속 해소**: 282→104, 288→226, 234→76, 226→146.
영점(288) 대비 최대 CC **−38%**. 행동 회귀 통산 **0건** 유지 (테스트 2,912/0 · OVR 추가
변동 0 · 매 추출 전수 게이트).

## 2. 산출물

| 신규 | 크기 | CC | 내용 |
|---|---|---|---|
| `RunEmitState`(8필드)+`RunEmitVars`(21필드) | struct | — | run 방출 캐리오버 값-왕복 / 읽기 스칼라 묶음 |
| `emit_line_runs` | 1,107줄 | 81 | run 방출 루프 통이동 (라벨 break 0·return 클로저 1곳 전수 확인) |
| `Hwp3DrawingCarry`(9필드) | struct | — | GSO 캐리오버 (디스패치가 채우고 후속이 소비) |
| `parse_hwp3_object_dispatch` | 812줄 | 68 | 개체 컨트롤 if-else 체인 통이동 (ch 10/11/14~17/29/5~8) |

커밋: `66265fba`(분석·계획) → `be694851`(② 디스패치) → `0ed5c329`(struct 도입, 분리 커밋)
→ `897b2d00`(① B 블록) → 본 보고서.

## 3. 정정 기록 (계획 대비, 전건 컴파일러/정밀 스캔 검증)

- **경계 정정(②)**: "ch==10 블록"은 실측상 개체 디스패치 if-else 체인 전체 — 통이동으로
  단일 책임 정합.
- **캐리오버 정정(①)**: baseline/raw_lh는 eprintln 포맷 문자열 오탐(읽기 전용, Vars로) /
  char_offset(+= 누적)·fn/shape_marker_inserted(인덱스 대입 — 정규식 사각지대) 추가.
  → **의존 스캔 도구의 알려진 사각지대 2종**(포맷 문자열, 인덱스 대입)을 다음 라운드
  분석 절차에 반영할 것.

## 4. 재평가

- CC>25: 84 → 86 (+2 과도기 — 상위 함수 2개 해소와 신규 4함수 경계 진입의 교환,
  v2 §5 허용). 최대 CC 179로 **처음으로 200 미만** 진입.
- 차기 1위 = `typeset_section_endnotes`(179, 라운드 1 산물) — `EndnoteFlowState` 설계
  (후보 ③)와 자연 연결.

## 5. 다음 라운드 후보

1. `typeset_section_endnotes`(179, 현 1위) — `EndnoteFlowState` 설계 (#1904 이연분)
2. `table_partial.rs::layout_partial_table`(163) — 표 계열 첫 진입
3. `emit_line_runs`(81)/`parse_hwp3_object_dispatch`(68) 내부 2차 분해
4. typeset 재성장 흡수(Phase P 연동) + `parse_paragraph_list` 후처리 `Hwp3FlowState`

## 6. 산출물 위치

계획 `plans/task_m100_2003{,_impl}.md` / 단계 보고 `working/task_m100_2003_stage{2,3}.md` /
스냅샷 `mydocs/metrics/2026-07-06-r4/`
