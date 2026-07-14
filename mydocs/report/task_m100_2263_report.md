# 최종 결과 보고서 — M100 #2263: BinData 지연 로딩

## 요약

`Document` IR 이 문서의 모든 내장 바이너리를 **파싱 시점에 압축 해제한 상태로 상주**시키던 구조를, 실제로 참조되는 항목만 요청 시 압축 해제하는 구조로 전환했다.

파싱 + 조판 경로의 최대 RSS 가 **244 MB → 49 MB** (무손실 비트맵 다수 내장 문서 기준)로 떨어졌고, 시각 출력은 **바이트 단위로 동일**하다.

## 문제

- `src/model/document.rs` — `bin_data_content: Vec<BinDataContent>`
- `src/model/bin_data.rs` — `BinDataContent.data: Vec<u8>`

1페이지만 렌더해도 문서 전체의 이미지가 메모리에 올라왔다. 컨테이너(ZIP deflate / CFB zlib)가 비트맵을 강하게 압축하므로, 원본이 작아도 압축 해제본은 수십 배가 된다.

측정된 경험 모델: `RSS ≈ 프로세스 바닥(약 5MB) + 압축해제 이미지 전량 + 페이지당 약 0.4MB 조판 구조`

## 해결

### 설계

`Vec<BinDataContent>` 배열 구조는 **유지**하고, 각 항목의 바이트만 지연화했다. 조회가 위치 기반(`find_bin_data` 의 `get(id-1)`)이고 직렬화기가 배열을 전량 소비·추가하므로, 배열의 순서·길이 의미를 건드리면 광범위하게 깨진다.

```
BinDataContent.data: BinDataBytes
  ├─ Loaded(Vec<u8>)                  메모리 상주 (직렬화기 push, HML/HWP3, lenient)
  └─ Lazy { resolver, key }           원본 컨테이너에서 요청 시 압축 해제
```

- `BinDataResolver` 트레이트를 model 에 두고 **파서 측이 구현**한다 (model → parser 역의존 회피).
- HWPX: `HwpxBinResolver` 가 ZIP 원본을 보유, 엔트리를 요청 시 inflate.
- HWP5: `Hwp5BinResolver` 가 CFB 원본을 보유, 스트림을 요청 시 읽고 `decompress_stream` 적용.
- **캐시를 두지 않는다.** 소비자(`ImageNode`, 직렬화기)가 어차피 바이트를 복사해 보유하므로, 캐시하면 이중 상주가 되어 목적에 반한다.

### 보존해야 했던 의미

| 항목 | 처리 |
|---|---|
| 위치 기반 조회 (`get(id-1)`) | HWP5 는 기존에 **읽기 실패 시 항목을 건너뛴다**. `has_stream()` 으로 압축 해제 없이 존재만 확인해 skip 의미를 보존 |
| OLE 선두 4-byte size prefix 스킵 (Task #195) | 지연 로드 시점에 동일 적용. 대상 스트림 집합을 리졸버가 보유 |
| HWPX 내부 OLE 정규화 | `normalize_ole_bytes` 로 바이트 전용 분리 후 지연 시점 적용 |
| 로드 실패 placeholder (#1917) | 엔트리는 등록 유지, 리졸버가 빈 바이트 반환 (manifest·binaryItemIDRef 보존, 이미지 데이터만 소실) |
| 저장 시 이미지 유실 방지 | 원본 컨테이너를 계속 보유하므로 직렬화 시 전량 재획득 가능. `hwpx_roundtrip_baseline` 이 방어 |

### 지연화하지 않은 경로 (의도적)

- **HWPX Chart XML** — 존재 여부 탐색이 필요하고 크기도 작다.
- **HWP5 lenient 경로** — 손상 파일 폴백으로 드물게 쓰인다. 정확성 우선.

## 검증 중 발견하고 고친 결함

`is_empty()` 를 "`Lazy` 면 엔트리가 존재하므로 비어 있지 않다"로 구현한 것이 잘못된 가정이었다. 매니페스트에는 있으나 실제 엔트리가 없거나 읽기에 실패하는 경우가 바로 #1917 이 다루는 상황이다. `issue1917_bindata_load_failure_preserves_pic_control` 가 이를 잡아냈고, 실제 바이트를 해석해 판정하도록 수정했다. 렌더 핫패스에서 쓰이지 않아 비용 부담은 없다.

## 검증 결과

### 메모리 — 파싱 + 조판 경로 최대 RSS (`/usr/bin/time -l`, 1페이지)

| 문서 특성 | 이전 | 이후 | 감소 |
|---|---|---|---|
| 무손실 비트맵 다수 내장 (HWP5, 20쪽) | 244 MB | **49 MB** | −80% |
| 무손실 비트맵 다수 내장 (HWPX, 20쪽) | 244 MB | **47 MB** | −81% |
| 이미지 다수 (HWP5, 41쪽) | 185 MB | **40 MB** | −78% |
| 이미지 중간 규모 (HWP5, 74쪽) | 78 MB | **39 MB** | −50% |
| JPEG 다수 (HWPX, 141쪽) | 97 MB | **77 MB** | −21% |
| 이미지 다수 (HWP5, 8쪽) | 84 MB | **30 MB** | −64% |

압축 해제 이미지 총량이 더 이상 RSS 를 지배하지 않는다. **완료 기준 충족.**

### 시각 회귀 — 변경 전/후 바이너리 SVG 바이트 비교

6개 문서(HWP5 3 + HWPX 3), 전체 페이지(20 / 74 / 20 / 141 / 41 / 8쪽) 내보내기 결과 **전부 바이트 동일**. SVG 에 이미지가 base64 로 임베드되므로(20쪽 문서 37 MB), 이 일치는 지연 로딩이 이미지 바이트까지 동일하게 재현함을 보인다.

### 테스트

| 게이트 | 결과 |
|---|---|
| `cargo test` | 전량 통과 |
| `cargo test --test hwpx_roundtrip_baseline` | 4 passed / 0 failed — **저장 시 이미지 유실 없음** |
| `cargo clippy --all-targets` | 경고 0 |
| `cargo check --target wasm32-unknown-unknown` | 오류 0 |

## 파급 효과

- **WASM / 브라우저 뷰어**: 동일 비용을 지불하고 있었으므로 그대로 이득이다.
- **macOS Quick Look 확장 (후속 예정)**: 확장 프로세스는 약 80MB 경고 / 120MB 강제 종료다. 본 변경으로 파싱·조판 경로가 전부 80MB 아래로 들어왔다.

## 남은 항목 (본 이슈 범위 밖)

**PDF 내보내기 경로가 여전히 122~195 MB 를 쓴다.** `src/renderer/pdf.rs:110` 의 `fontdb.load_system_fonts()` 가 문서와 무관하게 시스템 폰트를 전량 열거하며 +85~140 MB 를 더한다. Quick Look 확장 진입을 위해서는 이 항목도 제거해야 하며, **별도 이슈로 다룬다.**
