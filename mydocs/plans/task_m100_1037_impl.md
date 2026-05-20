# Task #1037 구현 계획서

**Issue**: [#1037 HWP5 변환본 paragraph height 과대 측정 (HWP3 대비 약 2배)](https://github.com/edwardkim/rhwp/issues/1037)
**Branch**: `local/task1037` (base = `local/devel` = `a52859de`)

---

## 1. 사전 단언 (이미 확인된 영역)

### 1.1 정량 측정 데이터 (Task #1035 Stage 3 진단)

HWP5 변환본 sample16 p23 pi=450 (계약상대자는 주전산센터...) :
- HWP3 pi=450: `h=59.5 (sb=7.5 lines=52.0 sa=0.0)`
- HWP5 pi=450: `h=118.5 (sb=7.5 lines=110.9 sa=0.0)`

분석:
- `sb` (spacing_before): 7.5 동일 ✓
- `sa` (spacing_after): 0.0 동일 ✓
- **`lines` (line content height): 52.0 → 110.9 (약 2.13배)**

→ **paragraph 의 "lines" 부분이 약 2배**. spacing_before/after 가 아닌 **line content 자체** 가 2배.

### 1.2 line content height 의 구성

`lines = number_of_lines × line_height × line_spacing_ratio`

가능한 차이 원인:
- A: `number_of_lines` 차이 (text wrap 위치 다름 → 줄 수 다름)
- B: `line_height` 차이 (같은 줄 수라도 줄 높이 다름)
- C: `line_spacing_ratio` 차이

### 1.3 Task #1008 격차 D fix 후 상태

Task #1008 PR #1034 가 머지된 상태 (또는 머지 대기 — 본 task 시작 시 단언):
- HWP3 폰트명 매핑 (신명조 → HY신명조) — 영향
- 그러나 paragraph height 자체에는 직접 영향 없음 (advance width 만 영향)

---

## 2. Stage 1 — 진단 (정밀 측정)

### 2.1 Step 1.1 — 같은 paragraph 메트릭 직접 비교

진단 test 작성 — HWP3 vs HWP5 변환본 의 pi=450 CharShape/ParaShape/LineSeg 비교:

```rust
// tests/diag_1037_height.rs
use rhwp::parser::hwp3::parse_hwp3;
use rhwp::parser::parse_hwp;

#[test]
fn diag_paragraph_height_components() {
    let hwp3 = parse_hwp3(&std::fs::read("samples/hwp3-sample16.hwp").unwrap()).unwrap();
    let hwp5 = parse_hwp(&std::fs::read("samples/hwp3-sample16-hwp5.hwp").unwrap()).unwrap();

    for pi in [450, 451, 452, 460] {
        for (label, doc) in [("HWP3", &hwp3), ("HWP5", &hwp5)] {
            let para = &doc.sections[0].paragraphs[pi];
            let cs = para.char_shapes.first()
                .and_then(|csr| doc.doc_info.char_shapes.get(csr.char_shape_id as usize));
            let ps = doc.doc_info.para_shapes.get(para.para_shape_id as usize);

            println!("=== {} pi={} ===", label, pi);
            println!("  CharShape: base_size={}, spacings[0]={}, ratios[0]={}",
                cs.map(|c| c.base_size).unwrap_or(0),
                cs.map(|c| c.spacings[0]).unwrap_or(0),
                cs.map(|c| c.ratios[0]).unwrap_or(0));
            println!("  ParaShape: line_spacing_type={:?}, line_spacing={}, sb={}, sa={}",
                ps.map(|p| p.line_spacing_type).unwrap_or_default(),
                ps.map(|p| p.line_spacing).unwrap_or(0),
                ps.map(|p| p.margin_spacing_before).unwrap_or(0),
                ps.map(|p| p.margin_spacing_after).unwrap_or(0));
            println!("  LineSegs ({})", para.line_segs.len());
            for (i, ls) in para.line_segs.iter().enumerate().take(3) {
                println!("    [{}] vpos={} lh={} ls={} cs={} sw={}",
                    i, ls.vertical_pos, ls.line_height, ls.line_spacing,
                    ls.char_spacing, ls.spread_width);
            }
        }
    }
}
```

**측정 후 단언**:
- CharShape.base_size 동일 여부
- ParaShape.line_spacing 동일 여부
- LineSeg.line_height 동일 여부 (vs 차이 — root cause 후보)
- line_segs.len() (number_of_lines) 동일 여부

### 2.2 Step 1.2 — height 산출 경로 추적

`src/renderer/layout/paragraph_layout.rs` / `text_measurement.rs` 의 height 계산:

```bash
grep -n "compute_para_height\|paragraph_height\|measure_paragraph\|line_height\|line_spacing" \
  src/renderer/layout/paragraph_layout.rs \
  src/renderer/layout/text_measurement.rs | head -30
```

height 계산 공식 식별 후 variant 한정 분기 후보 위치 단언.

### 2.3 Step 1.3 — root cause 단언

Step 1.1+1.2 결과 따라 4 가설 중 1개 단언:
- A (font metric line_height) : LineSeg.line_height 가 HWP3 vs HWP5 다름
- B (line_spacing_ratio) : ParaShape.line_spacing 또는 산출 식 다름
- C (spacing_before/after) : 이미 동일 단언 (sb=7.5, sa=0 동일) → 제외
- D (ParaShape 정의 자체) : ParaShape.line_spacing_type 다름

**산출물**: `mydocs/working/task_m100_1037_stage1.md` — root cause 단언 + Stage 2 fix 위치 결정

---

## 3. Stage 2 — Fix 적용

### 3.1 Stage 1 결과별 fix 방향

#### 옵션 A — LineSeg.line_height 정합

HWP5 변환본의 LineSeg.line_height 가 HWP3 보다 큰 경우, parser 또는 renderer 단계에서 정합:
- 위치 A1: `src/parser/body_text.rs` (HWP5 LineSeg 파싱) 의 line_height 산출
- 위치 A2: `src/renderer/layout/paragraph_layout.rs` (variant 가드 한정 line_height 보정)

variant 가드 (`is_hwp3_variant`) 한정 적용 — 일반 HWP5 무영향.

#### 옵션 B — ParaShape.line_spacing 정합

HWP5 변환본의 ParaShape.line_spacing 이 HWP3 보다 큰 경우:
- 위치 B1: `src/parser/doc_info.rs` 의 ParaShape 파싱
- 위치 B2: `src/renderer/style_resolver.rs` 의 line_spacing 처리 (Task #1008 격차 D 와 유사한 variant_div 패턴)

#### 옵션 D — variant ParaShape 보정

HWP5 변환본 ParaShape 의 line_spacing_type 또는 다른 필드 변환기 변형:
- 위치 D1: `src/renderer/style_resolver.rs` 의 variant 가드 분기

### 3.2 단위 테스트 추가

`tests/issue_1037_height.rs`:
- HWP5 변환본 pi=450 paragraph height = HWP3 동등 단언
- LineSeg.line_height 정합 단언

### 3.3 빌드 + 단위 테스트

```bash
cargo build --release
cargo test --release --test issue_1037_height
```

---

## 4. Stage 3 — 검증 + 회귀 sweep

### 4.1 sample16 p23 정합 단언

```bash
# pi=450 height 단언
./target/release/rhwp dump-pages samples/hwp3-sample16-hwp5.hwp -p 22 | grep "pi=450"
# 기대: h=59.5 (HWP3 동등)

# pi=460 FullParagraph fit (PartialParagraph split 회피)
./target/release/rhwp dump-pages samples/hwp3-sample16-hwp5.hwp -p 22 | grep "pi=460"
```

### 4.2 alignment 정합률 측정

PR #1036 의 60/64 (93.75%) 유지 또는 향상:

```bash
MATCH=0
for p in $(seq 0 63); do
  H3=$(./target/release/rhwp dump-pages samples/hwp3-sample16.hwp -p $p 2>/dev/null | grep "FullParagraph" | head -1 | grep -oE 'pi=[0-9]+' | head -1)
  H5=$(./target/release/rhwp dump-pages samples/hwp3-sample16-hwp5.hwp -p $p 2>/dev/null | grep "FullParagraph" | head -1 | grep -oE 'pi=[0-9]+' | head -1)
  [ "$H3" = "$H5" ] && MATCH=$((MATCH+1))
done
echo "alignment: $MATCH/64"
```

### 4.3 sample16-hwp5 페이지 수 측정

```bash
PAGES=$(./target/release/rhwp dump-pages samples/hwp3-sample16-hwp5.hwp 2>/dev/null | grep -c "^=== 페이지")
echo "sample16-hwp5: $PAGES"
# 기대: 64 유지 또는 HWP3 정합 향상
```

### 4.4 회귀 sweep

```bash
# 변환본 9 종
for f in samples/hwp3-sample{4,5,10,11,13,14,16,19}-hwp5.hwp samples/hwp3-sample16-hwp5.hwpx; do
  P=$(./target/release/rhwp dump-pages "$f" 2>/dev/null | grep -c "^=== 페이지")
  printf "  %-50s  %s 페이지\n" "$(basename $f)" "$P"
done

# 일반 HWP5 + HWP3
for f in samples/hwp3-sample16.hwp samples/exam_kor.hwp samples/exam_eng.hwp samples/exam_math.hwp samples/aift.hwp samples/biz_plan.hwp; do
  P=$(./target/release/rhwp dump-pages "$f" 2>/dev/null | grep -c "^=== 페이지")
  printf "  %-50s  %s 페이지\n" "$(basename $f)" "$P"
done
```

### 4.5 전체 테스트

```bash
cargo test --release --lib
cargo test --release --tests
cargo clippy --release --lib -- -D warnings
cargo fmt --all -- --check    # ← --all 필수 (feedback_cargo_fmt_all_required)
```

---

## 5. Stage 4 — 최종 보고 + PR

- `mydocs/working/task_m100_1037_stage{1,2,3}.md` 완성
- `mydocs/report/task_m100_1037_report.md` 작성
- `mydocs/orders/` 갱신 (현 또는 새 날짜)
- WASM 빌드 (Docker)
- 작업지시자 한컴 한글 정답지 시각 검증
- PR 생성 (upstream/devel base, closes #1037)

---

## 6. 변경 위치 후보 summary (Stage 1 결과 후 확정)

| Root cause | 위치 | 변경 규모 |
|-----------|------|----------|
| A: LineSeg.line_height (parser) | `src/parser/body_text.rs` | 중 (variant 가드 한정) |
| A: LineSeg.line_height (renderer) | `src/renderer/layout/paragraph_layout.rs` | 중 |
| B: ParaShape.line_spacing (style_resolver) | `src/renderer/style_resolver.rs` | 소 (variant_div 패턴 확장) |
| D: ParaShape 보정 | `src/renderer/style_resolver.rs` | 소 |

단위 테스트 신규: `tests/issue_1037_height.rs`

---

## 7. 위험 + 완화 (구체화)

| 위험 | Stage / 완화 |
|------|-------------|
| variant 한정 fix 가 일반 HWP5 / HWPX 회귀 | Stage 3 sweep 9 변환본 + 일반 fixture |
| paragraph height 변경이 페이지 수 회귀 | Stage 3 step 4.3 페이지 수 단언 + 4.2 alignment 측정 |
| Task #1035 (PR #1036) 의 paginator narrow 가드와 적층 영향 | Stage 1 진단 시 PR #1036 적용 상태에서 측정 |
| cargo fmt --all 미적용 CI failure | Stage 3 step 4.5 필수 |

---

## 8. 비대상

- 총 페이지 수 (한컴 38 vs rhwp 64) — 한컴 viewer 근사값
- HWP3 native paragraph height (이미 한컴 정합)
- WMF, 폰트 fallback
- WASM (Stage 4 최종 1회)
