# 최종 결과 보고서 — Task #409

## 1. 타스크 요약

- **이슈**: [#409](https://github.com/edwardkim/rhwp/issues/409) — 21페이지 2x1 표 위치 오류 (TopAndBottom Picture 다음 문단 vpos 보정 차트 높이 이중 반영)
- **마일스톤**: M100 (v1.0.0)
- **브랜치**: `local/task409`
- **단계**: Stage 1 진단 → Stage 2 구현 → Stage 3 회귀 검증 (총 3단계)

## 2. 증상 (수정 전)

`samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx` 21페이지에서:

- **PDF**: 차트(170×111mm) 바로 아래 2x1 빈 표 배치, 후속 콘텐츠는 22페이지 이후
- **SVG**: 차트와 2x1 표 사이 ~400px 빈 공간, 2x1 표가 페이지 하단(y≈937)으로 밀려 일부 잘림 + pi=192 (10x5 표) 521px overflow

LAYOUT_OVERFLOW 19건 (pi=174~192 연쇄 overflow).

## 3. 근본 원인

`src/renderer/layout.rs:1366-1370` 의 `prev_has_overlay_shape` 가드가 `Control::Shape` + `InFrontOfText|BehindText` 만 검사하여 다음 두 케이스를 처리하지 못함:

1. **`Control::Picture`** — 그림이 텍스트 흐름에 영향을 주는 경우
2. **`TopAndBottom + vert_rel_to=Para`** — 한컴이 후속 문단 vpos 에 개체 높이를 반영하므로 sequential y_offset 이 이미 개체 바닥까지 진행된 상태에서 `lazy_base = vpos_end - y_delta_hu` 산출 시 **개체 높이만큼 base 가 낮게 잡힘** → 후속 문단/표가 개체 높이만큼 추가 점프

본 케이스에서 차트 그림이 두 가지 모두에 해당:
- `Control::Picture` (170×111mm, bin_id=19)
- `TextWrap::TopAndBottom`
- `VertRelTo::Para`

`vpos(pi=173) - vpos(pi=172) = 31470 HU = 419.6 px` 가 정확히 차트 높이와 일치 → 한컴이 차트 높이를 후속 문단 vpos 에 반영했음을 확인.

## 4. 수정 내용

### `src/renderer/layout.rs:1365-1390`

기존 가드에 다음 분기 추가:
- `Control::Picture` (non-TAC) 분기
- `TopAndBottom + vert_rel_to=Para + !treat_as_char` 케이스

```rust
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

## 5. 검증 결과

### 5.1 21페이지 LAYOUT_OVERFLOW

| 항목 | 수정 전 | 수정 후 |
|------|--------|--------|
| 21페이지 OVERFLOW 건수 | **19** | **1** |
| pi=174 (2x1 표) overflow | 21.8px | 0 (제거) |
| pi=175~191 overflow | 17건 (35~268px) | 0 (제거) |
| pi=192 (10x5 표) overflow | 521.7px | 247.9px (잔여 — 별개 페이지네이션 결함) |

### 5.2 시각 검증

- **수정 전**: `mydocs/working/task_m100_409_stage1_before/p21_before.svg` — 2x1 표가 차트 한참 아래(y≈937)에서 페이지 하단으로 밀려 일부 잘림
- **수정 후**: `mydocs/working/task_m100_409_stage2_after.svg` — 2x1 표가 차트 바로 아래(y=532) 정상 위치, PDF 21페이지와 일치

### 5.3 회귀 테스트 (cargo test --release)

전체 10개 테스트 스위트 100% 통과:

| Suite | 결과 |
|-------|------|
| `lib` | **1023 passed**, 0 failed, 1 ignored |
| `svg_snapshot` | **6 passed**, 0 failed |
| `composition_alpha` | 14 passed |
| `find_replace_engine` | 25 passed |
| 기타 6 suites | 0~6 passed each |
| **합계 실패** | **0** |

### 5.4 10개 샘플 LAYOUT_OVERFLOW 비교

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

## 6. 잔여 이슈 (본 타스크 범위 외)

`page=20 pi=192 overflow 247.9px`:

- 사용자 지적("하단의 테이블 위치")은 21페이지 PDF 의 유일하게 보이는 2x1 빈 표를 의미하며, 본 타스크로 해결
- pi=192 (10x5 표)는 PDF 에서 22페이지에 위치해야 하는 표로, **페이지네이션** 단계에서 21페이지로 묶이는 별개 결함
- pagination engine 이 vpos 점프를 충분히 반영하지 못해 발생 (`dump-pages` 의 "items=22, used=803.3px" 추정 vs 실제 layout y=1275.9)
- 별도 이슈로 분리 권장

기타 잔여 (페이지 2, 27)는 본 타스크 이전부터 존재하던 항목으로 본 변경의 영향이 아님.

## 7. 변경 파일

- `src/renderer/layout.rs` (1행 → 22행으로 가드 확장)

## 8. 산출물

- `mydocs/plans/task_m100_409.md` (수행계획서)
- `mydocs/plans/task_m100_409_impl.md` (구현계획서)
- `mydocs/working/task_m100_409_stage1.md` (Stage 1 보고서)
- `mydocs/working/task_m100_409_stage1_before/` (수정 전 베이스라인)
- `mydocs/working/task_m100_409_stage2.md` (Stage 2 보고서)
- `mydocs/working/task_m100_409_stage2_after.svg` (수정 후 SVG)
- `mydocs/report/task_m100_409_report.md` (이 파일)

## 9. 결론

- **사용자 지적 해결**: 21페이지 2x1 표가 차트 바로 아래 정상 위치 (PDF 일치)
- **회귀 무**: 1023 lib + 6 svg_snapshot + 통합 테스트 100% 통과, 6개 다른 샘플 LAYOUT_OVERFLOW 변동 없음
- **개선 효과**: 타겟 샘플 LAYOUT_OVERFLOW 22 → 4 (-18)
- **잔여 이슈**: pi=192 (10x5 표) 페이지네이션 결함은 별도 타스크로 분리 필요

이슈 클로즈 승인 요청.
