# Stage 6 보고서 — Task #409 v3

## 변경 내용

`src/renderer/typeset.rs::typeset_paragraph` fit 분기에 atomic TAC top-fit 분기 추가 (line 913 직후, line ~954):

```rust
// [Task #409 v3] atomic TAC top-fit:
// 단일 라인 + TAC Picture/Shape (분할 불가능) 항목은 시작점이 본문 안이면
// 현재 페이지에 배치하고 하단 일부는 하단 여백 (15mm) 으로 흘림 허용.
let is_atomic_tac_singleton = fmt.line_heights.len() == 1
    && para.controls.iter().any(|c| match c {
        Control::Picture(p) => p.common.treat_as_char,
        Control::Shape(s) => s.common().treat_as_char,
        _ => false,
    });
if is_atomic_tac_singleton
    && st.current_height < available
    && !st.current_items.is_empty()
{
    let overflow = st.current_height + fmt.height_for_fit - available;
    // 60px 이내 초과 (대략 하단 여백 1.6cm 까지 허용; HWP 표준 15mm 여백 안)
    if overflow <= 60.0 {
        st.current_items.push(PageItem::FullParagraph {
            para_index: para_idx,
        });
        st.current_height += if st.col_count > 1 { fmt.height_for_fit } else { fmt.total_height };
        return;
    }
}
```

### 핵심
- `line_heights.len() == 1` + Picture/Shape TAC 만 대상 (분할 불가 atomic)
- `current_height < available` (시작점이 본문 안) + `overflow ≤ 60px` (하단 여백 1.6cm 안)
- bottom-fit fail 후, atomic top-fit 통과 시 현재 페이지에 배치

## 검증 결과

### 23/24 페이지 SVG (PDF 대조)

| 페이지 | 항목 | v2 후 | **v3 후** |
|--------|------|-------|----------|
| 23 | 막대 차트 (pi=208) | 24페이지로 밀림 ❌ | **23페이지 하단 정상 배치** ✓ |
| 24 | 페이지 시작 콘텐츠 | 차트로 시작 (PDF와 다름) ❌ | **2x1 표 → (6) 헤딩 → 표 → 파이차트 → 2x1 표** ✓ |

### 23페이지 dump-pages
```
items=6, used=944.1px (본문 933.5px, 하단 여백 56.7px 안으로 10.6px 초과 — HWP 시멘틱 부합)
  pi=204~207 + Table pi=206 + pi=208 (Picture, lh=316px)
```

차트 layout y=708.76 ~ 1024.76 → 본문 안에 안전 배치 (실제 layout은 vpos 보정 후 1024 < 1028).

### LAYOUT_OVERFLOW (대상 샘플 전체)

| 단계 | 건수 | 잔여 |
|------|------|------|
| v0 | 22 | page=2/20/27 다수 |
| v1 | 4 | page=2 / page=20 / page=27 (2건) |
| v2 | 1 | page=2 449.2 |
| **v3** | **1** | page=2 449.2 (변동 없음, 본 작업 무관) |

→ chart 가 본문 안에 잘 배치되어 overflow 없음.

### 6개 다른 샘플 무회귀

| 샘플 | v2 | v3 |
|------|----|----|
| `biz_plan.hwp` | 0 | 0 |
| `exam_kor.hwp` | 7 | 7 |
| `exam_math.hwp` | 0 | 0 |
| `aift.hwp` | 1 | 1 |
| `k-water-rfp.hwp` | 0 | 0 |
| `kps-ai.hwp` | 4 | 4 |

### 단위/스냅샷 테스트

- `cargo test --lib --release`: **1023 passed**, 0 failed
- `cargo test --release --test svg_snapshot`: **6 passed**, 0 failed

## 결론

- 23페이지 차트 정상 배치 (PDF 일치)
- 24페이지 시작 콘텐츠 정상화 (PDF 일치)
- 다른 샘플 무회귀
- HWP 의 atomic TAC top-fit 시멘틱을 코드로 표현

Stage 7 (전체 회귀 + 통합 최종 보고서) 진행 승인 요청.
