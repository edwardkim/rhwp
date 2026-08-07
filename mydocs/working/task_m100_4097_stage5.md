# Stage 5 — task_m100_4097 판정 번들과 AC④

- **이슈**: [#4097](https://github.com/edwardkim/rhwp/issues/4097)
- **계획서**: [`mydocs/plans/task_m100_4097.md`](../plans/task_m100_4097.md)
- **선행 단계**: [stage1](task_m100_4097_stage1.md) · [stage2](task_m100_4097_stage2.md) ·
  [stage3](task_m100_4097_stage3.md) · [stage4](task_m100_4097_stage4.md)
- **브랜치**: `task_m100_4097`
- **작업 시각**: 2026-08-07 KST
- **프로덕션 코드 변경**: 0

## 1. 판정 번들 재생성 — 그리고 예상 밖의 확증

```
CARGO_INCREMENTAL=0 cargo test --profile release-test \
  --test issue_4055_b1_chart_edit_probe -- --ignored --nocapture
```

이제 `mutate_nested` → `rebuild_cfb_preserving_clsid` 는 테스트 로컬 바이트 수술이 아니라
`ole_root_clsid` + `build_cfb_with_root_clsid` **프로덕션 경로**다(Stage 3). 그 경로로 변종 10종을 다시
만들었더니 생성기가 **전건 `unchanged`** 를 보고했다.

```text
  unchanged 00-control-원본.hwpx  (40484 bytes)  zip=Some(4.3) nested=Some(4.3) legacy=Some(4.3) emf=true
  unchanged 00-control-원본.hwp   (36864 bytes)  zip=None      nested=Some(4.3) legacy=Some(4.3) emf=true
  unchanged X-A-zip파트만.hwpx    (40385 bytes)  zip=Some(91.7) nested=Some(4.3)  legacy=Some(4.3)  emf=true
  unchanged X-B-레거시만.hwpx     (37605 bytes)  zip=Some(4.3)  nested=Some(4.3)  legacy=Some(91.7) emf=true
  unchanged X-C-셋다.hwpx         (37506 bytes)  zip=Some(91.7) nested=Some(91.7) legacy=Some(91.7) emf=true
  unchanged X-D-셋다+EMF제거.hwpx (25369 bytes)  zip=Some(91.7) nested=Some(91.7) legacy=Some(91.7) emf=false
  unchanged H-A-중첩OOXML만.hwp   (33792 bytes)  zip=None nested=Some(91.7) legacy=Some(4.3)  emf=true
  unchanged H-B-레거시만.hwp      (33792 bytes)  zip=None nested=Some(4.3)  legacy=Some(91.7) emf=true
  unchanged H-C-둘다.hwp          (33792 bytes)  zip=None nested=Some(91.7) legacy=Some(91.7) emf=true
  unchanged H-D-둘다+EMF제거.hwp  (21504 bytes)  zip=None nested=Some(91.7) legacy=Some(91.7) emf=false
```

`unchanged` 의 의미는 생성기 `:796-798` 에서 **디스크 내용과의 전체 바이트 비교**다.

```rust
let on_disk = std::fs::read(&target).ok();
let status = if on_disk.as_deref() == Some(bytes.as_slice()) { "unchanged" } else { ... };
```

그리고 디스크에 있던 파일들의 타임스탬프는 **2026-08-05**, 즉 #4055 가 한컴 판정에 넘긴 바로 그
산출물이다(같은 디렉터리의 판정 PDF 도 2026-08-05 16:34–16:35).

```text
-rw-r--r--  36864  2026-08-05 15:56  00-control-원본.hwp
-rw-r--r--  33792  2026-08-05 16:07  H-A-중첩OOXML만.hwp
-rw-r--r--  33792  2026-08-05 16:07  H-C-둘다.hwp
-rw-r--r--  21504  2026-08-05 16:07  H-D-둘다+EMF제거.hwp
-rw-r--r-- 100810  2026-08-05 16:35  H-A-중첩OOXML만.pdf      ← 당시 판정 PDF
```

**결론**: 프로덕션 경로가 **2026-08-05 한컴 판정을 통과한 파일을 바이트 단위로 재현한다.** 특히
`H-A`·`H-C`·`H-D` 는 중첩 CFB 를 실제로 재포장한 변종이라, 이번 수정이 지나가는 바로 그 경로다.

이는 Stage 3 의 `production_api_reproduces_the_hancom_validated_bytes`(코퍼스 56건 합성 대조)와 독립적인
두 번째 증거다 — 이쪽은 **판정에 실제로 쓰인 파일 그 자체**와의 대조다.

### 1.1 산출물 SHA-256

| 파일 | SHA-256 |
|---|---|
| `00-control-원본.hwp` | `12e8b4034cb236cc2fc20b777f90628825111e2ee8d8ec81e97f3fa60987d644` |
| `H-A-중첩OOXML만.hwp` | `806beef00e057f51f689c151f712a2a248ae1efabb8d19a4f18e12701508dd0a` |
| `H-B-레거시만.hwp` | `a836a9af34de68407431f19cdeef2f3b65612c5195380fe7c0a53a40c4bb0cb6` |
| `H-C-둘다.hwp` | `76d37400e7151b01d57e33bae9a8e6df1027251245a5b9a56a31431a6388128d` |
| `H-D-둘다+EMF제거.hwp` | `151badc428b5d8a4448a540b8b26111e2358920d8feec8a30abd6814414ffdf3` |
| `00-control-원본.hwpx` | `e8ffa2ef81e630b5ac7e0a230c101924b96bd4d61be958cf608161bfe9a7365a` |
| `X-A-zip파트만.hwpx` | `897e9a906659f8b321e130ce39fd5a86819b42f70a4adb02969fa98df82591c7` |
| `X-B-레거시만.hwpx` | `7e023ddd66a860c437503e1a5f8cb21e29f4c06f6e25ff3ac7c0d2cc2f7f8c5f` |
| `X-C-셋다.hwpx` | `66fe811e40812bd1419ec95938c79da5632da74c719ee45323ba13528ed2ea0c` |
| `X-D-셋다+EMF제거.hwpx` | `bfdd90ccea4455d3d95acb09d0c92f96b05c5c2242a7acf94115e1fd7884d67c` |

위치: `output/issue_4055_b1_spike/` (gitignore 대상, `.gitignore:15`). 판정표는 같은 디렉터리
`PANJEONG.md`.

## 2. AC④ — 남은 것과 넘기는 것

이슈 AC④ 는 "한컴에서 재포장본 개체가 정상 렌더되고 더블클릭 시 편집기가 열림 (원격 Windows)"이다.

### 2.1 자동화로 확보한 것

| 증거 | 내용 |
|---|---|
| 코퍼스 56건 바이트 동일성 | `production_api_reproduces_the_hancom_validated_bytes` — 프로덕션 출력 == (`build_cfb` + 사후 스탬프) 레시피 |
| 판정 파일 그 자체와의 동일성 | 위 §1 — 2026-08-05 판정 산출물 10종 전건 `unchanged` |
| CLSID 외 차이의 무해성 | #4055 stage4 `:64-70` 이 원본↔재포장본 디렉터리 엔트리를 **전수 비교**해 color flag(0→1)·타임스탬프를 무해 판정. 루트 CLSID 가 "유일한 유의미 차이"였고, 되박자 정상 렌더됐다 |

즉 AC④ 는 **"새로 측정할 미지"가 아니라 "이미 측정된 것의 재확인"** 이다.

### 2.2 넘기는 것 — 작업지시자 육안 판정

HWP 2020 MCP 는 현재 PDF 변환이 되지 않아(작업지시자 확인) 자동 재현 경로가 없다. 육안 판정은
작업지시자가 직접 수행한다.

판정 대상과 항목:

| 파일 | 확인할 것 |
|---|---|
| `H-A-중첩OOXML만.hwp` · `H-C-둘다.hwp` · `H-D-둘다+EMF제거.hwp` | **중첩 CFB 재포장 축** — ① 개체가 틀만 남고 비지 않는가 ② 첫 막대가 차트를 뚫고 솟는가(sentinel `91.7` 반영) ③ **더블클릭 시 편집기가 열리는가** ④ 오류·복구 대화상자가 없는가 |
| `H-B-레거시만.hwp` | 대조군 — 재포장은 했으나 한컴이 읽는 표현(OOXML)은 안 바꿈 → 개체는 정상이되 sentinel 미반영이어야 정상 |
| `00-control-원본.hwp` / `.hwpx` | 무변경 대조군 |
| `X-*` | HWPX 축(중첩 CFB 미재포장) — 회귀 확인용 |

기대 결과(2026-08-05 기준선): sentinel **반영** 6종(`X-A`·`X-C`·`X-D`·`H-A`·`H-C`·`H-D`), **미반영**
4종(대조군 2 + `X-B` + `H-B`). 그리고 **전 변종이 오류 대화상자 없이 열린다.**

> 2026-08-05 판정에서는 `H-A`·`H-B` 가 처음에 차트가 통째로 비었고, 그것이 이 이슈의 발단이었다.
> CLSID 를 되박은 뒤 정상화됐다. 이번 판정에서 다시 비면 이 수정이 그 경로를 못 탄 것이다.

## 2.3 HWP3 축 — 판정 산출물 생성 (수정 전/후 쌍)

차트 축과 달리 **HWP3 글맵시는 한컴 판정 이력이 전혀 없다.** 판정할 것이 있는 유일한 축이라 수정
전/후 쌍을 만들었다.

```bash
target/release-test/rhwp.exe convert     samples/SO-SUEOP.hwp output/issue_4097_hwp3/SO-SUEOP-after.hwp
target/release-test/rhwp.exe export-hwpx samples/SO-SUEOP.hwp output/issue_4097_hwp3/SO-SUEOP-after.hwpx
```

`before` 는 `hwp3/ole.rs` 를 `build_cfb(&refs)` 로 임시 되돌려 빌드한 바이너리로 같은 명령을 실행해
만들었다(생성 후 원복·재빌드 완료).

### 산출물 차이 — 설계대로인지 바이트로 확인

`.hwpx` 의 `BinData/image1.ole`(19972B, 선두 4바이트 LE size prefix + CFB):

```text
다른 바이트 수: 16 (그중 값이 실제로 바뀐 자리 5)
다른 구간     : 596 ~ 611  → CFB 내 오프셋 592 = 512(헤더) + 80(루트 디렉터리 엔트리 CLSID)
  before = 00000000000000000000000000000000
  after  = 1442040000000000c000000000000046   = {00044214-0000-0000-C000-000000000046}
```

**오직 루트 디렉터리 엔트리 +80 의 16바이트만 다르다.** 나머지 19956 바이트는 완전히 같다 — 재포장의
다른 어떤 것도 건드리지 않았다는 기계적 증명이다.

`.hwp` 는 13646 바이트가 다르다. BinData 가 deflate 로 압축돼 들어가 16바이트 변경이 압축 스트림
전체로 번지기 때문이며, 총 크기는 99840 으로 동일하다.

### 판정 안내

`output/issue_4097_hwp3/PANJEONG.md` 에 대상·확인 항목·**결과 해석 분기**를 정리했다. 특히 "둘 다
정상"인 경우(글맵시는 CLSID 없이도 한컴이 알아보는 경우)도 유효한 결과로 기록하도록 적어 뒀다 —
수정 자체는 원본 값을 버리지 않는 쪽이 옳지만, 그때는 이 축에 눈에 보이는 개선이 없다는 사실을
보고서에 남겨야 한다.

## 2.4 한컴 판정 결과 (2026-08-07, 작업지시자 실측)

작업지시자가 4개 파일(before/after × hwp/hwpx)을 한컴으로 열고, 원본·before·after 를 한컴에서
PDF 로 내보냈다. 판정 근거 PDF 는 관례(`mcp_hwp2020Convert_usage.md` — 리뷰 기준 PDF 는
`output/` 에만 두지 않는다)에 따라 `pdf/task4097/` 에 커밋했다:

| 파일 | SHA-256 |
|---|---|
| `SO-SUEOP.pdf` (원본 한컴 출력, 46쪽) | `78a4ca30f870e744aa13701c45d269dbe5c9d3741d3f3636311f20cfc945c629` |
| `SO-SUEOP-before-hwp.pdf` (수정 전 변환본) | `ee0a4be9ca20391fdc114f6ad0c2ac144b38609f61b48dd95241e409eb6e97cd` |
| `SO-SUEOP-after-hwp.pdf` (수정 후 변환본) | `7621bf3da6a84db2f144858c565a3907bd0d86ed189bfbdb56145009bd613537` |

| 항목 | 결과 |
|---|---|
| 오류·복구 대화상자 | **4개 전부 없음** |
| before vs after 렌더 | **46쪽 전부 픽셀 동일** (PyMuPDF 72dpi 픽스맵 SHA-256, 다른 쪽 0) |
| 개체 더블클릭 | 편집기 아닌 **속성 대화상자** (before/after 동일) |

**판정: CLSID 축은 HWP3 글맵시 렌더에 가시 효과 없음 — PANJEONG 의 "둘 다 정상" 분기.**
글맵시는 `OlePres000` 미리보기(EMF)로 그려지므로 CLSID 유무가 표시에 영향을 주지 않는 것으로
보인다(차트는 2026-08-05 실측에서 CLSID 부재 시 통째로 비었다 — 개체 종류에 따라 거동이 다르다).
수정은 "원본이 가진 값을 버리지 않는다"는 보존 원칙으로 정당하며, 효과가 증명된 곳은 차트 축이다.

**판정의 한계**: 이 실측에서 변환본 자체가 별개 결함(전 46쪽 글자 크기 ~100배 붕괴, §5)으로
사실상 백지라, "글맵시가 제대로 보이는가"·"편집기가 열리는가" 축은 CLSID 단독으로 판정할 수 없게
오염됐다. 확정 가능한 것은 **before==after(CLSID 무영향)** 하나다.

## 2.5 재검토 기록 (2026-08-07)

작업지시자 요청으로 "CLSID 로 차트 정상 여부가 판정되어야 하는 것 아닌가"를 재검토했다. 결론:

- **유지**: roundtrip 게이트 C1~C5 는 중첩 OLE CLSID 를 보지 않는다(C1 은
  `bin_data_content` 비어있지 않은 개수만, C2 는 내용 해시). C2 green 은 저장 passthrough 덕이다.
- **철회 3건**: ① "B1 편집 시 C2 실패 → 보호막 0" 인과 — 게이트는 무편집 왕복만 돌므로 성립하지
  않음(#4055 report §4-4 가 이미 "편집분만 분기"를 요구). ② "`OleContainer` 에 CLSID 를 지금
  넣었어야" — `diff_documents` 는 Document IR 만 보므로 무의미, 이연이 옳았음. ③ "별개 선행 과제
  이슈 등록" — **#3557(OPEN)** 이 같은 범주를 이미 다루고, **#3683 은 go/no-go 미결정**이라
  blocker 표현도 과장.
- **채택된 실질 조치**: C2b 게이트(중첩 OLE 루트 CLSID 보존)를 `hwp5_roundtrip_batch` 에 추가.
  현행 passthrough 저장에서는 자동 green — 재포장이 저장 경로에 들어오는 날 C2 를 편집 허용으로
  완화해도 개체 정체성을 계속 지키는 알람이다. 검출기의 비-공허성은
  `nested_clsid_fingerprint_sees_chart_corpus`(차트 코퍼스에서 정확히 1건·비-0 CLSID)로 고정했다.
  canonical 문서(`hwp5_roundtrip_baseline.md`) 검사 표에 C2b 를 등재했다.

## 3. 검증

```
CARGO_INCREMENTAL=0 cargo test --profile release-test \
  --test issue_4055_b1_chart_edit_probe -- --ignored --nocapture
  running 1 test ... test result: ok. 1 passed; 0 failed
```

생성기는 내보내기 전에 각 변종을 스스로 검증한다 — ① `rhwp::parse_document` 재개봉 ②
`Representations` 4중 표현이 라벨과 일치. 두 검증이 통과했으므로 파일이 라벨대로 조립됐다.

C2b 게이트 추가분 검증 (전부 직렬, `CARGO_INCREMENTAL=0`):

```
cargo test --profile release-test --lib hwp5_roundtrip_batch
  13 passed (신규: nested_clsid_fingerprint_sees_chart_corpus, multiset_lost_counts_only_missing)
cargo test --release --test hwp5_roundtrip_baseline
  3 passed; 0 failed  (51.8s — C2b 활성 상태로 전 샘플 통과)
cargo test --profile release-test --tests
  469개 바이너리 전부 ok — 5285 passed, 0 failed, 28 ignored
cargo fmt --check   → Diff in 0건
cargo clippy --all-targets -- -D warnings   → exit 0
```

## 5. 신규 발견 결함 — HWP3 변환본 글자 크기 ~100배 붕괴 (별도 이슈 대상)

한컴 판정 과정에서 #4097 과 **무관한** 결함이 드러났다. 변환본(before·after 공통)을 한컴이 열면
사실상 백지다.

한컴 PDF 내보내기 실측(PyMuPDF):

| | 원본 `SO-SUEOP.pdf` | 변환본 `SO-SUEOP-{before,after}-hwp.pdf` |
|---|---|---|
| 글자 크기 | 9.7 ~ 50.5 pt | **전 46쪽 10604개 span 전부 ~0.12 pt** |
| 글자 위치(origin) | 정상 | **정상** — 크기만 붕괴 |
| 텍스트 내용 | 정상 | 정상 (추출 텍스트는 오히려 원본보다 많음) |
| 개봉 | 정상 | 정상 (대화상자 없음) |

**before==after 46쪽 픽셀 동일**이므로 #4097 변경과 무관함은 증명돼 있다.

### 원인 실측 (임시 프로브, 측정 후 삭제)

`base_size` 100배 가설은 **기각**됐다 — 변환본을 rhwp 로 재파싱하면 base_size 분포가 원본과
완전히 동일하다(2512개, 최빈값 1000=10pt). 즉 rhwp 는 자기 산출물을 정상으로 읽는다 — #4097·#3557
과 같은 **"자기정합은 유지되고 한컴만 깨지는"** 계열이다.

진범 후보는 정답지 대조에서 나왔다:

| CharShape(base_size=1000 대표) | ratios(장평) | **relative_sizes(상대크기)** | spacings(자간) |
|---|---|---|---|
| 원본 HWP3 파싱 IR | 95 | **0** | -1 |
| 변환본 HWP5 재파싱 IR | 95 | **0** | -1 |
| **한컴산 HWP5 정답지** (차트 코퍼스) | 100 | **100** | 0 |

스펙상 상대크기 유효범위는 10~250(%)인데, `convert_char_shape`(`src/parser/hwp3/mod.rs:523-541`)가
`base_size`·`font_ids`·`ratios`·`spacings` 만 채우고 **`relative_sizes` 를 채우지 않아**
`CharShape::default()` 의 0 이 그대로 직렬화된다. 한컴이 `크기 × 상대크기%` 로 해석하면
10pt × ~0% ≈ 0.1pt — 실측 0.12pt 와 부합한다. rhwp 렌더러는 0 을 유효값으로 쓰지 않아
자체 렌더에서는 드러나지 않았다.

동일 증상 기존 이슈 없음 확인(#1950 은 렌더 변위 군집, #3676 은 개봉 거부 — 모두 다름).
→ 위 실측을 첨부해 별도 이슈로 등록한다(작업지시자 승인 완료).

## 6. 절차 상태

- 오늘할일(`mydocs/orders/YYYYMMDD.md`)은 **아직 갱신하지 않았다.** 저장소 관례상 "변경 범위·검증·merge
  판단·PR 생성 승인이 확정된 최종 준비 시점"에 최초 remote push 와 같은 PR diff 에 포함한다
  (`collaborator_self_merge.md:31-38`).
- **remote push 와 PR 생성은 작업지시자 승인 대기**다 (`docs_and_git_workflow.md:164-174`, `AGENTS.md:31`).
- #3557 코멘트(CLSID 축 제안)와 백지 결함 이슈 등록은 작업지시자 승인 완료(2026-08-07).
- 이슈 assignee 지정은 권한 부족으로 실패했다(Stage 1 §5).
