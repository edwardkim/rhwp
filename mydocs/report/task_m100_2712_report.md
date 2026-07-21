# task_m100_2712 처리결과 보고서 — 도형·그림 `hp:sz` 크기 기준·크기 보호 라운드트립 보존

- **이슈**: [#2712](https://github.com/edwardkim/rhwp/issues/2712)
- **브랜치**: `task/m100-2712-hwpx-shape-picture-sz` (base `devel` @ `49f38446`)
- **범위**: `src/serializer/hwpx/shape.rs`, `src/serializer/hwpx/picture.rs`,
  `src/parser/hwpx/section.rs`(그림 파서 arm 1곳)
- **분류**: 결함 수정 (직렬화 하드코딩으로 인한 IR 소실) — [#2697](https://github.com/edwardkim/rhwp/issues/2697)/[#2701](https://github.com/edwardkim/rhwp/pull/2701) 이 표에서 고친 결함의 형제 요소 잔여

## 1. 문제

HWPX `hp:sz` 의 `widthRelTo` / `heightRelTo` / `protect` 세 속성이 도형·그림 직렬화기에서
리터럴로 하드코딩돼, 파서가 이미 읽어 둔 IR 값이 저장 때 통째로 버려졌다.

| 파일 | 위치(수정 전) | 하드코딩 |
|---|---|---|
| `src/serializer/hwpx/shape.rs` | `:986`, `:988`, `:989` | `"ABSOLUTE"` / `"ABSOLUTE"` / `"0"` |
| `src/serializer/hwpx/picture.rs` | `:390`, `:392`, `:393` | 동일 |

대응 IR 필드는 `src/model/shape.rs:67`(`size_protect`), `:83`(`width_criterion`),
`:85`(`height_criterion`).

`shape.rs::write_sz` 는 도형 계열 방출기 4곳이 공유한다 — `write_rect`(`:40`),
`write_line`(`:144`), `write_container_close`(`:315`), `write_ole`(`:339`).
`picture.rs::write_sz` 는 `write_picture`(`:98`) 한 곳이다.

**파서는 이미 값을 읽고 있었다**(즉 읽기는 옳고 쓰기만 버리는 비대칭):

| 개체 | 파서 경로 | 읽는 속성 |
|---|---|---|
| rect / line / container | `parse_object_layout_child`(`parser/hwpx/section.rs:2878`) | `widthRelTo` `:2901`, `heightRelTo` `:2904`, `protect` `:2907` — 3개 모두 |
| OLE / chart | `parse_common_shape_children`(`:5935`) | `protect` `:5967` **만** |
| **그림** | `parse_picture`(`:2232`) 자체 인라인 `b"sz"` arm | `width`/`height` **만** — 3개 전부 없음 |

기존 통과 테스트 `src/parser/hwpx/section.rs::test_parse_rect_preserves_size_protect` 가
`<hp:sz ... protect="1"/>` → `rect.common.size_protect == true` 를 이미 단언한다.

### 실측 근거 (한글 산출 실물 파일)

`samples/hwpx/` 의 실제 한글 산출 파일 60개에서 `hp:sz` 1583개를 전수 조사했다.

| 소유 요소 | widthRelTo | heightRelTo | protect | 개수 |
|---|---|---|---|---|
| `hp:tbl` | ABSOLUTE | ABSOLUTE | 0 | 671 |
| `hp:rect` | ABSOLUTE | ABSOLUTE | 0 | 309 |
| `hp:pic` | ABSOLUTE | ABSOLUTE | 0 | 188 |
| `hp:checkBtn` | ABSOLUTE | ABSOLUTE | 0 | 182 |
| `hp:polygon` | ABSOLUTE | ABSOLUTE | 0 | 150 |
| `hp:equation` | ABSOLUTE | ABSOLUTE | 0 | 60 |
| **`hp:tbl`** | ABSOLUTE | ABSOLUTE | **1** | **13** |
| `hp:btn`/`hp:comboBox`/`hp:radioBtn`/`hp:edit` | ABSOLUTE | ABSOLUTE | 0 | 각 2 |
| **`hp:rect`** | ABSOLUTE | ABSOLUTE | **1** | **1** |
| **`hp:ole`** | ABSOLUTE | ABSOLUTE | **1** | **1** |

`protect="1"` 이 실물에 15회 존재하고, 그중 13회는 `hp:tbl`(#2697 이 처리), **나머지 2회가 본
작업 대상**이다 — `samples/hwpx/143E433F503322BD33.hwpx` 의 `hp:rect` 1개, `hp:ole` 1개.
두 개체 모두 수정 전에는 HWPX→HWPX 저장에서 `protect="0"` 으로 되쓰여 한글이 기록한
"크기 고정"이 실제로 소실됐다.

`widthRelTo`/`heightRelTo` 는 이 코퍼스 60개 안에서는 전부 `ABSOLUTE` 였다. **범주 소실은 본
코퍼스에서 재현되지 않는 잠복 결함**이며 HWP5 입력 경로(`src/parser/control/shape.rs:348`
bit20, `:380` bit15-17, `:388` bit18-19)와 문서 코어 편집 경로에서만 관측된다. 과장하지 않고
그대로 적는다.

## 2. 분석

### 2-1. 손실 경로

| # | 입력 | 저장 | 손실 | 재현 |
|---|---|---|---|---|
| 1 | HWPX `hp:rect`/`hp:line`/`hp:container` | HWPX | `protect` 1→0 | **실측** (`143E433F503322BD33.hwpx`) |
| 2 | HWPX `hp:ole` | HWPX | `protect` 1→0 | **실측** (같은 파일) |
| 3 | HWPX 도형 | HWPX | 범주 → ABSOLUTE 강제 | 파서는 읽음. 코퍼스에 표본 없어 잠복 |
| 4 | HWP5 도형·그림 | HWPX | 범주·보호 전부 소실 | HWP5 파서가 IR 을 채우므로 방출에서 확정 소실 |
| 5 | HWPX→HWPX→HWP5 | HWP5 | 중간 저장에서 죽은 값이 attr 비트까지 전파 (`document_core/converters/common_obj_attr_writer.rs:97/100/101`) | 경로 성립 |
| 6 | 문서 코어 편집 | HWPX | UI 로 켠 "크기 고정" 이 저장 즉시 소실 | `tests/issue_1436_size_protect_properties.rs:48,105` 가 편집 명령의 IR 반영을 보장 |

6번이 **그림 방출측 결함이 파서 공백과 무관하게 실재하는 이유**다.

### 2-2. `heightRelTo` 정확한 역함수 제약

파서는 높이를 `parse_size_criterion(_, allow_column_para = false)` 로 읽는다
(`parser/hwpx/section.rs:1844` 정의, 호출부는 너비 `:2901` `true` / 높이 `:2904` `false`).
따라서 높이 치역은 `{Paper, Page, Absolute}` 3값뿐이고 **방출도 같은 3값으로 접어야 왕복이
정확한 역**이 된다. 높이로 `COLUMN`/`PARA` 를 내보내면 되읽을 때 `Absolute` 로 떨어져 왕복이
깨진다. HWP5 측 `height_criterion_to_bits`(`common_obj_attr_writer.rs:160`) 및 모델 주석
(`model/shape.rs:85`, bit 18-19)과도 일치한다. 너비는 bit 15-17 로 3비트라 5값 전부를 담는다.

### 2-3. 선례와의 관계 (중요 — 이슈 작성 시점과 달라진 사실)

이슈 #2712 를 낼 당시 [#2701](https://github.com/edwardkim/rhwp/pull/2701) 은 **open** 상태였고
`origin/devel` 에 `size_criterion_str`/`height_criterion_str` 가 없었다. 작업 중 devel 이
10커밋 전진하면서 #2701 의 내용이 devel 에 반영됐다(PR 자체는 `closed`·`merged=false` 이며
메인테이너 통합 브랜치를 통해 들어옴). 그래서 base 를 `c4e6faa3` → `49f38446` 으로 옮겨
재검증했다.

현재 devel 의 `src/serializer/hwpx/table.rs:147/162` 에 **private** `size_criterion_str` /
`height_criterion_str` 가 있다. 본 작업은 `table.rs` 를 건드리지 않는다는 제약에 따라 동일
이름·치역·접기 규칙의 헬퍼를 `shape.rs` 에 `pub(super)` 로 두고 `picture.rs` 가 재사용한다
(`table.rs` 가 이미 `shape.rs::numbering_type_str` 를 `use super::shape::` 로 가져다 쓰는 기존
패턴과 동형). 결과적으로 같은 헬퍼가 `table.rs`(private)와 `shape.rs`(pub(super))에 중복
존재한다 — **통합은 후속 과제로 남긴다**(5장).

## 3. 변경

### 3-1. `src/serializer/hwpx/shape.rs`

- `write_sz`(`:978`) — 세 속성을 IR 통과로 교체.
  ```rust
  ("widthRelTo", size_criterion_str(c.width_criterion)),
  ("heightRelTo", height_criterion_str(c.height_criterion)),
  ("protect", bool01(c.size_protect)),
  ```
- `size_criterion_str`(`:1004`, 5값) / `height_criterion_str`(`:1019`, 3값 접기) 추가.
  둘 다 `pub(super)` — `picture.rs` 재사용용.
- import 에 `SizeCriterion` 추가.

### 3-2. `src/serializer/hwpx/picture.rs`

- `write_sz`(`:384`) — 동일 교체.
- `use super::shape::{height_criterion_str, size_criterion_str};` 로 헬퍼 재사용(관례 이중화 방지).
- import 에 `SizeCriterion` 추가.

### 3-3. `src/parser/hwpx/section.rs`

- `parse_picture` 인라인 `b"sz"` arm 에 `widthRelTo` / `heightRelTo` / `protect` 3개 arm 추가
  (`:2347` 주석 기점). 도형 파서 `:2901-2907` 과 동형이며 높이는 `allow_column_para=false`.
- 이 파서 보강이 없으면 그림은 HWPX→HWPX 왕복이 계속 막혀 방출 수정이 관측되지 않는다
  (#2701 이 표에서 파서 `protect` arm 을 함께 추가한 것과 같은 구조).
- 코퍼스의 `hp:pic` 188개가 전부 `ABSOLUTE/ABSOLUTE/0` 이므로 **기존 표본에 대해 무변화**다.

변경 규모: 3파일 `269 insertions(+), 10 deletions(-)` (테스트 포함).

## 4. 검증

### 4-1. RED→GREEN (각 수정을 개별로 되돌려 실제 실행·캡처)

신규 테스트 8개 (`task2712_*`) — 도형 4, 그림 4.

#### RED 1 — `shape.rs::write_sz` 만 되돌림

```
running 8 tests
test serializer::hwpx::shape::tests::task2712_shape_sz_criteria_and_protect_emitted_from_ir ... FAILED
test serializer::hwpx::shape::tests::task2712_line_sz_criteria_and_protect_emitted_from_ir ... FAILED

---- serializer::hwpx::shape::tests::task2712_shape_sz_criteria_and_protect_emitted_from_ir stdout ----

thread '...' (15452) panicked at src\serializer\hwpx\shape.rs:1667:9:
widthRelTo 가 IR(Column)로 방출돼야 함(종전 ABSOLUTE 하드코딩): <hp:rect id="0" ... ><hp:sz width="0" widthRelTo="ABSOLUTE" height="0" heightRelTo="ABSOLUTE" protect="0"/>...</hp:rect>

---- serializer::hwpx::shape::tests::task2712_line_sz_criteria_and_protect_emitted_from_ir stdout ----

thread '...' (41096) panicked at src\serializer\hwpx\shape.rs:1694:9:
<hp:line id="0" ... ><hp:sz width="0" widthRelTo="ABSOLUTE" height="0" heightRelTo="ABSOLUTE" protect="0"/>...</hp:line>

test result: FAILED. 6 passed; 2 failed; 0 ignored; 0 measured; 2452 filtered out
```

#### RED 2 — `picture.rs::write_sz` 만 되돌림

```
running 8 tests
test serializer::hwpx::picture::tests::task2712_pic_sz_criteria_and_protect_emitted_from_ir ... FAILED
test serializer::hwpx::picture::tests::task2712_pic_sz_round_trips_through_parser ... FAILED

---- serializer::hwpx::picture::tests::task2712_pic_sz_criteria_and_protect_emitted_from_ir stdout ----

thread '...' (21096) panicked at src\serializer\hwpx\picture.rs:931:9:
widthRelTo 가 IR(Column)로 방출돼야 함(종전 ABSOLUTE 하드코딩): <hp:pic id="0" ... ><hp:sz width="1000" widthRelTo="ABSOLUTE" height="500" heightRelTo="ABSOLUTE" protect="0"/>...</hp:pic>

---- serializer::hwpx::picture::tests::task2712_pic_sz_round_trips_through_parser stdout ----

thread '...' (34884) panicked at src\serializer\hwpx\picture.rs:959:9:
assertion `left == right` failed: 그림 widthRelTo 가 IR 로 되읽혀야 함
  left: Absolute
 right: Para

test result: FAILED. 6 passed; 2 failed; 0 ignored; 0 measured; 2452 filtered out
```

#### RED 3 — `parse_picture` 의 `hp:sz` arm 3개만 되돌림

```
running 8 tests
test serializer::hwpx::picture::tests::task2712_pic_sz_round_trips_through_parser ... FAILED

---- serializer::hwpx::picture::tests::task2712_pic_sz_round_trips_through_parser stdout ----

thread '...' (12704) panicked at src\serializer\hwpx\picture.rs:959:9:
assertion `left == right` failed: 그림 widthRelTo 가 IR 로 되읽혀야 함
  left: Absolute
 right: Para

test result: FAILED. 7 passed; 1 failed; 0 ignored; 0 measured; 2452 filtered out
```

#### GREEN — 세 수정 모두 복구

```
running 8 tests
test serializer::hwpx::picture::tests::task2712_pic_sz_criteria_and_protect_emitted_from_ir ... ok
test serializer::hwpx::picture::tests::task2712_pic_sz_defaults_unchanged ... ok
test serializer::hwpx::shape::tests::task2712_line_sz_criteria_and_protect_emitted_from_ir ... ok
test serializer::hwpx::shape::tests::task2712_shape_sz_defaults_unchanged ... ok
test serializer::hwpx::shape::tests::task2712_height_criterion_is_exact_inverse_of_parser ... ok
test serializer::hwpx::picture::tests::task2712_pic_sz_round_trips_through_parser ... ok
test serializer::hwpx::shape::tests::task2712_shape_sz_criteria_and_protect_emitted_from_ir ... ok
test serializer::hwpx::picture::tests::task2712_pic_height_criterion_is_exact_inverse ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 2452 filtered out
```

**정직한 부기**: `task2712_height_criterion_is_exact_inverse_of_parser` 와
`task2712_*_defaults_unchanged` 는 RED 1/2 에서도 통과한다. 전자는 헬퍼 함수의 접기 규칙을
직접 검사하는 성격이고(하드코딩 상태에서도 `ABSOLUTE` 기대와 우연히 일치), 후자는 기본값
무변화를 지키는 회귀 방지용이라 의도적으로 양쪽에서 통과해야 한다. 결함을 실제로 잡아내는
RED 테스트는 도형 2개·그림 2개다.

### 4-2. CI 3종 (base `49f38446` 리베이스 후 전부 재실행)

| 검사 | 결과 |
|---|---|
| `cargo fmt --all -- --check` | **통과** — `Diff in` 항목 0건 (이 Windows 체크아웃 특유의 "Incorrect newline style" 잡음은 손대지 않은 파일 전반의 기존 현상이라 제외) |
| `cargo clippy --all-targets -- -D warnings` | **통과** — exit 0, 경고 0 |
| `cargo test --profile release-test --tests` | **통과** — exit 0, 테스트 바이너리 291개, **3488 passed / 0 failed / 23 ignored** |

`--lib` 만으로는 `tests/` 전체가 건너뛰어지므로 `--tests` 로 통합 테스트를 포함해 돌렸다.
회귀 관심 지점이었던 `tests/issue_1251_ole_chart_contents.rs`(OLE `size_protect`,
attr `0x143A_2610`), `tests/issue_1436_size_protect_properties.rs`,
`tests/hwpx_roundtrip_baseline.rs`, `tests/hwpx_roundtrip_integration.rs` 모두 통과했다.

## 5. 미실행 항목 / 잔여

### 미실행

- **한글 프로그램 실물 열기 검증 미실시.** 방출 XML 이 스키마상 유효하고 파서 왕복이 정확한
  역임은 확인했으나, 한글에서 직접 열어 "단에 맞춤" 개체가 실제로 리플로하는지는 확인하지
  못했다. 근거는 실물 파일 정적 분석(1장 전수 조사)과 파서 대칭성에 한정된다.
- **비-ABSOLUTE 범주 실물 표본 확보 실패.** `samples/hwpx/` 60개 안에 `COLUMN`/`PAGE`/`PARA`
  표본이 없어 범주 보존은 합성 IR 테스트로만 검증했다. `protect` 는 실물 표본 2건으로 검증됐다.

### 잔여 (본 작업 범위 밖)

1. **`src/serializer/hwpx/section.rs:1912`** — `render_common_shape_xml`(`:1854`) 문자열 템플릿이
   `widthRelTo`/`heightRelTo` 를 하드코딩하고 `protect` 는 아예 방출하지 않는다. 타원·호·다각형·
   곡선 경로(코퍼스 `hp:polygon` 150개). 동일 결함 계열.
2. **`src/serializer/hwpx/section.rs:2030`** — `render_equation`(`:1995`) 도 같은 하드코딩,
   `protect` 미방출(코퍼스 `hp:equation` 60개).
3. **OLE/차트 범주 파싱 공백** — `parse_common_shape_children`(`:5935`)은 `hp:sz` 에서
   `width`/`height`/`protect` 만 읽고 `widthRelTo`/`heightRelTo` 를 읽지 않는다. 방출을 고쳐도
   OLE·차트는 HWPX→HWPX 로 범주가 살아나지 않는다(HWP5 입력 경로로는 살아난다).
4. **헬퍼 통합** — `table.rs:147/162`(private)와 `shape.rs:1004/1019`(pub(super))에 동일 헬퍼가
   중복 존재한다. `table.rs` 를 건드리지 않는다는 제약 때문에 이번엔 합치지 않았다. 공용 위치
   (예: `serializer/hwpx/utils.rs`)로 올리고 양쪽이 참조하도록 정리하는 후속을 권한다.
