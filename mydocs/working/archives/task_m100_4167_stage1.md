---
kind: working
status: completed
issue: 4167
last_verified: 2026-08-08
---

# Task #4167 Stage 1 - 셀 캐럿 rect fast path (페이지 트리 미빌드)

## 구현

- `get_cursor_rect_in_cell_native`를 fast→legacy 폴백 래퍼로 재구성, 기존 본문은
  `cursor_rect_in_cell_via_page_tree`로 추출해 패리티 기준으로 삼았다.
- fast path 는 "재구현"이 아니라 **생략 프로브** — production `layout_partial_table`를
  대상 셀 하나만 방출하도록 재사용한다(`PartialTableCellProbe`/`ProbeCutPlan`/
  `CellComposedStore` lazy compose). 셀 방출 루프는 셀-간 캐리가 없어(코드 명시 계약)
  생략이 좌표를 바꾸지 않는다 — 패리티가 구성적으로 보장되는 부분을 최대화했다.
- 셀 bbox/페이지 원점: pagination `PartialTable` 메타(행 범위·유닛 컷) + memoized
  `cell_units` + `measured_tables` + `PageContent.layout` col_area. 컬럼 선두 아이템 +
  단일 컬럼 + zone/wrap/미주 부재 게이트로 `y = col_area.y` 확정.
- 줄 찾기: 저장 line_segs 불신(#4149 — 무편집 ~16% divergence) — production compose 경로
  그대로(compose→recompose_for_cell_width→overflow 가드)를 컷 창 문단에만 lazy 적용,
  캐럿 문단 이후 중단(Top 정렬 게이트로 안전).
- 폴백 12종(캡션 센티널/다중컬럼/zone/wrap/미주/통페이지 표/TAC/반복 제목행/auto counter
  개체(표 포인터 키 메모)/rowspan>1/windowed 증명 실패/run 미발견) — 전부 legacy 전체 경로로.

## 검증 결과

- 패리티(fast=Some 전 지점 legacy JSON `assert_eq!`): 거대 문서 **185/185 (100%)**,
  hwp_table_test_saved 40/222, KTX 3/371 (저적중 원인은 컬럼 선두 PartialTable 전용 게이트 —
  폴백 시 출력은 정의상 legacy 동일).
- `--lib cursor_rect` 16 passed, 캐럿 전반 84 passed, getCursorRect 사용 통합 테스트 전부
  (issue_1951/2164/2214/4126/4128/table_vpos_01/850/2069/1071/1308) pass, **전체 lib 3303/0**.
- 적대 리뷰(별도 세션): ① 캐시 무효화 전수 추적(clear 호출자 2곳 — full paginate·deferred
  commit) 반박 실패, ② 프로브-전량 동치(셀-간 mutable 필드 한 줄씩 검사 + 줄 경계 ±1 offset
  11,962지점 차등) 반박 실패, ③ **편집 직후 일관성(최유망 각도)**: legacy 도 같은
  `self.pagination` stale 메타를 읽는 대칭 구조 — deferred insert 직후 861지점·delete 직후
  50지점·paginate 후 34지점 전부 일치, ④ 패리티 그리드 구멍 공격(추가 실문서 3종 — 다단+미주
  0/259 게이트 정상, 보건소 208/286=73% 적중 — 전부 일치), ⑤ 게이트 역손실 +0.6%(노이즈).
  CONFIRMED 0건.
- 실측: `get_cursor_rect_in_cell_native` 17.5ms → **0.61ms** (약 28배), 웜 지연 어서션
  `issue4149_fast_path_giant_cell_warm_latency_beats_legacy` 0.619ms/call.
- 계측 재현물: `issue4149_adjacent_giant_cell_cursor_rect_latency_decomposition`(페이즈 분해),
  `issue4149_adjacent_cursor_rect_profile_loop`(--ignored, macOS sample 용),
  `issue4149_cell_lineseg_roundtrip_survey`(저장 line_segs 신뢰성 — composed 사용 설계 근거).
- 알려진 한계: 세로쓰기 셀 실물 샘플 미확보(구조상 windowed 는 `text_direction != 0` 게이트
  차단, Uncut 은 전량 compose 와 문자 그대로 동일 경로). 향후 게이트 완화 시
  `reset_numbering_state` + replay 가 전제조건.
