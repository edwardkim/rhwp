# Task #324 Stage 1 보고서

**기준일**: 2026-04-25
**브랜치**: `local/task324`
**단계**: 재현 검증 + 기준선 캡처

---

## 1. 재현

```
./target/release/rhwp export-svg samples/hwpx/form-002.hwpx -o output/svg/task324_baseline/
```

10페이지 SVG 출력 완료. page 1 하단에 \"연구개발계획서 제출시…\" 가 표출되고, page 2 상단에는 본문 후속(\"산소탑재효율…\") 부분이 표시됨 (PDF/HWP 와 어긋남).

## 2. dump-pages 결과 (수정 전)

### 페이지 1 (global_idx=0)

```
body_area: x=75.6 y=94.5 w=642.5 h=933.5
단 0 (items=1, used=0.0px)
  PartialTable pi=0 ci=2 rows=0..20 cont=false 26x27 vpos=0
```

### 페이지 2 (global_idx=1)

```
body_area: x=75.6 y=94.5 w=642.5 h=933.5
단 0 (items=2, used=925.6px, hwp_used≈946.8px, diff=-21.2px)
  PartialTable pi=0 ci=2 rows=19..26 cont=true 26x27 vpos=0
  FullParagraph pi=1 h=13.3 vpos=70012 \"(빈)\"
```

- HWP 가용량 ≈ 946.8px, rhwp 사용량 925.6px (diff -21.2px)
- row 19 가 page 1/2 양쪽에 분할됨

## 3. 대상 셀 / 문단

`dump samples/hwpx/form-002.hwpx`:

```
셀[73] r=19,c=0 rs=1,cs=27 h=63539 w=47673 paras=29
  ...
  p[27] ps_id=44 ctrls=0 text_len=83 ls[0] vpos=23892 lh=1100 ls=384
  p[28] ps_id=63 ctrls=1 text_len=0 ls[0] vpos=25376 lh=4768 ls=420
  p[28] 내부표: 1행×1열, 셀=1
    셀[0] r=0,c=0 ... text=\"연구개발계획서 제출시 …\"
```

→ p[28] 가 page 2 로 가야 하나, page 1 에 그려짐.

## 4. 기준선 산출물

| 항목 | 경로 |
|------|------|
| Baseline SVG | `output/svg/task324_baseline/form-002_001.svg` ~ `_010.svg` |

## 5. 회귀 기준 샘플 후보

`samples/` 내 셀 분할 + 중첩 표 패턴은 form-002 외에는 즉시 식별 어려움.
대신 다음 카테고리 샘플을 회귀 비교 대상으로 채택:
- `samples/hwpx/form-001.hwpx` (form 류 일반)
- `samples/hwpx/` 내 다른 form 류 (있을 경우)

Stage 3 에서 SVG 출력 후 baseline 과 byte-level diff 로 회귀 검증.

## 6. 다음 단계

Stage 2 — `compute_cell_line_ranges()` 의 `has_table_in_para` 분기 수정.
