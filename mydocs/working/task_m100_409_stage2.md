# Stage 2 보고서 — Task #409

## 개요

`prev_has_overlay_shape` 가드를 `Control::Picture` (non-TAC) 분기 + `TopAndBottom + vert_rel_to=Para` 케이스로 확장.

## 변경 내용

### `src/renderer/layout.rs:1365-1390`

수정 전:
```rust
// 글앞으로/글뒤로 Shape가 있는 문단: vpos에 Shape 높이가 포함되어 과대 → bypass
let prev_has_overlay_shape = paragraphs.get(prev_pi).map(|p| {
    p.controls.iter().any(|c|
        matches!(c, Control::Shape(s) if matches!(s.common().text_wrap,
            crate::model::shape::TextWrap::InFrontOfText | crate::model::shape::TextWrap::BehindText)))
}).unwrap_or(false);
```

수정 후:
```rust
// 글앞으로/글뒤로/위아래 Shape·Picture가 있는 문단: vpos에 개체 높이가 포함되어 과대 → bypass
// - InFrontOfText/BehindText: 개체 vpos가 텍스트 라인 vpos와 별도 누적 → 합산 시 과대
// - TopAndBottom + vert=Para: 한컴이 후속 문단 vpos에 개체 높이를 더해 기록하므로
//   sequential y_offset이 이미 개체 바닥까지 진행된 상태에서 vpos 보정 lazy_base 산출
//   시 prev_pi의 텍스트 vpos_end만 쓰면 base가 개체 높이만큼 낮게 산출되어
//   다음 문단/표가 개체 높이만큼 추가 점프 (Task #409: 21페이지 차트→2x1 표 521px overflow)
let prev_has_overlay_shape = paragraphs.get(prev_pi).map(|p| {
    use crate::model::shape::{TextWrap, VertRelTo};
    p.controls.iter().any(|c| match c {
        Control::Shape(s) => {
            let cm = s.common();
            matches!(cm.text_wrap, TextWrap::InFrontOfText | TextWrap::BehindText)
                || (matches!(cm.text_wrap, TextWrap::TopAndBottom)
                    && matches!(cm.vert_rel_to, VertRelTo::Para)
                    && !cm.treat_as_char)
        }
        Control::Picture(pic) => {
            let cm = &pic.common;
            if cm.treat_as_char { return false; }
            matches!(cm.text_wrap, TextWrap::InFrontOfText | TextWrap::BehindText)
                || (matches!(cm.text_wrap, TextWrap::TopAndBottom)
                    && matches!(cm.vert_rel_to, VertRelTo::Para))
        }
        _ => false,
    })
}).unwrap_or(false);
```

### 핵심 변화
1. **Control::Picture (non-TAC) 추가** — 본 케이스의 차트는 Picture 컨트롤이며, 기존 가드는 Shape만 검사하여 우회 누락
2. **TopAndBottom + vert=Para 케이스 추가** — 한컴이 이 조합에서 후속 문단 vpos에 개체 높이를 반영하므로 vpos 보정에서 이중 점프 발생

## 검증 결과

### 21페이지 LAYOUT_OVERFLOW (수정 전 → 수정 후)
| 항목 | 수정 전 | 수정 후 |
|------|--------|--------|
| 21페이지 OVERFLOW 건수 | 19 | **1** |
| pi=174 (2x1 표) overflow | 21.8px | **0** (제거) |
| pi=175~191 overflow | 17건 (35~268px) | **0** (제거) |
| pi=192 (10x5 표) overflow | 521.7px | 247.9px (잔여) |

### 21페이지 시각 검증
- 차트 바로 아래 2x1 빈 표가 정상 위치 (y=532.56 ~ 644.81)
- PDF 21페이지와 일치 (`mydocs/working/task_m100_409_stage2_after.svg`)

### 회귀 테스트
- `cargo test --lib --release`: **1023 passed** (베이스라인 동일)
- `cargo test --release --test svg_snapshot`: **6 passed** (베이스라인 동일)

### 10개 샘플 LAYOUT_OVERFLOW 비교 (수정 전/후)
| 샘플 | 수정 전 | 수정 후 | 변화 |
|------|--------|--------|------|
| `biz_plan.hwp` | 0 | 0 | — |
| `exam_kor.hwp` | 7 | 7 | — |
| `exam_math.hwp` | 0 | 0 | — |
| `aift.hwp` | 1 | 1 | — |
| `k-water-rfp.hwp` | 0 | 0 | — |
| `kps-ai.hwp` | 4 | 4 | — |
| `2025년 기부·답례품_양식.hwpx` | **22** | **4** | **-18** |

→ 다른 샘플 무회귀, 타겟 샘플 18건 개선.

## 잔여 이슈 (별개)

`page=20 pi=192 overflow 247.9px`은 본 타스크 범위 외:

- 사용자 지적("하단의 테이블 위치")은 21페이지 PDF의 **유일하게 보이는 작은 표** (2x1 빈 표)를 의미하며, 이는 정상 위치 복원 완료
- pi=192 (10x5 표)는 PDF에서 22페이지에 위치하는 표로, **페이지네이션** 단계에서 21페이지에 묶이는 별개 결함
- 페이지네이션이 vpos 점프를 충분히 반영하지 못해 발생 (`dump-pages` 가 "items=22, used=803.3px"로 추정하지만 실제 layout y는 1275.9)
- 별도 이슈로 분리 권장

기타 잔여 (페이지 2, 27)는 본 타스크 이전부터 존재하던 항목으로 본 변경의 영향이 아님.

## 결론

- 사용자 지적 21페이지 2x1 표 위치 복원 완료
- 21페이지 LAYOUT_OVERFLOW 19건 제거 (잔여 1건은 별개 페이지네이션 결함)
- 다른 샘플 무회귀
- 단위/스냅샷 테스트 100% 통과

Stage 3 진행 승인 요청.
