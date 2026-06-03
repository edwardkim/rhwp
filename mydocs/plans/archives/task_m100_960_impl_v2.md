# 구현 계획서 V2 — Task #960 Stage 2 — Fix A 적용 계획

- 이슈: [#960](https://github.com/edwardkim/rhwp/issues/960)
- Stage 1 결과: `paragraph_layout.rs:1724-1727` filter 가 has_line_break line 의 end position control 누락
- 선택: **Fix A** — filter 에 `has_line_break && is_last_of_line` 조건 추가

## 1. 변경 위치

`src/renderer/layout/paragraph_layout.rs:1719-1727`

## 2. 변경 내용

### Before
```rust
let is_last_run = is_last_line_of_para && is_last_run_of_line(run_idx);
let is_line_break = comp_line.has_line_break && is_last_run_of_line(run_idx);

// treat_as_char 분기점: run 내 이미지 위치 목록 (rel_pos, width_px, control_index)
// 마지막 run에서는 run_char_end 위치의 TAC도 포함 (문단 끝 수식/그림)
let run_tacs: Vec<(usize, f64, usize)> = tac_offsets_px.iter()
    .filter(|(pos, _, _)| *pos >= run_char_pos 
        && (*pos < run_char_end 
            || (is_last_run && *pos == run_char_end)))
    .map(|(pos, w, ci)| (pos - run_char_pos, *w, *ci))
    .collect();
```

### After
```rust
let is_last_run = is_last_line_of_para && is_last_run_of_line(run_idx);
let is_line_break = comp_line.has_line_break && is_last_run_of_line(run_idx);

// treat_as_char 분기점: run 내 이미지 위치 목록 (rel_pos, width_px, control_index)
// 마지막 run에서는 run_char_end 위치의 TAC도 포함 (문단 끝 수식/그림)
// [Task #960] has_line_break line 의 마지막 run 도 end position 의 TAC 포함:
// HWP3 의 char_offsets gap 분석으로 매핑된 control 위치가 `\n` 문자에 떨어지면
// (예: 시험지 page 2 pi=117 의 cases formula at position 30 = '\n'),
// 그 line 의 chars range [start, end) 에서 end 가 `\n` 위치이므로 누락됨.
// has_line_break line 의 마지막 run 의 end position 도 TAC 포함하면 fix.
let allow_end_tac = is_last_run 
    || (comp_line.has_line_break && is_last_run_of_line(run_idx));
let run_tacs: Vec<(usize, f64, usize)> = tac_offsets_px.iter()
    .filter(|(pos, _, _)| *pos >= run_char_pos 
        && (*pos < run_char_end 
            || (allow_end_tac && *pos == run_char_end)))
    .map(|(pos, w, ci)| (pos - run_char_pos, *w, *ci))
    .collect();
```

## 3. 영향 분석

### 3.1 변경 직접 영향
- pi=117 line 1 의 run_char_end=30 위치의 cases formula 가 line 1 에 포함됨 → emit y ~347 (정상)
- shape_layout 의 default emit (y=329) 으로 가던 cases 가 paragraph_layout 의 line 1 inline emit 으로 변경

### 3.2 다른 sample 영향 (예상)
- has_line_break line + end position 의 control 가 있는 모든 paragraph 가 영향:
  - 동일 fix 적용 (정상화)
  - 기존 잘못된 위치 emit → 정확한 line 위치 emit

### 3.3 잠재적 회귀
- has_line_break line 의 end position 의 control 이 기존에 다른 path (shape_layout default) 로 emit 되어 visual coincidentally 한컴 정합했던 case → fix 적용 후 다른 위치로 변경 → 시각 회귀 가능성

## 4. 위험 평가

| 위험 | 평가 | 완화 |
|------|------|------|
| 다른 sample 의 has_line_break + end control case 회귀 | **중** | Stage 4 다중 sample 검증 |
| shape_layout default emit 에 의존하던 case 의 위치 변경 | **중** | cargo test + 시각 회귀 |
| Filter logic 추가로 인한 cargo test 회귀 | **낮음** (조건 추가만) | cargo test 1288 |

## 5. 검증 계획 (Stage 3-4)

### Stage 3 단위 검증
1. cargo build --release
2. 시험지 page 2 SVG render:
   - cases formula y 위치 ~347 (line 1) 확인
   - h(x)=lim formula y 위치 ~380 (line 2) 정합 유지
3. PNG render → 한컴 PDF 정합

### Stage 4 회귀 검증
1. `cargo test --release --lib` 전체 (1288 tests)
2. 다중 sample SVG render:
   - 시험지 4종 (3-09월/3-10월/3-11월)
   - exam_kor/math/eng (수식 다수 보유)
   - hwp3-sample14 (caption + 수식 보유)
   - hwp3-sample10/11/13
   - shortcut, biz_plan
3. golden SVG diff 회귀 0

## 6. Stage 5 (시각 검증 + 최종 보고서 + PR)

- 한컴 PDF 정합 비교 (시험지 page 2)
- rhwp-studio UI 시각 확인 (작업지시자)
- 최종 보고서 + commit + PR

## 7. 진행 규칙

- 자동진행 안함
- 각 stage 종료 시 보고서 + 명시 승인
- 회귀 발견 시 **즉시 revert + 보고**
