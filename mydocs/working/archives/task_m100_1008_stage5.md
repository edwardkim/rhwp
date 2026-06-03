# Task #1008 Stage 5 완료 보고서 — 격차 D 완료 (HWP3 CharShape dedupe + 폰트명 매핑)

**Issue**: [#1008 HWP3 sample16 Shape/Text 정합 격차 종합](https://github.com/edwardkim/regression-rhwp/issues/1008)
**Branch**: `local/task1008`
**작업 내용**: HWP3 char advance drift root cause 2 항목 해소 — CharShape dedupe + 폰트명 매핑

---

## 1. 격차 D 의 두 root cause

### 1.1 CharShape pos 중복 (Step 1)

HWP3 raw 의 char_shapes 빌드 시 rep CharShape + inline shape change 가 같은 pos=0 으로 모두 push (sample16 pi=4: rep id=57 base_size=1000 + inline id=58 base_size=1400). 한컴 변환기는 dedupe.

**Fix**: `mod.rs:1869~1900` 의 char_shapes 빌드 후 dedupe loop 추가.

### 1.2 HWP3 legacy 폰트명 (Step 2 — 핵심 root cause)

진단 test 로 단언:
- HWP3 CharShape id=59 (`"세계 3대...기업"` 영역): font_ids[0]=1 → group 0 idx 1 = **"신명조"**
- HWP5 변환본 id=27: font_ids[0]=4 → **"한양신명조"**

→ HWP3 raw 의 "신명조" 와 HWP5 변환본의 "한양신명조" 가 **다른 폰트 metric 으로 평가**되어 char advance 좌표 차이. SVG font-family 의 첫 폰트가 다름 → rhwp 의 폰트 metric 측정이 다른 폰트로 fallback → cumulative drift.

**Fix**: HWP3 parser 의 font_faces 로딩 시 legacy 명칭 → HWP5 정합 명칭 매핑 (한컴 변환기 mimic):

```rust
fn hwp3_font_name_to_hwp5(name: &str) -> String {
    match name.trim() {
        "신명조" | "신명" => "HY신명조".to_string(),
        "고딕" => "HY고딕".to_string(),
        "중고딕" => "HY중고딕".to_string(),
        "견고딕" => "HY견고딕".to_string(),
        "그래픽" => "HY그래픽".to_string(),
        _ => name.to_string(),
    }
}
```

`Font.alt_name` 에 원본 명칭 보존 (트레이싱용).

---

## 2. dump 단언

### 2.1 CharShape dedupe (Step 1)

```
HWP3 sample16 pi=4 (AFTER):
  [CS] pos=0 id=58 bold=true   ← id=57 (10pt) 제거됨
  [CS] pos=8 id=59 bold=false
  [CS] pos=31 id=60 bold=true
```

HWP5 변환본 구조와 동일 (3개).

### 2.2 폰트명 매핑 (Step 2)

```
HWP3 sample16 doc_info.font_faces[0]:
  idx=0 name="HY고딕"  alt="고딕"
  idx=1 name="HY신명조"  alt="신명조"
  idx=2 name="HY중고딕"  alt="중고딕"
  idx=3 name="HY견고딕"  alt="견고딕"
  idx=4 name="HY그래픽"  alt="그래픽"
```

---

## 3. SVG 좌표 정합 단언 (HWP3 vs HWP5 변환본)

| Char | HWP3 BEFORE | HWP3 AFTER | HWP5 변환본 | 정합? |
|------|-------------|------------|-------------|------|
| `"` | 131.69 | 131.69 | 131.69 | ✓ |
| 세 | 134.69 | **137.69** | 137.69 | ✓ |
| 계 | 151.12 | **154.12** | 154.12 | ✓ |
| 3 | 174.69 | **177.69** | 177.69 | ✓ |
| 대 | 181.69 | **186.69** | 186.69 | ✓ |
| 물 | 205.21 | **210.21** | 210.21 | ✓ |
| ... | ... | ... | ... | ✓ |
| 한 | 401.21 | **408.37** | 408.37 | ✓ |

→ **HWP3 SVG 좌표가 HWP5 변환본과 byte-for-byte 일치**.

---

## 4. SVG font-family 단언

```
HWP3 native cover (AFTER):
  font-family="HY신명조, 'Batang', '바탕', 'Nanum Myeongjo', ..."

HWP5 변환본 cover:
  font-family="HY신명조, 'Batang', '바탕', 'Nanum Myeongjo', ..."
```

→ 동일 font-family chain → 동일 폰트 metric → 동일 advance width.

---

## 5. 회귀 sweep

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✓ warning 0 |
| `cargo clippy --release --lib -- -D warnings` | ✓ clean |
| `cargo fmt --check` | ✓ clean |
| `cargo test --release --lib` | ✓ 1307 passed |
| `cargo test --release --test issue_1008_gradient` | ✓ **4 passed** (격차 A + B + C + D) |
| `cargo test --release --tests` | ✓ all passed (FAILED 0) |

### 5.1 페이지 수 sweep (HWP3 11 + HWP5 + 일반 fixture)

모든 fixture 페이지 수 회귀 0.

### 5.2 HWP5/HWPX 변환본 — 영향 없음 단언

HWP3 parser 측 변경. HWP5/HWPX parser 무수정. font_faces 매핑은 HWP3 only.

---

## 6. 단위 테스트 추가

`tests/issue_1008_gradient.rs::hwp3_sample16_font_name_mapped_to_hwp5_convention`:
- group 0 idx 0~4 의 매핑 단언 (고딕→HY고딕, 신명조→HY신명조 등)
- alt_name 원본 보존 단언

---

## 7. 성공 기준 충족

| 조건 | 결과 |
|------|------|
| C4: HWP3 한글 단어 공백 정합 (격차 D) | ✓ **byte-for-byte HWP5 변환본 정합** |
| C5: 페이지 수 64 유지 | ✓ |
| C6: 변환본/일반 fixture 회귀 0 | ✓ |
| C7: cargo test | ✓ |
| C8: 시각 검증 | (Stage 6 시점) |

---

## 8. 4 격차 종합 fix 위치 summary

| 격차 | 파일 | 라인 |
|------|------|------|
| A | `src/parser/hwp3/drawing.rs` | 792~830 (Fill IR gradient 매핑) |
| B | `src/parser/hwp3/drawing.rs` | 758~783 (LineType 2~7 → Solid normalize) |
| C | `src/parser/hwp3/mod.rs` | 2870~2960 (fixup_hwp3_heading_decoration) |
| D-1 | `src/parser/hwp3/mod.rs` | 1869~1900 (CharShape pos dedupe) |
| D-2 | `src/parser/hwp3/mod.rs` | 2570~2585, 2908~2924 (font name 매핑) |

→ **2 파일** (`drawing.rs` + `mod.rs`) 한정 수정. 다른 parser/renderer/model 무수정.

---

## 9. 다음 단계 (Stage 6)

최종 보고서 갱신 (4 격차 모두 완료 반영) + orders 갱신 + PR 생성 (작업지시자 승인 후).
