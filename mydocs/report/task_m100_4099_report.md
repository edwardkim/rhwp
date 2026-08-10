---
kind: report
status: active
canonical: mydocs/report/task_m100_4099_report.md
last_verified: 2026-08-10
---

# #4099 최종 보고 — HWPX→HWP5 변환이 차트를 통째로 잃던 결함

- **Issue**: [#4099](https://github.com/edwardkim/rhwp/issues/4099)
- **브랜치**: `task4099` (`upstream/devel = 629cd33db` 기준)
- **계획서**: [`task_m100_4099.md`](../plans/task_m100_4099.md)
- **검증 환경**: 로컬 Rust 테스트 + 한컴 2022 판정(작업지시자, 2026-08-10)

## 결론 한 줄

**차트 OleShape 를 `<hp:default>` fallback OLE 로 접는다.** 그것이 한컴 자신이 하는 일임을
정답지 바이트로 확인했고, **변환본의 한컴 렌더가 정답지와 픽셀 단위로 같음**을 확인했다.
프로덕션 변경은 어댑터 한 파일에 함수 둘, 그리고 그 fold 가 드러낸 저장 의미론 문제를
막는 `wasm_api` 1줄이다.

## 1. 무엇이 문제였나

HWPX 파서는 `<hp:switch>` 의 `<hp:case>` 브랜치를 채택해 **가상 id** `bin_data_id = 60000+N`
을 세우고, `<hp:default>` 의 진짜 OLE 를 `chart_switch_fallback` 에 매달아 둔다(#3546 —
HWPX 저장 시 원형 재방출의 재료). 그 가상 id 는 zip 파트 `Chart/chartN.xml` 을 가리키므로
**HWP5 에는 대응물이 없다.** 그런데 HWP5 저장 경로에 이 규약을 아는 코드가 **0건**이었다.

| 축 | 결과 |
|---|---|
| `serialize_ole_data` | `60001` 을 그대로 기록 → HWP5 DocInfo 에 없는 storage 참조(dangling) |
| `find_bin_data_info_with_compress` 폴백 | `/BinData/BINEA61.ooxml_chart` **DocInfo 미등록 정크 스트림** 생성 |
| 재파싱 | 그 스트림을 읽지 않아 `--verify` 가 `bin_data_content count: expected=2 actual=1` → exit 3 |
| rhwp 자신의 렌더 | `"OLE 개체 (BinData #60001)"` 회색 상자 |

`samples/chart/**` **28종 전건**이다.

### 왜 지금까지 안 잡혔나 — 두 가지가 겹쳤다

**① 바이트는 멀쩡히 보존된다.** 끊어진 것은 참조뿐이다. #4055 스파이크의
`observation_hwpx_to_hwp_conversion_keeps_the_chart` 는 변환본의 **바이트**를 확인해
②③④ 보존을 증명하고 "별도 이슈로 낼 결함은 없다"로 마쳤다. 그 판정은 바이트에 대해서는
지금도 옳다 — 다만 본문 컨트롤이 그것에 도달하지 못한다는 축을 보지 않았다.

**② 게이트가 그 축에서 비어 있었다.** `convert --verify` 는 이 결함을 처음부터 exit 3 으로
보고하고 있었다. `tests/convert_verify_corpus_ratchet.rs` 의 코퍼스가 `read_dir(samples/)`
**비재귀**라 `samples/chart/**` 가 통째로 제외돼 있었을 뿐이다. 결함보다 이쪽이 근인에 가깝다.

## 2. 정답지 — 한컴은 무엇을 하는가

`samples/chart/세로막대형/묶은세로막대형` 을 두 포맷으로 놓고 실측했다.

```text
오라클 .hwp        bin_data_id=1      instance_id=0           attr=0x140A2210  raw=30B
HWPX chart 브랜치  bin_data_id=60001  instance_id=1117817146  attr=0x140A2210  raw=0B
HWPX fallback      bin_data_id=1      instance_id=0           attr=0x140A2210  raw=0B
현재 변환본(수정 전) bin_data_id=60001  instance_id=1117817146                  raw=26B
```

**`instance_id` 가 유일한 판별자다.** `parse_hp_chart_element` 은 `@id` 를 읽어
`1117817146`, `parse_hp_ole_element` 은 `@id` arm 이 없고 `instid="0"` 만 읽어 `0` 이 된다.
오라클이 **0** 이라는 것은 **한컴 자신의 HWPX→HWP5 변환도 fallback 브랜치를 쓴다**는 뜻이다.

정답지 CFB 도 같은 말을 한다 — `BinData/BIN0001.OLE` **하나뿐**이고 `.ooxml_chart` 스트림이
없다. 그 OLE 안에 한컴이 실제로 읽는 중첩 `OOXMLChartContents` 가 들어 있다(#4055 H-A 변종:
그 사본만 고쳐도 렌더에 반영됨).

> **설계 근거를 한 번 갈아엎었다.** 처음에는 "fallback 이 정보 상위집합이라 통째로 채택한다"로
> 적었으나 그것은 **거짓**이다. `parse_common_shape_children` 은 여섯(+#4319 캡션) arm 만
> 처리해 `orgSz`/`curSz`/`flip`/`renderingInfo`/`lineShape` 를 IR 에 싣지 않는다. XML 상의
> 상위집합은 모델에 도달하지 못한다. 결론은 유지됐지만 근거는 오라클 실측으로 교체했다.
> 그래서 `issue4099_folded_ole_matches_hancom_oracle` 로 그 근거 자체를 고정했다 —
> 없으면 다음 사람이 "instance_id 를 chart 쪽에서 살려야 하지 않나"로 되돌린다.

## 3. 무엇을 고쳤나

결함 자체의 수정은 [`hwpx_to_hwp.rs`](../../src/document_core/converters/hwpx_to_hwp.rs)
**한 파일**이다(§3-1·3-2·3-4). §3-3 은 그 fold 가 드러낸 별도 축으로, `wasm_api.rs` 1줄이다.

### 3-1. `fold_hwpx_chart_ole_for_hwp`

`chart_id_ref` 가 있는 `OleShape` 에서 `chart_switch_fallback` 을 **move 로 통째 채택**하고,
`extension == "ooxml_chart"` 인 `BinDataContent` 를 전량 제거한다.

캡션은 명시적으로 이월한다. #4319(`7553410cd`, 이 작업 착수 전날)가 차트·OLE 캡션 파싱을
고쳐 양쪽 `<hp:caption>` 이 모두 IR 에 들어오게 됐으므로, chart 쪽에만 있는 캡션은 fallback
채택으로 조용히 사라진다. 코퍼스 28종은 캡션이 0건이라 **합성 문서로만 잴 수 있는 축**이다.

fallback 이 없는 변형(switch 밖 단독 `<hp:chart>`, `<hp:default>` 없는 case-only switch)은
코퍼스 0건이다. 접을 대상이 없으므로 참조를 비워 정크와 dangling 을 둘 다 막고 placeholder 로
남긴다. `mini_cfb` 로 CFB 를 합성하는 길은 #4097 로 도구가 갖춰졌으나, 참조할 원본 CLSID 가
없어 상수 하드코딩이 필요하고 `OOXMLChartContents` 하나만 든 CFB 를 한컴이 받아들이는지
미검증이라 주석으로만 남겼다.

### 3-2. 파이프라인 위치 — 도형을 건드리는 첫 패스

```text
normalize_file_header_for_hwp
normalize_page_border_fills_for_hwp
fold_hwpx_chart_ole_for_hwp          ← 신규
normalize_picture_geometry_for_hwp
normalize_doc_properties_for_hwp
materialize_hwp5_bin_data_order
normalize_bin_data_for_hwp
```

`materialize_hwp5_bin_data_order` 앞이라야 fold 가 올려놓은 진짜 `bin_data_id` 를 remap 이
본다. **덤으로 "`chart_switch_fallback` 을 어떤 워커도 방문하지 않는다"는 잠재 결함이 함께
풀린다** — fold 후에는 그 상자 자체가 없어지기 때문이다.
`normalize_picture_geometry_for_hwp` 보다도 앞인 이유는, 그 패스가 바깥 차트 OleShape 에 가한
변경이 fold 로 통째로 버려지기 때문이다(오늘은 무해하나 조용히 깨질 자리다).

### 3-3. `wasm_api::exportHwp` 를 스냅숏 저장으로 옮겼다

fold 는 IR 에서 `chart_id_ref` 와 `ooxml_chart` 를 **없앤다.** 어댑터가 살아 있는 IR 을
직접 정규화하므로, HWP 로 한 번 저장한 뒤 같은 핸들로 HWPX 를 내보내면
`write_ole_or_chart` 가 `hp:switch/case/default` 대신 `hp:ole` 단독을 방출하고
`Chart/chart1.xml` 파트가 빠진다 — **#3546 계약이 저장 한 번으로 깨진다.** 실측했다.

```text
[HWP 저장 전]  Chart 파트=1  hp:chart=1  hp:switch=1
[HWP 저장 후]  Chart 파트=0  hp:chart=0  hp:switch=0   ← 수정 전
```

**이건 새 종류의 문제가 아니다.** `export_hwp_with_adapter_snapshot` 이 정확히 이 부류를
위해 이미 있었고(주석: *"저장은 스냅숏이어야 하므로"*), 같은 원인의 다른 증상인 누름틀
`field_ranges` 어긋남도 거기 적혀 있다. CLI edit 경로만 이관돼 있고 wasm 은 남아 있었다.
#4099 가 그 미완 이관의 증상을 하나 더 늘렸으므로 여기서 마무리했다.

우회가 아니라 **원인 제거**다 — 저장이라는 읽기 연산이 IR 을 바꾸지 않게 한다. fold 는
복제본에서 그대로 돌아 HWP 산출물은 동일하다(정크 0, `bin_data_id=1` 확인).

| 측정 | 결과 |
|---|---|
| 전체 스위트 회귀 | **0건** — 503 blocks / 5572 passed, 전환 전 기준선과 완전 동일 |
| 되돌렸을 때 | T7 red: `(1,1,1) → (0,0,0)` |
| 비용 | 브라우저 HWP 저장 시 `Document` clone 1회 |

### 3-4. 순회 — 네 번째 복제 워커를 만들지 않았다

이 파일에는 거의 같은 재귀 워커가 이미 넷 있다. 좁은 타입 `for_each_ole_mut` 를 두고 골격만
`normalize_picture_geometry_for_hwp` 쪽에서 가져왔다 — 넷 중 커버리지가 가장 넓어
`Control::Field` 메모 문단과 `SectionDef.master_pages` 까지 닿는 유일한 워커다(remap 계열은
못 닿는다). 커버리지 표를 doc 주석에 남겼다. 다섯을 하나로 합치는 것은 커버리지가 서로 달라
동작이 바뀔 수 있는 별개 리팩터이므로 §7 로 분리했다.

## 4. 수용 기준 대조

| # | 기준 | 결과 |
|---|---|---|
| 1 | 코퍼스 28종 렌더에 `"OLE 개체 (BinData #"` 0건 | **충족** — `hwp-ooxml-chart` 렌더 확인, fallback placeholder 도 0건 |
| 2 | 28종 `convert --verify` exit 0 | **충족** — 전건 diff 0 |
| 3 | 산출 `.hwp` 에 `BIN*.ooxml_chart` 정크 0건 | **충족** |
| 4 | OLE `bin_data_id` 가 DocInfo 실재 `storage_id` 를 가리킴 | **충족** — `1`, 오라클과 동일. `Storage` 등록 확인 |
| 5 | 차트+그림 합성으로 `bin_count > 1` remap 경로 커버 | **충족** — §5-1 |
| 6 | 이 축이 다시 비지 않게 함 | **충족** — §5-2 |
| 7 | 한컴에서 변환본을 열어 차트가 보임 | **충족** — 정답지와 렌더 바이트 일치. §6 |
| 8 | 기존 계약 전건 green | **충족** — §5-3 |

## 5. 검증

### 5-1. `bin_count > 1` — 코퍼스로는 잴 수 없던 경로

코퍼스 28종은 전부 BinData 가 `ole1.ole` 하나뿐이라
`materialize_hwp5_bin_data_order` 의 `bin_count <= 1` 조기 반환에 걸린다 — **그 remap 은 차트
문서에서 한 번도 실제로 돌아본 적이 없다.**

차트 hwpx 에 그림을 얹어 런타임에 조립했다(`samples/chart/` 에 커밋하면
`issue_4055_b1_chart_edit_probe` 의 `checked == 56` 하드코딩이 깨진다). manifest 에 `image1`
을 `ole1` 앞에 넣어 순번 1(그림)/2(OLE)를 만들고, 그림 문단은 실 코퍼스에서 떼어왔다 —
XML 을 손으로 쓰면 `hc:imgRect`/`hp:imgClip`/`hp:imgDim` 을 빠뜨리기 쉽다. section XML 의
`binaryItemIDRef="ole1"` 은 일부러 그대로 뒀다. `canonicalize_bin_item_refs` 가 `image2` 로
정규화하는 것까지 함께 검증하기 위해서다.

`AdapterReport` 를 직접 단언해 새 경로가 열렸음을 증거로 남겼다.

```text
합성본        bin_data_order_materialized = 1   ← remap 이 실제로 돌았다
대조군(코퍼스) bin_data_order_materialized = 0   ← 조기 반환
```

fold 를 임시로 끄면 `ole.bin_data_id` 가 `60001` 로 남아 red 임을 확인했다.

### 5-2. 래칫 — 게이트가 실효인가

`convert_verify_corpus_ratchet.rs` 의 코퍼스에 `samples/chart` 를 재귀로 합쳤다.
`samples/` 전체를 재귀로 열지 않은 것은 범위 통제다 — 나머지 하위 68개 디렉터리는 이 이슈의
축이 아니고, 한꺼번에 넣으면 무관한 신규 실패를 이 PR 에서 등재해야 한다.

| 측정 | 결과 |
|---|---|
| 합병 후 신규 실패 | **0건** — `EXPECTED_FAILURES` 등재 불필요 |
| 한컴 저작 `.hwp` 28건 (노이즈 스트립 없는 strict 비교) | 전건 통과 |
| **fold 를 끄면** | 4개 조각에서 5+7+5+11 = **28건** 검출 (코퍼스 `.hwpx` 전건) |
| 파일명 충돌 / 1MB 초과 | 각 0건 → 56건 전부 검사 |
| 실행 시간 | 15초(4조각 병렬) |

계획 단계의 위험 R4("한컴 저작 `.hwp` 28건이 처음 strict 비교에 들어가 대량 실패할 것")는
**실현되지 않았다.**

### 5-3. 계약 테스트

| 테스트 | 결과 |
|---|---|
| `issue_3546_chart_preserved_on_save` | 2/2 green, 무수정 |
| `issue_1251_ole_chart_contents` | 10/10 green, 무수정 |
| `issue_3547_ole_size_prefix` | 1/1 green, 무수정 |
| `issue_4055_b1_chart_edit_probe` | 9/9 green + 1 ignored, 무수정 |
| `hwpx_to_hwp_adapter` | 50/50 green, 무수정 |
| `convert_verify_corpus_ratchet` | 4/4 green (코퍼스 확장) |
| `issue_4099_hwpx_chart_to_hwp` (신규) | 10/10 green + 1 ignored |

`issue_4055::editing_only_the_zip_part_is_lost_when_converting_to_hwp` 가 green 인 것은
**호출 순서 덕분이다** — `export_hwpx_native()` 를 `export_hwp_with_adapter()` 보다 먼저
부른다. 순서가 반대였으면 fold 가 zip 파트를 지워 깨졌을 것이다. 이것이 §7-2 의 축소판이다.

전체 스위트(`cargo test --profile release-test --tests`, 로그 `$TMPDIR/task4099_full_test.log`):

```text
503개 결과 블록 — 5573 passed, 0 failed, 36 ignored (exit 0)
```

`cargo fmt --check` 는 `Diff in` 0건(Windows CRLF 로 exit 1 이나 실제 차이 없음),
`cargo clippy --profile release-test --all-targets -- -D warnings` 통과.

## 6. 한컴 판정 (수용 기준 7) — 통과

`output/issue_4099/` 에 5파일을 냈다(gitignored).

fold 산출본과 정답지의 `BodyText/Section0` 레코드를 **전수 대조**해 변종을 실측으로 골랐다.
차트 개체 관련 차이는 **정확히 둘**이고, GenShape CTRL_HEADER 46B 는 **바이트까지 같다**.

```text
[12] CTRL_HEADER(gso)      46B   SAME          ← fold 가 instance_id 포함해 맞췄다
[13] SHAPE_COMPONENT      196B   @38: 0b→00    ← flip 워드 0x000B_0000
[14] SHAPE_COMPONENT_OLE   30B→26B             ← 앞 26B 동일, 꼬리 reserved u32 부재
```

나머지 세 곳(SectionDef CTRL_HEADER 47B/38B, `PAGE_BORDER_FILL` ×2)은 차트와 무관한 기존
축이다.

두 차이 모두 fold 이전부터 있었고 이 PR 이 만든 것이 아니다. 26B 는
`issue_1251_ole_chart_contents` 가 고정하고 있어 여기서 바꾸면 그 계약이 깨진다. 그래서
**고치는 대신 변종으로 함께 냈다.**

### 판정 결과 — A 가 정답지와 **렌더 바이트 일치**

작업지시자가 한컴 2022 로 두 파일을 열어 PDF 로 저장했다. 그 PDF 의 첫 쪽을 144DPI 로
렌더해 SHA-256 을 비교했다(#4055 가 쓴 방법과 같다 — 메타데이터·타임스탬프를 타지 않는다).

```text
00-oracle-한컴원본.pdf  78834a582ce0a39f758ec7e763b2b07af72d0be798210f3e340be0e2e6ac5e9a
A-fold.pdf              78834a582ce0a39f758ec7e763b2b07af72d0be798210f3e340be0e2e6ac5e9a
                        ^ 동일 (1190x1682 px)
```

렌더 내용도 확인했다 — 묶은 세로막대형 3계열 × 4항목, 제목·범례·축 눈금이 모두 그려진다.
"빈 페이지 둘이 우연히 같은" 경우가 아니다.

| 파일 | 내용 | 판정 |
|---|---|---|
| `00-oracle-한컴원본.hwp` | 목표 화면 | 기준 |
| `A-fold.hwp` | 이 PR 산출본 | **통과 — 기준과 렌더 바이트 일치** |
| `B-fold+ole30.hwp` | OLE 레코드 30B | 불필요 |
| `C-fold+flip.hwp` | flip `0x000B_0000` | 불필요 |
| `D-fold+ole30+flip.hwp` | 둘 다 | 불필요 |

### 부수 성과 — 남은 두 레코드 차이는 한컴이 무시한다

A 는 **26B OLE 레코드**와 **flip = 0** 을 가진 채로 정답지와 픽셀까지 같게 그려졌다.
즉 위 두 차이는 한컴 호환에 영향이 없다. 계획 단계에서 "기준 7 실패 시의 이분 후보"로
잡아 뒀던 위험 R2·R3 가 **실측으로 해소**됐고, 26→30 을 다루는 별도 PR 도 필요 없다.

`issue_1251_ole_chart_contents` 의 `len == 26` 단언은 그대로 유효하다.

## 7. 범위 밖으로 남긴 것

새 이슈는 열지 않는다. 아래 7-1 은 **이미 등록된 이슈 세 건의 근인**이라 네 번째를 만들면
중복만 늘어난다. 대신 PR 본문에 이슈 번호를 적어 GitHub 크로스 레퍼런스로 연결한다.

### 7-1. HWPX 직렬화기가 `bin_data_id`(위치)를 `storage_id` 로 오해한다

**이미 열린 이슈 세 건의 근인이다.**

| 이슈 | 샘플 | 보고된 증상 |
|---|---|---|
| [#3893](https://github.com/edwardkim/rhwp/issues/3893) | `pic-crop-01.hwp` | `binaryItemIDRef 미등록 bin_data_id=1` → `controls: expected=[pic] actual=[]` |
| [#4049](https://github.com/edwardkim/rhwp/issues/4049) | `exam_social.hwp` | `bin_data_id=5, 6 미등록` → 같은 형태 |
| [#4303](https://github.com/edwardkim/rhwp/issues/4303) | `exam_social.hwp` | **#4049 와 중복** (같은 샘플·id·차이 위치, 본문이 임시 경로만 다름) |

셋 다 자동 스윕이 등재한 것으로 *"자동 수정이 red→green 게이트를 넘지 못해 이슈로만 보고"*
상태이고 **근인이 규명돼 있지 않다.** 닫힌 [#3526](https://github.com/edwardkim/rhwp/issues/3526)
(`hwpspec.hwp`)에도 같은 경고(`bin_data_id=37 미등록`)가 있었으나 IR 차이 1149건 중 대부분이
`char_shapes` 축이라 그쪽으로 닫히면서 이 축은 남은 것으로 보인다. 즉 최소 2026-07-28 부터
관측됐는데 스윕이 같은 증상을 계속 재등록해 왔다.

#### 근인

HWP5 규격상 `ImageAttr.bin_data_id` 는 **DocInfo BinData 레코드 순번(1-based)** 이고
`storage_id`(CFB 스트림 이름을 정하는 번호)와 다를 수 있다
(`mydocs/troubleshootings/bin_data_id_index_mapping.md`). 렌더의 `find_bin_data` 는 이 규약대로
순번을 먼저 본다. **HWPX 직렬화기만 그 값을 `storage_id` 로 읽는다.**

```rust
// src/serializer/hwpx/context.rs
let manifest_id = format!("image{}", bd.id);   // bd = BinDataContent → id 는 storage_id
ctx.bin_data_map.insert(bd.id, ..);            // 키 = storage_id
..
pub fn resolve_bin_id(&self, bin_data_id: u16) -> Option<&str>   // 인자는 컨트롤의 "위치"
```

두 값이 같은 문서에서는 우연히 맞는다. 어긋나면 `resolve_bin_id` 가 실패하고 호출자가
**pic 컨트롤을 드롭한 뒤 stderr 경고만 남기고 저장을 성공으로 마친다.**

[#2038](https://github.com/edwardkim/rhwp/issues/2038)(닫힘)이 `next_bin_data_storage_id` 를
만들며 두 번호를 **의도적으로 분리**했고, 그 본문이 *"`ImageAttr.bin_data_id` 는 기존대로
순번(위치) 유지 — `find_bin_data` 의 위치 우선 해석과의 정합을 깨지 않음"* 이라고 적고 있다.
렌더 축만 확인했고 HWPX 직렬화기 축은 보지 않았다.

#### 영향 범위 — 차트 밖에서도 난다

`samples/` 최상위 `.hwp` 를 훑어 `storage_id` 가 순번과 어긋나는 문서를 골랐다.

```text
검사 83건 / storage_id 불일치 4건 / 그림 완전 소실 2건

· pic-crop-01.hwp         storage_id=[3]            IR그림=2 → hp:pic=0   ← 소실 (#3893)
· hwp3-sample10-hwp5.hwp  storage_id=[0,0,0]        IR그림=3 → hp:pic=0   ← 소실
· exam_social.hwp         storage_id=[1,2,3,4,7,8]  bin_data_id=5,6 미등록 (#4049·#4303)
· table-in-tbox.hwp       storage_id=[6,7,8,1,..]   본문 그림 0개 — 무증상
```

#### 차트 문서는 같은 근인의 한 변종이다

`Document::next_bin_data_storage_id` 는 `max(bin_data_content.id, bin_data_list.storage_id) + 1`
을 채번하는데, 차트 문서는 `bin_data_content` 에 sentinel **60001** 이 있어 첫 그림이
`storage_id = 60002` 를 받는다. 그러면 순번(3)과 갈라져 위와 같은 일이 난다. 현재 HEAD 로
재현했다.

```text
삽입 후 IR   content=[(1,"OLE"), (60001,"ooxml_chart"), (60002,"png")]
             list   =[(1,Storage), (60002,Embedding)]

HWPX 저장    Ok      hp:pic=0   재파싱 Picture 컨트롤 0개   ← 그림이 조용히 사라진다
HWP  저장    Ok      Picture bin_data_id=[3] / DocInfo storage_id=[1,60002] → dangling
                     /BinData/BINEA62.png (0xEA62 = 60002)
대조군(차트 없음)     position_id = 4 == storage_id = 4 → HWPX 저장 정상
```

**저장은 성공하고 오류도 없다. 그림만 없어진다.** 렌더는 positional 조회 덕에 살아남아
**화면은 멀쩡한데 저장만 깨지므로** 눈으로는 알아채기 어렵다.

> 이 진단은 두 번 고쳤다. 처음에는 "차트 sentinel 이 근인이고 HWPX 저장이 실패한다"로
> 적었으나 둘 다 틀렸다 — 저장은 `Ok` 를 반환하고(실패보다 나쁘다, 사용자가 알 수 없다),
> sentinel 은 근인이 아니라 위치↔storage_id 불일치를 **드러내는 계기**다. 작업지시자가
> "기존 이슈 중 비슷한 게 있는지 확인해 보라"고 짚어 준 덕에 이미 등록된 세 건에 닿았고,
> 그 과정에서 범위가 차트 밖으로 넓다는 것이 드러났다.

#### #4099 와의 관계

**이 PR 의 fold 로는 안 고쳐진다.** 오염은 편집 시점의 live IR 에서 일어나고 fold 는 HWP
저장 시점에 돈다. 재현은 반드시 저장 전에 해야 한다. `bin_count > 1` 회귀 테스트
(§5-1)의 합성은 매니페스트 순번으로 조립해 이 결함을 비켜 가므로 그대로 재사용할 수 없다.

#### 수정 방향

`resolve_bin_id` 가 위치로 조회하도록 `bin_data_map` 을 순번 키로 바꾸는 것이 규격에 맞는다.
HWPX 출처 문서는 파서가 `storage_id = i + 1` 로 만들어 두 값이 같으므로 그 축은 무영향이고,
**HWP→HWPX 축만 달라진다.** 차트 sentinel 은 `next_bin_data_storage_id` 에서 60000+N 대역을
제외하면 함께 정리된다(그 대역은 네 곳에 매직 넘버로 하드코딩돼 있어 상수화가 선행되면 좋다).

착수 시점은 미정이다.

### 7-2. `hwpx_to_hwp.rs` 의 IR 워커 다섯을 하나로 통합

거의 같은 재귀 워커가 다섯이고 커버리지가 제각각이다. 이 표가 근거다.

| 컨테이너 | bin order | bin ref remap | adapt | picture-geometry | for_each_ole_mut |
|---|---|---|---|---|---|
| 표 셀 · 그룹 자식 · 머리말/꼬리말/각주/미주/숨은설명 | ✓ | ✓ | ✓ | ✓ | ✓ |
| `pic`/`group`/`chart`/`ole` own caption | 일부 | 일부 | — | ✓ | ✓ |
| `Control::Field.memo_paragraphs` | ✗ | ✗ | ✗ | ✓ | ✓ |
| `Control::SectionDef.master_pages` | ✗ | ✗ | ✗ | ✓ | ✓ |

"한 워커가 컨테이너 하나를 놓쳤다"가 이 파일의 반복 결함이다(코드 주석 다섯 곳이 그 이력을
적고 있다). 통합은 동작 변경 위험이 있어 별개 작업이어야 한다.

## 8. 이 작업에서 배운 것

### 8-1. 조사 근거 자체가 낡을 수 있다

계획 수립 중 로컬 `devel` 이 `upstream/devel` 보다 **475 커밋** 뒤처진 상태였고, 작업지시자
지적으로 잡았다. 재검증 결과 골격(변환기 전체 무변경, 60000+N 주입, 정크 폴백, verify 판정,
래칫 비재귀)은 유지됐으나 **두 건이 설계를 바꿨다** — #4319 가 캡션 축을 만들었고, #4171 이
fold 불가 경로의 기각 근거를 무효화했다. 계획서에 기준 커밋 SHA 를 명시하는 관례를 함께 넣었다.

### 8-2. 근인을 찾았다고 새 이슈부터 열면 안 된다

§7-1 을 처음에는 "새로 등록할 spin-off 이슈 초안"으로 썼다. 작업지시자가 *"기존 이슈 중
비슷한 게 있는지 확인해 보라"* 고 짚어 준 뒤에야 **이미 세 건(#3893·#4049·#4303)이 같은
증상으로 열려 있고 그중 둘은 서로 중복**이라는 것을 알았다. 그대로 등록했으면 네 번째 중복이
됐다.

검색이 처음부터 쉬웠던 것도 아니다. "그림 소실", "storage_id 채번" 같은 **내가 이해한 근인**
으로는 하나도 안 나왔고, **도구가 뱉은 오류 문자열**(`binaryItemIDRef`)로 검색했을 때 한 번에
나왔다. 자동 스윕이 등재한 이슈는 사람이 요약한 제목이 아니라 CLI 출력을 그대로 담고 있기
때문이다. `docs_and_git_workflow.md` 가 *"패닉 메시지·오류 문자열은 외부 리포트에 원문 그대로
실리는 경우가 많아 검색어로 효과적"* 이라고 적어 둔 그대로였다.

그리고 그 검색이 **결함의 범위까지 바꿨다.** 차트 전용인 줄 알았던 것이 실은
`storage_id ≠ 순번` 인 모든 문서의 문제였고(코퍼스 83건 중 4건, 그중 2건 그림 완전 소실),
차트 sentinel 은 그것을 드러내는 계기였을 뿐이다.

## 9. 산출물

| 경로 | 내용 |
|---|---|
| `src/document_core/converters/hwpx_to_hwp.rs` | `fold_hwpx_chart_ole_for_hwp` · `for_each_ole_mut` · `AdapterReport` 3필드 |
| `src/wasm_api.rs` | `exportHwp` → `export_hwp_with_adapter_snapshot` (1줄) |
| `tests/issue_4099_hwpx_chart_to_hwp.rs` | 게이트 10건 + 판정 번들 생성기 1건(ignore) |
| `tests/support/issue_4055_chart_probe.rs` | `append_hwpx_entries` 추가 |
| `tests/convert_verify_corpus_ratchet.rs` | `samples/chart` 재귀 병합 |
| `output/issue_4099/` | 한컴 판정 5파일 + `PANJEONG.md` + 작업지시자가 한컴 2022 로 변환한 PDF 2건 + 144DPI PNG 2건 (gitignored) |
