# Task #962 Stage 1 — Root cause 정밀 식별

## 1. 증상 재정리

`samples/3-11월_실전_통합_2022.hwp` page 2 의 문14 <보기> textbox 내부 content scramble.

- 한컴: ㄱ. h(1)=3, ㄴ. 함수 h(x)는..., ㄷ. 함수 g(x)가 닫힌구간 [-1, 1]에서...
- rhwp: ㄱㄴㄷ prefix + 수식 위치 + 본문 text 가 시각상 overlap

## 2. 데이터 분석

### pi=118 구조 (dump)

- [0] [사각형] 글앞으로 (InFrontOfText), 글자처럼=true (TAC), z=1735
- 내부 글상자 (text box) paragraphs:
  - p[0]: "ㄱ. \t" + 수식 h(1)=3 (1 ctrl)
  - p[1]: "ㄴ. 함수 는 실수..." + 수식 h(x) (1 ctrl)
  - p[2]: "ㄷ. 함수 가 닫힌구간 ..." + 4 수식 (g(x), [-1,1], 함수, g(-1)=-2)

→ 총 6개 inline 수식 예상.

### SVG 실측

보기 textbox 영역 (y 440-540, x 400-760) 의 equation transforms:

**Set 1** (gap position, 정상):
- (444, 449) ㄱ. h(1)=3
- (474, 467) ㄴ. h(x)
- (469, 485) ㄷ. g(x)
- (560, 485) ㄷ. [-1,1]
- (443, 503) ㄷ. 함수
- (568, 503) ㄷ. g(-1)=-2

**Set 2** (duplicate at left edge x=406, 회귀):
- (406, 448) duplicate
- (406, 466) duplicate
- (406, 484), (431, 484), (475, 484), (540, 484) — left-to-right 누적

→ **6 equations 가 각각 2번 emit. Set 2 가 duplicate**.

## 3. Code path 추적

### Path 1 — layout_composed_paragraph (정상 위치)

`src/renderer/layout/shape_layout.rs:1440-1454`:
```rust
para_y = self.layout_composed_paragraph(
    tree,
    shape_node,
    composed,
    ...
    section_index, tb_para_idx, Some(cell_ctx),
    is_last_para,
    tb_inline_width,
    None, Some(para), None,
    None,
);
```

→ paragraph_layout 의 inline TAC 처리 (`paragraph_layout.rs:2078+`) 가 inline equation 을 gap 에 emit (Set 1).

### Path 2 — shape_layout 의 두번째 loop (duplicate)

`src/renderer/layout/shape_layout.rs:1537-1644`:
```rust
for (ctrl_idx_in_para, ctrl) in para.controls.iter().enumerate() {
    match ctrl {
        ...
        Control::Equation(eq) => {
            // 글상자 내 수식: 항상 글자처럼 인라인 배치
            ...
            let (eq_x, eq_y) = {
                let x = inline_x;
                inline_x += eq_w;
                (x, para_start_y)
            };
            ...
            shape_node.children.push(eq_node);
        }
        ...
    }
}
```

→ 같은 paragraph 의 controls 를 다시 iterate, equation 을 `inline_x` 에 emit (textbox 좌측 edge 부터 누적). 이는 **Set 2 의 source** (duplicate).

## 4. Root cause

**shape_layout 의 두번째 loop 가 inline Equation 을 emit** — Path 1 (paragraph_layout) 가 이미 emit 했음에도 중복 처리.

원래 두번째 loop 의 의도 (추정):
- 빈 paragraph (텍스트 없음 + 인라인 controls 만) 에서 paragraph_layout 이 미emit 시 fallback
- 또는 paragraph_layout 미지원 이전의 legacy 처리

현재 paragraph_layout 이 textbox 내부 inline TAC 를 정상 처리 → 두번째 loop 의 Equation emit 은 **불필요한 duplicate**.

## 5. Fix 후보

### A. shape_layout 두번째 loop 의 Equation branch 제거

paragraph_layout 가 이미 emit 하므로 두번째 loop 의 Equation emit 삭제.

- 위험: **중** — paragraph_layout 이 emit 안 하는 case 가 있다면 fallback 누락 가능성

### B. inline_shape_position 으로 이미 등록 확인

두번째 loop 에서 emit 전에 paragraph_layout 의 등록 확인 (`tree.get_inline_shape_position`).

- 위험: **낮음** — 양쪽 path 의 키 매칭 필요. cell_ctx 차이 주의
- 정밀: 정상 fallback case 보존

### C. paragraph_layout 이 textbox 내부 inline TAC 미emit (shape_layout 만 emit)

paragraph_layout 의 cell_ctx 분기에서 Equation emit 스킵.

- 위험: **고** — paragraph_layout 의 다른 case (cell context) 영향 가능

## 6. 권장 Fix

**Option B** — inline_shape_position 으로 중복 차단. 가장 안전.

또는 **Option A** — 두번째 loop 의 Equation branch 제거 (paragraph_layout 이 모든 textbox case 처리한다는 가정).

## 7. 다른 inline 컨트롤 영향 검토 필요

shape_layout 두번째 loop 는 Shape, Picture, Equation, Table 도 처리. 같은 duplicate 가능성:
- Shape (line 1539-1574): layout_shape_object 호출 — 자체 등록 + emit 시 duplicate?
- Picture (line 1576-1607): layout_picture 직접 호출 + inline_x 갱신
- Table (line 1646~): table emit

Stage 4 회귀 검증 시 확인.

## 8. 후속

- Stage 2: Fix 안 (A vs B vs C) 결정 + 구현 계획 V2
- Stage 3: 구현 + 단위 검증
- Stage 4: 다른 textbox + inline 수식 sample 회귀 검증
