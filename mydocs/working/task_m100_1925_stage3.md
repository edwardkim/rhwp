# 단계 완료 보고 — Task M100 #1925 3단계: 추출 2·3 (est 폭 추정 + 빈 runs 줄)

- 작성일: 2026-07-05 / 브랜치: `local/task1925`

## 수행 내용

### 추출 2 — `estimate_line_run_widths` (A 블록, 274줄)
- est 사전 폭 추정 run 패스(누산기 선언 포함)를 추출. 누산기 7종 중 **하류 소비는
  3개뿐**(est_x, est_x_start, included_tac_width_in_est)임을 실측 — est_x_start는 입력
  (`est_x_init`)으로 넘기고 **반환 struct `LineWidthEst` { est_x, included_tac_width } 2필드**로
  축소 (구현계획서의 7필드 안 대비 간결화, 소비 실측 근거).
- 읽기 의존은 주석 전용 가짜 의존 5개(col_area/end/endnote_marker_x_advance/inline_offset/
  tac_offsets_px)를 배제하니 **실의존 12개** — 계획의 struct 도입 임계(>12) 이내라 직접
  파라미터로 확정. 이동 조정: `tab_stops.clone()`→`to_vec()`(슬라이스 수령), `line_idx`
  파라미터 추가(1차 스캔 누락분, 컴파일러 검출).

### 추출 3 — `layout_empty_runs_line` (C 블록, 174줄)
- 빈 runs 줄 처리(빈 TextRun 생성 + 셀 외부 빈 줄의 TAC 이미지/Shape 인라인 렌더링) 추출.
- 실의존 28개로 임계 초과 → **줄-스코프 스칼라 18개를 `EmptyRunsLineVars` struct**로 묶고
  참조 9개 + mut 캐리오버 1개(`current_line_reserved_tac_picture_height: &mut Option<f64>`)를
  파라미터로. `para_topbottom_line_vpos_base`는 `.is_some()`만 쓰여 bool 필드로 축약.

두 추출 모두 로직 무변경(코드 이동 + 기계적 참조 치환). 커밋은 한 단계 1커밋으로 묶음
(구현계획서 "1 추출 = 1 커밋" 대비 편차 — 같은 단계 내 연속 작업으로 게이트는 결합 상태
1회 검증, 이하 결과).

## 게이트 결과 (전수 통과)

| 게이트 | 결과 |
|---|---|
| cargo fmt --check | 통과 |
| cargo clippy --profile release-test --all-targets | **경고 0** |
| cargo test --profile release-test --tests | **2,875 통과 / 실패 0** |
| OVR baseline 5샘플 (기준 00014ecf) | **5/5 개체 회귀 0건** |

## 계측 (라운드 2 누적)

| 함수 | 줄수 | 분기 지표(자체 추정) |
|---|---|---|
| `layout_composed_paragraph` | 3,771 → 3,467 → **3,071** (−700, −18.6%) | 605 → **490** (−115, −19%) |
| `layout_click_here_and_bookmark_markers` (신규) | 331 | 43 |
| `estimate_line_run_widths` (신규) | 292 | 46 |
| `layout_empty_runs_line` (신규) | 182 | 26 |

## 다음 단계

4단계 — 중간 재평가: 대시보드 스냅샷(`-r2`) + 공식 CC 재계측 + 대비표 + 다음 라운드
범위 제안(B run 방출 `RunEmitState` struct 설계 포함) + 최종 보고서. 승인 후 착수.
