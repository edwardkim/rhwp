# Task #902 v2 Stage 26 보고서 — textLength 강제 fit (visual stacking 해결)

**Stage**: 26 / 27 (v2 점진 포팅)
**상태**: 완료

## 1. ROOT CAUSE

사용자 보고: "WMF 네모 안에 정확하게 글자가 표시안되고 밀림"

**원인**: WMF DX 는 한컴 굴림체 (Windows 전용) 의 glyph metric 을 가정하여 작성됨. 우리가 사용하는 NanumGothic / Apple SD Gothic Neo 등의 fallback 폰트는 glyph 너비가 다름 (보통 더 넓음). 결과:
- 텍스트 위치 (origin) 는 정확
- 각 glyph 가 의도한 너비 (WMF DX) 보다 넓게 렌더 → 인접 요소와 겹침 / 박스 밖으로 밀림

## 2. 해결 — SVG textLength

각 tspan 에 `textLength="DX"` + `lengthAdjust="spacingAndGlyphs"` 추가:

```xml
<tspan x="291" textLength="117" lengthAdjust="spacingAndGlyphs">전</tspan>
```

브라우저가 glyph 너비를 textLength 에 강제 fit (수평 scale). 폰트 무관 정합.

## 3. 구현 (src/wmf/converter/svg/mod.rs)

```rust
// Stage 26 추가:
if dx_advance > 0 {
    tspan = tspan
        .set("textLength", dx_advance)
        .set("lengthAdjust", "spacingAndGlyphs");
}
```

## 4. 효과

| 환경 | Stage 25 (font embed only) | Stage 26 (+ textLength) |
|------|---------------------------|------------------------|
| rhwp-studio (WASM) | NanumGothic 사용, glyph 너비 NanumGothic 의존 | NanumGothic + WMF DX 강제 fit → 박스 정합 |

## 5. 검증

```
cargo build --release           — Finished
cargo test --release --all-targets — 1412 passed / 0 failed
wasm-pack build --release       — pkg/rhwp_bg.wasm 4.88 MB (May 16 00:08)
```

## 6. 잔존 한계

- 미세 왜곡 가능: glyph 가 강제 scaling 되어 약간 압축/늘어남
- 줄바꿈 / 텍스트 wrap 미적용 (WMF 는 single-line text 만)

## 7. 다음 단계

Stage 27: 최종 보고서 + PR
