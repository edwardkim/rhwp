# Task #402: 동일 문단 내 두 번째 line의 inline 그림이 첫 line과 겹침 — 구현계획서

## 핵심 원인 (사전 코드 조사 결과 확정)

`src/renderer/layout.rs`:

**문제 코드 — `layout_shape_item` (line 2513, 2521):**
```rust
para_start_y.entry(para_index).or_insert(y_offset);   // 없으면만 insert
let pic_y = para_start_y.get(&para_index).copied().unwrap_or(y_offset);
```

**참고 코드 — `layout_table_item` (line 1995-2002):**
```rust
let is_current_tac = paragraphs.get(para_index)
    .and_then(|p| p.controls.get(control_index))
    .map(|c| matches!(c, Control::Table(t) if t.common.treat_as_char))
    .unwrap_or(false);
if let Some(existing_y) = para_start_y.get(&para_index) {
    if is_current_tac && y_offset > *existing_y + 1.0 {
        para_start_y.insert(para_index, y_offset);   // TAC인 경우 갱신
    }
} else {
    para_start_y.insert(para_index, y_offset);
}
let para_y_for_table = *para_start_y.get(&para_index).unwrap_or(&y_offset);
```

**대조점:** 표는 TAC인 경우 `y_offset > existing_y + 1.0` 이면 갱신하지만, 그림은 무조건 `or_insert`만 한다.

**시나리오 (pi=57):**
1. PageItem::Table {pi=57, ci=0} 처리 → `layout_table_item` 진입
   - `para_start_y[57]` = y₀ (예: 539)
   - 표 본체 렌더 → y_offset = y₀ + 328.7 (예: 868)
2. PageItem::Shape {pi=57, ci=1} 처리 → `layout_shape_item` 진입
   - `para_start_y.entry(57).or_insert(y_offset)` → 이미 있어서 무시 (여전히 539)
   - `pic_y = para_start_y[57] = 539` ← **버그**: 표 시작 위치에 그림이 그려짐

## 구현 단계

### 1단계: 진단 로깅 + 가설 최종 확정

**목적:** 코드 수정 전, 실제 실행 시 두 번째 inline shape의 `pic_y`가 어떻게 결정되는지 로그로 확인. 구현 범위(다른 inline 컨트롤 조합도 영향 받는지) 확정.

**작업:**
- `layout_shape_item` 진입 시 `para_index`, `control_index`, `y_offset`, `para_start_y[para_index]`, 결정된 `pic_y` 를 임시 `eprintln!`로 출력 (커밋하지 않음)
- 샘플 SVG 7쪽 재생성하면서 pi=57 ci=1 그림 처리 시점의 값 확인
- 동일 패턴이 있는 다른 페이지/샘플 1~2건 확인 (회귀 위험 추정)

**산출물:** `mydocs/working/task_402_stage1.md` — 로그 결과, 가설 확정 여부, 영향 범위 추정

**검증:** 가설대로 `para_start_y[57] = 539` 이고 `y_offset ≈ 868` 임을 확인.

### 2단계: 수정 구현

**목적:** `layout_shape_item` 에 `layout_table_item` 과 동일한 갱신 로직을 적용.

**작업:**
- `layout_shape_item` 의 `para_start_y.entry(...).or_insert(...)` 를 `layout_table_item` 의 패턴과 동일하게 변경
  - inline TAC 그림이고 `y_offset > existing_y + 1.0` 이면 `para_start_y[para_index] = y_offset` 으로 갱신
- 갱신 후 `pic_y = para_start_y.get(&para_index).copied().unwrap_or(y_offset)` 그대로 사용
- 라인 2657 (else 분기, non-TAC picture)의 동일 패턴도 함께 점검 — 다만 non-TAC은 이번 수정 범위 밖이므로 변경하지 않음
- 진단 로그 제거

**핵심 변경 (의사 코드):**
```rust
// 기존
para_start_y.entry(para_index).or_insert(y_offset);

// 수정 후
let is_current_tac_pic = paragraphs.get(para_index)
    .and_then(|p| p.controls.get(control_index))
    .map(|c| matches!(c, Control::Picture(p) if p.common.treat_as_char))
    .unwrap_or(false);
if let Some(existing_y) = para_start_y.get(&para_index).copied() {
    if is_current_tac_pic && y_offset > existing_y + 1.0 {
        para_start_y.insert(para_index, y_offset);
    }
} else {
    para_start_y.insert(para_index, y_offset);
}
```

**산출물:** 코드 수정 + `mydocs/working/task_402_stage2.md`

**검증:** `cargo build` 통과.

### 3단계: 회귀 검증 + 시각 비교

**목적:** 수정이 의도한 대로 동작하고 기존 케이스를 깨지 않음을 확인.

**작업:**
1. **타겟 검증** — 7쪽 SVG 재생성, qlmanage로 PNG 변환, PDF 7쪽과 시각 비교
   - 표가 단독으로 7쪽에 정상 배치되는지
   - 파이 차트가 8쪽 이후로 흘러가는지
2. **회귀 테스트** — `cargo test`
3. **샘플 회귀 검증** — `re_sample_gen` 류 자동 검증 + 자주 깨지는 샘플(`hwp_table_test*`, `equation-lim`, `endnote-01` 등) 1~2건 SVG 비교
4. `clippy` 통과 확인

**산출물:**
- `mydocs/report/task_402_report.md` (최종 보고서)
- 샘플 SVG 비교 스크린샷 (필요 시 보고서에 첨부)
- `mydocs/orders/{yyyymmdd}.md` 갱신 (있는 경우)

**검증 기준 (수행계획서 그대로):**
1. 7쪽 SVG에 파이 차트가 더 이상 표시되지 않거나, 표와 겹치지 않음
2. 7쪽 표 영역이 PDF와 동일한 위치에 그려짐
3. 파이 차트가 다음 페이지로 흘러감
4. 기존 회귀 테스트 모두 통과

## 위험 / 영향 범위

- **수정 범위가 좁음**: `layout_shape_item` 한 함수, TAC 그림 분기만. non-TAC, 캡션, paper-anchored 그림은 무관.
- **참조 패턴이 검증된 코드**: `layout_table_item` 의 기존 로직과 동일한 패턴 → 새로운 설계 없음.
- **회귀 가능성**: 같은 문단에 inline TAC 그림 1개만 있는 케이스(가장 흔한 케이스)에서는 `y_offset == existing_y` (둘 다 paragraph 시작)이므로 동작 변화 없음. 차이는 inline shape가 다른 inline 컨트롤 뒤에 올 때만 발생.
