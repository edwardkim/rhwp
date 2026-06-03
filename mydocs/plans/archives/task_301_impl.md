# Task #301 구현계획서: z-table 셀 수식 이중 렌더링 수정

- **이슈**: #301
- **브랜치**: `local/task301`
- **수행계획서**: [task_301.md](./task_301.md)

## 1. 원인 분석 결과 (수행계획서 가설 B 확정)

### 셀 구조
페이지 12 좌측 컬럼 #29 안의 z-table(pi=27 ci=0)은 5×2 = 10개 셀이다. 각 셀은:
- `text=""` (빈 텍스트)
- `paras=1`, `ctrls=1` (수식 1개)
- 셀 안의 숫자/헤더(`0.5`, `0.1915`, `z`, `P(0≤Z≤z)` 등)는 **모두 Equation(수식) 컨트롤로 저장**되어 있음

### 중복 경로
셀 내 **빈 runs 문단의 TAC 수식**이 두 경로에서 렌더링된다:

1. **paragraph_layout.rs:1996-2057** (Task #287 추가)
   - `comp_line.runs.is_empty() && !tac_offsets_px.is_empty()` 조건에서 인라인 EquationNode emit
   - 이후 `tree.set_inline_shape_position(section_index, para_index, tac_ci, inline_x, eq_y)` 호출

2. **table_layout.rs:1602-1670** (Task #287 이전부터 존재)
   - `Control::Equation(eq)` 분기에서 `has_text_in_para = false`(빈 텍스트)일 때 직접 EquationNode emit
   - **paragraph_layout이 이미 그렸는지 검사하지 않음** → 중복 발생

### 중복 검증
```
SVG 출력 (line 297-352):
<g clip-path="url(#cell-clip-119)">
  <g transform="translate(350.44,417.93)..."><text>0.5</text></g>  ← paragraph_layout
  <g transform="translate(352.95,416.62)..."><text>0.5</text></g>  ← table_layout
</g>
```
두 좌표의 차이(Δx≈2.5, Δy≈-1.31)는 두 경로의 좌표 계산 방식 차이에서 비롯됨.

### 영향 범위
- 동일 패턴(빈 텍스트 셀 + Equation 컨트롤) 가진 모든 표에서 발생
- Task #287 도입 이전에는 paragraph_layout이 빈 runs 케이스를 처리하지 않아 table_layout 경로만 그렸으므로 중복 없음 → **#287 회귀 버그**

## 2. 수정 방향

`table_layout.rs:1602` 의 Equation 분기에서 paragraph_layout이 이미 렌더했는지 확인하고 중복을 회피한다.

판별 수단: `tree.get_inline_shape_position(section_index, cp_idx, ctrl_idx)`
- paragraph_layout의 두 경로(line 1689, 2052)는 모두 렌더 후 `set_inline_shape_position`을 호출
- 따라서 호출 결과가 `Some`이면 이미 그려진 것으로 판단하여 직접 렌더를 스킵

스킵해야 할 케이스 (이미 그려짐):
- 비-빈 runs + TAC equation (line 1645)
- 빈 runs + TAC equation, line_segs 존재 (line 2011, Task #287)

스킵하면 안 되는 케이스 (paragraph_layout이 안 그림):
- line_segs가 없거나 composed.lines가 비어 paragraph_layout이 호출조차 안 된 경우 → 기존 직접 렌더 경로 유지

## 3. 단계 분할 (4단계)

### Stage 1: 회귀 검증용 테스트/스크립트 추가
- `samples/exam_math.hwp` 페이지 12를 기반으로 z-table 수식 중복을 검출하는 검증 스크립트 또는 통합 테스트 작성
- 검증 방법: SVG에서 `0.1915`, `0.3413`, `0.4332`, `0.4772`(또는 cell-clip 그룹별 EquationNode 수)의 출현 횟수가 정확히 1이어야 함
- 통합 테스트로 추가 가능한지 검토 (기존 `tests.rs` 또는 신규 테스트)
- **수정 전에 테스트가 실패함을 확인** (RED 단계)
- **단계별 보고서**: `mydocs/working/task_m_301_stage1.md`
  - (CLAUDE.md의 표준 형식은 `task_{milestone}_{이슈}_stage{N}.md` 이지만 마일스톤 미지정이므로 `task_301_stage{N}.md` 형식 사용 — 작업지시자 확인 필요)

### Stage 2: 본 수정
- `src/renderer/layout/table_layout.rs:1602` Equation 분기 수정
- 추가 조건: `tree.get_inline_shape_position(section_index, cp_idx, ctrl_idx).is_some()` 이면 직접 렌더 스킵 (`inline_x += eq_w` 만 진행)
- 기존 `has_text_in_para` 분기는 보조 조건으로 유지하거나 통합 (paragraph_layout이 안 그리는 코너 케이스 보존)
- **단계별 보고서**: `mydocs/working/task_301_stage2.md`

### Stage 3: 회귀 확인 (다른 샘플)
- `cargo build --release` + `cargo test`
- `samples/` 내 비슷한 패턴(셀 안 수식, 다단 표 등) 가진 문서 SVG 출력 비교:
  - `exam_math_8.hwp`, `exam_math_no.hwp` 페이지 전체 export 후 시각/grep 비교
  - 추가로 1~2개 샘플 선정 회귀 확인
- **단계별 보고서**: `mydocs/working/task_301_stage3.md`

### Stage 4: 최종 정리
- 코드 cleanup (불필요해진 `has_text_in_para` 변수 정리 여부 결정)
- 최종 결과보고서 `mydocs/report/task_301_report.md` 작성
- orders 갱신 (오늘할일 yyyymmdd.md)
- **이슈 close는 작업지시자 승인 후에만 수행**

## 4. 위험 요소

- **paragraph_layout이 "그리지 않는" 코너 케이스**: line_segs가 비어 있는 경우. 이 경우 set_inline_shape_position도 호출되지 않으므로, 현재 제안 방식이 올바르게 fallback함
- **다른 컨트롤 타입(Picture, Shape)에도 동일 중복 가능성**: 현재 Picture는 1437의 `will_render_inline` 검사로 일부 처리하나 Task #287 빈-runs 케이스는 미처리 가능. 본 타스크 범위에는 Equation만 다루고 다른 타입은 별도 이슈로 분리 권장.
- **WASM canvas 경로**: `web_canvas.rs`도 동일 RenderTree를 사용하므로 SVG 수정으로 동시에 해결될 가능성. RenderTree 자체에서 노드가 1개만 생성되도록 수정하는 본 접근이 두 출력 경로 모두에 효과 있음.

## 5. 승인 요청

본 구현계획에 대한 작업지시자 승인 요청. 단계 수(4)와 단계별 산출물에 대한 의견 부탁드림.
