# Task 127 Execution Plan -- measureText LRU Caching

## Background

### Current Problem

rhwp's text width measurement operates as a 2-stage pipeline:

```
measure_char_width_hwp() (layout.rs:6848)
  +-- 1st: Embedded metric lookup (font_metrics_data.rs, 582 fonts)
  |       -> Hit: immediate return (no JS call)
  +-- 2nd: JS bridge fallback (js_measure_text_width)
          -> WASM->JS cross-boundary call -> Canvas measureText()
```

Task 125 embedded metrics for 582 fonts in WASM, so most characters are handled in the 1st stage. However, **the JS bridge fallback path has no caching at all**, causing repeated measurements for unregistered fonts or uncovered Unicode ranges.

### Cases Where JS Bridge Fallback Occurs

1. **Unregistered fonts**: Fonts not in the 582 metrics (special HWP fonts, user-installed fonts)
2. **Uncovered Unicode ranges**: Latin Extended (U+0100~), Greek (U+0370~), Cyrillic (U+0400~), Box Drawing (U+2500~), Arrows (U+2190~), etc.
3. **Double measurement**: `estimate_text_width()` and `compute_char_positions()` each call for the same characters

### Hancom webhwp Reference

Hancom webhwp uses an LRU cache (128 entries) (`mydocs/tech/webhwp/text_measurement.md`):
- Cache key: `char + fontName + (4096 * sizeCode + variant)`
- On capacity overflow: remove oldest 25%
- Estimated hit rate: 80~90%

### Current Code Analysis

**JS bridge declaration** (`layout.rs:37-44`):
```rust
#[wasm_bindgen(js_namespace = globalThis, js_name = "measureTextWidth")]
fn js_measure_text_width(font: &str, text: &str) -> f64;
```

**JS implementation** (`wasm-bridge.ts:45-60`):
- Canvas ctx reused, only single `lastFont` cache
- No measurement result cache

**Key point**: `js_measure_text_width` always measures at 1000px fixed size (`build_1000pt_font_string`). The result (`raw_px`) is independent of font_size, so caching with `(measure_font, char)` pair as key allows reuse across all font_sizes.

## Solution Direction

Introduce an LRU cache on the Rust side to cache `js_measure_text_width` call results. This eliminates the WASM-JS bridge overhead itself.

## Implementation Phases (3 Phases)

---

### Phase 1: Rust LRU Cache Implementation

**File**: `src/renderer/layout.rs`

- `MeasureCache` struct (Vec-based LRU, 256 entries)
- `thread_local!` instance (WASM single-threaded)
- `measure_cache_key()` hash function
- `cached_js_measure()` wrapper function

**Cache structure**:
```rust
struct MeasureCache {
    entries: Vec<(u64, f64)>,   // (key_hash, raw_px) -- access order (most recent at end)
    capacity: usize,            // 256
}
```

- On hit: move entry to the end (MRU)
- On capacity overflow: remove oldest 25% (webhwp pattern)

---

### Phase 2: Cache Application

**File**: `src/renderer/layout.rs`

Replace `js_measure_text_width` calls with `cached_js_measure` at 2 locations:

1. `measure_char_width_hwp()` (line 6860) -- individual character measurement
2. `measure_hangul_width_hwp()` (line 6877) -- Korean 'ga' surrogate measurement

---

### Phase 3: Integration Testing and Verification

| Item | Method |
|------|--------|
| 571 regression tests | `docker compose run --rm test` |
| WASM build | `docker compose run --rm wasm` |
| Cache integrity | Verify rendering results identical with/without cache |

---

## Changed Files Summary

| File | Changes | Scope |
|------|---------|-------|
| `src/renderer/layout.rs` | MeasureCache struct + cached_js_measure function + 2 call site replacements | ~60 lines |

## Design Decision Rationale

| Decision | Reason |
|----------|--------|
| Rust-side cache (not JS) | Eliminates WASM-JS bridge overhead itself |
| (measure_font, char) key | 1000px fixed measurement means font_size-independent -> maximizes reusability |
| Vec-based LRU (not HashMap) | 256-entry linear search = few us, faster than 1 JS bridge call (~50us). Saves WASM binary size |

## Expected Benefits

| Item | Current | After |
|------|---------|-------|
| Unregistered font double measurement | estimate + compute each make JS calls | 1 JS call + 1 cache hit |
| Repeated characters (same style) | JS call every time | Only first call to JS, cache thereafter |
| WASM binary size | 1.83MB | Negligible increase (~200B code) |
| Change scope | -- | 1 file, ~60 lines |
