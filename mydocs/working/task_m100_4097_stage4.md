# Stage 4 — task_m100_4097 HWP3 축과 전수 회귀

- **이슈**: [#4097](https://github.com/edwardkim/rhwp/issues/4097)
- **계획서**: [`mydocs/plans/task_m100_4097.md`](../plans/task_m100_4097.md)
- **선행 단계**: [`stage1`](task_m100_4097_stage1.md) · [`stage2`](task_m100_4097_stage2.md) ·
  [`stage3`](task_m100_4097_stage3.md)
- **브랜치**: `task_m100_4097`
- **작업 시각**: 2026-08-07 KST

## 1. HWP3 축 — 승격 시 CLSID 이동

`extract_ole_payloads`(`src/parser/hwp3/ole.rs:166`)는 원본 CFB 의 root 직속 **서브 스토리지를 루트로
승격**하고 그 직속 스트림만 평탄하게 재포장한다. 그래서 되박아야 할 CLSID 는 원본 CFB 의 루트가 아니라
**승격되는 서브 스토리지 엔트리**의 것이다 — 이슈가 제안한 `ole_root_clsid(cfb)` 시그니처만으로는 이
축을 덮지 못한다.

`cfb` 0.14 가 `Entry::clsid() -> &Uuid` 를 노출하므로 raw 디렉터리 파싱이 필요 없었다. 이미 `walk()` 로
서브 스토리지 `Entry` 를 쥐고 있어 수집 지점에서 함께 들고 오면 된다.

```rust
let storages: Vec<(std::path::PathBuf, [u8; 16])> = comp
    .walk()
    .filter(|e| e.is_storage() && !e.is_root())
    .filter(|e| e.path().parent() == Some(root))
    .map(|e| (e.path().to_path_buf(), e.clsid().to_bytes_le()))
    .collect();

// ...
crate::serializer::mini_cfb::build_cfb_with_root_clsid(&refs, storage_clsid)
```

프로덕션 로직 변경은 **3줄**이다(수집 타입, 루프 구조분해, 빌더 호출).

## 2. 테스트

### 2.1 배선 증명 — `src/parser/hwp3/ole.rs` 신규 `mod tests` (fixture 무의존)

`mini_cfb` 로 `/00000000.OOO/{Contents, \x02OlePres000}` 뼈대를 만들고, 그 **스토리지 엔트리 +80** 에
CLSID 를 스탬프한 뒤 `extract_ole_payloads` 를 호출한다.

| 테스트 | 고정하는 것 |
|---|---|
| `task4097_promoted_sub_storage_clsid_becomes_the_new_root_clsid` | 원본 서브 스토리지의 CLSID 가 **출력 루트**에 나타난다 + 스트림 내용도 그대로 넘어간다(`parse_ole_container` 재통과) |
| `task4097_zero_sub_storage_clsid_stays_zero` | CLSID 가 없는(0) 스토리지는 0 인 채로 승격된다 — 없는 값을 지어내지 않는다 |

스탬프 헬퍼는 `cfb::set_storage_clsid` 를 쓰지 않는다 — `Uuid` 타입 이름이 필요해 `uuid` 를
dev-dependency 로 끌어들이기 때문이다. mini_cfb 출력은 sector_shift=9, first dir sector=0 고정이라
엔트리가 파일 오프셋 512 부터 128바이트씩 이어지므로, 이름으로 찾아 직접 스탬프한다.

### 2.2 실물 회귀 — `task3363_hwp3_embedded_ole_payload_extraction`

기존 단언들(`parse_ole_container` 개봉, `has_preview`, `is_hmapsi_ole_container`)은 전부 **스트림 이름**
기반이라 CLSID 가 0 이어도 통과한다. 그래서 실측값 단언을 새로 붙였다.

```rust
let clsid = crate::parser::cfb_reader::root_clsid(&bytes).expect(..);
assert_eq!(clsid, [0x14,0x42,0x04,0x00, ...0x46],
    "원본 서브 스토리지의 OLE 클래스 ID 가 보존되어야 함");
```

Stage 1 실측값 `{00044214-0000-0000-C000-000000000046}`(글맵시 서버 클래스)를 그대로 고정한다. 합성
테스트가 배선을, 이 테스트가 **실물 파일에서의 값**을 지킨다.

```
CARGO_INCREMENTAL=0 cargo test --profile release-test --lib -- hwp3::ole task3363_hwp3_embedded
  running 3 tests
    task4097_zero_sub_storage_clsid_stays_zero ... ok
    task4097_promoted_sub_storage_clsid_becomes_the_new_root_clsid ... ok
    task3363_hwp3_embedded_ole_payload_extraction ... ok
  test result: ok. 3 passed; 0 failed
```

## 3. 명시 회귀 게이트

```
CARGO_INCREMENTAL=0 cargo test --profile release-test \
  --test issue_3546_chart_preserved_on_save \
  --test issue_3547_ole_size_prefix \
  --test issue_1251_ole_chart_contents
```

| 테스트 파일 | 결과 |
|---|---|
| `issue_1251_ole_chart_contents` (10건) | ok. 10 passed |
| `issue_3546_chart_preserved_on_save` (2건, 암호 축 포함) | ok. 2 passed |
| `issue_3547_ole_size_prefix` (실제 fn 명 `issue_3547_internal_ole_size_prefix_roundtrips`) | ok. 1 passed |

> `issue_1251_ole_chart_contents` 는 이슈 AC⑤ 에 빠져 있으나 #4055 report §8 이 "깨지면 안 되는 것"으로
> 지목한 항목이라 게이트에 넣었다.

## 4. 호출자 무영향 확인

`build_cfb` 호출자를 전수 재확인했다.

| 호출자 | 호출 API | 판단 |
|---|---|---|
| `src/serializer/cfb_writer.rs:293` (`write_hwp_cfb`) | `build_cfb` | 바깥 HWP5 CFB — 원본도 루트 CLSID 가 0(Stage 1 실측). 유지가 맞다 |
| `src/diagnostics/hwp5_*_probe.rs` 6종 | `build_cfb` | 같은 바깥 CFB 재조립. 무영향 |
| `src/parser/hwp3/ole.rs:229` | **`build_cfb_with_root_clsid`** | 이번에 고친 지점 |

`build_cfb` 의 시그니처는 바뀌지 않았고, 출력 바이트 동일성은
`production_api_reproduces_the_hancom_validated_bytes`(Stage 3)가 코퍼스 56건에서 고정한다.

## 5. 전수 검증

`local_validation.md` §4.3 의 **Rust parser/model/CLI** 행 + `visual_verification_governance.md:57` 의
직렬화 roundtrip 게이트. 모든 Cargo 명령은 **직렬 실행**, `CARGO_INCREMENTAL=0`.

| 게이트 | 명령 | 결과 |
|---|---|---|
| roundtrip (HWP5) | `cargo test --release --test hwp5_roundtrip_baseline` | **3 passed**, 0 failed (46.3s) |
| roundtrip (HWPX) | `cargo test --test hwpx_roundtrip_baseline` | **4 passed**, 0 failed (50.7s) |
| release-test 전체 | `cargo test --profile release-test --tests` | **5284 passed, 0 failed, 28 ignored** (469개 테스트 바이너리 전부 `ok`) |
| fmt | `cargo fmt --check` | 실제 포맷 지적(`Diff in`) **0건** — §6 참조 |
| 공백 | `git diff --check` | 오류 0 |
| clippy | `cargo clippy --all-targets -- -D warnings` | exit 0, 경고 0 |
| wasm 안전성 | `cargo check --target wasm32-unknown-unknown --lib` | exit 0 (59.9s) |

전체 로그: `$TMPDIR/4097_full_test.log`.

### 5.1 wasm 게이트를 대체한 이유

계획은 `wasm-pack build --target web` 1회 스모크였으나 **이 머신에 `wasm-pack` 이 설치돼 있지 않다**
(`wasm32-unknown-unknown` 타깃은 설치돼 있다). `cargo check --target wasm32-unknown-unknown --lib` 로
대체했다.

이 이슈의 wasm 논점은 **`usize` 가 32비트인 타깃에서의 산술**이다 — `root_clsid` 의
`checked_add`/`checked_mul` 과 `1usize << sector_shift` 의 범위 검증. 그 논점은 타깃 컴파일로 검증되고,
`wasm-pack` 이 추가로 보는 것은 바인딩 생성·번들링이라 이번 변경과 무관하다. 다만 **계획된 명령을
그대로 실행하지는 못했다**는 사실을 여기 남긴다.

## 6. `cargo fmt --check` 의 newline style 경고

Stage 2 §4 와 같다 — 이 체크아웃 전역의 CRLF 상태이며 손대지 않은 파일(`tools/`, `tests/*_contract.rs`
등)이 대거 포함된다. 이번 변경에서 발생한 **실제 포맷 지적**은 `hwp3/mod.rs:5575` 의 바이트 배열 줄바꿈
1건이었고 rustfmt 제안대로 고쳤다. 현재 `Diff in` 은 0건이다.

## 7. AC 대응 현황

| AC | 상태 |
|---|---|
| ① 방향 반전 | 완료 (Stage 3, 변이 검증) |
| ② 28×2 = 56건 | 완료 (Stage 3) |
| ③ `\` 경로 | 완료 (Stage 2) |
| ④ 한컴 | 바이트 동일성 고정 완료. 육안 판정은 Stage 5 |
| ⑤ 회귀 green | **완료** — 명시 4종 + roundtrip 2종 + release-test 전체 |
