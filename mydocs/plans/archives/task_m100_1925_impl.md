# 구현계획서 — Task M100 #1925: layout_composed_paragraph 해체 (라운드 2)

- 이슈: #1925 / 수행계획서: `task_m100_1925.md` / 작성일: 2026-07-05
- 1단계(의존 정밀 분석) 결과를 반영한 추출 설계. 검증 방법: 단어 경계 매칭 + 실사용
  라인 검수(클로저 파라미터/주석/재선언 오탐 배제) — 라운드 1 방식.

## 1. 의존 분석 결과 (1단계 산출)

| 블록 | 크기 | 읽기 의존 | mut 쓰기 | 캐리오버-out | 판정 |
|---|---|---|---|---|---|
| E ClickHere (4627~4949) | 323줄 | 11 | 2 (`line_node.children` push, `x += shift`) | **0** (child/guide는 오탐) | **추출 1 — 최적** |
| A est 폭 추정 (2283~2556) | 274줄* | 23 | 0 (누산기 6개는 블록 산출물) | 누산기 6개 → **반환 struct화** | **추출 2** |
| C 빈 runs (4204~4377) | 174줄 | 24 | 1 (`current_line_reserved_tac_picture_height`) | **0** (char_shape_id는 필드 리터럴 오탐) | **추출 3** |
| B run 방출 (2994~4049) | 1,056줄 | **45** | **11** (y·char_x_map·baseline·raw_lh 등) | 다수 | **이연** — 라운드 1 축소 전례(32/9)와 동급 이상. `RunEmitState` struct 설계 선행 필요 → 다음 라운드 입력 |

\* A는 누산기 선언(2283~2290)을 블록에 포함해 추출 — 선언이 함수 내부로 들어가고
반환 struct 필드가 된다.

CC 기여 추정(분기 지표, 함수 전체 605): E 44 + A 46 + C 27 = **117 (~19%)** →
예상 CC 288 → **~230**, 줄수 3,771 → **~3,000**. B(128)는 이연이므로 이번 라운드
목표는 "1위 함수의 안전한 1차 감량 + B 해체의 설계 입력 확보"다.

## 2. 추출 설계

### 추출 1 (수행계획 2단계) — `layout_click_here_markers`
- 원본: 4627~4949 (ClickHere 필드 안내문/조판부호 마커, `if let Some(p) = para` 블록).
- 시그니처(안): `fn layout_click_here_markers(&self, tree: &mut PageRenderTree, line_node: &mut RenderNode, para: &Paragraph, comp_line: &ComposedLine, char_x_map: &..., styles: &ResolvedStyleSet, <컨텍스트 소수>, x: &mut f64)`
  — 읽기 11개는 개별 파라미터로 수용 가능(라운드 1의 endnotes 10개 전례 내).
- 소스분기 0. 동작 불변 확인: 게이트 전수.

### 추출 2 (수행계획 3단계 전반) — `estimate_line_run_widths` + 반환 struct
- 원본: 2283~2556 (est 사전 폭 추정 패스).
- 반환: `struct LineWidthEst { est_x: f64, est_x_start: f64, pending_right_tab: Option<(f64,u8,u8)>, pending_right_leader_digit: bool, run_char_pos: usize, included_tac_width: f64, inline_tab_cursor: usize }`
  (누산기 6개 + est_x_start — 이후 코드가 이 값들을 소비).
- 읽기 23개 중 self 경유·comp_line 계열을 정리하면 파라미터 ~10개 수준 예상.
  12개 초과 시 `LineEstContext` 참조 struct 도입(설계 여지).

### 추출 3 (수행계획 3단계 후반) — `layout_empty_runs_line`
- 원본: 4204~4377 (빈 TextRun 생성).
- mut 1개는 `&mut Option<f64>` 파라미터 또는 반환값으로 처리.

각 추출 = 1 커밋 (수행계획 "1 추출 단위 = 1 PR" — 내부 타스크이므로 브랜치 내 커밋 단위로
적용, 라운드 1과 동일).

## 3. 이연 기록 (다음 라운드 입력)

- **B run 방출(1,056줄, 의존 45/mut 11)**: `RunEmitState` mut-묶음 struct(y, char_x_map,
  baseline, raw_lh, pending 탭 상태, tac 예약 높이) 설계 선행 — `EndnoteFlowState`(라운드 1
  이연)와 같은 패턴. 내부에 서브블록 3297~4043(747줄)이 단일 `if run_tacs.is_empty()`
  덩어리라 재분할 이득도 제한적 → struct 설계가 정공법.
- 루프 중반 2557~2993(정렬/줄배경 준비, 분기 지표 109, 소스분기 0): brace 단일 블록이
  아닌 문장 연속 구간 — 의존 폭 실측 후 다음 라운드 후보로 평가.
- D(tac 개체 라인, 분기 1곳 4444): 수행계획대로 이번 라운드 제외.

## 4. 게이트 (매 추출 공통)

수행계획서 §4와 동일 — fmt/clippy 0, 전체 테스트 FAILED 0, OVR 5샘플 무변동
(기준 00014ecf). 추출 간 병행 금지, 각 추출 후 커밋.

## 5. 완료 기준

- 추출 3건 게이트 전수 통과 + `layout_composed_paragraph` 실측 재계측(CC/줄수).
- 4단계 재평가: 대시보드 스냅샷 `-r2`, 대비표, 다음 라운드 제안(B struct 설계 포함).
