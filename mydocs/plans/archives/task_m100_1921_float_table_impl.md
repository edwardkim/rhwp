# 구현계획서 — #1921/#2004 트랙 A: 부동 표 콘텐츠 페이지네이션 (156714340)

- 수행계획서: `mydocs/plans/task_m100_1921_float_table.md` (트랙 B sliver 는 PR #2092 완료, 59043 48→42)
- 브랜치: `local/task1921-float-table-pagination` (base: local/devel 통합 = origin/devel 27b3461f R11 + PR #2073/#2082/#2086/#2088/#2090/#2092)
- 선행 RCA: `mydocs/tech/investigations/issue-2004/floating_object_family_rca.md` — 이미지/셀 레벨 단독 수정 3종 무효 배제, 유효 = 표 레벨 결합

## 0. 실측 기준선 (2026-07-09, 통합 브랜치)

- 156714340.hwp = **4쪽** (한글 8쪽). pi=42(1×1 RowBreak 부동 표, 셀에 전면 Square 이미지 5장 스택, varying offset 0/−3360/−2940…):
  - `TABLE_DRIFT: eff_h=871.9 table_total=871.9 mt_sum=871.9 mt_rows=1 cur_h=40.8 avail=944.9` → `typeset.rs:13558` fit 게이트 통과, **원자 배치·분할 스캔 미진입**(TABLE_SPLIT_RESULT 없음), 이미지 5장 겹침.
- 59043 = 42쪽 (한글 37, PR #2092 반영) — 본 타스크 스코프 아님(잔여 +5 = 2단 배치 밀도, 별건).

## 1. 결함 기전 (3경로 연동)

1. **측정**: `height_measurer.rs` 셀 content 측정(`:1201~1317`)에서 Square 부동 이미지는
   `non_inline_control_flow_height`(`:285`, TopAndBottom 만 카운트) = 0,
   `cell_wrap_objects_bottom_height`(`:1316`) = max(bottom) ≈ 이미지 1장(스택 겹침) → 셀 required ≈ 871px = 저장 높이.
2. **배치**: `typeset_block_table`(`typeset.rs:13558`) — `table_total(871.9) ≤ available(944.9)` → `place_table_with_text` 원자 배치. 분할 머시너리(`scan_block_table_split_rows`) 미진입.
3. **분할 불가**: 부동 이미지는 flow 에 없어 진입해도 intra-cell 컷이 fragment 로 못 나눔.

## 2. 수정 설계 — 정규화(재분류 N분할) + 셀 측정 정합 결합

3차 실증에서 재분류 N분할 **단독** 무효 원인 = 분할된 빈-텍스트 inline 이미지 문단이 셀 composition 에서
placeholder 1줄(≈400HWPU)로 붕괴 → 측정 871 불변. 본문 경로에는 줄합성 선례가 이미 있음(`rendering.rs:3108~3133`,
line_height=이미지 높이). **셀 경로에 동일 원리를 적용**한다:

- (A) `compute_render_normalized`(`rendering.rs`) 정규화 확장 — 셀 재귀 스택 검출 + 이미지 1장/문단 N분할 재분류.
  스택 판정 `para_is_floating_image_stack` 은 동일-offset 요구를 **겹침 band**(offset spread ≤ 이미지 높이)로 완화
  (156714340 varying offset 대응). ※ 이전 세션 실증 코드 stash 보존분 재사용
  (`stash: invalidated: #2004 재분류 셀재귀 실험`, fix/2004-float-table-content 기준).
- (B) 분할된 각 셀 문단에 **합성 line_seg(line_height=이미지 높이 HWPU)** 를 부여(`line_segs.clear()` 대신)
  → 셀 측정 `text_height` 가 스택 총높이(≈5×880px)를 자연 반영 → `required_height > combined` 로 행높이 팽창
  → `table_total ≈ 4400 > available` → RowBreak 분할 스캔 **자연 진입**.
- (C) 분할 진입 후 intra-cell 컷(cell_units)이 이미지 문단 단위로 fragment 분배되는지 검증,
  렌더(table_partial)가 fragment 별 이미지 1장을 배치하는지 확인·정합.

원본 IR 무손상(정규화본 전용, save 무결) 원칙 유지. 수정 스코프는 스택 검출 문단에 한정(비스택 표 무영향).

## 3. 단계

### Stage 1 — 계측·측정 실증
- stash 복원(재분류 셀재귀 + RHWP_2004DBG 계측), 셀 content_height/text_h/non_inline_h 실측 로그 확보.
- (B) 합성 line_seg 를 정규화 분할 문단에 부여, TABLE_DRIFT 로 `mt_sum ≈ 4400` 도달 확인 (배치 변경 없이 측정만 검증).
- 산출: 측정 정합 수치 + stage1 보고서.

### Stage 2 — 분할 진입·컷 분배
- fit 게이트 초과 → `scan_block_table_split_rows` 진입 확인(TABLE_SPLIT_RESULT), fragment 경계가 이미지 문단 단위인지 검증.
- 진입 후 과분할/스퓨리어스(1차 실증의 +1 spurious 재발) 시 컷 유닛 정합 수정.
- 검증: 156714340 4→8(±1), `--debug-overlay`/export-png 시각으로 쪽당 이미지 1장.

### Stage 3 — 렌더 정합
- table_partial 렌더가 fragment 별 이미지를 올바른 y 에 1장씩 그리는지 확인·수정(겹침 소거).
- 검증: 시각 오라클(visual_oracle_native.py, 한글 PDF 대조 `output/poc/survey10k_0708/visual/156714340*.hangul.pdf`).

### Stage 4 — 회귀
- `cargo test` 전체(기준 2946/0), 10k 표본 A/B 2500(변화 문서 전수 판독), 부동표 문서군 핀(59043=42 유지, 86712=65 유지, 1790387=141 유지 등).
- 산출: A/B 무회귀 보고.

### Stage 5 — 최종 보고서·PR
- `mydocs/report/task_m100_1921_float_table_report.md`, fork 브랜치 push + edwardkim:devel PR.

## 4. 리스크

- 측정 팽창이 스택 아닌 셀에 누출되면 광범 과분할 → 스택 검출 술어(count≥2 ∧ 전면급 높이 ∧ 겹침 band)로 3중 게이트, 정규화본에만 적용.
- 분할 스캔 진입 자체는 기존 SSOT(#993/#1022/#1025 계층) 재사용 — 스캔 코드 수정 최소화, 진입 조건은 측정치로만 변경.
- Stage 2에서 컷 유닛 정합이 대형화되면 정직하게 중간 경계(측정 진입까지) 보고 후 승인 재요청.
