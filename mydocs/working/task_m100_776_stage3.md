# Task #776 Stage 3 (GREEN) — H3b 정정

**Issue**: [#776](https://github.com/edwardkim/rhwp/issues/776)
**Stage**: 3 — H3b (zone 전환 ColumnDef.spacing/2) 정정
**작성일**: 2026-05-10

---

## 정정 코드

`src/renderer/layout.rs:1237-1257`:

```diff
             let is_new_zone = (col_content.zone_y_offset - last_zone_y_offset).abs() > 0.1;
             if is_new_zone {
                 if col_content.zone_y_offset > 0.0 {
-                    current_zone_start_y = prev_zone_y_end;
+                    // [Task #776 H3b] zone 전환 시 진입 zone 의 ColumnDef.spacing / 2
+                    // 가산 (한컴 PDF 정합). RFC #774 stage 5 분석:
+                    // shortcut.hwp 의 zone 전환 측정 ~18.9 px = 10mm spacing / 2.
+                    let zone_top_extra = col_content.items.first()
+                        .map(|item| item.para_index())
+                        .and_then(|pi| paragraphs.get(pi))
+                        .and_then(|para| para.controls.iter().find_map(|c| {
+                            if let crate::model::control::Control::ColumnDef(cd) = c {
+                                Some(hwpunit_to_px(cd.spacing as i32, self.dpi) / 2.0)
+                            } else { None }
+                        }))
+                        .unwrap_or(0.0);
+                    current_zone_start_y = prev_zone_y_end + zone_top_extra;
                 } else {
                     current_zone_start_y = 0.0;
                 }
                 last_zone_y_offset = col_content.zone_y_offset;
             }
```

핵심 동작:
- 새 zone 진입 시 (`zone_y_offset > 0.0`): 진입 zone 의 첫 PageItem 의 paragraph 에서 ColumnDef control 추출
- ColumnDef.spacing / 2 (HWPUNIT → px) 를 `current_zone_start_y` 에 가산
- ColumnDef 없거나 spacing=0 인 경우: `unwrap_or(0.0)` → 변경 없음

## 검증 결과

### issue_776 가드 (4건 모두 GREEN)

| Test | Stage 1 (RED) | Stage 2 (H1') | Stage 3 (H3b) | 기대 (PDF) |
|------|--------------|--------------|--------------|----------|
| shortcut.hwp pi=0 | 0.00 | 26.45 | **26.45** | 26.83 ✓ |
| shortcut.hwp pi=2 | 73.76 | 100.21 | **138.01** | 137.87 ✓ |
| sungeo.hwp pi=0 | 0.00 | 13.33 | **13.33** | 12.63 ✓ |
| treatise sample.hwp pi=0 | 0.00 | 24.00 | **24.00** | 23.69 ✓ |

### 회귀 검증

`cargo test --release`: **1217 passed, 0 failed** (모든 테스트 GREEN).

H3b 정정의 회귀: **없음**.

## H3b 정정의 효과 분석

### shortcut.hwp 페이지 1 zone 전환

3개 zone (단 0 / 단 1 / 단 2):
- 단 0: 첫 zone (zone_y_offset = 0) → H3b 적용 안 됨
- 단 1: 새 zone (zone_y_offset = 69.1) → pi=1 ColumnDef.spacing 10mm = 37.8 px → /2 = 18.9 px 가산
- 단 2: 새 zone (zone_y_offset = 100.2) → pi=2 ColumnDef.spacing 10mm = 37.8 px → /2 = 18.9 px 가산

누적: 18.9 × 2 = **37.80 px** → pi=2 offset 100.21 → 138.01 (정확히 37.80 가산) ✓

### sungeo.hwp / treatise (단일 zone)

zone 전환 없음 → H3b 효과 없음 (변경 없음) ✓

### ColumnDef.spacing = 0 케이스

`unwrap_or(0.0)` 또는 `cd.spacing = 0` → zone_top_extra = 0 → 변경 없음. 회귀 안전.

## H1' + H3b 누적 정정 효과

### shortcut.hwp 페이지 1 (3 zone)

- pi=0 (heading): 0.00 → 26.45 (+26.45 = H1' sb)
- pi=1 (TAC table): _ → _ + 26.45 (= 단 0 의 sb 전파)
- pi=2 (body): 73.76 → 138.01 (+64.25 = H1' 26.45 + H3b 18.9 × 2)

PDF 정합 확인: 138.01 ≈ 137.87 (오차 0.14 px) ✓

### 누적 결함 분해 정합 (RFC #774 예측 검증)

RFC #774 stage 5 예측:
```
H1' (26.45) + H3b 누적 (37.8) = 64.25 px ≈ PDF 측정 64.11 px (±0.14)
```

실제 정정 결과:
```
shortcut.hwp pi=2 offset 변화: 73.76 → 138.01 = +64.25 px
PDF 정합 차이: 138.01 - 137.87 = 0.14 px
```

**RFC 예측 정확** ✓.

## 다음 단계

Stage 4 — 회귀 검증 (cargo test 전체 + 다단 layout + 다중 시각 샘플).
