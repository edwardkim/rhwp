# Stage 3 — task_m100_4097 테스트 하네스 전환

- **이슈**: [#4097](https://github.com/edwardkim/rhwp/issues/4097)
- **계획서**: [`mydocs/plans/task_m100_4097.md`](../plans/task_m100_4097.md)
- **선행 단계**: [`stage1`](task_m100_4097_stage1.md) · [`stage2`](task_m100_4097_stage2.md)
- **브랜치**: `task_m100_4097`
- **작업 시각**: 2026-08-07 KST
- **프로덕션 코드 변경**: 0 (Stage 2 결과물 그대로)

## 1. AC① — 방향 전환은 이름만 바꿔서는 안 된다

이슈 AC① 은 `mini_cfb_repack_drops_the_ole_class_id` 가 "방향을 뒤집어 통과"하기를 요구한다. 그런데
**기존 `build_cfb` 를 `[0u8;16]` 위임으로 남기면 종전 단언이 그대로 통과한다** — `rebuild_cfb` 는
여전히 `build_cfb` 를 부르므로 CLSID 가 0 인 게 맞다. 이름과 기대값만 손대면 "고쳤다고 주장하지만
아무것도 검증하지 않는 테스트"가 된다.

실제 전환점은 **`rebuild_cfb_preserving_clsid` 가 바이트 수술을 그만두고 프로덕션 API 를 부르는 것**이다.

```rust
// 종전 — 테스트 로컬 바이트 수술
let mut rebuilt = rebuild_cfb(streams);
stamp_root_clsid(&mut rebuilt, root_clsid(original));   // cfb[at..at+16] 직접 대입

// 변경 후 — 프로덕션 경로
let clsid = rhwp::parser::ole_container::ole_root_clsid(original).expect(..);
rhwp::serializer::mini_cfb::build_cfb_with_root_clsid(&refs, clsid).expect(..)
```

테스트 이름은 `mini_cfb_repack_preserves_the_ole_class_id` 로 바꿨다 — "drops" 라는 이름이 보존을
단언하면 다음 독자에게 지뢰다. 독스트링에 계보를 남겼다.

세 단언의 의미도 정리했다.

| 단언 | 종전 | 변경 후 |
|---|---|---|
| 원본이 비-0 | 유지 | 유지 |
| `build_cfb` 결과가 0 | "mini_cfb 는 CLSID 를 0 으로 고정한다" (**결함 고정**) | "build_cfb 는 CLSID 없음으로 위임한다 — 이것이 **계약**이다" (바깥 HWP5 CFB 는 원본도 0 이라 그 호출자가 이 쪽을 쓴다) |
| 보존 경로가 원본과 같음 | "되박으면 유지된다" (테스트 로컬 수술) | "프로덕션 재포장이 보존해야 한다" (**여기가 전환점**) |

### 1.1 변이 검증 — 정말로 red 가 되는가

주장을 말로 남기지 않고 실제로 확인했다. `build_cfb_with_root_clsid` 에 `entries[0].clsid = [0u8;16]`
한 줄을 임시로 넣어 결함을 되살린 뒤 실행했다.

```text
test nested_cfb_repack_preserves_every_stream ... FAILED
test mini_cfb_repack_preserves_the_ole_class_id ... FAILED

panicked at tests\issue_4055_b1_chart_edit_probe.rs:242:13:
  ...가로막대형\3차원누적가로막대형.hwpx: 재포장이 OLE 클래스 ID 를 보존해야 한다
panicked at tests\issue_4055_b1_chart_edit_probe.rs:320:5:
  프로덕션 재포장이 OLE 클래스 ID 를 보존해야 한다 (#4097)

test result: FAILED. 7 passed; 2 failed; 1 ignored
```

의도한 두 단언에 정확히 걸렸다. 변이를 되돌린 뒤 다시 green 임을 확인했다.

## 2. AC② — 28종 × 2포맷 = 56건

`nested_cfb_repack_preserves_every_stream` 의 루프를 2포맷으로 감쌌다.

```rust
for hwpx in corpus() {
    for path in [hwpx.clone(), hwpx.with_extension("hwp")] {
```

Stage 1 측정대로 **`hwp_nested_cfb()` 헬퍼는 쓰지 않았다** — HWP5 파서가 BinData 의 4바이트 LE size
prefix 를 이미 `drain(..4)` 하므로 `.hwp` 축도 `bin_data_content` 에서 raw 중첩 CFB 가 그대로 나온다.

함께 고친 것: 중첩 CFB 를 못 찾으면 종전에는 `continue` 로 **조용히 건너뛰었다**. 카운트 단언이 결국
잡긴 하지만 메시지가 "28이어야 하는데 27" 이라 어느 파일인지 알 수 없다. 하드 실패로 바꿨다.

`known` 목록(4종)과 `unknown_seen` 단언은 그대로 뒀다 — Stage 1 에서 56건 전부 3종(`\x02OlePres000`,
`Contents`, `OOXMLChartContents`)뿐임을 확인해 추가할 것이 없었다.

## 3. AC④ 지원 — 바이트 동일성 오라클 (신규 `tests/issue_4097_mini_cfb_root_clsid.rs`)

support 모듈을 쓰지 않는 자족형 파일이다. `#[path]` 로 support 를 두 번 포함하면 이 파일이 안 쓰는
helper 가 `dead_code` 경고를 내 `clippy -D warnings` 가 깨진다.

| 테스트 | 고정하는 것 |
|---|---|
| `production_api_reproduces_the_hancom_validated_bytes` | #4055 가 한컴 판정에 넘긴 레시피(`build_cfb` 출력 + 루트 엔트리 +80 사후 스탬프)를 **동결 사본**으로 인라인해 두고, 프로덕션 `build_cfb_with_root_clsid` 가 **코퍼스 56건 전건에서 바이트 동일**한 것을 만든다고 단언 |
| `production_reader_agrees_with_the_cfb_crate` | `ole_root_clsid` == `cfb` 크레이트 값, 그리고 위임 래퍼 == 유일 구현 (56건) |
| `reader_returns_none_for_malformed_input` | 빈 입력·511B·매직 불일치·섹터 지수 0/65535·`dir_start`=ENDOFCHAIN·절단 → 전부 `None`, 패닉 0 |

첫 번째가 이 PR 에서 가장 중요한 테스트다. 두 가지를 동시에 지킨다.

1. **AC④ 를 "새로 측정할 미지"에서 "이미 측정된 것의 재확인"으로 강등시킨다.** #4055 stage4 는
   CLSID 를 되박은 산출물을 한컴에서 판정해 정상 개봉·렌더를 확인했고, CLSID 외 차이(color flag 0→1,
   타임스탬프, 섹터 배치)는 무해 판정을 받았다(`task_m100_4055_stage4.md:64-68`). 프로덕션 산출물이
   **그 파일과 바이트 동일**하면 한컴 거동도 같다.
2. **기존 호출자 9곳 무영향의 기계적 증명이다.** 대조 상대가 `build_cfb` 출력이므로, `build_cfb` 의
   출력이 CLSID 16바이트 외에 조금이라도 달라지면 즉시 red 다.

## 4. 독립 오라클 유지

판정을 프로덕션 리더로 하면 `assert_eq!(read(write(x)), read(x))` 가 되어 **오프셋이 통째로 틀려도
통과한다.** support 의 `root_clsid` 를 raw 오프셋 산술에서 `cfb` 크레이트 기반으로 갈아끼웠다.

```rust
pub(super) fn root_clsid(cfb: &[u8]) -> [u8; 16] {
    cfb::CompoundFile::open(..).expect(..).root_entry().clsid().to_bytes_le()
}
```

즉 **쓰기는 프로덕션, 채점은 크레이트**다. 프로덕션 리더는
`production_reader_agrees_with_the_cfb_crate` 에서 **별도로** 크레이트와 대조한다.

삭제한 것: `root_dir_entry_offset`(`support:239`), `stamp_root_clsid`(`support:260`). 후자의 레시피는
`production_api_reproduces_the_hancom_validated_bytes` 안에 동결 사본으로 1회만 남아 있다.

유지한 것: `all_streams` 의 `path.replace('\\', "/")`. `mini_cfb` 가 이제 스스로 정규화하지만, 이
함수의 반환값을 **이름으로 비교**하는 쪽(`mutate_nested` 의 `== "/OOXMLChartContents"`, bundle
생성기의 `starts_with("/BinData/")`)이 있어 플랫폼 무관 표기가 여전히 필요하다. 주석을 그렇게 갱신했다.

## 5. 검증

```
CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_4055_b1_chart_edit_probe
  running 10 tests ... test result: ok. 9 passed; 0 failed; 1 ignored

CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_4097_mini_cfb_root_clsid
  running 3 tests ... test result: ok. 3 passed; 0 failed

CARGO_INCREMENTAL=0 cargo fmt --check      → 실제 포맷 지적(Diff in) 0건
CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings   → exit 0, 경고 0
```

| AC | 상태 |
|---|---|
| ① 방향 반전 | 완료 — 변이 검증으로 red 전환 확인 |
| ② 28×2 = 56건 | 완료 — `assert_eq!(checked, 56)`, `unknown_seen.is_empty()` |
| ③ `\` 경로 | Stage 2 에서 완료 (`mini_cfb mod tests`) |
| ④ 한컴 | 바이트 동일성 고정 완료. 실측은 Stage 5 |
| ⑤ 회귀 green | Stage 4 |
