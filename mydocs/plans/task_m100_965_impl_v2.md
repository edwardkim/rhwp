# 구현 계획 V2 — Task #965 Stage 2 — PR #918 Stage 33-A 핵심 cherry-pick

- 이슈: [#965](https://github.com/edwardkim/rhwp/issues/965)
- Stage 1 결과: WMF baseline 측정 완료 (sample16/14/4 의 WMF page 식별)
- 원본 commit: `f53235c6` (PR #918 Stage 33-A)

## 1. 변경 위치 (3 영역, 단일 파일)

`src/wmf/converter/svg/mod.rs`:
1. **`ext_text_out` baseline 보정** (~813-828)
2. **`text_out` baseline 보정** (~1541-1553) — PR #918 미포함, 동일 결함 패턴
3. **`set_text_align` vertical bits 분기** (~2186-2197)

> **Note**: PR #918 의 commit `f53235c6` 은 ext_text_out 만 fix (set_text_align 도 fix). 그러나 `text_out` (META_TEXTOUT) 에 동일 버그 존재 → 본 task 에서 추가 fix.

## 2. 변경 내용

### 2.1 `ext_text_out` baseline 보정 (line 813-828)

#### Before
```rust
} + match self.context_current.text_align_vertical {
    VerticalTextAlignmentMode::VTA_TOP => {
        let em = font.height.abs();
        (em as f64 * 0.8) as i16
    }
    VerticalTextAlignmentMode::VTA_BASELINE
        | VerticalTextAlignmentMode::VTA_BOTTOM
        if font.height < 0 => {
        -font.height
    }
    _ => 0,
},
```

#### After (PR #918 f53235c6)
```rust
} + match self.context_current.text_align_vertical {
    // [Task #965 / PR #918 Stage 33-A] WMF 의 ExtTextOut y 는 text_align_vertical
    // 에 따라 reference point 가 결정된다:
    //   VTA_BASELINE (default): y 가 baseline — 그대로 사용
    //   VTA_TOP: y 가 cell 의 top — baseline = y + ascent
    //   VTA_BOTTOM: y 가 cell 의 bottom — baseline = y - descent
    // font.height 의 부호는 magnitude 의 해석 (cell vs char) 만 바꾸며
    // reference point 와 무관 (이전 구현이 font.height < 0 일 때
    // -font.height 만큼 y 를 더했던 것은 잘못된 보정).
    VerticalTextAlignmentMode::VTA_TOP => {
        let em = font.height.abs();
        (em as f64 * 0.8) as i16
    }
    VerticalTextAlignmentMode::VTA_BOTTOM => {
        let em = font.height.abs();
        -((em as f64 * 0.2) as i16)
    }
    VerticalTextAlignmentMode::VTA_BASELINE => 0,
    _ => 0,
},
```

### 2.2 `text_out` baseline 보정 (line 1541-1553) — 동일 패턴 fix

Same change as 2.1 applied to text_out function.

### 2.3 `set_text_align` vertical bits 분기 (line 2186-2197)

#### Before
```rust
let align_vertical = [
    VerticalTextAlignmentMode::VTA_BOTTOM,
    VerticalTextAlignmentMode::VTA_TOP,    // VTA_TOP = 0x0000
]
.into_iter()
.find(|a| record.text_alignment_mode & (*a as u16) == *a as u16)
.unwrap_or(VerticalTextAlignmentMode::VTA_BASELINE);
```

#### After (PR #918 f53235c6)
```rust
// [Task #965 / PR #918 Stage 33-A] SetTextAlign vertical bits 정합.
// WMF [MS-WMF] 2.1.2.18 TextAlignmentMode 의 vertical 부분:
//   TA_TOP = 0x0000 (default)
//   TA_BOTTOM = 0x0008
//   TA_BASELINE = 0x0018 (TA_BOTTOM | extra bit)
// 우리 enum 의 VTA_* 매핑:
//   VTA_TOP = 0x0000, VTA_BASELINE = 0x0018, VTA_LEFT = 0x0008.
// 이전 구현은 `mode & VTA_TOP(=0)` 가 항상 true 라서 BASELINE/BOTTOM 인 mode 도
// VTA_TOP 으로 매핑되어 baseline 이 cell-top 보정 (+ascent) 만큼 아래로 shift 되는
// 박스 외부 표시 회귀의 원인. 우선순위: BASELINE → BOTTOM → TOP.
let v_bits = record.text_alignment_mode & 0x0018;
let align_vertical = if v_bits == 0x0018 {
    VerticalTextAlignmentMode::VTA_BASELINE
} else if v_bits == 0x0008 {
    // TA_BOTTOM
    VerticalTextAlignmentMode::VTA_BOTTOM
} else {
    VerticalTextAlignmentMode::VTA_TOP
};
```

## 3. 제외 (별도 concern, 본 fix 와 무관)

| 영역 | PR #918 commit | 본 task |
|------|---------------|---------|
| `build_font_face_style` 호출 제거 (line 101-114) | 포함 | **제외** (woff2 의존 정리는 별도) |
| `build_font_face_style` 함수 제거 (line 2425+) | 포함 | **제외** |
| nested `<svg>` inline embed (renderer/svg.rs) | 포함 | **제외** (별도 issue) |
| LibreOffice emfio 포팅 | 포함 | **제외** |
| WASM RasterPlayer | 포함 | **제외** |
| DX byte-aware indexing | 포함 | **제외** |
| POLYPOLYGON fill-rule | 포함 | **제외** |

## 4. 영향 분석

### 4.1 직접 영향
- sample16 page 18 WMF 다이어그램: 박스 내부 한글 텍스트 정합
- 다른 WMF sample (sample14, sample4 등) — BASELINE 모드인 텍스트의 위치 변경 (정상화 방향)

### 4.2 잠재적 회귀
- WMF 의 일부 sample 에서 기존 잘못된 baseline 위치가 visual coincidentally 정합했을 case → fix 후 다른 위치 (사실상 정상화)

## 5. 위험 평가

| 위험 | 평가 | 완화 |
|------|------|------|
| WMF 보유 모든 sample 영향 | **중** | Stage 4 baseline 비교 |
| 다른 alignment case 회귀 (TA_CENTER 등) | **낮음** (vertical 만 변경) | cargo test |
| Filter 단순화로 인한 corner case 누락 | **낮음** (WMF spec 정확 반영) | - |

## 6. 검증 계획 (Stage 3-4)

### Stage 3 단위 검증
1. cargo build --release
2. sample16 page 18 WMF 다이어그램 SVG/PNG render
3. 박스 내부 한글 텍스트 정합 확인

### Stage 4 회귀 검증
1. `cargo test --release --lib` 전체 (1288 tests)
2. WMF 보유 sample baseline 비교 (Stage 1 PNG vs Post-Fix PNG)
3. golden SVG diff 회귀 0

## 7. 진행 규칙

- 자동진행 안함
- 각 stage 종료 시 보고서 + 명시 승인
- 회귀 발견 시 **즉시 revert + 보고**
