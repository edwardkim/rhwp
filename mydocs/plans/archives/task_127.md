# 타스크 127 수행계획서 — measureText LRU 캐싱

## 배경

### 현재 문제

rhwp의 텍스트 폭 측정은 2단계 파이프라인으로 동작한다:

```
measure_char_width_hwp() (layout.rs:6848)
  ├─ 1차: 내장 메트릭 조회 (font_metrics_data.rs, 582개 폰트)
  │       → 히트: 즉시 반환 (JS 호출 없음)
  └─ 2차: JS 브릿지 폴백 (js_measure_text_width)
          → WASM→JS 크로스바운더리 호출 → Canvas measureText()
```

타스크 125에서 582개 폰트 메트릭을 WASM에 내장하여 대부분의 문자는 1차에서 처리된다. 그러나 **JS 브릿지 폴백 경로에는 캐싱이 전혀 없어**, 미등록 폰트나 미커버 유니코드 범위의 문자가 반복 측정된다.

### JS 브릿지 폴백이 발생하는 경우

1. **미등록 폰트**: 582개 메트릭에 없는 폰트 (특수 HWP 폰트, 사용자 설치 폰트)
2. **미커버 유니코드 범위**: Latin Extended (U+0100~), Greek (U+0370~), Cyrillic (U+0400~), Box Drawing (U+2500~), 화살표 (U+2190~) 등
3. **이중 측정**: `estimate_text_width()`와 `compute_char_positions()`가 동일 문자를 각각 호출

### 한컴 webhwp 참조

한컴 webhwp는 LRU 캐시(128 엔트리)를 사용한다 (`mydocs/tech/webhwp/text_measurement.md`):
- 캐시 키: `char + fontName + (4096 × sizeCode + variant)`
- 용량 초과 시 가장 오래된 25% 제거
- 추정 히트율: 80~90%

### 현재 코드 분석

**JS 브릿지 선언** (`layout.rs:37-44`):
```rust
#[wasm_bindgen(js_namespace = globalThis, js_name = "measureTextWidth")]
fn js_measure_text_width(font: &str, text: &str) -> f64;
```

**JS 구현** (`wasm-bridge.ts:45-60`):
- Canvas ctx 재사용, `lastFont` 단일 캐시만 보유
- 측정 결과 캐시 없음

**핵심 포인트**: `js_measure_text_width`는 항상 1000px 고정 크기(`build_1000pt_font_string`)로 측정한다. 결과(`raw_px`)는 font_size와 무관하므로, `(measure_font, char)` 쌍을 키로 캐싱하면 모든 font_size에서 재사용 가능하다.

## 해결 방향

Rust 측에 LRU 캐시를 도입하여 `js_measure_text_width` 호출 결과를 캐싱한다. WASM↔JS 브릿지 오버헤드 자체를 제거한다.

## 구현 단계 (3단계)

---

### 1단계: Rust LRU 캐시 구현

**파일**: `src/renderer/layout.rs`

- `MeasureCache` 구조체 (Vec 기반 LRU, 256 엔트리)
- `thread_local!` 인스턴스 (WASM 단일 스레드)
- `measure_cache_key()` 해시 함수
- `cached_js_measure()` 래퍼 함수

**캐시 구조**:
```rust
struct MeasureCache {
    entries: Vec<(u64, f64)>,   // (key_hash, raw_px) — 접근 순서 (최근이 뒤)
    capacity: usize,            // 256
}
```

- 히트 시 엔트리를 맨 뒤(MRU)로 이동
- 용량 초과 시 가장 오래된 25% 제거 (webhwp 방식)

---

### 2단계: 캐시 적용

**파일**: `src/renderer/layout.rs`

2곳의 `js_measure_text_width` 호출을 `cached_js_measure`로 교체:

1. `measure_char_width_hwp()` (라인 6860) — 개별 문자 측정
2. `measure_hangul_width_hwp()` (라인 6877) — 한글 '가' 대리 측정

---

### 3단계: 통합 테스트 및 검증

| 항목 | 방법 |
|------|------|
| 571개 회귀 테스트 | `docker compose run --rm test` |
| WASM 빌드 | `docker compose run --rm wasm` |
| 캐시 무결성 | 캐시 유무에 따른 렌더링 결과 동일 확인 |

---

## 변경 파일 요약

| 파일 | 변경 내용 | 규모 |
|------|-----------|------|
| `src/renderer/layout.rs` | MeasureCache 구조체 + cached_js_measure 함수 + 2곳 호출 교체 | ~60줄 |

## 설계 결정 근거

| 결정 | 이유 |
|------|------|
| Rust 측 캐시 (JS 아님) | WASM↔JS 브릿지 오버헤드 자체를 제거 |
| (measure_font, char) 키 | 1000px 고정 측정이므로 font_size 무관 → 재사용성 극대화 |
| Vec 기반 LRU (HashMap 아님) | 256 엔트리 선형탐색 = 수 μs, JS 브릿지 1회(~50μs)보다 빠름. WASM 바이너리 절약 |

## 기대 효과

| 항목 | 현재 | 적용 후 |
|------|------|---------|
| 미등록 폰트 이중 측정 | estimate + compute 각각 JS 호출 | 1회 JS + 1회 캐시 히트 |
| 반복 문자 (같은 스타일) | 매번 JS 호출 | 첫 1회만 JS, 이후 캐시 |
| WASM 바이너리 크기 | 1.83MB | 미미한 증가 (~200B 코드) |
| 변경 규모 | — | 1개 파일, ~60줄 |
