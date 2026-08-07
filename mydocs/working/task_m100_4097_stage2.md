# Stage 2 — task_m100_4097 프로덕션 API

- **이슈**: [#4097](https://github.com/edwardkim/rhwp/issues/4097)
- **계획서**: [`mydocs/plans/task_m100_4097.md`](../plans/task_m100_4097.md)
- **선행 단계**: [`task_m100_4097_stage1.md`](task_m100_4097_stage1.md)
- **브랜치**: `task_m100_4097` (`upstream/devel` `d634e608b` 기준)
- **작업 시각**: 2026-08-07 KST

## 1. 구현

### 1.1 `mini_cfb` — 루트 CLSID 를 받는 API

`DirEntry` 에 `clsid: [u8;16]` 필드를 넣고 `write_dir_entry` 가 **엔트리 필드를** 오프셋 +80 에 쓴다.
루트 값을 특별 취급해 직접 쓰지 않는 이유는, 나중에 스토리지별 CLSID 가 필요해지면 `build_entries` 에
조회 한 줄만 추가하면 되고 `write_dir_entry` 는 손대지 않아도 되게 하기 위해서다.

```rust
pub fn build_cfb(named_streams: &[(&str, &[u8])]) -> Result<Vec<u8>, String> {
    build_cfb_with_root_clsid(named_streams, [0u8; 16])
}

pub fn build_cfb_with_root_clsid(
    named_streams: &[(&str, &[u8])],
    root_clsid: [u8; 16],
) -> Result<Vec<u8>, String> {
    let mut entries = build_entries(named_streams)?;
    entries[0].clsid = root_clsid;
    // ... 이하 기존 파이프라인 그대로
```

`build_cfb` 의 **시그니처는 글자 하나 바뀌지 않았다** — 기존 호출자 9곳이 컴파일 오류를 보지 않는다.
독스트링에 "중첩 OLE CFB 재포장에는 `_with_root_clsid` 를 써야 한다"는 경고와 #4097 근거를 넣었다.

`write_header` 의 헤더 CLSID(8..24)는 **손대지 않았다** — MS-CFB 스펙상 0 이 맞고, 이슈도 이쪽은 결함이
아니라고 명시했다.

### 1.2 `build_entries` — 경로 정규화와 잠재 결함 2건

```rust
let normalized = path.replace('\\', "/");
let parts: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty()).collect();
if parts.is_empty() {
    return Err(format!("CFB 경로에 이름 세그먼트가 없다: {path:?}"));
}
```

네 가지를 함께 처리한다.

| 항목 | 종전 동작 | 변경 후 |
|---|---|---|
| `\` 구분자 | 이름의 일부가 되어 스토리지가 사라짐 | `/` 로 정규화. MS-CFB §2.6.1 이 이름에서 `/ \ : !` 를 금지하므로 **무손실** |
| 빈 세그먼트 (`//`, 후행 `/`) | 이름 없는 엔트리 생성 — `/A/` 는 이름 없는 스트림에 데이터가 들어가 소실 | 버린다. 남는 게 없으면 `Err` |
| Root Entry 충돌 | dedup 후보에 인덱스 0(Root, `parent==0`)이 들어가 `/Root Entry` 스트림이 **루트 데이터를 덮어씀** | `.skip(1)` 로 루트를 후보에서 제외 |
| 스토리지↔스트림 이름 충돌 | 스토리지에 `.data` 만 채우고 `write_dir_entry:533-537` 이 type 1 에 크기 0 을 써 **데이터가 조용히 소실** | `Err` 로 승격 |

`build_entries` 반환형이 `Result<Vec<DirEntry>, String>` 이 됐다. `build_cfb` 는 원래부터
`Result<_, String>` 인데 지금까지 `Err` 를 한 번도 내지 않았다 — 그 팔을 처음 쓰는 것이다.

### 1.3 `cfb_reader::root_clsid` — 읽기 짝

`cfb_reader.rs` 가 CFB 바이트 레이아웃 지식의 단일 소유자이므로 여기에 유일 구현을 뒀다.
전체 파싱을 하지 않고 헤더 두 필드(`0x1E` 섹터 지수, `0x30` first dir sector)만 본다.

기존 테스트 헬퍼(`tests/support/issue_4055_chart_probe.rs:239-250`)와 결정적으로 다른 점은
**바운드 검증이다.** 헬퍼는 `cfb[0x1E]`·`cfb[0x30..0x34]`·`cfb[at..at+16]` 을 전부 무검증 인덱싱하고
`1usize << shift` 도 무검증이라, 짧거나 조작된 바이트에 패닉한다. 프로덕션은 전 구간을 검사한다.

- 길이 512 미만·매직 불일치 → `None`
- 섹터 지수 `9..=12` 밖 → `None` (`LenientCfbReader::open:350-364` 와 같은 근거 — wasm32 에서
  `1usize << 32` 는 debug 패닉 / release 마스킹)
- `checked_add`/`checked_mul` — **wasm32 는 `usize` 가 32비트**라 `(dir_start+1)*sector_size` 가 실제로
  넘칠 수 있다. `ENDOFCHAIN(0xFFFFFFFE)` 같은 특수값도 여기서 걸린다
- 최종 슬라이스 끝이 입력 길이를 넘으면 → `None`

섹터 오프셋을 `(sid + 1) * sector_size` 로 계산한다. 같은 파일의 `LenientCfbReader` 쪽
`512 + sid * sector_size` 는 v4(4096)에서 틀리므로 그 식을 상속하지 않았다(루트만 읽으면 체인 순회가
없어 무관하다).

### 1.4 `ole_container::ole_root_clsid` — 이슈가 지정한 이름

`cfb_reader::root_clsid` 위임 래퍼. 이슈 본문이 명시한 API 이름을 그대로 제공해 B1(#3683)이 문서대로
찾게 하고, "OLE 서버 식별"이라는 의미론을 `ole_container` 에 둔다. 바이트 해석은 한 곳에만 있다.

`OleContainer` 구조체에는 필드를 추가하지 않았다 — `parse_ole_container` 의 `Some` 게이트가 렌더러
분기(`renderer/layout/shape_layout.rs:1964`)를 좌우하므로, B1 이 실제로 필요해질 때 넣는다.

### 1.5 변경 파일

| 파일 | 변경 |
|---|---|
| `src/serializer/mini_cfb.rs` | +186 / −7 |
| `src/parser/cfb_reader.rs` | +40 / −0 |
| `src/parser/ole_container.rs` | +12 / −0 |

계획대로 **3파일에 한정**됐다.

## 2. 테스트

`src/serializer/mini_cfb.rs` `mod tests` 에 6건 추가. 전부 fixture 무의존·결정적이다.

| 테스트 | 고정하는 것 |
|---|---|
| `test_build_cfb_normalizes_backslash_path_separator` | `/BinData\BIN0001.OLE` 와 `/BinData/BIN0001.OLE` 가 **바이트 동일한** CFB 를 만들고, `cfb` 크레이트로 `/BinData/BIN0001.OLE` 스트림이 열린다 |
| `test_build_cfb_collapses_empty_path_segments` | `//A//B/` == `/A/B` |
| `test_build_cfb_rejects_degenerate_paths` | `""`, `"/"`, `"\\"`, `"///"` → `Err` |
| `test_build_entries_does_not_clobber_root_entry` | `/Root Entry` 스트림이 루트를 덮지 않는다 |
| `test_build_cfb_rejects_storage_stream_conflict` | `/A/B`+`/A` 양방향 → `Err` |
| `test_build_cfb_with_root_clsid_writes_dir_entry_offset_80` | 헤더 8..24 는 0 / 루트 엔트리 512+80 에 값 / **Stream 엔트리 +80 은 0** / `cfb` 크레이트가 같은 값을 읽는다 / `build_cfb` 는 0 위임 |

판정은 `cfb` 크레이트로 한다(`read_stream` 헬퍼, `root_entry().clsid()`) — 우리 리더로 판정하면
읽기·쓰기가 같은 오해를 공유해도 통과한다.

```
CARGO_INCREMENTAL=0 cargo test --profile release-test --lib mini_cfb
  running 13 tests ... test result: ok. 13 passed; 0 failed   (기존 7 + 신규 6)

CARGO_INCREMENTAL=0 cargo test --profile release-test --lib cfb_reader
  running 10 tests ... test result: ok. 10 passed; 0 failed
```

기존 7건이 그대로 통과했다 — `build_cfb` 무회귀.

## 3. 검증 게이트 (`local_validation.md` §4.3 — Rust parser/model/CLI 범위)

| 게이트 | 결과 |
|---|---|
| focused test (`--lib mini_cfb`) | 13 passed |
| focused test (`--lib cfb_reader`) | 10 passed |
| `cargo clippy --all-targets -- -D warnings` | exit 0, 경고 0 |
| `cargo fmt --check` | 실제 포맷 지적(`Diff in`) **0건** — 아래 §4 참조 |
| release-test 전체 | Stage 4 에서 실행 |

## 4. `cargo fmt --check` 의 newline style 경고에 대해

이 체크아웃에서 `cargo fmt --check` 는 `Incorrect newline style` 을 **1055건** 보고한다. 이는 이번
변경과 무관한 **체크아웃 전역 CRLF 상태**다. 근거:

- 손대지 않은 파일이 대거 포함된다 — `tools/batch-convert/src/*.rs`, `tests/*_contract.rs` 등
- **`src/parser/hwp3/ole.rs` 는 HEAD 대비 diff 가 0인데도 목록에 있다** (Stage 1 임시 계측을 원복한
  파일이라 이 판별이 명확하다)
- Git 이 `warning: CRLF will be replaced by LF the next time Git touches it` 을 함께 낸다

내 변경 파일에서 발생한 **실제 포맷 지적 2건**(`mini_cfb.rs` 의 `assert_eq!` 줄바꿈)은 rustfmt 제안대로
고쳤고, 현재 `Diff in` 은 0건이다.
