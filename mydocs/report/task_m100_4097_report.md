# 최종 보고서 — task_m100_4097

- **Issue**: [#4097](https://github.com/edwardkim/rhwp/issues/4097) `[serializer/mini_cfb] 재포장이 OLE
  루트 CLSID 를 0 으로 떨군다 — 한컴이 개체를 알아보지 못해 내용을 비워 그린다`
- **브랜치**: `task_m100_4097` (`upstream/devel` `d634e608b` 기준)
- **계획서**: [`mydocs/plans/task_m100_4097.md`](../plans/task_m100_4097.md)
- **단계 기록**: [stage1](../working/task_m100_4097_stage1.md) ·
  [stage2](../working/task_m100_4097_stage2.md) · [stage3](../working/task_m100_4097_stage3.md) ·
  [stage4](../working/task_m100_4097_stage4.md) · [stage5](../working/task_m100_4097_stage5.md)
- **작성 시각**: 2026-08-07 KST

## 1. 요약

`mini_cfb::build_cfb` 가 CFB 디렉터리 엔트리의 CLSID(오프셋 +80)를 항상 0 으로 고정해, 중첩 OLE CFB 를
재포장하면 OLE 서버 식별자가 사라졌다. 한컴은 개체를 알아보지 못해 **틀과 선택 핸들만 그리고 내용을
비웠다**(2026-08-05 실측). rhwp 는 스트림 **이름**으로만 개체를 판별하므로 자체 왕복 검증으로는 잡히지
않는 종류였다.

루트 CLSID 를 받는 `build_cfb_with_root_clsid` 와 그 짝인 리더 `cfb_reader::root_clsid` /
`ole_container::ole_root_clsid` 를 넣고, 실제로 손실이 일어나던 프로덕션 호출자(HWP3 승격 재포장)를
새 API 로 전환했다. 함께 `build_entries` 의 경로 구분자 정규화와 잠재 결함 2건을 고쳤다.

## 2. 원인

| 위치 | 내용 |
|---|---|
| `src/serializer/mini_cfb.rs:32-43` | `DirEntry` 에 CLSID 필드 자체가 없음 → 항상 0 |
| `src/serializer/mini_cfb.rs:513` | `write_dir_entry` 가 +80 에 주석만 남기고 아무것도 쓰지 않음 |
| `src/parser/ole_container.rs:63-165` | `parse_ole_container` 가 스트림 이름으로만 판별 → 손실을 감지하지 못함 |
| `src/serializer/mini_cfb.rs:330` | `build_entries` 가 `/` 로만 경로를 쪼갬 → Windows 의 `/BinData\BIN0001.OLE` 에서 스토리지 소실 |

> 같은 파일 `:422` 의 CFB **파일 헤더** CLSID 는 MS-CFB 스펙상 0 이 맞다 — 손대지 않았다.

## 3. 수정

### 3.1 쓰기측 — `src/serializer/mini_cfb.rs`

`DirEntry` 에 `clsid: [u8;16]` 필드를 넣고 `write_dir_entry` 가 **엔트리 필드를** +80 에 쓴다. 루트 값을
특별 취급해 직접 쓰지 않았으므로, 나중에 스토리지별 CLSID 가 필요해지면 `build_entries` 에 조회 한 줄만
추가하면 되고 `write_dir_entry` 는 무변경이다.

```rust
pub fn build_cfb(named_streams: &[(&str, &[u8])]) -> Result<Vec<u8>, String> {
    build_cfb_with_root_clsid(named_streams, [0u8; 16])   // 시그니처 불변
}
pub fn build_cfb_with_root_clsid(
    named_streams: &[(&str, &[u8])],
    root_clsid: [u8; 16],
) -> Result<Vec<u8>, String>
```

**루트만 받는 이유**: CLSID 를 실을 수 있는 엔트리는 MS-CFB 상 Root(5)와 Storage(1) 둘뿐이고 Stream(2)은
0 이어야 한다(`cfb` 크레이트는 Permissive 에서 비-0 스트림 CLSID 를 0 으로 뭉갠다). 그런데 **코퍼스 56건
전부 서브 스토리지가 0개**이고(Stage 1 실측), HWP3 축도 서브 스토리지를 루트로 승격하므로 출력에
스토리지가 없다. B1 편집 대상도 같은 평탄 구조다 — per-path CLSID 는 소비자가 0 인 speculative
generality다.

### 3.2 `build_entries` — 경로 정규화와 잠재 결함 2건

| 항목 | 종전 | 변경 후 |
|---|---|---|
| `\` 구분자 | 이름의 일부가 되어 스토리지 소실 | `/` 로 정규화 (MS-CFB §2.6.1 이 이름에서 `/ \ : !` 를 금지하므로 무손실) |
| 빈 세그먼트 | 이름 없는 엔트리 생성 (`/A/` 는 데이터 소실) | 버린다. 남는 게 없으면 `Err` |
| Root Entry 충돌 | `/Root Entry` 스트림이 루트 데이터를 덮어씀 | dedup 후보에서 인덱스 0 제외 |
| 스토리지↔스트림 충돌 | 데이터가 **조용히 소실** | `Err` 로 승격 |

### 3.3 읽기측 — `cfb_reader::root_clsid` + `ole_container::ole_root_clsid`

바이트 레이아웃 지식은 `cfb_reader.rs` 한 곳에만 둔다(이미 `0x1E`·`0x30` 을 읽고 `9..=12` 범위 검증을
가진 모듈). `ole_container` 는 이슈가 지정한 이름의 위임 래퍼다.

기존 테스트 헬퍼와 결정적으로 다른 점은 **바운드 검증**이다 — 헬퍼는 전 구간을 무검증 인덱싱했다.
프로덕션은 길이·매직·섹터 지수 범위·`checked_add`/`checked_mul`·최종 슬라이스 끝을 모두 검사하고 `None`
을 돌려준다. **wasm32 는 `usize` 가 32비트**라 `(dir_start+1)*sector_size` 가 실제로 넘칠 수 있어
`checked_mul` 이 필수다. 패닉은 WASM 모듈 전체를 죽여 편집 중인 다른 문서까지 잃게 한다.

### 3.4 HWP3 축 — `src/parser/hwp3/ole.rs`

`extract_ole_payloads` 는 서브 스토리지를 **루트로 승격**하므로, 옮겨야 할 CLSID 는 원본 루트가 아니라
승격되는 서브 스토리지 엔트리의 것이다. `cfb` 0.14 가 `Entry::clsid()` 를 노출해 raw 파싱 없이 수집
지점에서 함께 들고 올 수 있었다(로직 3줄).

Stage 1 실측: `samples/SO-SUEOP.hwp` 의 `00000000.OOO` 서브 스토리지 CLSID =
`{00044214-0000-0000-C000-000000000046}` — **비-0**. 즉 이 축은 실제로 유효한 값을 버리고 있었다.

## 4. 변경 파일

| 파일 | 변경 |
|---|---|
| `src/serializer/mini_cfb.rs` | API 2개, `build_entries` 재작성, `write_dir_entry` +80 기록, 단위 테스트 6건 |
| `src/parser/cfb_reader.rs` | `root_clsid` 신설 |
| `src/parser/ole_container.rs` | `ole_root_clsid` 위임 래퍼 |
| `src/parser/hwp3/ole.rs` | 서브 스토리지 CLSID 승격, `mod tests` 2건 |
| `src/parser/hwp3/mod.rs` | `task3363_...` 에 실측 GUID 단언 |
| `tests/support/issue_4055_chart_probe.rs` | 되박기 → 프로덕션 API, 판정 오라클을 `cfb` 크레이트로 |
| `tests/issue_4055_b1_chart_edit_probe.rs` | AC① 재작성·이름 변경, AC② 56건 확장 |
| `tests/issue_4097_mini_cfb_root_clsid.rs` | 신설 3건 |

## 5. 수용 기준 대응

| AC | 결과 |
|---|---|
| ① 회귀 테스트 방향 반전 | `mini_cfb_repack_preserves_the_ole_class_id` 로 재작성. **이름·기대값만 바꾸면 안 됐다** — `build_cfb` 가 `[0u8;16]` 위임이라 종전 단언은 그대로 통과한다. 실제 전환점은 `rebuild_cfb_preserving_clsid` 가 프로덕션 API 를 부르는 것이고, **변이 검증으로 red 전환을 확인**했다 |
| ② 코퍼스 28×2 | `nested_cfb_repack_preserves_every_stream` `checked == 56`, `unknown_seen.is_empty()` |
| ③ `\` 혼용 경로 | `mini_cfb mod tests` 6건 (백슬래시·빈 세그먼트·퇴화 경로·Root 보호·충돌·CLSID 기록) |
| ④ 한컴 실측 | 프로덕션 산출물이 **#4055 가 한컴 판정을 통과시킨 산출물과 바이트 동일**함을 코퍼스 56건에서 고정. 육안 판정은 작업지시자 수행 (§7) |
| ⑤ 회귀 green | `issue_3546`(2) · `issue_3547`(1) · `issue_1251`(10) · HWP3 축 · roundtrip 게이트 |

## 6. 검증

*(Stage 4 결과로 확정)*

## 7. 한컴 실측 (AC④)

*(Stage 5)*

## 8. 남긴 것

- `OleContainer` 구조체에 `root_clsid` 필드를 넣지 않았다 — `parse_ole_container` 의 `Some` 게이트가
  렌더러 분기(`renderer/layout/shape_layout.rs:1964`)를 좌우하므로, B1(#3683)이 실제로 필요해질 때
  넣는 편이 리뷰 부담이 적다.
- `cfb_reader.rs` 의 `LenientCfbReader` 쪽 섹터 오프셋 산식 `512 + sid * sector_size` 는 v4(4096)에서
  틀리다. 이번 `root_clsid` 는 그 식을 상속하지 않았으나(체인 순회 없음), **기존 결함은 그대로 남아
  있다** — 별도 이슈 대상이다.
