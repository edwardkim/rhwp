# 구현계획서 — Task M100 #2003: 라운드 4 (B 블록 RunEmitState + GSO 내부 분해)

- 이슈: #2003 / 수행계획서: `task_m100_2003.md` / 작성일: 2026-07-06
- 1단계 의존 재실측 결과 반영.

## 1. 의존 재실측 결과

### ① B 블록 (run 방출 루프, paragraph_layout.rs 2786~3841, 1,056줄)
- **mut 캐리오버 10종 확정** (라운드 2의 11에서 정제 — line_idx는 오탐, x/y는 길이-1
  필터 누락분 복원): `x`(대입 8) `y`(1) `baseline` `raw_lh` `run_char_pos`
  `inline_tab_cursor_render` `pending_right_tab_render` `pending_right_leader_digit_render`
  `current_line_reserved_tac_picture_height` (이상 Copy 스칼라 9종) + `char_x_map`(Vec, 1종).
- 읽기 ~40종 (오탐 제거 후): 참조군(tree/line_node/comp_line/composed/para/styles/cell_ctx/
  bin_data_content/tab_stops/tac_offsets_px/shape_markers/fn_positions 등) + 스칼라군
  (alignment/available_width/effective_margin_left/line_height/max_fs/... ~20종).
- 캐리오버-out: base_style/eq_node/form_node 등 후보 전건 **재선언 오탐** — 실 캐리오버는
  mut 10종뿐.

### ② ch==10 블록 (hwp3/mod.rs, parse_object_control_char 내부)
- 블록 1: 511~1290 (**780줄**, 표/글상자/수식/버튼 본체). break/continue 전부 내부 루프
  대상(중첩0 = 0). **조기 return 15곳**(라운드 3 치환 산물) + 후속 코드로 넘기는 캐리오버
  4종(`parsed_drawing_object`/`parsed_obj_type`/`parsed_is_hypertext`/`info_buf`).
- 블록 2: 1327~1496 (170줄, 후속 처리). 조기 return 0 — 단순 이동 가능.

## 2. 추출 설계

### 2단계 — ② `parse_hwp3_drawing_object` (블록 1) + `parse_hwp3_drawing_object_tail`(블록 2)
- 블록 1 반환 설계 (조기 return 15곳의 전파):
  `Result<Option<(usize, u32, bool)>, Hwp3Error>` — `Some` = 조기 종료(호출자가 그대로
  return), `None` = 통과(후속 코드 진행). 캐리오버 4종은 `&mut` 파라미터.
- 블록 2는 동일 시그니처 계약(조기 return 없음 → 값 반환만).
- `Hwp3CharScan` 재사용 + 파라미터군(doc_*, 기하)은 라운드 3 계약 그대로.

### 3단계 — ① `emit_line_runs` + `RunEmitState`
- **커밋 1 — struct 도입**: 
  ```rust
  /// [#2003] run 방출 루프의 줄-간 캐리오버 묶음 (Copy 스칼라 9종).
  #[derive(Clone, Copy)]
  struct RunEmitState {
      x: f64, y: f64, baseline: f64, raw_lh: f64,
      run_char_pos: usize, inline_tab_cursor_render: usize,
      pending_right_tab_render: Option<(f64, u8, u8)>,
      pending_right_leader_digit_render: bool,
      current_line_reserved_tac_picture_height: Option<f64>,
  }
  ```
  값 전달 + 반환(라운드 3 검증 패턴) — 함수 진입 destructure(`let RunEmitState { mut x, .. }`)
  와 말미 재조립으로 **본문 무변경 이동** 보장. `char_x_map`은 `&mut Vec` 별도.
- **커밋 2 — 본 추출**: `for (run_idx, run)` 루프 전체를 `emit_line_runs(...) -> RunEmitState`로.
  읽기 스칼라 ~20종은 `RunEmitVars`(Copy 묶음, EmptyRunsLineVars 전례) + 참조 ~12개.
  루프 통째 이동이라 내부 break/continue는 자체 루프 대상 — 제어 흐름 수술 불요
  (외부 line 루프 대상 break/continue 존재 여부는 착수 시 중첩 스캔으로 최종 확인).
- **v2 §1 가드**: 인접 소스분기(is_hwpx_source 계열)가 루프 내부에 있으면 해당 서브구간을
  경계 밖으로 — 착수 시 소스분기 위치 재확인(라운드 2 실측: 루프 내 0곳이었음, #1927
  유입분 재확인 필요).

## 3. 게이트/완료 기준

수행계획서 §3와 동일 (OVR "추가 변동 0" — 착수 전 기지 시그니처 재확인). 예상 효과:
`layout_composed_paragraph` 226 → **~150 이하** (B 블록이 분기 밀도 최대 구간),
`parse_object_control_char` 104 → **~40대**.
