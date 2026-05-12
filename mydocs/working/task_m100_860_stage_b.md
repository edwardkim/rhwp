# Task #860 Stage B 단계 보고서

**선행**: Stage A (BMP URI 미지원 본질 식별)
**브랜치**: `local/task860`
**작성일**: 2026-05-13

## 작업 요약

`dib_to_bmp_data_url` (`src/emf/converter/player.rs:368`) 정정 후보 평가 + 회귀 위험.

## 정정 후보

### 후보 1 (권장): 기존 `bmp_bytes_to_png_bytes` 재사용

`src/renderer/svg.rs:2398` 에 이미 `bmp_bytes_to_png_bytes` 함수 존재 (image crate, BMP→PNG 변환). 이를 재사용.

```rust
// src/emf/converter/player.rs:368
fn dib_to_bmp_data_url(bmi: &[u8], bits: &[u8]) -> String {
    let bmi_size  = bmi.len() as u32;
    let bits_size = bits.len() as u32;
    let file_size = 14 + bmi_size + bits_size;
    let data_offset = 14 + bmi_size;

    let mut bmp = Vec::with_capacity(file_size as usize);
    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&file_size.to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());
    bmp.extend_from_slice(&data_offset.to_le_bytes());
    bmp.extend_from_slice(bmi);
    bmp.extend_from_slice(bits);

    // [Task #860] SVG renderer (rsvg-convert, 브라우저) 가 data:image/bmp URI 미지원.
    // BMP → PNG 재인코딩 후 data:image/png URI 로 embed (svg.rs:1118 / shape_layout.rs:1063
    // 와 동일 정책). image crate (Cargo.toml 의존성, features=["bmp","png"]) 사용.
    if let Some(png) = crate::renderer::svg::bmp_bytes_to_png_bytes(&bmp) {
        let b64 = base64::engine::general_purpose::STANDARD.encode(&png);
        format!("data:image/png;base64,{b64}")
    } else {
        // fallback: BMP (기존 동작)
        let b64 = base64::engine::general_purpose::STANDARD.encode(&bmp);
        format!("data:image/bmp;base64,{b64}")
    }
}
```

**장점**:
- 단일 위치 변경 (`dib_to_bmp_data_url` 만)
- 기존 함수 (`bmp_bytes_to_png_bytes`) 재사용 → 코드 중복 없음
- BMP decode 실패 시 fallback 으로 기존 BMP URI 유지 (graceful degradation)
- 다른 sample 의 EMF 처리에도 동일 개선 효과

**단점**:
- emf module 이 renderer module 에 의존 (= cross-module dependency 추가)
- `bmp_bytes_to_png_bytes` 의 visibility `pub(crate)` → 같은 crate 안 호출 가능

### 후보 2: EMF module 내부 BMP→PNG 변환 (renderer 의존성 회피)

`src/emf/converter/player.rs` 안에 자체 BMP→PNG 변환 함수 추가.

**장점**: emf module 의 독립성 유지.
**단점**: 코드 중복 (이미 svg.rs 에 있음).

### 후보 3: SVG 임베딩 단계에서 BMP→PNG (renderer 측)

`dib_to_bmp_data_url` 유지 + SVG 임베드 호출 위치에서 BMP→PNG 변환.

**장점**: emf module 변경 최소.
**단점**: emf module 의 출력이 BMP URI 라는 의미가 모호해짐 + 새 코드 추가.

## 권장

**후보 1** (기존 함수 재사용).

## 회귀 위험 평가

### 영향 범위

`dib_to_bmp_data_url` 의 caller:
- `src/emf/converter/player.rs:184` `emit_bitmap` → `Record::StretchDIBits` 처리
- EMF 안의 모든 DIB image 가 영향

### 잠재적 회귀

| 회귀 위치 | 영향 | 평가 |
|---|---|---|
| `src/emf/tests.rs:712` `assert!(svg.contains("href=\"data:image/bmp;base64,"))` | test 실패 | **test update 필요** (= `data:image/png`) |
| EMF image 보유 다른 sample (모든 sample) | image format 변경 (BMP→PNG) | lossless 변환 → **시각 동일 또는 개선** |
| 기존 BMP URI 의존 코드 | 없음 (확인) | OK |

### 회귀 검증 대상

- `cargo test --release --lib` (1246 passed 기대, emf test 1 개 update 필요)
- HWP3 sample (sample14 본 fix 대상)
- HWP3 sample13 (다른 HWP3 sample 회귀 검사)
- HWP5/HWPX 변환본 (hwp3-sample14-hwp5.hwp / .hwpx)
- 기존 EMF 보유 sample (필요 시 추가 검사)

## 정정 순서

1. **`bmp_bytes_to_png_bytes` 가용성 확인**: `pub(crate)` → emf module 에서 호출 가능
2. **`dib_to_bmp_data_url` 변경**: BMP 생성 → PNG 변환 → data URI
3. **test update**: `src/emf/tests.rs:712` 의 BMP assert → PNG assert
4. **검증**:
   - cargo build --release
   - cargo test (1246 passed)
   - sample14 page 2 PNG 시각 — 박스 내부 텍스트 표시 확인
   - sample13 회귀 검사
   - HWP5/HWPX 변환본 검증

## 산출 아티팩트

- 본 보고서

## 후속 단계

Stage C: 정정 구현 (후보 1) + 검증.
