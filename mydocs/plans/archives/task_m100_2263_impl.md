# 구현 계획서 — M100 #2263: BinData 지연 로딩

수행계획서: `task_m100_2263.md`

## 사전 조사로 확정된 사실

| 항목 | 확인 결과 | 설계 영향 |
|---|---|---|
| 조회 경계 | `find_bin_data()` (`src/renderer/layout/utils.rs:22`) 로 수렴. 호출부 23곳 | 교체 지점 단일 |
| 조회 방식 | **위치 기반** `get(bin_data_id - 1)` + id 폴백 | 배열의 **순서·길이 보존 필수** |
| 호출부 사용 패턴 | 대부분 이미 `.data.clone()` 으로 바이트를 **복사**함 | 저장소는 **캐시 불필요**, 요청 시 압축 해제 후 반환 |
| 렌더 트리 | `ImageNode { data: Vec<u8> }` (`src/renderer/layout.rs:90`) 가 바이트를 복사 보유, 페이지 단위 캐시 | 페이지 단위라 유계. 지배항 아님 |
| 직렬화 | `cfb_writer` / `hwpx` 가 `bin_data_content` 를 **전량 소비**하고 `push` 도 함 | 저장 시 전체 바이트 접근 가능해야 함 |

→ **결론**: `Vec<BinDataContent>` 배열 구조는 유지하고, **각 항목의 바이트만 지연화**한다. 파급이 가장 작고 위치 기반 조회·직렬화 의미가 보존된다.

## 설계

```rust
// src/model/bin_data.rs
pub struct BinDataContent {
    pub id: u16,
    pub extension: String,
    pub data: BinDataBytes,   // 기존: Vec<u8>
}

pub enum BinDataBytes {
    Loaded(Vec<u8>),          // 메모리 상주 (직렬화기 push, HML/HWP3, 테스트)
    Lazy { source: Arc<BinSource>, key: BinKey },  // 요청 시 압축 해제
}

impl BinDataBytes {
    pub fn load(&self) -> Vec<u8>;   // 호출부의 기존 `.data.clone()` 자리를 대체
    pub fn is_empty(&self) -> bool;
}
```

- `BinSource` 는 **원본 컨테이너 바이트**(`Arc<Vec<u8>>`)를 보관한다. ZIP(deflate)/CFB(zlib) 안의 이미지는 압축 상태이므로 원본 보관 비용은 파일 크기에 그친다.
- `BinKey` 는 재로딩에 필요한 최소 정보만 담는다 (HWPX: href + 내부 OLE 여부 / HWP5: 스토리지 스트림명).
- 캐시를 두지 않는다 — 소비자가 어차피 복사하므로 이중 상주를 만들지 않는 편이 목적에 부합한다.

## 단계

### 1단계 — `BinDataBytes` 타입 도입 (순수 리팩터링, 동작 불변)

- `BinDataContent.data` 를 `Vec<u8>` → `BinDataBytes` 로 교체, 모든 생산자는 `Loaded` 로 생성.
- 호출부 23곳 + `&[BinDataContent]` 시그니처 35곳 마이그레이션 (`.data.clone()` → `.data.load()`).
- **검증**: `cargo build`, `cargo test`, `cargo clippy` 통과. 동작·메모리 변화 없음.

### 2단계 — HWPX 지연 로딩

- HWPX 파서(`src/parser/hwpx/mod.rs:317~`)가 ZIP 원본 바이트를 `Arc` 로 보관하고, BinData 항목을 `Lazy` 로 등록.
- `normalize_internal_ole_data` 후처리를 지연 로드 시점에 동일 적용.
- 로드 실패 placeholder(`#1917`) 의미 보존.
- **검증**: `hwpx_roundtrip_baseline`, SVG 회귀, 메모리 재측정.

### 3단계 — HWP5 지연 로딩

- HWP5 파서(`src/parser/mod.rs:1275` / `:579` lenient)가 CFB 원본 바이트를 `Arc` 로 보관하고 `Lazy` 등록.
- 스트림 압축 해제(`decompress_stream`) 를 지연 시점에 적용, 실패 시 원본 폴백 의미 보존.
- **검증**: 저장(`cfb_writer`) 무결성, SVG 회귀, 메모리 재측정.

### 4단계 — 검증 및 보고

- 메모리 재측정 (`/usr/bin/time -l`, 1페이지 렌더 RSS) — 스파이크와 동일 조건 비교.
- `cargo test`, `cargo clippy`, `hwpx_roundtrip_baseline`, WASM 빌드.
- 단계별/최종 보고서 작성.

## 통과 기준

- 압축 해제 이미지 총량이 RSS를 지배하지 않는다.
- SVG 시각 회귀 없음.
- `hwpx_roundtrip_baseline` 기존 통과 유지 (이미지 유실 없음).
- `cargo test` / `cargo clippy` 통과.
