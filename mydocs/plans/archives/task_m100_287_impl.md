# 구현 계획서: Task #287

## 타스크

[#287](https://github.com/edwardkim/rhwp/issues/287) — 수식 SVG 레이아웃: 큰 디스플레이 TAC 수식이 줄 상단(y=col_area.y)으로 올라감 (exam_math_8)

- 브랜치: `local/task287`
- 선행: `mydocs/plans/task_m100_287.md`

## 접근 방침 확정 (수행계획 (A)/(B) 중 선택)

사전 조사 결과:

- `compose_paragraph` (`src/renderer/composer.rs:113-177`) 가 `para.line_segs.vpos` 를 **완전히 버림**. `ComposedLine` 구조체에 vpos 필드 자체가 없음 (L44-52).
- 일반 문단의 y 갱신은 **순수 누적** (`paragraph_layout.rs:2219` 부근: `y += line_height + line_spacing_px`). vpos 는 마스터페이지에서만 사용됨.
- ls[1].line_height(4095 HU ≈ 54.6 px) 는 `corrected_line_height` 통과 후에도 보존됨 (폰트보다 커서 그대로).
- `has_tac_shape` 조건(L716-729) 이 `Control::Shape` 만 검사하여 TAC Equation 제외 → `text_y = y` 유지 → 큰 수식이 줄 상단에 고정.

→ **(A) 근본 수정은 파이프라인 전반(composer + layout + render tree) 수정이 필요 → 위험·범위 과대**. 본 타스크는 **(B) 우회 수정**으로 진행한다. vpos 전파 개선은 별도 이슈로 분리 가능.

## 단계 구성 (4단계)

---

### 단계 1 — 파이프라인 실측 및 수정 지점 확정

**목적**: `exam_math_8.hwp` 문단 0.0 의 `ComposedLine` 배열이 실제로 어떻게 나오는지, 큰 수식이 어느 `comp_line` 에 속하는지를 확정한다. 수정 전에 가설을 데이터로 검증.

**작업**
1. `compose_paragraph` 결과를 디버그 출력하는 임시 로그 또는 기존 `dump` 확장 활용.
2. 확인 항목:
   - `composed.lines` 개수 = ls 개수인가?
   - 큰 수식이 속한 comp_line 의 `line_height`, `baseline_distance`, 포함 runs/tac
   - 해당 라인에서 `max_fs`, `raw_lh`, `has_tac_shape`, `tac_w` 값
   - 단계 1 완료 시점의 현재 y 누적 값
3. 수정 지점 확정:
   - `paragraph_layout.rs:716-720` `has_tac_shape` 판정 확장 필요성
   - `paragraph_layout.rs:721-729` line_height/baseline 보정 공유 여부
   - `paragraph_layout.rs:744-751` text_y 시프트 공유 여부
   - `paragraph_layout.rs:1584-1618` eq_y / eq_h / x 계산 수정 지점

**산출물**: `mydocs/working/task_m100_287_stage1.md` (실측 덤프 결과 + 수정 지점 결정)

**완료 기준**: 큰 수식이 기록된 comp_line 인덱스, line_height(px), y 누적값이 수치로 확정됨.

---

### 단계 2 — 빈 runs comp_line 의 TAC 인라인 처리 추가

> **단계 1 결과로 방침 조정됨.**
> 원안(`has_tac_shape` 조건 확장)은 폐기. 실제 원인은 빈 `runs` 줄에서 TAC 인라인 분기가 실행되지 않는 것이었다.

**목적**: `comp_line.runs` 가 비어있는 줄(큰 수식이 단독으로 차지하는 ls) 에서도, 그 줄이 소유한 TAC 수식이 인라인 경로로 렌더되도록 한다.

**작업**
1. `paragraph_layout.rs` comp_line 루프에서 **이 줄의 char 범위** `[line_start, line_end)` 를 산출.
   - `line_start = comp_line.char_start`
   - `line_end = composed.lines.get(line_idx+1).map(|l| l.char_start).unwrap_or(<문단 총 char 수>)`
2. `tac_offsets_px` 중 `line_start <= pos < line_end` 인 TAC 를 이 줄의 소유물로 식별.
3. run 루프가 한 번도 돌지 않는 경우(`runs.is_empty()` 또는 run 모두 소비 후 남은 TAC), 해당 TAC 를 인라인 렌더 경로로 처리:
   - 기존 인라인 Equation 렌더 블록(L1574-1620) 과 동일한 방식으로 `EquationNode` 생성
   - 이 줄의 `y`, `baseline`, `line_height` 는 단계 1 에서 정확함을 확인했으므로 그대로 사용
   - `eq_y = (y + baseline - layout_box.baseline).max(y)` 유지 (음수 보정 없음 — 이 줄 y 가 이미 맞음)
   - x 는 `col_area.x + effective_margin_left` 부터 시작, 여러 TAC 가 있으면 폭(tac_w) 누적
4. `tree.set_inline_shape_position(...)` 을 등록해 `shape_layout.rs:133-182` display 경로가 동일 수식을 중복 렌더하지 않게 한다.

**범위 밖 (단계 3 이후)**
- `has_tac_shape` 리팩터링
- x 수평 정렬 조정 (중앙 정렬 등) — 단계 3 시각 비교 후 필요 시
- Shape/Picture/Table TAC 처리 — 본 단계는 Equation 만

**테스트**
- `exam_math_8.hwp` 재내보내기 → 큰 수식의 `translate(x, y)` 가 (71.8, 147.38) 이 아니라 line 1 의 y(≈172.69) 와 줄 베이스라인에 맞는 좌표로 바뀜
- `RHWP_DUMP_287=1` 로 `[287-eq] para=0 ci=3` 로그가 새로 찍히는지 확인 (단계 1 시점에는 없었음)
- `cargo test --lib` 전수 통과, clippy clean
- `samples/exam_math.hwp`, `samples/equation-lim.hwp` 회귀 없음

**산출물**: `mydocs/working/task_m100_287_stage2.md`

**완료 기준**:
- 큰 수식이 `shape_layout` display 경로가 아닌 인라인 경로로 렌더됨을 덤프 로그로 확인
- SVG 에서 `translate(71.8, 147.38)` 이 아닌 line 1 y(≈172.69) 로 이동함을 확인

---

### 단계 3 — `eq_y` / x 계산 정합화 + 회귀 체크

**목적**: 큰 수식의 최종 (x, y) 를 한컴 PDF 와 시각적으로 일치시킨다.

**작업**
1. `paragraph_layout.rs:1584-1618` 의 `eq_y` 계산 재검토:
   - 큰 수식은 `layout_box.baseline > baseline` 이 일반적 → `.max(y)` clamp 대신 줄 상단 or 줄 중앙 정렬 적용.
   - 보정된 `line_height` 안에서 수식이 잘리지 않도록 `eq_h` 와 `line_height` 정합성 확인.
2. x 축: 현재 `x` 가 인라인 커서이므로 줄 안에서의 가용폭 중 어디에 놓일지 결정. `tac_w` (수식 폭) 와 현재 줄 margin 을 반영.
   - 한컴 PDF 확인 결과 수식이 줄 내에서 들여쓰기(대략 중앙 부근)로 배치됨. 이를 달성하기 위해 HWP 의 `ParaShape.align` / `TabDef` 반영 혹은 수식 자체 정렬값 사용.
3. 작은 인라인 수식(같은 문단 내 `n`, `m`, `3`, `|a_m|=|a_{m+2}|`) 배치가 변하지 않음을 확인.

**테스트**
- `cargo test --lib` (특히 `equation`, `paragraph_layout` 관련)
- `cargo clippy --lib -- -D warnings`
- `cargo test --test svg_snapshot`

**산출물**: `mydocs/working/task_m100_287_stage3.md` + 전/후 SVG·PDF 시각 비교 PNG

**완료 기준**: `exam_math_8.hwp` SVG 의 큰 수식 위치가 한컴 PDF 와 육안 동등. 기존 수식 샘플 회귀 없음.

---

### 단계 4 — 최종 보고서 + orders 갱신 + merge 준비

**작업**
1. `mydocs/report/task_m100_287_report.md` 작성.
2. `mydocs/orders/20260424.md` (또는 merge 일자) 에 Task #287 섹션 추가.
3. `git status` 로 미커밋 파일 없는지 확인.
4. devel merge 준비: `git checkout devel && git merge local/task287 --no-ff` (승인 후).

**완료 기준**: 작업지시자 승인 완료, 이슈 #287 클로즈 승인, devel merge 준비.

---

## 전체 검증 체크리스트

- [ ] `cargo test --lib` 통과
- [ ] `cargo clippy --lib -- -D warnings` clean
- [ ] `cargo test --test svg_snapshot` 통과
- [ ] `samples/exam_math_8.hwp` SVG ↔ PDF 시각 동등 (큰 수식 위치·크기)
- [ ] `samples/exam_math.hwp` 16개 수식 회귀 없음
- [ ] `samples/equation-lim.hwp` 회귀 없음
- [ ] HWPX 동등 샘플 보유 시 회귀 점검

## 위험과 완화

- **위험 1**: `has_tac_shape` 조건을 공유 네이밍으로 바꿀 때 Shape 전용 로직에 regression 발생 가능.
  - 완화: Shape 전용 분기와 Equation 확장 분기 동작을 조건문 내부에서 분기 유지. 네이밍만 통일.
- **위험 2**: x 정렬을 위해 `ParaShape.align` 을 건드리면 다른 문단에 광범위 영향.
  - 완화: x 는 현재 `x`(인라인 커서) 유지하고, `text_y` 만 고침으로 Phase 1 마무리. PDF 와의 x 차이는 단계 3 에서 데이터로 확인 후 범위 결정.
- **위험 3**: `corrected_line_height` 가 큰 수식에 대해 font-based 로 축소되면 수식이 줄 밖으로 돌출.
  - 완화: `raw_lh` (원본) 기반으로 누적 y 를 진행하고, 내부 text_y 만 하단 정렬 시프트.

## 승인 요청

본 구현계획(4단계, (B) 우회 접근)으로 단계 1 시작 가능 여부 승인 부탁드립니다.
