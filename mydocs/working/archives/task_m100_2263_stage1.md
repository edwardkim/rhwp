# 단계별 완료 보고서 — M100 #2263 (1~3단계): BinData 지연 로딩 구현

## 단계 병합 사유

구현 계획서의 1단계(타입 도입) / 2단계(HWPX 지연) / 3단계(HWP5 지연)를 **하나의 커밋**으로 묶었다.

`BinDataContent.data` 의 타입을 `Vec<u8>` → `BinDataBytes` 로 바꾸는 순간 **모든 생산자·소비자가 동시에 마이그레이션되어야 컴파일된다**. 따라서 1단계만 담은 "개별적으로 빌드 가능한 커밋"을 만들 수 없다. 단계별 검증은 순차로 수행했으나(1단계 완료 시점에 `cargo check` / `clippy` 통과 확인), 커밋은 원자적으로 남긴다.

## 구현 내용

### 1단계 — `BinDataBytes` 타입 도입

`src/model/bin_data.rs`:

- `BinDataContent.data: Vec<u8>` → `BinDataBytes`
- `BinDataBytes` = `Loaded(Vec<u8>)` | `Lazy { resolver, key }`
- `BinDataResolver` 트레이트 — 원본 컨테이너를 보유한 **파서 측**이 구현한다. model → parser 역의존을 피하기 위한 경계.
- 접근자: `load()`(바이트 획득), `len()`, `is_empty()`. `From<Vec<u8>>` 로 기존 생성 지점의 churn 최소화.

**캐시를 두지 않는다.** 소비자(레이아웃의 `ImageNode`, 직렬화기)가 어차피 바이트를 복사해 보유하므로, 여기서 캐시하면 이중 상주가 되어 목적에 반한다.

마이그레이션: 생성 지점 31곳(`.into()`), 접근 지점 다수(`.data.clone()` → `.data.load()`). `get_bin_data()` 는 빌림을 줄 수 없어 `Option<&[u8]>` → `Option<Vec<u8>>` 로 변경(호출부는 테스트뿐).

### 2단계 — HWPX 지연 로딩

`src/parser/hwpx/mod.rs`:

- `HwpxBinResolver` — ZIP 원본(`HwpxReader`)을 `Mutex` 로 보유하고 요청 시 엔트리를 inflate.
- `normalize_internal_ole_data` 를 바이트 전용 `normalize_ole_bytes` 로 분리해 지연 시점에도 동일 정규화 적용. 대상 href 집합을 리졸버가 보유.
- BinData 루프에서 바이트를 읽지 않고 `Lazy` 로 등록.
- Chart XML(`Chart/chartN.xml`)은 존재 여부 탐색이 필요하고 크기도 작아 **eager 유지**.

### 3단계 — HWP5 지연 로딩

`src/parser/mod.rs`:

- `Hwp5BinResolver` — CFB 원본(`CfbReader`)을 `Mutex` 로 보유하고 요청 시 스트림을 읽어 `decompress_stream` 적용. OLE Storage 선두 4-byte size prefix 스킵(Task #195)도 지연 시점에 재현.
- **순서 보존이 관건**: 기존 코드는 읽기 실패 시 항목을 배열에 넣지 않고 건너뛴다. 조회가 위치 기반(`find_bin_data` 의 `get(id-1)`)이므로 항목이 밀리면 그대로 깨진다. `has_stream()` 으로 **압축 해제 없이 존재만 확인**해 기존 skip 의미를 보존했다.
- 리졸버 생성 실패 시 기존 즉시 로드 경로로 폴백.
- lenient 경로(손상 파일 폴백)는 드물게 쓰이므로 **eager 유지** — 정확성 우선.

## 검증 중 발견하고 고친 결함

`is_empty()` 를 처음에 "`Lazy` 면 엔트리가 존재하므로 비어 있지 않다"로 구현했는데, **매니페스트에는 있으나 실제 엔트리가 없거나 읽기에 실패하는 경우**(#1917 이 다루는 상황)를 놓친 잘못된 가정이었다. `serializer::hwpx::tests::issue1917_bindata_load_failure_preserves_pic_control` 가 이를 잡아냈고, 실제 바이트를 해석해 판정하도록 수정했다. 렌더 핫패스에서 이 메서드를 쓰는 곳이 없어 비용 부담은 없다.

## 검증 결과

### 메모리 (`/usr/bin/time -l`, 1페이지 렌더 최대 RSS)

파싱 + 조판 경로(렌더링 없음):

| 문서 특성 | 이전 | 이후 |
|---|---|---|
| 무손실 비트맵 다수 내장 (HWP5) | 244 MB | **49 MB** |
| 무손실 비트맵 다수 내장 (HWPX) | 244 MB | **47 MB** |
| 이미지 중간 규모 (HWP5) | 78 MB | **39 MB** |
| JPEG 다수 내장, 141쪽 (HWPX) | 97 MB | **77 MB** |
| 이미지 다수, 41쪽 (HWP5) | 185 MB | **40 MB** |

압축 해제 이미지 총량이 더 이상 RSS를 지배하지 않는다. **완료 기준 충족.**

### 시각 회귀

변경 전 바이너리와 변경 후 바이너리로 각각 전체 페이지 SVG를 내보내 **바이트 단위 비교**:

| 문서 | 페이지 | 결과 |
|---|---|---|
| 6개 문서 (HWP5 3 + HWPX 3) | 20 / 74 / 20 / 141 / 41 / 8 | **전부 바이트 동일** |

SVG 에 이미지가 base64 로 임베드되므로(예: 20쪽 문서 37 MB), 이 일치는 지연 로딩이 **이미지 바이트까지 동일하게 재현**함을 보인다.

### 테스트

- `cargo test`: 2213 passed → (`is_empty` 수정 후) 전량 통과
- `cargo test --test hwpx_roundtrip_baseline`: 4 passed / 0 failed (`baseline_all_samples_roundtrip`, `baseline_large_samples_roundtrip` 포함) — **저장 시 이미지 유실 없음**
- `cargo clippy --all-targets`: 경고 0

## 남은 항목

4단계(최종 검증·보고)에서 WASM 빌드 확인 후 최종 보고서를 작성한다.
