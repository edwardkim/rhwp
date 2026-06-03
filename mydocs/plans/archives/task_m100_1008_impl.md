# Task #1008 구현 계획서 (v2 — 재작성)

**Issue**: [#1008 HWP3 sample16 Shape/Text 정합 격차 종합](https://github.com/edwardkim/rhwp/issues/1008)
**Branch**: `local/task1008` (base = `local/devel` = `65c8e693`)

---

## 1. 사전 단언 (HWP3 parser 구조 파악 완료)

### 1.1 HWP3 parser 모듈 구조

```
src/parser/hwp3/
├── drawing.rs          (863 lines) — Shape control / Drawing object 파싱 ★
├── encoding.rs         (66 lines)
├── johab.rs            (90 lines) — 한글 johab encoding
├── johab_map.rs        (5900 lines) — johab → unicode 매핑 테이블
├── mod.rs              (3248 lines) — top-level parser ★
├── ole.rs              (151 lines)
├── paragraph.rs        (103 lines) — paragraph 파싱
├── records.rs          (465 lines)
└── special_char.rs     (210 lines)
```

### 1.2 격차 A — gradient 데이터는 **이미 파싱됨**, IR 매핑만 누락

`src/parser/hwp3/drawing.rs:149-170`:
```rust
pub struct Hwp3DrawingObjectGradientAttr {
    pub start_color: u32,
    pub end_color: u32,
    pub kind: u32,       // gradient_type 으로 매핑
    pub angle: u32,
    pub center_x: u32,
    pub center_y: u32,
    pub step: u32,
}
```

`drawing.rs:216`: `Hwp3DrawingObjectHeader.gradient_attr: Option<Hwp3DrawingObjectGradientAttr>`
`drawing.rs:252-253`: `if basic_attr.has_gradient() { Some(Hwp3DrawingObjectGradientAttr::read(...)) }`

**그러나** `drawing.rs:792-806` 에서 최종 `Fill` IR 구축 시:
```rust
let fill = Fill {
    fill_type: crate::model::style::FillType::Solid,  // ← 하드코딩
    solid: Some(...),
    gradient: None,                                    // ← 누락
    image: None,
    alpha: 0,
};
```

→ **격차 A fix 는 단순 매핑**: `header.gradient_attr.as_ref()` 가 `Some` 이면 `fill_type=Gradient` + `fill.gradient=Some(GradientFill { ... })` 주입.

매핑 표:

| HWP3 (`Hwp3DrawingObjectGradientAttr`) | IR (`model::style::GradientFill`) |
|---------------------------------------|-----------------------------------|
| `kind: u32`                           | `gradient_type: i16` (1=줄무늬, 2=원형, 3=원뿔, 4=사각) |
| `angle: u32`                          | `angle: i16` |
| `center_x: u32`                       | `center_x: i16` |
| `center_y: u32`                       | `center_y: i16` |
| `start_color: u32`, `end_color: u32`  | `colors: vec![start, end]` (2-stop) |
| (없음, 2-stop 기본)                   | `positions: vec![0, 100]` |
| `step: u32`                           | `blur: i16` 또는 `step_center: u8` (Stage 2 단언) |

### 1.3 격차 B, C, D — Stage 1 진단 필요

위치 후보:
- **격차 B (border style)**: `drawing.rs:758~775` (`raw_attr` border 비트 처리). 정확한 비트 매핑 단언 필요
- **격차 C (HEAD numbering)**: `src/parser/hwp3/mod.rs` 의 HEAD/auto-numbering 처리 (위치 미정 — Stage 1)
- **격차 D (한글 공백)**: `src/parser/hwp3/encoding.rs` / `johab.rs` / paragraph text 처리 (위치 미정 — Stage 1)

---

## 2. Stage 진행 계획 (v2 — 6 stages)

### Stage 1 — 종합 진단

**Step 1.1 — 격차 A 추가 단언**:
- `step` 필드의 IR 매핑 (blur vs step_center) — HWP5 변환본의 gradient parsing 코드와 비교
- gradient 2-stop 외 multi-stop 가능 여부 단언
- HWP3 다른 sample 의 gradient Shape 분포 (sample10/11/13/14 등)

**Step 1.2 — 격차 B 단언**:
- `drawing.rs:758~775` border attr 비트 해석 분석
- HWP3 `style=0x0002` 의 LineType 의미 확인 — 실제로 점선인지 한컴이 실선 처리하는지
- HWP5 변환본의 `style=0xc0010043` 와 비교 (왜 변환 시 다른 비트로 저장되는지)
- 다른 fixture 의 점선 box 수집 (시험지/aift) — 실제 점선 정합 사례

**Step 1.3 — 격차 C 단언**:
- HWP3 의 HEAD/numbering record 위치 (mod.rs 내 grep)
- "■1.추진목적■" 의 "■" 가 어디서 오는지 추적 (auto-numbering prefix? special_char?)
- HWP5 변환본 parser 의 HEAD 처리 비교

**Step 1.4 — 격차 D 단언 + 영향 범위**:
- HWP3 sample16 cover paragraph 의 text run dump (`rhwp dump -s 0 -p 0`)
- 공백 누락이 char_shape 분할 단계인지 johab 디코딩 단계인지 단언
- HWP3 sample 전체 (sample4/5/10/11/13/14/16/19/sample10) text 공백 sweep — 회귀 risk 정량화
- **본 task 포함 / 분리 결정** (작업지시자 결정)

**산출물**:
- baseline SVG 보존 (`output/poc/pr1008/before/`)
- `mydocs/working/task_m100_1008_stage1.md` — 격차별 root cause 위치 + 영향 범위 단언 + 격차 D 분리 여부

**커밋**: "Task #1008 Stage 1: 4 격차 진단 + root cause 위치 + 격차 D 범위 단언"

### Stage 2 — 격차 A fix (HWP3 Shape gradient IR 매핑)

**Step 2.1**: `src/parser/hwp3/drawing.rs:792~806` 의 Fill 구축 변경
```rust
let (fill_type, gradient) = if let Some(g) = header.gradient_attr.as_ref() {
    use crate::model::style::{ColorRef, GradientFill};
    let grad = GradientFill {
        gradient_type: g.kind as i16,
        angle: g.angle as i16,
        center_x: g.center_x as i16,
        center_y: g.center_y as i16,
        blur: g.step as i16,  // Stage 1 결과 따라 step_center 일 수 있음
        step_center: 0,        // 또는 g.step
        colors: vec![g.start_color, g.end_color],
        positions: vec![0, 100],
    };
    (crate::model::style::FillType::Gradient, Some(grad))
} else {
    (crate::model::style::FillType::Solid, None)
};
let fill = Fill {
    fill_type,
    solid: ..., // 기존
    gradient,
    image: None,
    alpha: 0,
};
```

**Step 2.2**: 단위 테스트 — `tests/issue_1008_gradient.rs`
- HWP3 sample16 → pi=71 Shape control → `fill.fill_type == Gradient` 단언
- `fill.gradient.is_some()` + colors / positions 정합

**Step 2.3**: 시각 단언
- sample16 p2 export-svg → SVG 에 `<linearGradient>` 또는 `<radialGradient>` 노드 존재 단언
- 작업지시자 한컴 viewer 시각 검증 (rsvg-convert 셀프 + 한컴 비교)

**커밋**: "Task #1008 Stage 2: 격차 A — HWP3 Shape gradient IR 매핑 (이미 파싱된 gradient_attr 활용)"

### Stage 3 — 격차 C fix (HWP3 HEAD numbering)

**Step 3.1**: Stage 1 진단 결과 위치 변경
**Step 3.2**: 단위 테스트 + 시각 단언
**Step 3.3**: 다른 HWP3 sample 의 HEAD 라벨 sweep — 회귀 0 단언

**커밋**: "Task #1008 Stage 3: 격차 C — HWP3 HEAD numbering 형식 정합"

### Stage 4 — 격차 B fix (Shape border 실선)

**Step 4.1**: Stage 1 진단 결과 위치 변경 (parser 또는 renderer)
**Step 4.2**: 시각 단언 + 다른 fixture (시험지/aift 의 점선 box) 회귀 0 단언

**커밋**: "Task #1008 Stage 4: 격차 B — LineType 비트 해석 정합"

### Stage 5 — 격차 D fix (HWP3 한글 공백) — 조건부

Stage 1 step 1.4 에서 본 task 포함 결정 시 진행. 분리 결정 시 새 issue 등록 후 본 stage 생략.

**Step 5.1**: Stage 1 진단 결과 위치 변경
**Step 5.2**: HWP3 sample 전체 text sweep — 회귀 0 단언

**커밋**: "Task #1008 Stage 5: 격차 D — HWP3 한글 공백 정합" (조건부)

### Stage 6 — 종합 회귀 sweep + 최종 보고 + PR

**Step 6.1**: 종합 sweep
```bash
# 변환본 9 종
for f in samples/hwp3-sample{4,5,10,11,13,14,16,19}-hwp5.hwp samples/hwp3-sample16-hwp5.hwpx; do ...; done

# HWP3 native 9 종
for f in samples/hwp3-sample{4,5,10,11,13,14,16,19}.hwp samples/hwp3-sample16.hwp; do ...; done

# 일반 HWP5
samples/sample10.hwp samples/exam_{kor,eng,math,science,social}.hwp samples/aift.hwp samples/biz_plan.hwp
```

**Step 6.2**: 전체 테스트
- `cargo test --release --lib` (1307+ passed)
- `cargo test --release --tests` (68+ passed)
- `cargo clippy --release --lib -- -D warnings` (clean)
- `cargo fmt --check` (clean)
- golden SVG snapshot diff 분류 (격차 A/B/C/D 영향 외 0 단언)

**Step 6.3**: 문서 작업
- `mydocs/working/task_m100_1008_stage{1,2,3,4,5}.md` 완성
- `mydocs/report/task_m100_1008_report.md` 작성
- `mydocs/orders/20260520.md` 또는 새 날짜 갱신
- WASM 빌드 (Docker, rhwp-studio/public 동기화)

**Step 6.4**: PR 생성 (작업지시자 승인 후)
- title: "Task #1008: HWP3 sample16 Shape/Text 정합 격차 종합 정정 (closes #1008)"

**커밋**: "Task #1008 Stage 6: 종합 회귀 sweep + 최종 보고"

---

## 3. 변경 위치 summary (Stage 1 진단 후 확정)

| 격차 | 추정 변경 위치 | Stage |
|------|---------------|-------|
| A | `src/parser/hwp3/drawing.rs:792~806` (IR 매핑) | Stage 2 |
| B | `src/parser/hwp3/drawing.rs:758~775` (border attr) 또는 renderer | Stage 4 |
| C | `src/parser/hwp3/mod.rs` (HEAD numbering 처리) | Stage 3 |
| D | `src/parser/hwp3/encoding.rs` / `johab.rs` / paragraph text | Stage 5 (조건부) |

---

## 4. 위험 + 완화

| 위험 | 완화 |
|------|------|
| 격차 A `step` 필드 IR 매핑 모호 (blur vs step_center) | Stage 1 step 1.1 에서 HWP5 변환본 동일 필드 비교 |
| 격차 B fix 가 시험지/aift 등 의도된 점선 회귀 | Stage 4 시각 sweep — 회귀 발견 시 fix scope 좁힘 (sample16 한정 case-specific) |
| 격차 C/D root cause 위치 모호 | Stage 1 진단 충실 — 단언 못하면 작업지시자에게 보고 후 방향 재결정 |
| 격차 D 광범위 회귀 risk | Stage 1 step 1.4 에서 영향 범위 정량화 + 분리 결정 |
| 격차 간 fix 상호 간섭 | 격차별 stage 분리 + 각 stage 끝 sweep |

---

## 5. 비대상 (수행계획서 §10 재확인)

- 페이지 수 격차 64/38 (별도 issue #1000 등)
- WMF rendering, 폰트 fallback
- Picture 객체 simplify / 둥근모서리 (round_radius)
- 일반 HWP5 / HWPX 시각 변경 (영향 없음 보장)
- WASM 빌드 (Stage 6 최종 1회)

---

## 6. 검증 명령 모음 (참고)

```bash
# Stage 1
./target/release/rhwp dump samples/hwp3-sample16.hwp -s 0 -p 71 | grep -A 20 "그림\|사각형\|채우기"
./target/release/rhwp dump samples/hwp3-sample16-hwp5.hwp -s 0 -p 71 | grep -A 20 "그림\|사각형\|채우기"
grep -n "gradient_attr\|has_gradient\|HEAD\|HwpTag.*Num" src/parser/hwp3/

# Stage 2 (격차 A)
cargo build --release 2>&1 | grep -E "error|Finished" | tail -3
./target/release/rhwp dump samples/hwp3-sample16.hwp -s 0 -p 71 | grep "채우기"
./target/release/rhwp export-svg samples/hwp3-sample16.hwp -p 2 -o /tmp/h3-p2-after/
grep -c "linearGradient\|radialGradient" /tmp/h3-p2-after/*.svg
cargo test --release --test issue_1008_gradient

# Stage 6 종합 sweep
cargo test --release --lib && cargo test --release --tests
cargo clippy --release --lib -- -D warnings && cargo fmt --check
for f in samples/hwp3-sample*.hwp samples/hwp3-sample*-hwp5.hwp samples/hwp3-sample*-hwp5.hwpx samples/exam_*.hwp samples/aift.hwp samples/biz_plan.hwp; do
  P=$(./target/release/rhwp dump-pages "$f" 2>/dev/null | grep -c "^=== 페이지")
  printf "  %-50s %s\n" "$(basename $f)" "$P 페이지"
done
```
