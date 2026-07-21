# task_m100_2697 처리결과 보고서 — `<hp:tbl>` 크기·번호 범주 속성 라운드트립

- **이슈**: [#2697](https://github.com/edwardkim/rhwp/issues/2697)
- **브랜치**: `task/m100-2697-hwpx-table-attr-roundtrip` (base `devel` @ `2cd4d78b`)
- **범위**: `src/parser/hwpx/section.rs`, `src/serializer/hwpx/table.rs`
- **분류**: 결함 수정 (직렬화 하드코딩 + 파서 arm 누락)

## 1. 문제

`<hp:tbl>` 직렬화기가 표 크기·번호 범주 관련 속성 3종 4개를 IR 대신 리터럴로 방출한다.

> 1·2절의 `file:line` 인용은 모두 **수정 전** 기준(base `2cd4d78b`)이다. 3절 이후는 수정 후 기준.

| # | 속성 | 종전 방출 | 파서 | IR 필드 |
|---|------|-----------|------|---------|
| 1 | `hp:sz@widthRelTo` | `"ABSOLUTE"` | 읽음 (`section.rs:1689`) | `Table.common.width_criterion` |
| 1 | `hp:sz@heightRelTo` | `"ABSOLUTE"` | 읽음 (`section.rs:1693`) | `Table.common.height_criterion` |
| 2 | `hp:sz@protect` | `"0"` | **arm 부재** | `Table.common.size_protect` |
| 3 | `hp:tbl@numberingType` | `"TABLE"` | **arm 부재** | `Table.common.numbering_type` |

사용자 관점 증상:

- 단/쪽/문단에 맞춘 표가 저장 한 번에 절대값으로 바뀌어, 이후 단·여백·용지를 바꿔도
  따라오지 않는다(레이아웃 회귀가 저장 시점이 아니라 다음 편집 때 드러난다).
- "표 크기 보호"가 왕복에서 풀린다.
- 표에 그림 번호 캡션(`numberingType="PICTURE"`)을 붙인 문서에서 캡션 자동 번호가
  저장할 때마다 재배치된다.

## 2. 분석

### 2-1. IR-불가시 문제가 아님

세 값 모두 IR 안에서 이미 소비된다. 따라서 "IR 에 없어서 못 낸다"가 아니라
**HWPX 로 되쓸 때만 끊긴다**.

- `width_criterion`/`height_criterion` → `pack_hwpx_common_obj_attr`
  (`section.rs:1918-1928`) 가 attr bit 15-17 / 18-19 로 패킹하고, HWP5 저장 경로도
  동일하게 패킹한다(`document_core/converters/common_obj_attr_writer.rs:100-101`).
  기존 테스트 `section.rs` `assert_eq!(table.common.attr, 0x082a_2211)` 가 이 비트를
  이미 고정하고 있다.
- `size_protect` → HWP5 파서는 `parser/control/shape.rs:348` 에서 attr bit 20 을 정상
  파싱한다. 그런데 HWPX 로 저장하면 `protect="0"` 로 나가고, 다시 HWP5 로 저장할 때
  bit 20 은 IR 에서 재유도되므로(`common_obj_attr_writer.rs:97`)
  **HWP → HWPX → HWP 왕복에서 "표 크기 보호"가 영구 소실**된다.
- `numbering_type` → 필드 주석 자체가 `#1379` 의 라운드트립 보존 목적이라고 밝히고 있다
  (`model/shape.rs:88-92`).

### 2-2. 대칭성 — 설계 결정이 아니라 누락

같은 `hp:sz` 요소를 다루는 코드끼리 비교하면 표만 빠져 있다.

파서: 도형/그림 공통(`section.rs:2901/2904/2907`), 사각형 계열(`:5967`),
양식 개체(`:5582/5586/5590`)는 `widthRelTo`/`heightRelTo`/`protect` 를 모두 읽는다.
표(`:1689/1693`)만 `protect` arm 이 없다.

직렬화: **`serializer/hwpx/form.rs:178-188` 이 결정적 반례** — 완전히 같은 `hp:sz`
요소를 세 속성 모두 보존값으로 방출한다. 즉 "HWPX `hp:sz` 는 ABSOLUTE 고정" 같은 설계
결정은 이 저장소에 존재하지 않는다.

`numberingType`: 도형 파서(`section.rs:2855-2861`, `[Task #1379]` 주석), 차트(`:5764`),
사각형(`:5839`) 이 읽고, 도형 직렬화기 4곳(`shape.rs:70,172,291,367`)이
`numbering_type_str()` 로 방출한다. 표만 계약 밖이다.

같은 파일 내부에서도 `write_pos` 는 `holdAnchorAndSO`(#1594), `flowWithText`(#1637),
`allowOverlap` 순으로 이미 "리터럴 → IR" 정리가 끝났는데, 바로 위 `write_sz` 만 손이
닿지 않았다.

### 2-3. 정확한 역함수 제약 (heightRelTo)

`heightRelTo` 는 파서에서 `parse_size_criterion(_, allow_column_para = false)` 로 읽힌다
(`section.rs:1693`). 파서 치역이 `{Paper, Page, Absolute}` 3값뿐이므로, 방출측도 같은
3값만 내야 왕복이 정확한 역이 된다. height 에 너비용 5값 매핑을 그대로 쓰면
`Column`/`Para` 가 담긴 IR 이 `heightRelTo="COLUMN"` 을 방출하고, 재파싱 시 `Absolute` 로
접혀 **왕복이 비가역**이 된다. HWP5 측 `height_criterion_to_bits`
(`common_obj_attr_writer.rs:160-167`, `_ => 2`) 도 동일하게 접으므로, 제안 매핑은 기존
규약과 일치한다.

## 3. 변경

### `src/serializer/hwpx/table.rs`

- `numberingType` → `numbering_type_str(table.common.numbering_type)`
  (`shape.rs:1047` 의 기존 공용 헬퍼 재사용, 새 매핑 도입 없음).
- `size_criterion_str(SizeCriterion) -> &'static str` 신설 — 너비용 5값 전체 매핑,
  `parse_size_criterion(_, true)` 의 정확한 역.
- `height_criterion_str(SizeCriterion) -> &'static str` 신설 — `Column`/`Para` 를
  `"ABSOLUTE"` 로 접는 3값 매핑(2-3 제약).
- `write_sz` 가 `c.width_criterion` / `c.height_criterion` / `c.size_protect` 를 사용.

### `src/parser/hwpx/section.rs`

- `parse_table` 의 `b"sz"` arm 에 `b"protect" => table.common.size_protect = parse_bool(&attr)`
  추가 (형제 파서 `:2907` 과 동일 코드).
- `parse_table` 루트 속성 루프에 `numberingType` arm 추가 (도형 파서 `:2855-2861` 과 동형).
  속성 부재 시 기본값은 `ObjectNumberingType::Table` — 표의 자연 기본값이자 종전 방출
  리터럴과 같으므로 기존 문서의 동작이 바뀌지 않는다.
- `materialize_hwpx_table_attrs` 의 `HWPX_TABLE_NUMBERING_BIT` 무조건 OR 을
  `numbering_type == Table` 조건부로 변경. 종전 동작은 `numberingType="PICTURE"` 표에서
  IR 모순(`numbering_type = Picture` ↔ attr 은 TABLE 번호)을 만든다. 차트 파서
  (`:5800-5801`)가 PICTURE 를 별도 비트로 분기하는 것과 같은 취지다.

기능 변경은 위 6개뿐이며, 리팩터링·포맷 변경은 하지 않았다.

## 4. 검증

### 4-1. 신규 테스트 (`src/serializer/hwpx/table.rs` 기존 `#[cfg(test)] mod tests`)

| 테스트 | 내용 |
|--------|------|
| `task2697_sz_criteria_and_protect_emitted_from_ir` | IR `Column`/`Paper`/`true` → `widthRelTo="COLUMN"`, `heightRelTo="PAPER"`, `protect="1"` |
| `task2697_sz_defaults_unchanged` | 기본 IR 은 종전 출력(`ABSOLUTE`/`ABSOLUTE`/`0`)과 동일 — 무변화 보장 |
| `task2697_numbering_type_emitted_from_ir` | IR `Picture` → `numberingType="PICTURE"`, `Table` → `"TABLE"` |
| `task2697_tbl_attrs_survive_xml_ir_xml_roundtrip` | XML → IR → XML 전 구간. IR 4값 단언 + attr 번호 비트 단언 + 재방출 4속성 단언 |
| `task2697_height_criterion_str_is_exact_inverse_of_parser` | height 가 `COLUMN`/`PARA` 를 절대 내지 않음을 고정 (`page_break_str_is_exact_inverse_of_parser` 와 동형) |

### 4-2. red→green 실증 (3건 각각 개별 되돌림 후 실제 실행)

#### 결함 1 — `widthRelTo`/`heightRelTo` 되돌림 (`table.rs` `write_sz` 를 `"ABSOLUTE"` 리터럴로)

```
running 5 tests
test serializer::hwpx::table::tests::task2697_height_criterion_str_is_exact_inverse_of_parser ... ok
test serializer::hwpx::table::tests::task2697_sz_defaults_unchanged ... ok
test serializer::hwpx::table::tests::task2697_numbering_type_emitted_from_ir ... ok
test serializer::hwpx::table::tests::task2697_sz_criteria_and_protect_emitted_from_ir ... FAILED
test serializer::hwpx::table::tests::task2697_tbl_attrs_survive_xml_ir_xml_roundtrip ... FAILED

---- serializer::hwpx::table::tests::task2697_sz_criteria_and_protect_emitted_from_ir stdout ----

thread 'serializer::hwpx::table::tests::task2697_sz_criteria_and_protect_emitted_from_ir' (19452) panicked at src\serializer\hwpx\table.rs:1212:9:
widthRelTo 가 IR(Column)로 방출돼야 함(현재 ABSOLUTE 하드코딩): <hp:tbl id="0" zOrder="0" numberingType="NONE" textWrap="SQUARE" textFlow="BOTH_SIDES" lock="0" dropcapstyle="None" pageBreak="NONE" repeatHeader="0" rowCnt="1" colCnt="1" cellSpacing="0" borderFillIDRef="0" noAdjust="0"><hp:sz width="0" widthRelTo="ABSOLUTE" height="0" heightRelTo="ABSOLUTE" protect="1"/><hp:pos treatAsChar="0" affectLSpacing="0" flowWithText="0" allowOverlap="0" holdAnchorAndSO=

---- serializer::hwpx::table::tests::task2697_tbl_attrs_survive_xml_ir_xml_roundtrip stdout ----

thread 'serializer::hwpx::table::tests::task2697_tbl_attrs_survive_xml_ir_xml_roundtrip' (18064) panicked at src\serializer\hwpx\table.rs:1334:13:
widthRelTo="COLUMN" 유실: <hp:tbl id="0" zOrder="0" numberingType="PICTURE" ... <hp:sz width="42520" widthRelTo="ABSOLUTE" height="10000" heightRelTo="ABSOLUTE" protect="1"/> ...

test result: FAILED. 3 passed; 2 failed; 0 ignored; 0 measured; 2452 filtered out; finished in 0.01s
```

#### 결함 2 — `protect` 되돌림 (파서 arm 제거 + `table.rs` `("protect", "0")`)

```
running 5 tests
test serializer::hwpx::table::tests::task2697_height_criterion_str_is_exact_inverse_of_parser ... ok
test serializer::hwpx::table::tests::task2697_sz_defaults_unchanged ... ok
test serializer::hwpx::table::tests::task2697_numbering_type_emitted_from_ir ... ok
test serializer::hwpx::table::tests::task2697_sz_criteria_and_protect_emitted_from_ir ... FAILED
test serializer::hwpx::table::tests::task2697_tbl_attrs_survive_xml_ir_xml_roundtrip ... FAILED

---- serializer::hwpx::table::tests::task2697_sz_criteria_and_protect_emitted_from_ir stdout ----

thread 'serializer::hwpx::table::tests::task2697_sz_criteria_and_protect_emitted_from_ir' (33528) panicked at src\serializer\hwpx\table.rs:1222:9:
protect 가 IR(size_protect=true)로 방출돼야 함(현재 0 하드코딩): <hp:tbl id="0" zOrder="0" numberingType="NONE" textWrap="SQUARE" textFlow="BOTH_SIDES" lock="0" dropcapstyle="None" pageBreak="NONE" repeatHeader="0" rowCnt="1" colCnt="1" cellSpacing="0" borderFillIDRef="0" noAdjust="0"><hp:sz width="0" widthRelTo="COLUMN" height="0" heightRelTo="PAPER" protect="0"/><hp:pos treatAsChar="0" affectLSpacing="0" flowWithText="0" allowOverlap="0" holdAnchorAndSO="0" v

---- serializer::hwpx::table::tests::task2697_tbl_attrs_survive_xml_ir_xml_roundtrip stdout ----

thread 'serializer::hwpx::table::tests::task2697_tbl_attrs_survive_xml_ir_xml_roundtrip' (32656) panicked at src\serializer\hwpx\table.rs:1309:9:
protect=1 이 IR(size_protect)에 남아야 함(파서 arm 누락)

test result: FAILED. 3 passed; 2 failed; 0 ignored; 0 measured; 2452 filtered out; finished in 0.00s
```

#### 결함 3 — `numberingType` 되돌림 (파서 arm + attr 비트 게이트 + `table.rs` `"TABLE"` 리터럴)

```
running 5 tests
test serializer::hwpx::table::tests::task2697_height_criterion_str_is_exact_inverse_of_parser ... ok
test serializer::hwpx::table::tests::task2697_sz_criteria_and_protect_emitted_from_ir ... ok
test serializer::hwpx::table::tests::task2697_sz_defaults_unchanged ... ok
test serializer::hwpx::table::tests::task2697_numbering_type_emitted_from_ir ... FAILED
test serializer::hwpx::table::tests::task2697_tbl_attrs_survive_xml_ir_xml_roundtrip ... FAILED

---- serializer::hwpx::table::tests::task2697_numbering_type_emitted_from_ir stdout ----

thread 'serializer::hwpx::table::tests::task2697_numbering_type_emitted_from_ir' (12460) panicked at src\serializer\hwpx\table.rs:1250:9:
numberingType 이 IR(Picture)로 방출돼야 함(현재 TABLE 하드코딩): <hp:tbl id="0" zOrder="0" numberingType="TABLE" textWrap="SQUARE" textFlow="BOTH_SIDES" lock="0" dropcapstyle="None" pageBreak="NONE" repeatHeader="0" rowCnt="1" colCnt="1" cellSpacing="0" borderFillIDRef="0" noAdjust="0"><hp:sz width="0" widthRelTo="ABSOLUTE" height="0" heightRelTo="ABSOLUTE" protect="0"/><hp:pos treatAsChar="0" affectLSpacing="0" flowWithText="0" allowOverlap="0" holdAnchorAndSO

---- serializer::hwpx::table::tests::task2697_tbl_attrs_survive_xml_ir_xml_roundtrip stdout ----

thread 'serializer::hwpx::table::tests::task2697_tbl_attrs_survive_xml_ir_xml_roundtrip' (13608) panicked at src\serializer\hwpx\table.rs:1313:9:
assertion `left == right` failed: numberingType=PICTURE 가 IR 에 남아야 함(파서 arm 누락)
  left: Table
 right: Picture

test result: FAILED. 3 passed; 2 failed; 0 ignored; 0 measured; 2452 filtered out; finished in 0.00s
```

#### 복원 후 (GREEN)

```
running 5 tests
test serializer::hwpx::table::tests::task2697_height_criterion_str_is_exact_inverse_of_parser ... ok
test serializer::hwpx::table::tests::task2697_sz_defaults_unchanged ... ok
test serializer::hwpx::table::tests::task2697_sz_criteria_and_protect_emitted_from_ir ... ok
test serializer::hwpx::table::tests::task2697_tbl_attrs_survive_xml_ir_xml_roundtrip ... ok
test serializer::hwpx::table::tests::task2697_numbering_type_emitted_from_ir ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 2452 filtered out; finished in 0.01s
```

### 4-3. 회귀

```
cargo test --lib hwpx   →  476 passed / 0 failed / 0 ignored  (1981 filtered out)
cargo test --lib table  →  330 passed / 0 failed / 3 ignored  (2124 filtered out)
cargo test --lib        → 2450 passed / 0 failed / 7 ignored  (0 filtered out, 186.86s)
```

기존 `test_parse_hwpx_table_materializes_hwp_common_attrs`
(`assert_eq!(table.common.attr, 0x082a_2211)`) 가 그대로 통과한다 — `numberingType="TABLE"`
표에서 `HWPX_TABLE_NUMBERING_BIT` 가 유지됨을 기존 테스트가 직접 보증한다.

### 4-4. 미실행 항목 (투명 고지)

- **PR CI 전체 검증**(`cargo test --verbose`, `cargo clippy -- -D warnings`): 저장소 규약
  (`mydocs/manual/codex/docs_and_git_workflow.md`)상 작업지시자 별도 승인 사항이라
  실행하지 않았다.
- **한컴 오피스 실물 대조**: `widthRelTo="COLUMN"` 표를 한글에서 열어 "단에 맞춤"으로
  표시되는지의 시각 검증은 수행하지 않았다. 파서 치역·HWP5 attr 비트 매핑·형제 개체
  구현이 모두 일치함을 코드로 대조 확인하는 데 그쳤다.
- **통합 테스트(`tests/`) 추가 없음**: 결함이 파서/직렬화기 단위에 한정되고 기존 표
  테스트 하네스(`serializer/hwpx/table.rs`)로 XML→IR→XML 전 구간을 덮을 수 있어
  단위 테스트로 마무리했다.

## 5. 잔여 (범위 밖)

1. **`hp:cellzone` 확장형 미인식** — `parse_table` 이 `cellzone` 을 `Event::Empty` arm
   에서만 처리한다. `<hp:cellzone ...></hp:cellzone>` 처럼 펼쳐 쓴 형태는 `Event::Start`
   arm(`tr`/`tc`/`caption` 만 처리)에 걸리지 않아 셀 영역 테두리/배경이 유실된다.
   직렬화기는 항상 self-closing 으로 내므로 rhwp 자신이 만든 파일에서는 드러나지 않고,
   다른 생성기가 만든 HWPX 에서만 발생한다.
2. **IR 필드가 없는 하드코딩 속성** — `hp:tbl@lock`, `hp:tbl@dropcapstyle`,
   `hp:pos@affectLSpacing`. 담을 IR 필드가 없어 모델 확장이 선행돼야 한다.
3. **도형/그림 `write_sz` 의 동일 결함** — `shape.rs:986-988`, `picture.rs:390-392` 도
   `widthRelTo`/`heightRelTo` 를 `"ABSOLUTE"` 로 하드코딩한다. 파서는 읽으므로 표와 똑같은
   손실이 있다. 변경 범위를 표 경로로 좁히기 위해 분리했다.
4. **HWPX→HWP5 어댑터의 무조건 번호 비트** —
   `document_core/converters/hwpx_to_hwp.rs:1619-1631` 이 `HWPX_TABLE_NUMBERING_BIT` 를
   무조건 OR 한다. 파서 쪽은 본 수정으로 정리됐지만 HWP5 저장 경로는 여전히 TABLE 로
   강제한다. 어댑터는 본 수정의 대상 파일 밖이라 분리했다.
