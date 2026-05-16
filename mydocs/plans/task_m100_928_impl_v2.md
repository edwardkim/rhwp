# Task #928 구현 계획서 v2

## 변경 사유

v1 의 Stage 3 fix (Shape 분기 will_render_inline 가드) 로 텍스트 중복 회귀는 해소되었으나, Stage 4 시각 검증 단계에서 한컴 2022 PDF 와 비교하여 **2 건의 잔존 회귀** 발견:

1. **사각형 크기 / 내부 폰트** — 사각형 [A 단계] 가 한컴 대비 좁고, 내부 텍스트 폰트가 6.88px (정상 15.33px). Fix 전후 동일 회귀이므로 기존 별도 회귀. shape size scaling 로직 (Task #874 #3) 또는 사각형 IR width 결정 단계의 문제 가능.
2. **㉢ 그림 중복 emit** — `p[2]` 의 그림 3개 (ctrl[0..2]) 중 ctrl[2] (㉢ 격자) 가 두 다른 x 위치 (376.02, 432.02) 에 그려짐. IR 에는 그림 3개인데 SVG 에 image 4개. Picture 분기 `will_render_inline` 가드의 boundary 조건 (`abs_pos < line.char_start + line_chars`) 이 paragraph 마지막 위치에 있는 컨트롤을 통과시키지 못해 paragraph_layout + table_layout Picture 분기 양쪽에서 emit.

작업지시자 결정: 본 task 안에서 시각 정합까지 완료.

## Stage 구성 (v2, 총 6 stages)

| Stage | 상태 | 내용 |
|-------|------|------|
| 1 | ✅ | 정밀 재현 + 텍스트 중복 ROOT CAUSE |
| 2 | ✅ | 코드 trace + Fix 방향 (Shape 가드) |
| 3 | ✅ | Fix 구현 (table_layout.rs Shape 분기 가드) |
| 4 | ⚠️ | 시각 검증 — 텍스트 중복 ✅, 잔존 회귀 2건 발견 |
| **5** | **진행 예정** | **잔존 회귀 ROOT CAUSE + Fix 구현 + 단위 회귀 차단** |
| **6** | 예정 | 시각 검증 (한컴 PDF) + 최종 보고서 + 오늘할일 갱신 |

## Stage 5 세부 계획

### 5.1 잔존 회귀 정밀 측정 (코드 수정 전)

**㉢ 그림 중복**:
- 사각형 fix 의 가드 식 (`abs_pos < line.char_start + line_chars`) 을 Picture 분기 (라인 1698) 도 동일하게 사용 → 마지막 inline 컨트롤이 paragraph 끝에 있을 때 가드 미통과 가설 검증
- p[2] 의 `composed.tac_controls` 좌표와 `composed.lines[0].char_start + line_chars` 값 비교 (RHWP_DEBUG 또는 trace 추가)
- paragraph_layout 의 `run_tacs` filter (`is_last_run && *pos == run_char_end` 포함) 와 비교 → boundary 처리 차이 확인

**사각형 크기/폰트**:
- 사각형 ShapeCommon 의 `width`, `height`, `original_width`, `original_height`, `current_width`, `current_height` IR 값 확인 (dump 확장 또는 trace)
- `shape_layout.rs:1206-1223` 의 ratio 축소 로직 (Task #874 #3) 발동 여부 측정 (max_ratio 값)
- paragraph_layout 의 `tac_offsets_px` 에 reserve 된 gap 폭과 IR width 비교
- 한컴이 그리는 사각형 폭과 IR width 의 매칭 (paragraph_layout 의 inline shape 처리에서 width 결정 단계)

### 5.2 Fix 방향 결정

측정 결과에 따라:

**㉢ 그림 중복 Fix**:
- 가드 식의 boundary 조건 정정: `abs_pos < line.char_start + line_chars` → `abs_pos <= line.char_start + line_chars` (또는 paragraph_layout 의 `is_last_run && *pos == run_char_end` 와 정합되는 조건)
- Shape 분기 (v1 fix) 와 Picture 분기 둘 다 동일 패턴 적용
- 단위 회귀 차단: 기존 가드가 false 였던 케이스 (Picture inline at last position) 에서 paragraph_layout 이 실제로 emit 했는지 확인

**사각형 크기/폰트 Fix**:
- ratio 축소 로직 오작동 시: 셀 내부 inline shape 에 대한 가드 추가 (`treat_as_char=true && wrap=TopAndBottom` 케이스 ratio 검사 비활성화)
- IR width 결정 단계 문제 시: `src/parser/hwp3/` 안에서 보정 (CLAUDE.md 규칙)
- gap width 불일치 시: paragraph_layout 의 reserve gap 폭과 IR width 의 정합 보정

### 5.3 구현 + 단위 회귀 차단

- `cargo build --release` clean
- `cargo test --release` 전체 통과
- svg_snapshot 회귀 0건 (변경 시 작업지시자 승인 후 갱신)

→ 단계 보고서 `task_m100_928_stage5.md` 작성, 승인 요청

## Stage 6 세부 계획

### 6.1 시각 검증

- `samples/exam_kor.hwp` 5쪽 다이어그램 행 한컴 2022 PDF (`pdf/exam_kor-2022.pdf`) 와 시각 일치 확인
- 다이어그램 3 요소: `(가) ⇨ [A 단계] ⇨ (나)`, 사각형 폭/내부 폰트 정상, ㉠ ㉡ ㉢ 그림 3개만 적정 위치
- 다른 HWP3/HWP5/HWPX 샘플 회귀 검사 (Stage 4 와 동일 범위)

### 6.2 최종 보고서

`mydocs/report/task_m100_928_report.md`:
- 회귀 3건의 원인, 각 fix, 검증 결과
- 영향 범위 + 회귀 차단 패턴 정착 노트
- 향후 잔존 가능 회귀 (다중 shape 혼재 will_render_inline 케이스 등) 노트

### 6.3 오늘할일 갱신

`mydocs/orders/20260516.md` 에 Task #928 완료 갱신.

→ 최종 승인 요청, merge 준비

## 위험 요소 (v2 갱신)

| 위험 | 가능성 | 영향 | 완화 |
|------|--------|------|------|
| 가드 boundary 조건 변경이 다른 회귀 유발 | 중 | 중 | svg_snapshot + 다중 샘플 검증 (Stage 6) |
| 사각형 크기/폰트 fix 가 다른 글상자 케이스 회귀 | 중 | 중 | Task #874 의 참조 케이스 (shortcut.hwp 등) 시각 회귀 검사 |
| HWP3 파서 수정 필요 시 다른 포맷 영향 | 낮음 | 중 | CLAUDE.md 규칙대로 `src/parser/hwp3/` 안에서만 |
| 다중 shape 혼재 케이스 잔존 (v1 의 위험 그대로) | 낮음 | 중 | 본 task 비범위, 발견 시 별도 이슈 |

## 비범위 (v1 동일)

- 본문 paragraph (셀 외부) 동일 케이스 (별도 코드 경로)
- 다중 shape 혼재 will_render_inline=true/false 시나리오 (이론적 잔존 회귀, 관측 사례 없음)
- HWP3 외 포맷 동일 케이스 (별도 이슈로 분리)
