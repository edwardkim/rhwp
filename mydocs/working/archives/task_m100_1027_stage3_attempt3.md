# Stage 3-3 — #1027: VPOS_CORR 클램프 공유(옵션 1) 시도 결과

- 타스크: #1027 / 브랜치 `local/task1027`
- 작성일: 2026-05-20
- 단계: Stage 3 — 옵션 1(렌더러 VPOS_CORR 클램프 공유)

## 1. 렌더러 VPOS_CORR 클램프 규칙 (layout.rs:2440~2468)

```rust
const MAX_BACKWARD_PX: f64 = 8.0;
let applied = end_y >= col_area.y && end_y <= col_area.y + col_area.height
    && end_y >= y_offset - MAX_BACKWARD_PX && !stale_table_host_vpos;
if applied { y_offset = end_y; }
```
- 렌더러는 **단계당 ≤8px 백워드**로만 vpos 보정. 단, 렌더러 `y_offset` 자체는 **실제 레이아웃 높이**(paragraph_layout body advance)로 누적 → 단락당 drift ≤8px → 8px 보정으로 충분히 추종.

## 2. typeset 페이지네이터에 적용 시도 (3 변형)

`page_vpos_base` + fit 직전 앵커 (`current_height = vpos − base`):

| 변형 | 노트 | 페이지(185) | LAYOUT_OVERFLOW(12) |
|------|------|------------|------|
| full snap, base=첫 텍스트 vpos | 8쪽 ✅ | 168 | **136** ❌ |
| 8px 백워드 클램프 | 9쪽 ❌ | 186 | 13 ✅ |
| full snap, **base=페이지상단 vpos**(소비분 역산) | 8쪽 ✅ | 181 | **69** ❌ |

## 3. 근본 한계 (왜 fit 앵커만으론 안 되는가)

- 페이지네이터(typeset)는 누적에 **formula `total_height`**(1606행) 사용 → 단락당 **+sb+trailing_ls ≈ 19.6px drift**(#1027 Stage1).
- 8px 클램프는 19.6px/단락 drift 를 못 따라잡음 → 노트(43.6px 누적 보정 필요) 미이동.
- 무제한 full snap 은 **vpos < 실제 렌더 위치인 ~57 단락**에서 과밀 → overflow +57.
  - 이런 단락은 vpos 가 실측 높이를 과소표현(인라인 객체/큰 줄높이 등). 렌더러는 실제 레이아웃으로 알지만 vpos offset 은 모름.

→ **fit 앵커(측정 공간 동기화)만으론 불충분.** 렌더러와 진짜 일치하려면 페이지네이터의 **누적(1606)을 render-consistent advance** 로 바꿔, 단락당 drift 를 렌더러처럼 ≤8px 로 만든 뒤 8px 클램프 앵커를 적용해야 한다.

## 4. 다음 설계 (Stage 3-4) — 누적 정합

- `typeset_paragraph` 의 `current_height += total_height`(1606) 를 **렌더러 body advance 와 동일한 vpos-consistent 높이**로 교체:
  - 후보: `fmt.height_for_fit`(trailing_ls 제외) 또는 LINE_SEG vpos span 기반.
  - col==1 단단의 #359(k-water-rfp 311px drift) 회귀 주의 — total_height 도입 사유였음.
- 누적 정합 후 8px 클램프 앵커 재적용 → 노트 8쪽 + overflow≈12 동시 달성 검증.

## 5. 평가
옵션 1 의 클램프 규칙은 확인했고 노트 이동도 재현되나, fit 앵커 단독으론 누적 모델 불일치로 부작용. **누적 정합(1606)** 이 핵심 잔여 작업. (소스 되돌림, 클린.)
