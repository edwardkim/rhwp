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

`local_validation.md` §4.3 의 **Rust parser/model/CLI** 행 + `visual_verification_governance.md:57` 의
직렬화 roundtrip 게이트. 모든 Cargo 명령은 **직렬 실행**, `CARGO_INCREMENTAL=0`.

| 게이트 | 결과 |
|---|---|
| `cargo test --profile release-test --tests` | **5284 passed, 0 failed, 28 ignored** (469개 테스트 바이너리 전부 ok) |
| `cargo test --release --test hwp5_roundtrip_baseline` | 3 passed |
| `cargo test --test hwpx_roundtrip_baseline` | 4 passed |
| `cargo clippy --all-targets -- -D warnings` | exit 0, 경고 0 |
| `cargo fmt --check` | 실제 포맷 지적(`Diff in`) 0건 |
| `git diff --check` | 오류 0 |
| `cargo check --target wasm32-unknown-unknown --lib` | exit 0 |

focused 회귀: `issue_3546_chart_preserved_on_save`(2, 암호 축 포함) ·
`issue_3547_internal_ole_size_prefix_roundtrips`(1) · `issue_1251_ole_chart_contents`(10) ·
`issue_4055_b1_chart_edit_probe`(9 + 1 ignored) · `issue_4097_mini_cfb_root_clsid`(3) ·
`mini_cfb`(13) · `cfb_reader`(10) · HWP3 축(3).

전체 로그: `$TMPDIR/4097_full_test.log`.

### 6.1 계획과 다르게 실행한 것

계획의 wasm 스모크는 `wasm-pack build --target web` 이었으나 **이 머신에 `wasm-pack` 이 설치돼 있지
않아** `cargo check --target wasm32-unknown-unknown --lib` 로 대체했다. 이 이슈의 wasm 논점은 `usize` 가
32비트인 타깃에서의 산술(`checked_add`/`checked_mul`, `1usize << sector_shift` 범위 검증)이고 그것은
타깃 컴파일로 검증된다. `wasm-pack` 이 추가로 보는 바인딩 생성·번들링은 이번 변경과 무관하다.

### 6.2 `cargo fmt --check` 의 newline style 경고

이 체크아웃은 전역 CRLF 상태라 `Incorrect newline style` 이 1055건 나온다 — 손대지 않은 파일
(`tools/`, `tests/*_contract.rs` 등)이 대거 포함되며 이번 변경과 무관하다. 실제 포맷 지적(`Diff in`)은
작업 중 3건 발생했고 전부 rustfmt 제안대로 고쳐 현재 0건이다.

## 7. 한컴 실측 (AC④)

### 7.1 자동화로 확보한 증거 두 가지

1. **코퍼스 56건 합성 대조** — `production_api_reproduces_the_hancom_validated_bytes`. #4055 가 한컴에
   넘긴 레시피(`build_cfb` 출력 + 루트 엔트리 +80 사후 스탬프)를 동결 사본으로 두고, 프로덕션
   `build_cfb_with_root_clsid` 출력이 전건 바이트 동일함을 단언한다.
2. **판정 파일 그 자체와의 동일성** — Stage 5 에서 판정 번들을 프로덕션 경로로 재생성했더니 생성기가
   변종 10종 전건 **`unchanged`**(= 디스크 내용과 전체 바이트 비교 일치)를 보고했다. 디스크의 파일은
   **2026-08-05** 한컴 판정에 실제로 쓰인 산출물이다(같은 디렉터리의 판정 PDF 도 같은 날짜).

즉 프로덕션 경로는 **한컴이 이미 정상 판정한 파일을 바이트 단위로 재현한다.** 특히
`H-A`·`H-C`·`H-D` 는 중첩 CFB 를 실제로 재포장한 변종이라 이번 수정이 지나가는 바로 그 경로다.

여기에 #4055 stage4 `:64-70` 의 디렉터리 엔트리 전수 비교(color flag 0→1·타임스탬프 무해 판정, 루트
CLSID 가 유일한 유의미 차이)를 합치면, AC④ 는 **"새로 측정할 미지"가 아니라 "이미 측정된 것의
재확인"** 이다.

### 7.2 HWP3 글맵시 축 — 판정 완료: 가시 효과 없음 (2026-08-07 실측)

차트 축(HWPX/HWP5)은 §7.1 대로 이미 판정된 파일을 바이트 단위로 재현하므로 재판정 대상이 아니었고,
**한컴 판정 이력이 없는 HWP3 글맵시 축만** 수정 전/후 쌍(`output/issue_4097_hwp3/`,
`PANJEONG.md` 포함)으로 판정했다. 두 파일의 차이는 **CFB 오프셋 592(=512+80)의 16바이트뿐**이다.

| 파일 | `BinData/image1.ole` 루트 CLSID |
|---|---|
| `SO-SUEOP-before.{hwp,hwpx}` | `{00000000-0000-0000-0000-000000000000}` |
| `SO-SUEOP-after.{hwp,hwpx}` | `{00044214-0000-0000-C000-000000000046}` |

작업지시자 실측 결과:

| 항목 | 결과 |
|---|---|
| 오류·복구 대화상자 | 4개 전부 없음 |
| before vs after 한컴 렌더 (한컴 PDF 내보내기) | **46쪽 전부 픽셀 동일** (PyMuPDF 72dpi 해시) |
| 개체 더블클릭 | 속성 대화상자 (before/after 동일) |

**판정: "둘 다 정상" 분기 — CLSID 축은 HWP3 글맵시 렌더에 가시 효과가 없다.** 글맵시는
`OlePres000` 미리보기(EMF)로 그려지므로 CLSID 유무가 표시에 영향을 주지 않는 것으로 보인다
(차트는 2026-08-05 실측에서 CLSID 부재 시 통째로 비었다 — 개체 종류에 따라 거동이 다르다).
HWP3 축 수정은 "원본이 가진 값을 버리지 않는다"는 보존 원칙으로 정당하며, **효과가 증명된 곳은
차트 축**이다.

판정의 한계: 이 실측에서 변환본 자체가 별개 결함(§9)으로 사실상 백지라 "글맵시가 보이는가"·
"편집기가 열리는가" 축은 오염됐다. 확정 가능한 것은 before==after(CLSID 무영향)다.

### 7.3 절차 상태

- 오늘할일(`mydocs/orders/`)은 **미갱신** — 저장소 관례상 PR 생성 최종 준비 시점에 같은 PR diff 로 넣는다.
- **remote push 와 PR 생성은 작업지시자 승인 대기**다.
- 이슈 assignee 지정은 contributor 권한으로 실패했다(Stage 1 §5).

## 8. 재검토와 C2b 게이트 (2026-08-07)

작업지시자의 "CLSID 로 차트 정상 여부가 판정되어야 하는 것 아닌가"라는 지적을 받아 검증 체계를
재검토했다 (`stage5` §2.5 에 상세).

- **확인된 사실**: roundtrip 게이트 C1~C5 는 중첩 OLE CLSID 를 보지 않는다. C1 은
  `bin_data_content` 의 비어있지 않은 **개수**만(`roundtrip.rs:668-679`), C2 는 내용 해시만 본다.
  C2 가 green 인 것은 저장이 중첩 CFB 를 passthrough 하는 덕이다(`cfb_writer.rs:236-249`).
- **채택한 조치 — C2b 게이트**: `hwp5_roundtrip_batch` 에 "중첩 OLE 루트 CLSID 보존" 검사를
  추가했다(`nested_ole_clsid_fingerprint` + `baseline_check`/`roundtrip_one`/TSV/상태
  `OLE_CLSID_LOSS`). 현행 passthrough 저장에서는 자동 green — 존재 이유는 **재포장이 저장 경로에
  들어오는 날**(차트 편집 #3683 등)이다. 그때 C2(내용 해시)는 "편집분은 달라도 된다"로 완화될
  수밖에 없지만 개체 정체성은 C2b 가 계속 지킨다. 검출기 비-공허성은
  `nested_clsid_fingerprint_sees_chart_corpus` 가 고정한다. canonical 문서
  (`hwp5_roundtrip_baseline.md`)에 등재했다.
- **기각한 방향**: "B1 편집 시 C2 실패" 인과(게이트는 무편집 왕복만 돎), `OleContainer` 필드
  선반영(diff 는 Document IR 만 봄), 별도 선행 과제 이슈(**#3557 OPEN** 이 같은 범주를 이미 다룸,
  **#3683 은 go/no-go 미결정**). #3557 에 CLSID 축을 코멘트로 제안한다(승인 완료).

## 9. 판정 중 발견한 별개 결함 — HWP3 변환본 글자 크기 ~100배 붕괴

한컴 판정 과정에서 #4097 과 무관한 결함이 드러났다: 변환본(before·after 공통)을 한컴으로 열면
**전 46쪽 10604개 텍스트 span 이 전부 ~0.12pt** 로 사실상 백지다(원본 9.7~50.5pt, 위치는 정상).
0.12×100=12pt — 정확히 100배 패턴. before==after 46쪽 픽셀 동일이므로 #4097 무관은 증명돼 있다.
동일 증상 기존 이슈 없음 확인 → CharShape `base_size` 실측 보강 후 별도 이슈 등록(승인 완료).
상세는 `stage5` §5.

## 10. 남긴 것

- `OleContainer` 구조체에 `root_clsid` 필드를 넣지 않았다 — `parse_ole_container` 의 `Some` 게이트가
  렌더러 분기(`renderer/layout/shape_layout.rs:1964`)를 좌우하므로, B1(#3683)이 실제로 필요해질 때
  넣는 편이 리뷰 부담이 적다. (재검토에서도 이 이연이 옳았음을 확인 — `diff_documents` 는
  Document IR 만 보므로 이 필드는 검증 게이트에 기여하지 않는다.)
- `cfb_reader.rs` 의 `LenientCfbReader` 쪽 섹터 오프셋 산식 `512 + sid * sector_size` 는 v4(4096)에서
  틀리다. 이번 `root_clsid` 는 그 식을 상속하지 않았으나(체인 순회 없음), **기존 결함은 그대로 남아
  있다** — 별도 이슈 대상이다.
