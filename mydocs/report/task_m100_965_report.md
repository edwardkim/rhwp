# Task #965 — 최종 보고서

- 이슈: [#965](https://github.com/edwardkim/rhwp/issues/965)
- 마일스톤: M100 / v1.0.0
- 브랜치: `local/task965`
- 기간: 2026-05-17 (1일)

## 1. 작업 범위

WMF 텍스트 박스 외부 표시 — SetTextAlign vertical bits 파싱 버그 fix.

PR [#918](https://github.com/edwardkim/rhwp/pull/918) (closed, 5082 additions) 의 Stage 33-A 핵심 height/baseline 보정 로직만 단독 포팅. PR #918 의 광범위 변경 (LO emfio, WASM raster, woff2, DX, POLYPOLYGON 등) 은 모두 **제외**.

## 2. Root cause

### 2.1 증상

`samples/hwp3-sample16.hwp` page 18 (한컴 page 16) 의 WMF 다이어그램 (주전산센터 목표시스템 구성안) 내부 박스의 한글 텍스트 ("PE6450", "기록서버", "Windows 서버군", "Unix 서버군" 등) 가 박스 외부로 벗어남.

### 2.2 코드 분석

`src/wmf/converter/svg/mod.rs:2191-2197` (Fix 전):

```rust
let align_vertical = [
    VerticalTextAlignmentMode::VTA_BOTTOM,
    VerticalTextAlignmentMode::VTA_TOP,    // VTA_TOP = 0x0000
]
.into_iter()
.find(|a| record.text_alignment_mode & (*a as u16) == *a as u16)
.unwrap_or(VerticalTextAlignmentMode::VTA_BASELINE);
```

`mode & VTA_TOP(=0x0000) == 0x0000` 가 **항상 true** → first match 로 BASELINE/BOTTOM 인 mode 도 VTA_TOP 으로 잘못 매핑.

### 2.3 결과

`ext_text_out` 에서 VTA_TOP 분기 발동 → y 에 +ascent (~em × 0.8) 추가 → text baseline 이 cell-top 보정만큼 아래로 shift → 박스 하단 라인 걸침.

WMF spec [MS-WMF] 2.1.2.18:
- TA_TOP = 0x0000 (default)
- TA_BOTTOM = 0x0008
- TA_BASELINE = 0x0018

vertical bits (0x0018 mask) 값 기준 정확 분기 필요.

## 3. Fix

`src/wmf/converter/svg/mod.rs` 3 영역:

### 3.1 `set_text_align` (~2186-2208)

```rust
let v_bits = record.text_alignment_mode & 0x0018;
let align_vertical = if v_bits == 0x0018 {
    VerticalTextAlignmentMode::VTA_BASELINE
} else if v_bits == 0x0008 {
    VerticalTextAlignmentMode::VTA_BOTTOM
} else {
    VerticalTextAlignmentMode::VTA_TOP
};
```

### 3.2 `ext_text_out` (~813-833) baseline y shift 정합

```rust
match self.context_current.text_align_vertical {
    VerticalTextAlignmentMode::VTA_TOP => {
        let em = font.height.abs();
        (em as f64 * 0.8) as i16     // +ascent
    }
    VerticalTextAlignmentMode::VTA_BOTTOM => {
        let em = font.height.abs();
        -((em as f64 * 0.2) as i16)  // -descent
    }
    VerticalTextAlignmentMode::VTA_BASELINE => 0,
    _ => 0,
}
```

### 3.3 `text_out` (~1541-1556) — PR #918 미포함, 본 task 추가

`META_TEXTOUT` 동일 baseline 보정 (ext_text_out 와 일관성).

## 4. 검증

### 4.1 cargo test
- `cargo test --release --lib`: **1288 passed, 0 failed, 2 ignored**

### 4.2 단위 검증 (sample16 page 18)
- WMF 박스 내부 한글 텍스트 정상 위치 ✓
- 한컴 viewer 정합

### 4.3 회귀 검증
- sample14 page 0~8 (WMF terminal screenshot): PNG diff <1% (정상)
- sample4 page 1 (WMF): identical (BASELINE 미사용)

## 5. 영향 평가

| 영역 | 영향 |
|------|------|
| WMF BASELINE/BOTTOM 모드 텍스트 | 정상 위치 (회귀 fix) |
| WMF TOP 모드 텍스트 | 기존 동작 (영향 없음) |
| 비-WMF | 영향 없음 (svg/mod.rs WMF converter 만 변경) |
| WASM 환경 | 정합 개선 예상 (Canvas2D 가 동일 SVG 사용) |

## 6. PR #918 와의 차이

| PR #918 (closed, 5082+) | 본 task #965 (~60) |
|------------------------|-------------------|
| LibreOffice emfio 포팅 | ❌ 제외 |
| WASM RasterPlayer | ❌ 제외 |
| nested SVG inline embed | ❌ 제외 |
| woff2 base64 임베드 제거 | ❌ 제외 |
| DX byte-aware indexing | ❌ 제외 |
| POLYPOLYGON fill-rule | ❌ 제외 |
| Stage 33-A height/baseline (set_text_align + ext_text_out) | ✅ 포팅 |
| **text_out baseline (PR #918 미포함)** | ✅ **본 task 추가** |

PR #918 close 사유 (다양한 부작용) 회피 + root cause fix 만 도입한 **작은 PR**.

## 7. 관련

- PR #918 (closed) — 본 fix 의 원본 root cause 발견
- 원 issue #952 의 5 sub-issue 와 유사 패턴 (root cause 식별 + 최소 침습 fix)

## 8. 후속

- 작업지시자가 PR 머지
- 추후 별도 task (선택):
  - nested `<svg>` inline embed (WASM Canvas2D 정합)
  - woff2 base64 임베드 정리
