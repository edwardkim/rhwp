# task_m100_2734 처리결과 보고서 — PARA_SHAPE 말미 4바이트(개요 수준) 복원

- **이슈**: [#2734](https://github.com/edwardkim/rhwp/issues/2734)
- **브랜치**: `task/m100-2734-docinfo-parashape-outline-level` (base `origin/devel` @ `1658d0bb`)
- **범위**: `src/parser/doc_info.rs`, `src/serializer/doc_info.rs`, `src/serializer/doc_info/tests.rs`
- **분류**: 결함 수정 (HWP5 DocInfo 라운드트립 — 필드 미파싱 + 하드코딩 리터럴)

## 1. 문제

`HWPTAG_PARA_SHAPE` 레코드 payload 말미 4바이트(offset 54~57)는 **문단 개요 수준(0~9 = 1수준~10수준)** 이다.
`attr1` bit 25~27 은 3비트뿐이라 한컴이 6에서 포화시키므로, **8·9·10수준은 이 4바이트에만 존재**한다.

rhwp 는 이 필드를 양쪽에서 다뤘어야 하는데 둘 다 놓쳤다.

1. **파서** (`src/parser/doc_info.rs` `parse_para_shape`) — 4바이트를 아예 읽지 않았다.
   `para_level` 이 `attr1` 비트에서만 나오므로 8·9·10수준 문단이 전부 **7수준으로 붕괴**했다.
2. **직렬화기** (`src/serializer/doc_info.rs` `serialize_para_shape`) — `w.write_u32(0)` 리터럴이었다.
   재직렬화되는 문단 모양마다 개요 수준이 **1수준으로 리셋**되고, `attr1` 이 말하는 수준과
   tail 이 말하는 수준이 **서로 모순되는 레코드**가 만들어졌다.

이 4바이트를 zero 로 materialize 하는 코드는 #1110 stage26 에서 **"58바이트 길이 계약"만 맞추려고**
들어간 것이고(`mydocs/working/archives/task_m100_1110_stage1.md:2225-2258`), 값의 정체는 당시 규명되지 않았다.
이번 작업은 그 값을 규명해 채운 것이며 **길이 계약은 그대로**다(58바이트 불변, 값만 채움).

## 2. 분석

### 2.1 `raw_stream_dirty` 통과 경로 분석 (이 결함이 가려지지 않는 이유)

DocInfo 는 2단 통과 계층을 갖는다.

- **1단 (스트림)** `serializer/doc_info.rs:22-33` — `!raw_stream_dirty && raw_stream.is_some()` 이면
  원본 스트림을 그대로 반환한다.
- **2단 (레코드)** `serializer/doc_info.rs:107-113` — dirty 여도 각 ParaShape 은 `raw_data` 가 `Some` 이면
  원본 바이트를 쓴다.

따라서 `serialize_para_shape()` 가 실제로 도는 조건은 **dirty == true 이면서 그 ParaShape 의
`raw_data == None`** 이다. 이 결함이 "편집 안 한 문서에선 안 돈다"로 무마되지 않는 이유는,
**dirty 를 세우는 바로 그 동작이 동시에 `raw_data = None` 인 ParaShape 을 만들기 때문**이다.

`model/document.rs:466-491` `find_or_create_para_shape()` 는 한 함수 안에서
`ParaShapeMods::apply_to()`(`model/style.rs:918` 에서 `raw_data = None`)로 새 ParaShape 을 만들고,
push 한 뒤 `raw_stream_dirty = true` 를 세운다. 그리고 편집된 문단이 바로 그 새 id 를 참조한다.

| 조건 | 성립 | 근거 |
|---|---|---|
| `raw_stream_dirty = true` | O | `document.rs:489` |
| 새 ParaShape 의 `raw_data == None` | O | `style.rs:918` |
| 편집된 문단이 그 ParaShape 참조 | O | `find_or_create_para_shape()` 반환 id |

호출 지점(= 이 경로를 타는 사용자 동작): `formatting.rs:1374,1530,1720,1742`,
`footnote_ops.rs:391`, `header_footer_ops.rs:843,1037`.
즉 **정렬·줄간격·여백·들여쓰기·문단수준·탭·문단 테두리** 중 하나만 바꿔도 이 경로다.
특히 `ParaShapeMods::para_level` 로 **사용자가 개요 수준을 직접 지정**하는 경로에서도
tail 은 0 으로 나갔다 — 방금 지정한 수준이 한컴이 읽는 필드에 반영되지 않았다.

정직하게 덧붙이면, `find_or_create_para_shape()` 는 동일 ParaShape 이 있으면 **재사용**하고
그 경우엔 원본 `raw_data` 가 쓰여 손실이 없다. 손실은 **새로 push 되는 경우**에 발생한다.

한편 **읽기 쪽 손실은 dirty 와 무관하게 항상** 발생했다. 소비처는 개요번호 렌더링
(`renderer/layout.rs:1826`, `renderer/layout/paragraph_layout.rs:6126`), 구조 추출
(`document_core/queries/structure.rs:82`), HWPX `level` 속성(`serializer/hwpx/header.rs:1075`),
HML `Level`(`serializer/hml/head.rs:176`) 이다.

### 2.2 필드 정체 규명

`samples/*.hwp` 252개 파일에서 58바이트 `PARA_SHAPE` 11,913건의 tail 과 `attr1` bit25~27 을 대조했다.

| tail | `attr1` 수준 | 건수 | 판정 |
|---:|---:|---:|---|
| 0 | 0 | 11,032 | 일치 |
| 1 | 1 | 97 | 일치 |
| 2 | 2 | 94 | 일치 |
| 3 | 3 | 305 | 일치 |
| 4 | 4 | 84 | 일치 |
| 5 | 5 | 77 | 일치 |
| 6 | 6 | 77 | 일치 |
| 7 | **6** | 46 | attr1 포화 |
| 8 | **6** | 46 | attr1 포화 |
| 9 | **6** | 46 | attr1 포화 |
| 0 | 1~6 | 9 | 단일 파일 예외 |

tail 0~6 구간이 1:1 완전 일치하고, 7·8·9 에서 `attr1` 이 6 으로 포화한다.
어긋나는 유일한 사례는 `issue2439_zero_offset_coanchored_float_exclusion.hwp`(ver 5.1.0.0) 한 파일의
9건뿐이며, 나머지 **125개 파일 / 11,904건에서 어긋남 0건**이다.

**교차 검증**: `tail >= 7` 인 ParaShape 을 가진 파일은 46개이고 각 파일에서 정확히 3개(7,8,9)씩 나온다
(46×3 = 138). 그중 42개 파일이 `HWPTAG_NUMBERING` 에도 확장 수준을 갖는다. 실제 바이트를 뜯으면
8·9·10수준 문단머리정보 3개 + 시작번호 3개이며, 8수준 번호 형식 문자열이 `5e 00 38 00` = `"^8"` 이다.
즉 코퍼스의 한컴 문서들이 실제로 8~10수준 개요를 정의하고 있고, 대응 문단모양이 tail 7·8·9 를 갖는다.

## 3. 실측

집계 코드는 `src/serializer/doc_info/tests.rs` 에 임시 테스트로 붙여 실행하고 커밋 전 제거했다
(커밋에 포함되지 않음). 측정 항목은 (a) tail ↔ attr1 상관, (b) 각 레코드의 `raw_data` 와
`serialize_X(parse 결과)` 의 바이트 비교다.

### 3.1 수정 전

```
파일: 성공 252 / 실패 18 (총 270)
PARA_SHAPE  총 15043  동일 11041  다름 4002   firstdiff={42:112 46:84 54:3806}
  원본 크기 분포: 42B:112  46B:84  54B:2934  58B:11913
```

`firstdiff` offset 54 의 3,806건 = 54바이트 레코드 2,934건(tail 신규 부착, 손실 아님)
**+ 58바이트 레코드 872건(원본 tail 1~9 를 0 으로 덮어씀 = 손실)**.

### 3.2 수정 후

```
파일: 성공 252 / 실패 18 (총 270)
PARA_SHAPE  총 15043  동일 11904  다름 3139   firstdiff={42:112 46:84 54:2943}
```

- **바이트 동일 11,041 → 11,904 (+863)**. 손실 872건 중 863건이 사라졌다.
- 54바이트 레코드 2,934건의 tail 신규 부착은 #1110 계약 그대로 유지된다.

**남은 9건을 숨기지 않고 밝힌다.**

- 전부 **`issue2439_zero_offset_coanchored_float_exclusion.hwp` 한 파일**(FileHeader ver 5.1.0.0)의
  ParaShape 9개다. 나머지 125개 파일 11,904건에는 이런 레코드가 없다.
- 원본이 **`attr1` 수준 = 1,2,3,4,5,6,6,6,6 인데 tail = 0** 인 **자기모순** 레코드다.
- 수정 후 rhwp 는 tail 에 `attr1` 값을 써서 두 필드를 **정합**시킨다. 값을 지어내는 게 아니라
  이미 레코드 안에 있는 수준을 다른 필드에도 반영하는 것이다.
- **IR `para_level` 은 종전과 완전히 동일하다.** 파서가 `max(attr1, tail)` 을 쓰므로 `max(3,0) = 3` 으로
  기존 값이 유지된다. 이 회귀 없음을 테스트로 고정했다
  (`para_shape_recovers_outline_level_8_to_10_from_tail` 의 `tail=0/attr1=1~6` 루프).
- 이 파일 하나만 왜 다른지는 **규명하지 못했다**(다른 파일은 ver 5.1.0.1 / 5.1.1.0). tail 을 채우지
  않는 구버전 writer 산출물로 **추정**하지만 확인하지 않았으므로 추정이라고 명시한다.

### 3.3 다른 DocInfo 레코드 무영향 확인 (실측)

같은 census 를 수정 전/후로 돌려 전 레코드 종류를 비교했다. **PARA_SHAPE 외 전부 동일**하다.

| 레코드 | 총 | 다름 (수정 전) | 다름 (수정 후) |
|---|---:|---:|---:|
| BIN_DATA | 1,627 | 0 | 0 |
| FACE_NAME | 10,093 | 171 | 171 |
| BORDER_FILL | 6,237 | 82 | 82 |
| CHAR_SHAPE | 13,991 | 1,706 | 1,706 |
| TAB_DEF | 950 | 8 | 8 |
| NUMBERING | 347 | 203 | 203 |
| BULLET | 62 | 62 | 62 |
| STYLE | 6,436 | 101 | 101 |
| **PARA_SHAPE** | **15,043** | **4,002** | **3,139** |

### 3.4 실측 vs 잠재

- **실측**: 위 모든 수치, tail 값 분포와 상관, 872 → 9 감소, 8수준 번호 형식 바이트 덤프.
- **잠재(코드 판독)**: "한컴 편집기가 tail 을 authoritative 로 읽어 1수준으로 표시한다"는 부분.
  한컴 실행 검증은 하지 않았다. 확실한 것은 종전 코드가 **attr1 과 tail 이 모순되는 레코드**를
  만들었고 한컴 산출물 11,904건은 예외 없이 정합했다는 점, 그리고 8~10수준은 attr1 이 표현할
  수단 자체가 없어 tail 유실 = 수준 유실이 확정적이라는 점이다.
- **잠재**: HWPX(.hwpx) → .hwp 변환 산출물을 한컴으로 열어 확인하지 않았다.

## 4. 변경

### 4.1 파서 (`src/parser/doc_info.rs`)

`line_spacing_v2` 다음에 말미 4바이트를 읽고, `para_level` 을 두 출처의 **큰 쪽**으로 정한다.

```rust
let outline_level_tail = if r.remaining() >= 4 { r.read_u32().unwrap_or(0) } else { 0 };
...
let para_level = (((attr1 >> 25) & 0x07) as u8).max(outline_level_tail.min(9) as u8);
```

`max` 를 쓰는 근거는 실측이다. 두 값이 어긋나는 실측 사례는 (a) tail 7~9 / attr1 6(포화) 138건,
(b) tail 0 / attr1 1~6 단일 파일 9건뿐이므로, `max` 가 (a)에서 8~10수준을 복원하면서
(b)에서 종전 동작을 그대로 보존한다. tail 이 없는 42/46/54바이트 레코드는 `remaining() < 4` 라
`outline_level_tail = 0` → 종전과 동일하다.

### 4.2 직렬화기 (`src/serializer/doc_info.rs`)

```rust
// bits 25-27
attr1 |= (ps.para_level.min(6) as u32) << 25;   // 종전: (ps.para_level as u32 & 0x07) << 25
...
w.write_u32(ps.para_level.min(9) as u32).unwrap();  // 종전: w.write_u32(0)
```

`min(6)` 은 한컴 실측 규약이다(개요 8~10수준 문단모양 138건이 모두 attr1 비트 6 + tail 7/8/9).
종전 `& 0x07` 은 `para_level` 이 7 이상일 때 8→0, 9→1 로 엉뚱한 수준을 박는다.

### 4.3 건드리지 않은 것

- `ParaShape` 모델 — 필드 추가 없음. `para_level` 이 이미 같은 개념이다.
- `ParaShapeMods::apply_to` — attr1 비트를 `v & 0x07` 로 쓰지만 직렬화기가 그 비트를 무조건
  다시 계산하므로 출력에 영향이 없다. 무관 변경 금지 원칙에 따라 손대지 않았다.
- 58바이트 길이 계약(#1110 stage26), HWPX/HML/렌더러. 렌더러의 `para_level` 소비처는 이미
  `layout.rs:984`, `paragraph_layout.rs:6139` 에서 `.min(6)` 으로 방어하므로 값이 7~9 로 확장돼도
  인덱스 초과가 없다.

## 5. 검증

### 5.1 red → green (실제 실행 캡처)

수정 2건(파서 tail 반영, 직렬화기 tail 기록·attr1 포화)을 되돌리고 테스트만 남긴 상태.

```
running 5 tests
test serializer::doc_info::tests::test_serialize_para_shape_roundtrip ... ok
test serializer::doc_info::tests::serialize_para_shape_writes_outline_level_into_tail ... FAILED
test parser::doc_info::tests::para_shape_edit_path_keeps_outline_level_in_tail ... FAILED
test parser::doc_info::tests::para_shape_outline_level_roundtrips_0_to_9 ... FAILED
test parser::doc_info::tests::para_shape_recovers_outline_level_8_to_10_from_tail ... FAILED

---- serializer::doc_info::tests::serialize_para_shape_writes_outline_level_into_tail stdout ----
thread '...' panicked at src\serializer\doc_info\tests.rs:283:9:
assertion `left == right` failed: 말미 4바이트에 개요 수준 1 이 실려야 함
  left: 0
 right: 1

---- parser::doc_info::tests::para_shape_edit_path_keeps_outline_level_in_tail stdout ----
thread '...' panicked at src\parser\doc_info.rs:1292:13:
assertion `left == right` failed: 개요 1 수준 레코드 재직렬화 시 말미 4바이트가 보존돼야 함
  left: [0, 0, 0, 0]
 right: [1, 0, 0, 0]

---- parser::doc_info::tests::para_shape_outline_level_roundtrips_0_to_9 stdout ----
thread '...' panicked at src\parser\doc_info.rs:1310:13:
assertion `left == right` failed: 개요 수준 8 왕복 보존
  left: 0
 right: 8

---- parser::doc_info::tests::para_shape_recovers_outline_level_8_to_10_from_tail stdout ----
thread '...' panicked at src\parser\doc_info.rs:1255:13:
assertion `left == right` failed: tail=7 이면 개요 수준 7 로 복원돼야 함(attr1 은 6 에서 포화)
  left: 6
 right: 7

test result: FAILED. 1 passed; 4 failed; 0 ignored; 0 measured; 2470 filtered out; finished in 0.00s
```

**RED 에서도 통과하는 테스트가 있다 — 어느 것이고 왜인지 밝힌다.**

- `test_serialize_para_shape_roundtrip`(기존 테스트)은 RED 에서도 통과한다. `para_level: 0` 인
  ParaShape 만 쓰므로 `write_u32(0)` 과 `write_u32(para_level)` 의 출력이 같다.
  이 테스트의 `assert_eq!(&data[54..58], &[0,0,0,0])` 단언은 수정 후에도 그대로 유효하므로
  변경하지 않았다.
- `para_shape_outline_level_roundtrips_0_to_9` 는 **수준 8에서 처음 실패**한다. 수준 0~7 은
  RED 상태에서도 `attr1` bit25~27(3비트, `& 0x07`)만으로 rhwp 내부 왕복이 성립하기 때문이다.
  이 테스트가 잡는 것은 3비트를 넘는 8·9수준뿐이다. **한컴이 읽는 tail 값이 사라지는 본체 결함은
  `serialize_para_shape_writes_outline_level_into_tail`(수준 1에서 즉시 실패)과
  `para_shape_edit_path_keeps_outline_level_in_tail`(수준 1에서 즉시 실패)이 잡는다.**

수정 복원 후:

```
running 5 tests
test serializer::doc_info::tests::serialize_para_shape_writes_outline_level_into_tail ... ok
test serializer::doc_info::tests::test_serialize_para_shape_roundtrip ... ok
test parser::doc_info::tests::para_shape_edit_path_keeps_outline_level_in_tail ... ok
test parser::doc_info::tests::para_shape_outline_level_roundtrips_0_to_9 ... ok
test parser::doc_info::tests::para_shape_recovers_outline_level_8_to_10_from_tail ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 2470 filtered out; finished in 0.00s
```

### 5.2 CI 3종

```
$ cargo clippy --all-targets -- -D warnings
    Checking rhwp v0.7.19 (C:\Users\swsz9\Downloads\moneyflow\rhwp-wt-c)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 43s
→ 경고 0건
```

```
$ git diff --name-only origin/devel...HEAD -- '*.rs'
src/parser/doc_info.rs
src/serializer/doc_info.rs
src/serializer/doc_info/tests.rs

$ for f in $(git diff --name-only origin/devel...HEAD -- '*.rs'); do rustfmt --edition 2021 "$f"; done
$ git diff --name-only
[빈 출력]
```

빈 출력 = 커밋된 3개 파일이 모두 포맷을 준수한다. (커밋 전 1차 적용 시 신규 테스트 1곳의
줄바꿈 3줄이 조정됐고 그 결과를 커밋에 포함했으므로, 커밋 후 재적용에서는 변화가 없다.)
`cargo fmt --all -- --check` 는 이 CRLF 체크아웃에서 `Incorrect newline style` 만 찍고
diff 를 내지 않아 false pass 하므로 **사용하지 않았다.**

```
$ cargo test --profile release-test --tests
→ exit 0
→ 테스트 타깃 292개 전부 ok
→ 합계 3,494 passed / 0 failed / 23 ignored / 0 measured
   · lib 단위 테스트(src/lib.rs) 2,468 passed / 0 failed / 7 ignored — 신규 4건 + 기존 1건 포함:
       test parser::doc_info::tests::para_shape_edit_path_keeps_outline_level_in_tail ... ok
       test parser::doc_info::tests::para_shape_outline_level_roundtrips_0_to_9 ... ok
       test parser::doc_info::tests::para_shape_recovers_outline_level_8_to_10_from_tail ... ok
       test serializer::doc_info::tests::serialize_para_shape_writes_outline_level_into_tail ... ok
       test serializer::doc_info::tests::test_serialize_para_shape_roundtrip ... ok
   · 나머지 291개 타깃(tests/ 통합 + bin) 1,026 passed / 0 failed / 16 ignored
→ 출력 전체에서 `FAILED` / `panicked at` 0건
```

DocInfo 변경은 파급이 넓어 실파일 왕복·바이트 baseline 게이트를 특히 확인했다. 전부 통과:

```
tests/hwp5_roundtrip_baseline.rs    ok. 3 passed (baseline_all_samples_roundtrip / large / xfail)
tests/hwpx_roundtrip_baseline.rs    ok. 4 passed
tests/visual_roundtrip_baseline.rs  ok. 3 passed
tests/opengov_corpus_snapshot.rs    ok. 2 passed
tests/svg_snapshot.rs               ok. 8 passed
tests/hwp5_strikeout_shape_parity.rs ok. 2 passed
serializer::cfb_writer::tests::test_serialize_real_hwp_files ... ok
```

내가 작성하지 않은 테스트 중 실패한 것은 **없다**. 따라서 기존 단언을 수정한 곳도 없다.

### 5.3 코퍼스 재측정

3.2·3.3 참조. PARA_SHAPE 손실 872 → 9(자기모순 레코드 정합화), 다른 레코드 종류 무변화.

## 6. 미실행 항목

- **한컴 편집기 실물 판정 없음.** 수정 후 .hwp 를 한글에서 열어 개요 수준을 눈으로 확인하지 않았다.
  근거는 전적으로 코퍼스 실측(11,913건 전수 상관)과 코드 경로 분석이다.
- **HWPX→HWP 변환 산출물 실물 검증 없음.** 이 경로에서도 tail 이 채워지도록 개선되지만
  변환 결과를 한컴으로 열어보지는 않았다.
- **8~10수준 렌더링 개선 여부 미검증.** 파서가 이제 7·8·9 를 반환하지만 렌더러/HWPX/HML 이
  이 값으로 무엇을 다르게 그리는지는 이번 범위에서 측정하지 않았다(소비처가 모두 `.min(6)` 으로
  클램프하므로 렌더 결과는 종전과 동일할 가능성이 높다). 이 이슈의 성과는 **저장 시 유실 차단**과
  **IR 정확도 회복**이다.

## 7. 잔여 (범위 밖)

같은 코퍼스 조사에서 함께 관측된 DocInfo 결함들. 이슈 #2734 의 7절에도 기록했다.
수치는 실측, "영향"은 코드 판독 기반 추정이다.

| 대상 | 관측 | 실측 규모 |
|---|---|---|
| `HWPTAG_BULLET` | 실제 레코드는 23/25바이트인데 `serialize_bullet` 은 항상 24바이트. 이미지 글머리 정보가 5바이트(밝기1+명암1+효과1+binID2)인데 파서는 `image_data: [u8;4]` 로 4바이트만 읽어 `check_bullet_char` 가 1바이트 밀린다 | 62/62 불일치. 25바이트 37건 중 13건에서 `U+0020` 대신 `U+2000`, 2건에서 `U+F0A4` 대신 `U+A400` 으로 파싱 |
| `HWPTAG_NUMBERING` | 8·9·10수준 문단머리정보 + 번호 형식 + 시작번호 3개 미파싱 → 재직렬화 시 60바이트 축소 | 347건 중 200건(57.6%)이 확장 수준 보유 |
| `HWPTAG_FACE_NAME` | 대체 글꼴 앞 "대체 글꼴 유형" 1바이트를 `font.alt_type & 0x03`(자기 자신의 유형)으로 덮어씀 | 109건 |
| `HWPTAG_CHAR_SHAPE` | 밑줄 종류 `(attr>>2)&3 == 2` 를 `UnderlineType::None` 으로 접고 0 으로 씀 | 13,991건 중 1,514건 |
| `HWPTAG_CHAR_SHAPE` | 취소선 비트(18~20)를 `is_real_strike_shape_id` 판정으로 재기록해 원본 비트가 1→0 / 1→2 로 변함 | 1,620건 |
| `HWPTAG_BORDER_FILL` | fill type 이 비트마스크인데 모델이 단일 enum 이라 `0x03`(단색+이미지)이 이미지 단독으로 붕괴 | 6,237건 중 28건 |
| `HWPTAG_DOCUMENT_PROPERTIES` | `serialize_document_properties` 가 `raw_data` 있으면 무조건 원본 반환 → 모델 필드 변경 무반영 | 코드 판독만 |
