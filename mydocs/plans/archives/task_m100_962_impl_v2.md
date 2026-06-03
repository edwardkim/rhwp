# 구현 계획서 V2 — Task #962 Stage 2 — Fix B 적용 계획

- 이슈: [#962](https://github.com/edwardkim/rhwp/issues/962)
- Stage 1 결과: shape_layout 두번째 loop 가 inline Equation 을 중복 emit
- 선택: **Fix B** — inline_shape_position 으로 paragraph_layout 등록 확인 시 emit 스킵

## 1. 변경 위치

`src/renderer/layout/shape_layout.rs:1609-1644` (Equation branch in 두번째 loop)

## 2. inline_shape_position 키 분석

`paragraph_layout.rs:2130` 에서 등록:
```rust
tree.set_inline_shape_position(section_index, para_index, tac_ci, cell_ctx.as_ref(), x, eq_y);
```
- 키: `(sec, para_index=tb_para_idx, tac_ci, cell_path)`
- `cell_path`: shape_layout 첫번째 loop 의 cell_ctx 로부터 derive
  - parent_cell_path + `[control_index=textbox's index, cell_index=0, cell_para_index=tb_para_idx]`

shape_layout 두번째 loop 에서 동일 키로 query 가능.

## 3. 변경 내용

### Before
```rust
Control::Equation(eq) => {
    // 글상자 내 수식: 항상 글자처럼 인라인 배치
    let eq_w = hwpunit_to_px(eq.common.width as i32, self.dpi);
    let eq_h = hwpunit_to_px(eq.common.height as i32, self.dpi);
    let (eq_x, eq_y) = {
        let x = inline_x;
        inline_x += eq_w;
        (x, para_start_y)
    };
    // ... equation node 생성 + push
    shape_node.children.push(eq_node);
}
```

### After
```rust
Control::Equation(eq) => {
    // 글상자 내 수식: 항상 글자처럼 인라인 배치
    let eq_w = hwpunit_to_px(eq.common.width as i32, self.dpi);
    let eq_h = hwpunit_to_px(eq.common.height as i32, self.dpi);
    // [Task #962] 글상자 내부 paragraph 의 inline equation 은 paragraph_layout 가
    // 정확한 gap 위치 (text 사이) 에 emit 한다. 두번째 loop 의 본 분기는 paragraph_layout
    // 미지원 이전의 legacy fallback. paragraph_layout 가 이미 등록한 경우 중복 emit
    // 차단.
    // (시험지 page 2 문14 <보기> textbox 의 6개 inline 수식이 paragraph_layout +
    //  본 분기에서 각각 emit → 중복. 본 fix 는 본 분기에서 등록 확인 후 skip)
    let equiv_cell_ctx = CellContext {
        parent_para_index: para_index,
        path: {
            let mut p = parent_cell_path.to_vec();
            p.push(CellPathEntry {
                control_index,
                cell_index: 0,
                cell_para_index: pi,
                text_direction: 0,
            });
            p
        },
    };
    if tree.get_inline_shape_position(
        section_index, pi, ctrl_idx_in_para, Some(&equiv_cell_ctx)
    ).is_some() {
        // paragraph_layout 이 이미 emit — inline_x 만 advance
        inline_x += eq_w;
    } else {
        // legacy fallback: paragraph_layout 미emit
        let (eq_x, eq_y) = {
            let x = inline_x;
            inline_x += eq_w;
            (x, para_start_y)
        };
        // ... equation node 생성 + push (기존 코드)
        shape_node.children.push(eq_node);
    }
}
```

## 4. 영향 분석

### 4.1 변경 직접 영향
- pi=118 textbox 의 6 equations: paragraph_layout 이 등록 → 본 fix 가 skip → **duplicate 제거**
- 다른 textbox 의 inline equation: 동일하게 paragraph_layout 가 정상 emit → skip → 영향 없음

### 4.2 paragraph_layout 미emit case (fallback)
- legacy case 보존 (paragraph_layout 이 cell_ctx 분기에서 emit 안 하는 케이스가 있다면)
- 본 분기가 fallback emit (기존 동작 유지)

### 4.3 잠재적 회귀
- paragraph_layout 의 등록 키 (cell_ctx) 와 shape_layout 의 query 키 mismatch 가능성 → 등록 검출 실패 → 여전히 중복 emit (= 결함 미해결)
- 또는 다른 textbox 의 정상 case 가 등록 검출 false negative → 영향 없음

## 5. 위험 평가

| 위험 | 평가 | 완화 |
|------|------|------|
| cell_ctx 키 mismatch (등록 검출 실패) | **중** | Stage 3 단위 검증으로 확인 |
| Shape/Picture/Table 의 동일 duplicate (별도) | 본 fix 미해결 | Stage 4 다중 sample 검증 시 확인 |
| paragraph_layout 미emit case 의 fallback 유실 | **낮음** (else 분기 보존) | - |

## 6. 검증 계획 (Stage 3-4)

### Stage 3 단위 검증
1. cargo build --release
2. 시험지 page 2 SVG render:
   - 보기 textbox 영역 equations 수: 12 → 6 ✓
   - ㄱ.ㄴ.ㄷ. prefix + 본문 + 수식 한컴 정합
3. PNG render

### Stage 4 회귀 검증
1. `cargo test --release --lib` 전체 (1288 tests)
2. textbox + inline 수식 sample 검증:
   - 시험지 4종
   - exam_kor/math/eng
   - hwp3-sample14
   - 글상자 보유 sample (shortcut)

## 7. Stage 5 (시각 검증 + 최종 보고서 + PR)

## 8. 진행 규칙

- 자동진행 안함
- 각 stage 종료 시 보고서 + 명시 승인
- 회귀 발견 시 **즉시 revert + 보고**
