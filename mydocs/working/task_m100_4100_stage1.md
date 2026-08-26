---
kind: working
status: done
canonical: mydocs/working/task_m100_4100_stage1.md
last_verified: 2026-08-10
---

# #4100 Stage 1 — 구조 스캐너 + 최소 diff 패처

- **계획서**: [`mydocs/plans/task_m100_4100.md`](../plans/task_m100_4100.md)
- **기준 커밋**: `devel = dd9ecdc4b`
- **산출**: `src/ooxml_chart/data.rs`(신규) · `src/ooxml_chart/patch.rs`(신규) ·
  `src/ooxml_chart/mod.rs`(선언 2줄) · `tests/issue_4100_chart_data_edit.rs`(신규)

## 1. 무엇을 만들었나

값 하나를 고치기 위해 **차트 XML 전체를 다시 쓰지 않는다.** `c:v` 텍스트가 놓인
바이트 구간만 찾아 그 구간만 갈아끼운다.

| 모듈 | 책임 |
|---|---|
| `data::scan_chart_values(xml) -> Result<ChartData, ChartScanError>` | 계열별로 값·라벨의 `(idx, 바이트 구간, 텍스트)` 목록을 만든다 |
| `patch::apply_value_edits(xml, &ChartData, &[ValueEdit]) -> Result<Vec<u8>, PatchError>` | 그 구간만 치환한 새 바이트를 만든다 |

계약은 한 줄이다 — **`&xml[point.span] == point.text.as_bytes()`**. 이게 성립하는 한
"자기 텍스트를 되쓰면 바이트가 그대로"가 따라 나온다.

## 2. 결정과 근거

### 2-1. 오프셋 추적은 `Reader::from_str` + `buffer_position()` 이다

문자열 검색이 아니다. 여는 태그 `<c:v>` 직후 위치를 시작으로 잡고, **닫는 태그를 읽기
직전의 위치**를 끝으로 잡는다. 닫는 태그의 길이를 계산하지 않으므로 접두어가
`c:` 든 `chart:` 든 같은 코드가 돈다(`prefix_is_not_hardcoded` 가 고정).

`src/parser/hwpx/header.rs:2067` 의 `paraHead` 원본 보존과 같은 관용구다. `from_reader`
가 아니라 `from_str` 이어야 `buffer_position()` 이 곧 입력 바이트 오프셋이다 — 기존
`ooxml_chart/parser.rs:63` 은 `from_reader` 라 그대로 재사용할 수 없었다.

### 2-2. `c:tx` 만 문맥 가드가 느슨하다

값 거둠 조건은 `캐시/리터럴 안 && c:pt 안` 이다. 이 가드가 `c:f`(참조 수식)와
`c:formatCode` 를 값으로 오인하는 것을 막는다 — 실제로 분산형 `c:f` 는
`Sheet1!$B$2:$A$4` 같은 문자열이라 값으로 잡히면 즉시 오염된다
(`reference_formulas_are_not_mistaken_for_values`).

`c:tx` 만 예외로 블록 안이면 곧바로 거둔다. `<c:tx><c:v>이름</c:v></c:tx>` 처럼 참조
없이 쓰는 산출기가 있어서다. 코퍼스는 전건 `strRef` 형태지만 가드를 좁힐 이유가 없다.

### 2-3. 거부하는 것 셋

| 사유 | 왜 |
|---|---|
| `EmptyValueElement` — `<c:v/>` | 텍스트 구간이 없어 제자리 치환이 불가능하다. 코퍼스 0건. 반쪽만 새 값인 파일을 내보내느니 스캔에서 드러낸다 |
| `UnsafeText` — `<`, `>`, `&`, 제어문자 | **이스케이프해서 넣지 않는다.** 최소 diff 의 전제가 "쓴 텍스트가 곧 파일의 바이트"인데, 몰래 이스케이프하면 왕복이 그 전제를 잃는다 |
| `LabelNotEditable` — 카테고리 라벨 | 구조 변경이라 B2 다. 분산형 X 는 수치라 편집 대상이다 |

### 2-4. 다층 카테고리는 표지만 남긴다

`c:multiLvlStrRef`(코퍼스 0건)는 층을 평탄화해 싣고 `labels_multi_level = true` 만
세운다. 스캐너에서 거부하지 않는 이유: 라벨을 표현하지 못하는 것과 **값을 편집하지
못하는 것은 다르다.** 값 편집은 그대로 가능해야 하고, 라벨을 행 머리로 쓰는 CSV 층
(Stage 5)이 이 표지를 보고 거부하면 된다.

### 2-5. 패처는 의미를 모른다

값이 수치인지, 행·열 수가 맞는지는 **코어 한 곳**에서 본다(계획서 §4-4, 검증기를
코어와 CLI 로 가르지 않는다). 패처가 거부하는 것은 주소 오류·중복 지목·XML 안전성뿐이다.

## 3. 판정

### 3-1. 코퍼스 56건 게이트 — 전건 green

```text
tests/issue_4100_chart_data_edit.rs   8 passed
```

| 테스트 | 무엇을 고정하나 |
|---|---|
| `scanner_agrees_with_the_model_parser_across_the_corpus` | 56건에서 계열 수·값·분산형 X 가 `OoxmlChart::parse` 와 일치. **오라클이 완전히 다른 경로(SAX 모델 빌드)라 공모하지 않는다** |
| `every_span_slices_back_to_its_own_text` | 구간이 자기 텍스트를 정확히 가리킨다 (훑은 점 200 초과) |
| `identity_patch_is_byte_identical_across_the_corpus` | **무편집 왕복이 56건 전건 바이트 동일** — 수용 기준 2 의 XML 층 |
| `a_single_edit_changes_only_that_value` | 한 값을 고치면 그 값만 바뀌고 길이 차이는 텍스트 길이 차이뿐 |
| `numeric_literal_chart_is_scanned_like_a_cached_one` | M3 파일(`특이케이스/…단일시리즈제목`) 지목 |
| `scatter_series_expose_editable_x_values_shared_across_series` | M2 — 분산형 10건(5종×2포맷)에서 계열 간 X 동일 |
| `category_labels_are_not_editable` | B1/B2 경계 |
| `patcher_rejects_bad_addresses_duplicates_and_unsafe_text` | 쓰기 전에 거부 |

단위 테스트 18건(`ooxml_chart::data` 9 + `ooxml_chart::patch` 9)도 green.

### 3-2. 지켜야 할 계약 — 전건 무수정 green

| 테스트 | 결과 |
|---|---|
| `issue_3546_chart_preserved_on_save` | 2 passed |
| `issue_3547_ole_size_prefix` | 1 passed |
| `issue_1251_ole_chart_contents` | 10 passed |
| `issue_4055_b1_chart_edit_probe` | 9 passed, 1 ignored(판정 번들) |

### 3-3. 게이트

```text
cargo fmt --check           Diff in 0건
cargo clippy --all-targets -- -D warnings    exit 0
git diff --check            통과
```

clippy 지적 1건(`manual_contains`, 테스트 단언)은 수정했다.

## 4. Stage 1 에서 확인된 것

계획서 §3 의 코퍼스 실측이 **코드로 재확인됐다.**

- 분산형 5종 × 2포맷 = 10건 전부 계열 간 X 동일 → 이슈가 제안한 `X,계열1,계열2` 한 장
  레이아웃이 코퍼스에서 성립한다. 다만 **포맷 보장이 아니므로** Stage 5 의
  `sharedXRequired` 거부는 그대로 필요하다
- `c:numLit` 문서 1건도 캐시형과 같은 경로로 잡히고 왕복이 바이트 동일이다
- `c:f` 를 값으로 오인하지 않는다 — 분산형의 비대칭 범위(`Sheet1!$B$2:$A$4`)가 값으로
  섞이면 즉시 드러났을 텐데, 56건 전건 모델 파서와 일치했다

## 5. 다음 (Stage 2)

`all_ole_streams` 승격 + `replace_ole_stream` 신설. `ole_root_clsid` 는 #4097 이
이미 `src/parser/ole_container.rs:184` 로 승격해 뒀으므로 잔량은 둘이다.
