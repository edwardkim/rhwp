# Task #776 Stage 2 (GREEN) — H1' 정정

**Issue**: [#776](https://github.com/edwardkim/rhwp/issues/776)
**Stage**: 2 — H1' (단/페이지 첫 paragraph sb 누락) 정정
**작성일**: 2026-05-10

---

## 정정 코드

`src/renderer/layout/paragraph_layout.rs:744-749`:

```diff
-        // 문단 앞 간격 (첫 줄일 때만)
-        // 단/페이지의 맨 처음 문단은 spacing_before 적용하지 않음
-        let is_column_top = (y - col_area.y).abs() < 1.0;
-        if start_line == 0 && spacing_before > 0.0 && !is_column_top {
-            y += spacing_before;
-        }
+        // 문단 앞 간격 (첫 줄일 때만)
+        // [Task #776 H1'] 단/페이지의 맨 처음 문단도 sb 적용 (한컴 PDF 정합).
+        // 셀 안 paragraph 는 cell padding 과 sb 중복 방지를 위해 skip 유지.
+        // RFC #774 stage 3 분석: shortcut/sungeo/treatise 3 샘플 ±1 px 정합 검증.
+        if start_line == 0 && spacing_before > 0.0 && cell_ctx.is_none() {
+            y += spacing_before;
+        }
```

핵심 변화: `is_column_top` 가드 → `cell_ctx.is_none()` 가드.
- 본문 column top: sb 적용 (회복)
- 셀 안 paragraph: sb skip (회귀 방지)

## 검증 결과

### issue_776 가드

| Test | 정정 전 | 정정 후 | 기대 (PDF) | 결과 |
|------|--------|--------|----------|------|
| shortcut.hwp pi=0 | 0.00 | **26.45** | 26.83 | ✓ GREEN |
| sungeo.hwp pi=0 | 0.00 | **13.33** | 12.63 | ✓ GREEN |
| treatise sample.hwp pi=0 | 0.00 | **24.00** | 23.69 | ✓ GREEN |
| shortcut.hwp pi=2 | 73.76 | **100.21** | 137.87 | RED (H3b 미적용) |

### 회귀 검증

`cargo test --release`: **1217 passed, 0 failed** (issue_776 H3b 단일 RED 제외).

H1' 정정의 회귀: **없음**.

## H1' 정정의 효과 분석

### 본문 column top paragraph (회복)

shortcut.hwp pi=0 의 spacing_before = 3968 HU = 26.45 px 가 정상 적용됨. heading paragraph 가 body_top 으로부터 26.45 px 아래로 이동.

연쇄 효과:
- pi=0 → pi=1 transition: pi=1 도 26.45 px 아래로 이동
- pi=1 → pi=2 transition: pi=2 도 26.45 px 아래로 이동
- pi=2 offset: 73.76 → 100.21 (+26.45)

### 셀 안 paragraph (회귀 방지)

`cell_ctx.is_some()` 인 경우 sb 적용 skip. 셀 padding 과 중복 방지.

## 다음 단계

Stage 3 — H3b 정정 (layout.rs:1240 영역). zone 전환 시 ColumnDef.spacing / 2 가산. shortcut.hwp pi=2 의 잔여 차이 (100.21 → 137.87 = +37.66 px) 를 H3b 정정으로 해소.
