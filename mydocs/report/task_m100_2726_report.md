# [#2726] HWPX 공용 도형 경로 `hp:sz` — 크기 기준·크기 보호 라운드트립 보존

## 이슈 / 브랜치 / 범위

| 항목 | 값 |
|---|---|
| 이슈 | [#2726](https://github.com/edwardkim/rhwp/issues/2726) |
| 브랜치 | `task/m100-2726-hwpx-common-shape-sz` (분기점 `origin/devel` = `49f38446`) |
| 변경 파일 | `src/serializer/hwpx/section.rs`, `src/parser/hwpx/section.rs` |
| 범위 밖 | `render_equation`(수식 서브시스템 별도 작업 중), `serializer/hwpx/{table,shape,picture}.rs`(#2697·#2712 소관) |

`#2697`/`#2701`(표, devel 병합 `8b77bb15`·`14e4b806`)과 `#2712`/`#2719`
(rect/line/container/picture, 열림)가 고친 것과 **같은 결함 클래스의 잔여**다.
`#2719` 가 본문에 명시적으로 잔여로 남긴 두 지점이 본 작업 대상이다.

---

## 1. 문제

### 1-1. 직렬화 — `src/serializer/hwpx/section.rs:1912` (`render_common_shape_xml`)

```rust
r#"<hp:sz width="{w}" height="{h}" widthRelTo="ABSOLUTE" heightRelTo="ABSOLUTE"/>"#,
```

- `widthRelTo` / `heightRelTo` — IR `CommonObjAttr::width_criterion` / `height_criterion`
  이 있는데도 `"ABSOLUTE"` 리터럴로 고정.
- `protect` — IR `size_protect` 가 있는데 **속성이 아예 방출되지 않음**.

이 경로를 타는 태그는 `:1776`–`:1805` 디스패치 기준 **ellipse / arc / polygon / curve /
chart** 5종이다.

### 1-2. 파싱 — `src/parser/hwpx/section.rs:5985` (`parse_common_shape_children`)

`hp:sz` arm 이 `width` / `height` / `protect` 만 읽고 `widthRelTo` / `heightRelTo` arm 이
없다. 호출자는 `:5822`(chart) · `:5906`(ole) 두 곳이므로 **차트·OLE 의 크기 기준이
파싱 단계에서 소실**된다.

---

## 2. 분석

### 2-1. 과잉 주장 배제 — 파서는 4개 태그에 대해 이미 무결하다

`ellipse` / `arc` / `polygon` / `curve` 는 `parse_common_shape_children` 을 **타지 않는다**.
이들은 `parse_shape_object`(`:3698`) → `parse_object_layout_child`(`:2901`, `hp:sz` arm
`:2909`)로 파싱되며, 그 arm 은 `widthRelTo`(`:2925`) · `heightRelTo`(`:2928`) ·
`protect`(`:2930`)를 **이미 정상적으로 읽고 있다**.

따라서 이 4개 태그에 대해서는 **파서는 손실원이 아니고 직렬화기만 손실원**이다.
"공용 자식 파서가 안 읽어서 4개 태그의 왕복이 막혀 있다"는 서술은 사실이 아니므로
이슈·본 보고서 어디에도 그렇게 적지 않았다.

파서 arm 5개 대조:

| arm | 위치 | 담당 | `widthRelTo` | `heightRelTo` | `protect` |
|---|---|---|---|---|---|
| 표 | `:1692` | `hp:tbl` | O | O | O (#2697) |
| 그림 | `:2331` | `hp:pic` | O (#2712) | O (#2712) | O (#2712) |
| 도형 공용 | `:2909` | rect/ellipse/arc/polygon/curve/line | O | O | O |
| 양식 | `:5599` | `hp:checkBtn` 등 | O | O | O |
| **공용 자식** | **`:5985`** | **chart / ole** | **X** | **X** | O |

차트만 파싱·직렬화 **양쪽**이 막혀 있어 두 수정이 각각 독립적으로 필요하다.

### 2-2. HWP5 경로가 실재함을 코드로 확인

`src/parser/control/shape.rs:339 parse_common_obj_attr`(호출 `:30`)는 **모든 HWP5 도형**에
대해 공용으로 3값을 적재한다.

- `:348` `size_protect = attr & (1 << 20) != 0`
- `:380` `width_criterion = (attr >> 15) & 0x07` → Paper/Page/Column/Para/Absolute
- `:388` `height_criterion = (attr >> 18) & 0x03` → Paper/Page/Absolute

기록측도 `document_core/converters/common_obj_attr_writer.rs:150 width_criterion_to_bits` /
`:160 height_criterion_to_bits` 로 완비돼 있다. 끊긴 고리는 HWPX 직렬화기 한 곳뿐이었다.

---

## 3. 코퍼스 실측

`samples/hwpx` 의 실제 한글 산출 `.hwpx` **60개**를 열어 모든 `section*.xml` 의 `hp:sz` 를
**부모 태그와 함께** 전수 집계했다(zip + 태그 스택 파싱). 총 **1583개**로,
`#2719` 가 보고한 수치와 정확히 일치해 방법론이 교차 검증된다.

```
샘플 파일 수: 60          hp:sz 총 개수: 1583
부모태그          sz수  protect속성  protect="1"  파일수
tbl               684        684         13        47
rect              314        314          1        21
checkBtn          182        182          0         3
pic               177        177          0        33
polygon           150        150          0         8   <- 본 작업
equation           60         60          0         2   <- 범위 밖(잔여)
container           7          7          0         6
btn                 2          2          0         2
comboBox            2          2          0         2
radioBtn            2          2          0         2
edit                2          2          0         2
ole                 1          1          1         1   <- 파싱측만 본 작업
widthRelTo  전역 분포: {'ABSOLUTE': 1583}
heightRelTo 전역 분포: {'ABSOLUTE': 1583}
```

### 3-1. 실측 — `hp:polygon` 150개의 구조 이탈

- **`hp:sz` 1583개 전부(1583/1583, 100%)가 `protect` 속성을 갖는다.** 한글은 이 속성을
  예외 없이 쓴다. 전수이므로 표본 편향이 아니다.
- `render_common_shape_xml` 은 `protect` 를 방출하지 않았다. 따라서 8개 실제 한글 파일의
  `hp:polygon` **150개**가 HWPX→HWPX 저장에서 이 속성을 **잃었다**.
- 값이 `0` 이라 *의미* 손실은 아니다. 그러나 한컴 원본 대비 **XML 구조 이탈**이며,
  한컴 자신의 1583/1583 불변식을 깨는 것이다. 이 부분은 지금 재현 가능한 실측이다.

### 3-2. 잠재 — 값 손실은 이 코퍼스로 증명되지 않는다

| 태그 | 코퍼스 인스턴스 | `protect="1"` | 비-ABSOLUTE 범주 | 판정 |
|---|---|---|---|---|
| `hp:polygon` | 150 (8파일) | 0 | 0 | **실측 구조 이탈** |
| `hp:ellipse` | **0** | — | — | 잠재 |
| `hp:arc` | **0** | — | — | 잠재 |
| `hp:curve` | **0** | — | — | 잠재 |
| `hp:chart` | **0** | — | — | 잠재 |
| `hp:ole` | 1 | 1 | 0 | 파싱측만 해당 |

- `protect="1"` 은 본 경로 담당 태그에 코퍼스 인스턴스가 **0개**다. 전체 15개는
  `hp:tbl`(13, #2697) · `hp:rect`(1) · `hp:ole`(1, #2712)로 모두 선행 PR 소관이다.
- `widthRelTo` / `heightRelTo` 는 **1583개 전부 `ABSOLUTE`** 다. 비-ABSOLUTE 표본이 없다.
- 즉 **값 손실 절반은 잠복(latent)** 이며 HWP5 입력 경로(2-2) 또는 문서 코어 편집
  경로로만 도달한다. `#2719` 가 같은 상황을 밝힌 것과 동일하게 적는다.
  **추론을 측정으로 제시하지 않았다.**

`hwpx-01.hwpx` 1개는 zip 이 아니어서 집계에서 제외됐다(스크립트가 경고 출력). 위 60은
실제로 열린 파일 수다.

---

## 4. 변경

### 4-1. `src/serializer/hwpx/section.rs`

1. `hp:sz` 포맷 리터럴을 IR 통과로 교체하고 `protect` 를 추가.
   ```rust
   r#"<hp:sz width="{w}" height="{h}" widthRelTo="{wrt}" heightRelTo="{hrt}" protect="{prot}"/>"#,
   ```
   인자: `wrt = size_criterion_str(c.width_criterion)`,
   `hrt = height_criterion_str(c.height_criterion)`,
   `prot = if c.size_protect { "1" } else { "0" }`.
2. `size_criterion_str`(5값) · `height_criterion_str`(3값 접기) 헬퍼 추가.
3. `use crate::model::shape::…` 에 `SizeCriterion` 추가.

**높이 정확-역 제약을 유지했다.** 파서가 높이를
`parse_size_criterion(_, allow_column_para = false)`(`parser/hwpx/section.rs:1860`)로 읽어
치역이 `{Paper, Page, Absolute}` 3값뿐이므로, `height_criterion_str` 는 `Column`/`Para` 를
`ABSOLUTE` 로 접는다. 접지 않으면 되읽기에서 `Absolute` 로 접혀 왕복이 비-멱등이 된다.
HWP5 측 `height_criterion_to_bits` 도 동일하게 접으므로 세 표현이 일관된다.
이 성질은 5값 전수 대조 테스트로 못 박았다.

### 4-2. `src/parser/hwpx/section.rs`

`parse_common_shape_children` 의 `hp:sz` arm 에 두 arm 추가 — 같은 파일 `:2925`/`:2928`
(도형 공용) 및 `:1702`/`:1706`(표)과 **동형**이며 새 관례를 만들지 않았다.

```rust
b"widthRelTo"  => { common.width_criterion  = parse_size_criterion(&attr_str(&attr), true); }
b"heightRelTo" => { common.height_criterion = parse_size_criterion(&attr_str(&attr), false); }
```

영향 범위는 chart(`:5822`) · ole(`:5906`). 코퍼스의 해당 `hp:sz`(chart 0개, ole 1개)가
전부 `ABSOLUTE` 이므로 **기존 표본에는 무변화**다. OLE `#1283` 계약 테스트 무회귀는
6장에서 확인했다.

### 4-3. 헬퍼 재사용 방침

`devel` 기준 `size_criterion_str` / `height_criterion_str` 는
`serializer/hwpx/table.rs:147`/`:162` 에 **private(`fn`)** 로만 존재해 `section.rs` 에서
도달할 수 없다. `#2712` 가 `shape.rs` 에 추가한 `pub(super)` 사본은 아직 `devel` 에 없다.
따라서 `section.rs` 에 **동일 의미** 사본을 두고 doc 주석에 복제 사유를 명시했으며,
공용 위치 1벌 통합은 잔여로 신고한다(8장). **두 번째 관례를 만들지 않았다.**

---

## 5. 신규 테스트 6종

| # | 테스트 | 위치 | 확인 |
|---|---|---|---|
| 1 | `issue2726_common_shape_sz_preserves_criteria_and_protect` | 직렬화 | 5개 태그 전부 `COLUMN`/`PAGE`/`protect="1"` 보존 |
| 2 | `issue2726_common_shape_sz_always_emits_protect_attribute` | 직렬화 | `size_protect=false` 여도 `protect="0"` **속성 방출** (3-1 대응) |
| 3 | `issue2726_height_criterion_never_emits_column_or_para` | 직렬화 | 높이 5값 전수 → `COLUMN`/`PARA` 절대 미방출 (정확-역 고정) |
| 4 | `issue2726_width_criterion_emits_all_five_values` | 직렬화 | 너비 5값 전수 대조 |
| 5 | `issue2726_parse_chart_preserves_size_criteria` | 파싱 | 차트 `widthRelTo="COLUMN"`/`heightRelTo="PAGE"` → IR 적재 |
| 6 | `issue2726_parse_chart_height_folds_column_and_para_to_absolute` | 파싱 | 높이 `COLUMN`/`PARA` → `Absolute` 접힘, 너비는 원문 보존 |

---

## 6. 검증

### 6-1. RED → GREEN (실제 실행, 캡처)

두 수정은 서로 독립이므로 **각각 따로 되돌려** 개별로 load-bearing 임을 증명했다.

#### RED 1 — 직렬화 수정만 되돌림 (파서 수정은 유지)

`cargo test --lib issue2726`

```
running 6 tests
test serializer::hwpx::section::tests::issue2726_width_criterion_emits_all_five_values ... ok
test parser::hwpx::section::tests::issue2726_parse_chart_preserves_size_criteria ... ok
test parser::hwpx::section::tests::issue2726_parse_chart_height_folds_column_and_para_to_absolute ... ok
test serializer::hwpx::section::tests::issue2726_height_criterion_never_emits_column_or_para ... FAILED
test serializer::hwpx::section::tests::issue2726_common_shape_sz_preserves_criteria_and_protect ... FAILED
test serializer::hwpx::section::tests::issue2726_common_shape_sz_always_emits_protect_attribute ... FAILED

---- issue2726_common_shape_sz_preserves_criteria_and_protect stdout ----
panicked at src\serializer\hwpx\section.rs:2437:13:
ellipse: 너비 기준 COLUMN 이 보존되어야 함: <hp:ellipse …><hp:sz width="4000" height="3000"
widthRelTo="ABSOLUTE" heightRelTo="ABSOLUTE"/>…

---- issue2726_common_shape_sz_always_emits_protect_attribute stdout ----
panicked at src\serializer\hwpx\section.rs:2469:9:
size_protect=false 여도 protect="0" 속성이 방출되어야 함: <hp:polygon …><hp:sz width="0"
height="0" widthRelTo="ABSOLUTE" heightRelTo="ABSOLUTE"/>…

---- issue2726_height_criterion_never_emits_column_or_para stdout ----
panicked at src\serializer\hwpx\section.rs:2506:13:
Paper → heightRelTo="PAPER" 이어야 함: <hp:polygon …><hp:sz width="0" height="0"
widthRelTo="ABSOLUTE" heightRelTo="ABSOLUTE"/>…

test result: FAILED. 3 passed; 3 failed; 0 ignored; 0 measured; 2471 filtered out
```

**RED 1 에서도 통과한 테스트 3개 (공시)**

| 테스트 | 통과 사유 |
|---|---|
| `issue2726_width_criterion_emits_all_five_values` | 헬퍼 `size_criterion_str` 를 직접 호출할 뿐 포맷 리터럴을 타지 않는다 |
| `issue2726_parse_chart_preserves_size_criteria` | 파서 테스트 — 직렬화 수정과 무관 |
| `issue2726_parse_chart_height_folds_column_and_para_to_absolute` | 파서 테스트 — 직렬화 수정과 무관 |

#### RED 2 — 파서 수정만 되돌림 (직렬화 수정은 유지)

```
running 6 tests
test serializer::hwpx::section::tests::issue2726_width_criterion_emits_all_five_values ... ok
test serializer::hwpx::section::tests::issue2726_common_shape_sz_always_emits_protect_attribute ... ok
test serializer::hwpx::section::tests::issue2726_common_shape_sz_preserves_criteria_and_protect ... ok
test serializer::hwpx::section::tests::issue2726_height_criterion_never_emits_column_or_para ... ok
test parser::hwpx::section::tests::issue2726_parse_chart_height_folds_column_and_para_to_absolute ... FAILED
test parser::hwpx::section::tests::issue2726_parse_chart_preserves_size_criteria ... FAILED

---- issue2726_parse_chart_height_folds_column_and_para_to_absolute stdout ----
panicked at src\parser\hwpx\section.rs:7575:13:
assertion `left == right` failed: 너비는 COLUMN 를 그대로 보존해야 한다
  left: Absolute
 right: Column

---- issue2726_parse_chart_preserves_size_criteria stdout ----
panicked at src\parser\hwpx\section.rs:7527:9:
assertion `left == right` failed: widthRelTo="COLUMN" 이 IR 에 적재되어야 한다
  left: Absolute
 right: Column

test result: FAILED. 4 passed; 2 failed; 0 ignored; 0 measured; 2471 filtered out
```

**RED 2 에서도 통과한 테스트 4개 (공시)** — 직렬화 테스트 3개 + 헬퍼 테스트 1개.
전부 파서 경로를 타지 않으므로 파서 수정과 독립이다. 이는 두 수정이 서로 다른
결함을 덮고 있음을 보이는 증거이기도 하다.

#### GREEN — 두 수정 모두 복원 후

```
running 6 tests
test serializer::hwpx::section::tests::issue2726_common_shape_sz_always_emits_protect_attribute ... ok
test parser::hwpx::section::tests::issue2726_parse_chart_preserves_size_criteria ... ok
test parser::hwpx::section::tests::issue2726_parse_chart_height_folds_column_and_para_to_absolute ... ok
test serializer::hwpx::section::tests::issue2726_common_shape_sz_preserves_criteria_and_protect ... ok
test serializer::hwpx::section::tests::issue2726_width_criterion_emits_all_five_values ... ok
test serializer::hwpx::section::tests::issue2726_height_criterion_never_emits_column_or_para ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 2471 filtered out
```

### 6-2. CI 3종 — 전부 통과

| 검사 | 명령 | 결과 |
|---|---|---|
| fmt | 변경 `.rs` 2개에 `rustfmt --edition 2021` 후 `git diff --name-only` | 1차 실행에서 정형화 적용됨 → 2차 실행 md5 불변(고정점 도달) |
| clippy | `cargo clippy --all-targets -- -D warnings` | **exit 0**, 경고 0건 |
| test | `cargo test --profile release-test --tests` | **exit 0** — 291개 테스트 바이너리, **3486 passed / 0 failed / 23 ignored** |

`cargo fmt --all -- --check` 는 **쓰지 않았다.** 이 Windows 체크아웃에서는 CRLF 파일에
대해 `Incorrect newline style` 만 출력하고 diff 를 내지 않아 **거짓 통과**한다.

### 6-3. 민감 테스트 무회귀 확인

파서 변경이 OLE 파싱 경로(`:5906`)를 건드리므로 `#1283` 계약 회귀를 개별 확인했다.

```
test ole_chart_contents_probe_is_stable ... ok
test ole_chart_contents_parse_result_is_stable ... ok
test ole_chart_contents_exposes_renderer_neutral_ir ... ok
test ole_chart_contents_renders_rust_svg_fragment ... ok
test ole_chart_contents_renders_standalone_rust_svg ... ok
test issue_1251_svg_uses_legacy_ole_chart_renderer ... ok
test issue_1283_hwpx_internal_ole_is_loaded_even_when_isembeded_zero ... ok
test issue_1283_hwpx_svg_uses_legacy_ole_chart_renderer ... ok
test issue_1283_hwpx_to_hwp_save_keeps_ole_as_storage ... ok

test issue_1436_picture_properties_round_trip_size_protect ... ok
test issue_1436_shape_properties_round_trip_size_protect ... ok
test parser::hwpx::section::tests::test_parse_rect_preserves_size_protect ... ok
```

시각 회귀도 무변화 — `visual_baseline_all_samples`, `visual_xfail_entries_still_fail`,
`svg_snapshot`(8종) 전부 통과.

---

## 7. 미실행 항목

- **한글(HWP) 실물 대조 없음.** 수정 결과를 한글에서 열어 "단에 맞춤"이 유지되는지
  육안 확인하지 않았다. 검증은 IR ↔ XML 계약 수준(단위 테스트)과 기존 회귀 스위트
  무변화까지다.
- **비-ABSOLUTE 크기 기준의 실물 코퍼스 확보 실패.** 3-2 에 적은 대로 표본이 없어
  값 손실은 합성 IR/XML 로만 검증했다. 실물 표본을 만들려면 한글에서 타원/다각형에
  "단에 맞춤"을 지정해 저장해야 하는데 이번 작업에서는 하지 않았다.
- **HWP5→HWPX→HWP5 왕복 실측 없음.** 2-2 는 코드 경로 확인이며, 실제 바이트 왕복
  비교는 수행하지 않았다.

---

## 8. 잔여

1. **`render_equation`** — `serializer/hwpx/section.rs:2030` 의 `hp:sz` 도
   `widthRelTo="ABSOLUTE" heightRelTo="ABSOLUTE"` 하드코딩 + `protect` 미방출로
   **완전히 동일한 결함**이다. 코퍼스 실측 `hp:equation` **60개(2파일, `protect` 60/60
   존재)** 로 3-1 과 같은 구조 이탈이 있다. 수식 서브시스템이 별도 작업 중이라
   본 작업에서 건드리지 않았다.
2. **헬퍼 3중화** — 본 수정 후 `size_criterion_str`/`height_criterion_str` 가
   `table.rs`(private) · `shape.rs`(#2712, `pub(super)`) · `section.rs`(본 PR) 3벌이 된다.
   `serializer/hwpx/mod.rs` 등 공용 위치로 1벌 통합이 필요하다. `#2712` 병합 전에는
   `devel` 에서 도달 가능한 사본이 없어 지금 통합할 수 없다.
3. **OLE 직렬화측** — `serializer/hwpx/shape.rs write_ole` 는 `#2712` 소관.
   본 PR 은 OLE 의 **파싱측**만 건드렸다.
4. **`hp:container` 등 나머지** — `#2712` 소관.
