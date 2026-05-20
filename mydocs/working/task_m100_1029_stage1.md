# Task #1029 Stage 1 완료 보고서

**Issue**: [#1029 HWP3 외곽선 paper-edge 정합 회귀](https://github.com/edwardkim/rhwp/issues/1029)
**Branch**: `local/task1029`
**작업 내용**: PR #1003 cherry-pick `--theirs` 충돌 해소로 revert 된 PR #1011 의 `src/renderer/layout.rs` 변경 4 hunk 복원

---

## 1. 사전 단언 (Stage 1 시작 시 verify) — 모두 통과

| 항목 | 위치 | 결과 |
|------|------|------|
| `PageBorderBasis` enum | `src/model/page.rs:74` | ✓ 존재 |
| `PageBorderFill::basis` 필드 | `src/model/page.rs:69` | ✓ 존재 |
| HWP3 parser `basis=PaperBased` | `src/parser/hwp3/mod.rs:2840` | ✓ 주입 |
| HWP5 parser `basis=PaperBased` | `src/parser/body_text.rs:869` | ✓ 주입 |
| HWPX parser `basis=PaperBased` | `src/parser/hwpx/section.rs:527` | ✓ 주입 |

→ parser 측은 PR #1011 의 변경이 모두 살아있음. fix 는 renderer (`layout.rs`) 한정.

---

## 2. 복원 hunk

### 2.1 변경 A — `page_number_baseline_y()` (line ~947)

```diff
 let pbf = page_border_fill.filter(|p| p.border_fill_id > 0)?;
-let paper_based = (pbf.attr & 0x01) != 0;
+use crate::model::page::PageBorderBasis;
+let paper_based = matches!(pbf.basis, PageBorderBasis::PaperBased);
```

### 2.2 변경 B — `build_page_borders()` paper_based + 주석 + 디버그 로그

```diff
-// 외곽선 위치 기준: attr bit 0 (textBorder=PAPER) 존중.
-//   bit0 = 1 → paper 기준, bit0 = 0 → body 기준 (HWPX/HWP5 본래 의미).
+// 외곽선 위치 기준: PageBorderFill.basis (PaperBased/BodyBased).
 // 회귀 history:
 //   - task877: paper_based = (attr & 0x01) != 0 — sample16 정합, 시험지 회귀
 //   - #920: paper_based = (attr & 0x01) == 0 — 시험지 정합, sample16 회귀
 //   - #952: paper_based = true 전역 — 당시 모든 sample 정합 판정
-// [Task #987] #952 의 "sample16 = paper 정합" 은 bfid off-by-one
-//   ... HWP3 는 파서가 attr=0(body) 주입 ...
-let paper_based = (pbf.attr & 0x01) != 0;
+//   - #987: bfid 정정 + attr 존중 — 변환본 logo overlap 회귀 (#1006)
+// 정답 (#1006): 포맷별 분리 (PageBorderFill.basis) + 모두 PaperBased.
+// [Task #1029] PR #1003 cherry-pick `--theirs` 충돌 해소로 본 로직이
+// PR #987 attr 비트 해석으로 revert 되어 HWP3 native (attr=0) 만
+// body-edge 로 좁아진 시각 회귀 발생 — 본 task 에서 PR #1011 상태 복원.
+use crate::model::page::PageBorderBasis;
+let paper_based = matches!(pbf.basis, PageBorderBasis::PaperBased);
+let footer_inside = (pbf.attr & 0x04) != 0;
 if std::env::var("RHWP_DEBUG_PAGE_BORDER").is_ok() {
     eprintln!(
-        "PAGE_BORDER: attr=0x{:08x} bit0={} paper_based={} bfid={} ...",
-        pbf.attr, pbf.attr & 0x01, paper_based, ...
+        "PAGE_BORDER: attr=0x{:08x} bit0={} bit1={} bit2={} paper_based={} footer_inside={} bfid={} ...",
+        pbf.attr, pbf.attr & 0x01, (pbf.attr >> 1) & 0x01, (pbf.attr >> 2) & 0x01,
+        paper_based, footer_inside, ...
     );
 }
```

### 2.3 변경 C — `footer_inside` clip 복원 + 변수 mutability (변경 D)

```diff
-let (bx, by, bw, bh) = if paper_based { ... };
+let (bx, mut by, bw, mut bh) = if paper_based { ... };
+// [Task #1006 part 2] header_inside 는 clip 미적용 (cover logo 가 외곽선 내부),
+// footer_inside 만 clip 적용 (페이지 번호가 외곽선 바깥 — PR #1011).
+if !footer_inside {
+    let footer_top = layout.body_area.y + layout.body_area.height;
+    if by + bh > footer_top {
+        bh = footer_top - by;
+    }
+}
+let _ = &mut by;
```

---

## 3. 빌드 단언

```
$ cargo build --release
    Finished `release` profile [optimized] target(s) in 1m 38s
```

컴파일 통과, warning 0.

---

## 4. 단일 페이지 즉시 단언

```bash
RHWP_DEBUG_PAGE_BORDER=1 ./target/release/rhwp export-svg samples/hwp3-sample16.hwp -p 0 -o /tmp/v1/
```

```
HWP3 native   : PAGE_BORDER: attr=0x00000000 bit0=0 bit1=0 bit2=0 paper_based=true footer_inside=false bfid=2705
HWP5 변환본   : PAGE_BORDER: attr=0x00000001 bit0=1 bit1=0 bit2=0 paper_based=true footer_inside=false bfid=2
HWPX 변환본   : PAGE_BORDER: attr=0x00000041 bit0=1 bit1=0 bit2=0 paper_based=true footer_inside=false bfid=2
```

→ 3 포맷 모두 `paper_based=true` (basis 필드 기반, attr 비트 무관) 단언.

```
hwp3-sample16        -- border_top y=17.883333333333333  (paper-edge 복원, PR #1011 baseline)
hwp3-sample16-hwp5   -- border_top y=17.883333333333333  (무변동 유지)
```

→ **C1 (HWP3 paper-edge 복원), C2/C3 (HWP5/HWPX 무영향) 단언**

---

## 5. 변경 위치 summary

| 파일 | 변경 라인 수 |
|------|------|
| `src/renderer/layout.rs` | 4 hunk (변경 A/B/C/D 통합) |

parser 무수정, model 무수정, 시그니처 무변경. 단일 파일 한정 복원.

---

## 6. 다음 단계 (Stage 2)

전체 회귀 sweep 검증.
