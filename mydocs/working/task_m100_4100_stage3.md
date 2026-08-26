---
kind: working
status: done
canonical: mydocs/working/task_m100_4100_stage3.md
last_verified: 2026-08-11
---

# #4100 Stage 3 — 주소 → ①② 슬롯 해석 + `get_chart_data_native`

- **계획서**: [`mydocs/plans/task_m100_4100.md`](../plans/task_m100_4100.md)
- **기준 커밋**: `devel = dd9ecdc4b`
- **산출**: `src/document_core/queries/chart_extract.rs`(신규) ·
  `src/document_core/commands/object_ops/chart.rs`(신규) ·
  `src/renderer/layout/utils.rs`(`find_bin_data_index` 분리) ·
  `tests/issue_4100_chart_data_edit.rs`(Stage 3 테스트 6건)

## 1. 결정과 근거

### 1-1. 인덱스↔id 이중성은 한 함수에만 둔다 (계획서 R2)

`bin_data_id` 는 보통 **1-based 인덱스**이고 60000+N 만 진짜 id 다. 이 규칙이 읽기와 쓰기로
갈리면 **서로 다른 슬롯을 가리키게 된다.** 그래서 규칙을 복제하지 않고
`find_bin_data_index` 를 새로 두고 기존 `find_bin_data` 가 그것에 위임하게 바꿨다 —
렌더러가 쓰던 조회와 편집이 쓰는 조회가 같은 함수 하나에서 나온다.

편집이 인덱스를 필요로 하는 이유는 슬롯 바이트를 **제자리에서** 바꾸기 때문이다.

### 1-2. 편집은 컨트롤을 건드리지 않는다 — 그래서 컨테이너 안 차트도 된다

바꾸는 것은 `bin_data_content` 슬롯의 바이트뿐이다. 컨트롤을 mutable 로 잡을 필요가 없으므로
**열거만 되면 글상자·표 셀·머리말 안의 차트도 똑같이 편집된다.**

그래서 `collect_charts` 는 `table_extract` 와 같은 모양으로 컨테이너를 재귀한다
(글상자·표 셀·머리말·꼬리말·각주·미주, 깊이 상한 8).

주소 축은 둘이다.

| 축 | 닿는 범위 | 쓰는 곳 |
|---|---|---|
| `(section, paragraph, control)` 3인자 | **본문 직속만** | 코어 API (그림 API 와 동형) |
| 문서 순번 0-based | 컨테이너 안까지 전부 | `--chart N` (Stage 5) |

3인자가 컨테이너 안을 표현하지 못하는 것은 그림 API 와 같은 한계다. 숨기지 않고
`ChartRef::is_top_level()` 로 드러내고, 순번 경로(`get_chart_data_by_index_native`)를 함께 낸다.

### 1-3. 차트 판별은 이름이 아니라 내용으로 한다

HWP5 의 `OleShape` 는 수식·글맵시 등 무엇이든 될 수 있다. ①은 슬롯의
`extension == "ooxml_chart"` 로, ②는 중첩 CFB 안 `OOXMLChartContents` 유무로 본다 —
렌더러가 쓰는 판별과 같다. 둘 중 하나도 차트로 해소되지 않으면 열거에 담지 않는다.

### 1-4. 값은 원본 텍스트 그대로 싣는다

`get_chart_data_native` 는 값을 `4.3` 이라는 **문자열**로 돌려준다. 실수로 파싱했다가 되쓰면
표기가 달라져(`4.3` → `4.30`) 무편집 왕복의 바이트 동일이 깨진다.

### 1-5. 주소 오류만 `Err`

데이터 문제(스캔 실패·두 표현 모두 비어 있음)는 `Ok` + `{"ok":false,"invalid":[…]}` 다.
`invalid[]` 항목 모양은 CLI 관례(`reason` + `message`)에 맞췄다 — CLI 가 이 봉투를 그대로
실어 나르면 검증기가 한 곳에 남는다(계획서 R7).

## 2. 판정

### 2-1. 코퍼스 — 전건 green

| 테스트 | 무엇을 고정하나 |
|---|---|
| `every_corpus_document_resolves_its_chart_slots` | 56건에서 차트가 정확히 하나 열거되고, **HWPX 는 ①②, HWP5 는 ②만** 해소된다 (28/28 로 갈림) |
| `both_representations_carry_the_same_xml` | ①==② 를 28건에서 바이트로 확인 — #4055 의 SHA-256 전건 일치를 코드로 고정 |
| `get_chart_data_native_matches_the_model_parser` | 56건에서 봉투의 값이 모델 파서와 일치. `representations`·`source` 가 포맷대로 |
| `values_keep_their_original_spelling` | 값 표기 보존 (§1-4) |
| `only_address_errors_are_err` | 없는 구역·문단·컨트롤, 차트 아닌 컨트롤 → `Err` |
| `index_addressing_covers_the_same_chart` | 주소 경로와 순번 경로가 같은 결과 |

`chart_extract` 단위 1건 — 인덱스→id 폴백 규칙(R2)을 표로 고정.

### 2-2. 게이트

```text
cargo fmt --check                            Diff in 0건
cargo clippy --all-targets -- -D warnings    exit 0
issue_4100_chart_data_edit                   18 passed
```

## 3. 설계 재검토에서 고친 것 (Stage 3 마감 전)

Stage 3 을 끝내고 설계를 냉정하게 다시 본 결과 **판정의 자신감이 실제 커버리지보다 컸다.**
두 건은 코드로 고쳤고, 두 건은 결정으로 못박았다.

### 3-1. 오라클이 결함을 공유하고 있었다 (계획서 R9)

스캐너는 접두어를 고정하지 않으려고 `local_name` 만 본다. 그 성질이 `c:` ↔ `chart:` 에는
필요하지만 **외래 네임스페이스에서는 위험**이다 — `c:extLst` 는 표준 확장 지점이라
`c15:filteredCategoryTitle` 처럼 `c:cat`·`c:pt`·`c:v` 를 통째로 품는 확장이 들어올 수 있다.

더 나쁜 것은 오라클로 세운 `OoxmlChart::parse` **도 `local_name` 만 본다**는 것이다
(`ooxml_chart/parser.rs`). 이 축에서는 둘이 같이 틀리므로 "코퍼스 56건 전건 일치"가 아무것도
보장하지 않았다. 게다가 실측하니 코퍼스는 `c:ser` 안 `extLst` 가 **0건**이라 그 경로를 한 번도
밟지 않았다.

→ `extLst`·`dLbls`·`dLbl`·`trendline`·`errBars` 서브트리를 **통째로 건너뛴다.** 네임스페이스
URI 판정보다 싸고 확실하며 B1 은 그 안의 무엇도 읽거나 쓰지 않는다. 코퍼스가 못 밟는
경로이므로 합성 테스트로만 덮인다 — `extension_subtrees_are_not_scanned_as_values` ·
`data_label_text_does_not_become_the_series_name`.

### 3-2. 결측치가 문서 전체 읽기를 막고 있었다 (계획서 R10)

`<c:v/>` 를 만나면 스캔이 통째로 `Err` 였다. 보수적 선택 자체는 맞지만 **범위가 틀렸다** —
거부는 그 점의 *쓰기*에 걸려야지 문서 전체의 *읽기*에 걸리면 안 된다. 결측은 실데이터에서
흔하고, 값 하나 때문에 `chart-to-csv` 가 통째로 실패하면 못 쓴다.

→ `ChartPoint::span` 을 `Option` 으로 바꾸고 `PatchError::ValueNotPatchable` 로 **그 점의
편집만** 거부한다. `ChartScanError::EmptyValueElement` 는 없앴다.

### 3-3. 열거 범위를 넓히지 않고 못박았다

`collect_charts` 는 `SectionDef.master_pages`·`Field.memo_paragraphs`·캡션을 빠뜨린다.
#4099 보고서 §7-2 가 *"한 워커가 컨테이너 하나를 놓쳤다"* 를 이 저장소의 반복 결함으로 적어
뒀고, 이 작업이 그 패턴을 재생산한 것이다.

검증할 합성 문서 없이 재귀 가지를 늘리는 것이 바로 그 결함을 만드는 방식이므로 **가지를
늘리는 대신 범위를 계획서에 명시**했다(§4-2). 워커 통합은 #4099 가 이미 후속으로 지목한
별개 작업이다.

### 3-4. 주소 축의 정본을 정했다

문서 순번이 정본, 3인자는 본문 직속 편의 경로다(계획서 §4-2). 두 축을 두고 결론을 미룬
상태를 남기지 않는다.

## 4. 다음 (Stage 4)

`set_chart_data_native` — ①② 동시 기록. 여기서 계획서 R1(`bin_data_epoch` 불변식을 이
작업이 처음 깬다)이 실제로 걸린다. 슬롯 바이트를 제자리에서 바꾸면 렌더 캐시가 옛 차트를
주므로 `bump_bin_data_epoch()` 를 부르고 그 주석의 목록에 이 경우를 추가해야 한다.
