# task_m100_2727 처리결과 보고서 — 수식 EQEDIT attribute(lineMode) 왕복 유실

- **이슈**: [#2727](https://github.com/edwardkim/rhwp/issues/2727)
- **브랜치**: `task/m100-2727-equation-lineMode` (base `devel` @ `49f38446`)
- **범위**: `src/model/control.rs`, `src/parser/control.rs`, `src/serializer/control.rs`(수식 arm 한정),
  `src/parser/hwpx/section.rs`, `src/serializer/hwpx/section.rs`, `src/serializer/hwpx/mod.rs`(테스트 픽스처),
  `tests/issue_2727_equation_line_mode.rs`(신규)
- **분류**: 결함 수정 (수식 속성 왕복 유실)

## 1. 문제

수식(`eqed`) 컨트롤의 자식 레코드 `HWPTAG_EQEDIT` 선두 UINT32(HWP5 스펙 표 105 attribute)가
파서에서 버려지고 직렬화기에서 상수 `0` 으로 덮어써졌다.

```rust
// src/parser/control.rs:866 (정정 전)
// attr: u32 (4바이트) — bit0: 스크립트 범위
let _attr = r.read_u32().unwrap_or(0);      // 의미를 주석에 적어두고 버린다

// src/serializer/control.rs:2393 (정정 전)
// attr: u32
w.write_u32(0).unwrap();                    // IR 이 없으므로 상수 0
```

이 UINT32 의 bit 0 은 HWPX `hp:equation@lineMode`(`LINE` / `CHAR`, OWPML 문서 문구
"수식이 차지하는 범위")와 대응한다. HWPX 쪽도 같은 값이 양방향으로 끊겨 있었다.

- `src/parser/hwpx/section.rs::parse_equation` 의 속성 match 에 `lineMode` arm 없음
- `src/serializer/hwpx/section.rs::render_equation` 템플릿에 `lineMode` 속성 자체가 없음

결과적으로 수식 범위 설정이 **HWP5→HWP5(편집 후) / HWP5→HWPX / HWPX→HWP5 / HWPX→HWPX 네
경로 전부**에서 `CHAR`(0)로 초기화됐다.

같은 EQEDIT 레코드 안에서 **스펙에 문서화조차 되어 있지 않은** `unknown: u16` 은 Task #1061 이
IR 필드로 승격해 원본 그대로 왕복시키고 있었다. 문서화된 필드만 버리는 반대 방향 비대칭이었다.

## 2. 분석

### 2.1 패스스루가 가려주지 않는다

| 층 | 판정 |
|----|------|
| `Equation::raw_ctrl_data` | **가리지 않음.** `CTRL_HEADER(eqed)` 의 ctrl_data 만 담는다. attribute 는 자식 레코드 `HWPTAG_EQEDIT` 안이고 EQEDIT 은 raw 필드가 없어 매번 IR 에서 재생성된다. 게다가 `adapt_equation`(`hwpx_to_hwp.rs`)과 `apply_equation_properties`(`object_ops/equation.rs:176`)가 이 필드를 명시적으로 `clear()` 한다. |
| `Paragraph::ctrl_data_records[i]` | **가리지 않음.** `src/serializer/control.rs:98` 의 수식 arm 이 `ctrl_data_record` 인자를 아예 전달하지 않는다(Picture/Shape/Bookmark arm 은 받는다). 한컴 실측본의 `eqed` 자식은 EQEDIT 하나뿐이라 CTRL_DATA 자체가 없다. |
| `Section::raw_stream` | **일부만 가린다.** 아무것도 건드리지 않은 HWP5→HWP5 복사에서는 원본 바이트가 그대로 재생돼 손실이 드러나지 않는다(실측 확인, 3.3 참조). 그러나 수식 관련 편집 API 는 모두 `section.raw_stream = None` 을 실행하고(`set_equation_properties_native`·`delete_equation_control_native`·`insert_equation_native`), HWPX 출력 경로는 raw_stream 을 참조하지 않는다. |
| DocInfo `raw_stream_dirty` | 무관 (EQEDIT 은 BodyText). |

### 2.2 의도된 상수인지 확인

- `git log -S "bit0: 스크립트 범위" -- src/parser/control.rs` → `f0f7f1a4` (최초 커밋) 1건
- `git log -S "serialize_equation_control" -- src/serializer/control.rs` → `f0f7f1a4` 1건

둘 다 최초 커밋 이후 손대지 않은 코드이며, "0 으로 고정한다"는 계약을 설명하는 주석·정답지 대조
기록이 없다. `adapt_equation:1204-1208` 의 bit 27(0x08000000) 보강처럼 정답지 근거가 남아 있는
의도적 상수와는 성격이 다르다(그 코드는 이번 작업에서 손대지 않았다).

### 2.3 근거 자료

- OWPML 스키마 `mydocs/manual/OWPML SCHEMA/ParaList XML schema.xml:2189-2199` — `EquationType/@lineMode`,
  `default="CHAR"`, enum `LINE|CHAR`, 문서 문구 "수식이 차지하는 범위."
- `mydocs/tech/webhwp/05_other_controls.md:35-43` — 한컴 웹한글 역분석 기록의 수식 JSON 스키마에
  `lm: lineMode // 줄 모드 (boolean)`, 임포트 코드 `e.qbn = h.lm ? 1 : 0`(불리언 1비트).
- rhwp 자신의 파서 주석 `// attr: u32 (4바이트) — bit0: 스크립트 범위`.

### 2.4 말뭉치 전수 조사

`samples/**.hwpx` 중 `<hp:equation>` 포함 파일 전수:

| 항목 | 값 |
|------|-----|
| 파일 수 | 19 |
| `<hp:equation` 요소 총수 | 23,138 |
| `lineMode` 명시 요소 수 | 23,138 (100%) |
| 값 `CHAR` / `LINE` | 23,138 / 0 |

한컴은 기본값이어도 예외 없이 이 속성을 기록한다. 말뭉치에 `LINE` 사례가 0건이라는 사실은 그대로
밝힌다 — 본 결함은 **잠복 손실**이며, 정정해도 기존 23,138개 수식의 출력값은 `CHAR`/`0` 그대로다
(3.4 회귀 실측).

## 3. 변경

### 3.1 코드

| 파일 | 내용 |
|------|------|
| `src/model/control.rs` | `pub const EQUATION_LINE_MODE_BIT: u32 = 0x0000_0001;` 신규. `Equation` 에 `pub attr: u32` 신규(UINT32 전체 보존 — bit0 외 비트도 유실시키지 않기 위함). 기본값 0 = OWPML `lineMode` 기본값 `CHAR`. |
| `src/parser/control.rs:866` | `let _attr = ...` → `equation.attr = r.read_u32().unwrap_or(0);` |
| `src/serializer/control.rs:2393` | `w.write_u32(0)` → `w.write_u32(eq.attr)` (수식 arm 한정, 3줄) |
| `src/parser/hwpx/section.rs` | `parse_equation` 에 `b"lineMode"` arm 추가 — `LINE`(대소문자 무시)이면 bit0 set. `Equation` 생성 시 `attr: eq_attr`. |
| `src/serializer/hwpx/section.rs` | `render_equation` 이 `baseUnit` 과 `font` 사이(한컴 저장본과 동일 자리)에 `lineMode="{LINE\|CHAR}"` 방출. |
| `src/serializer/hwpx/mod.rs` | 기존 테스트 픽스처의 전량 초기화 리터럴에 `attr: 0,` 1줄 추가(컴파일 보정). |

`hwpx_to_hwp.rs::adapt_equation` 은 변경 없음 — attribute 는 `common` 이 아니라 `Equation` 에 있고,
그 함수가 지우는 것은 `raw_ctrl_data`(CTRL_HEADER)뿐이라 EQEDIT 재생성 경로에 그대로 실린다.

### 3.2 테스트

- `tests/issue_2727_equation_line_mode.rs` (신규 4건) — 한컴 실제 저장본
  `samples/수식-문자처럼취급-아님.hwp` 를 입력으로,
  1. `issue_2727_hwp5_line_mode_survives_roundtrip` — bit0 set 후 HWP5 직렬화→재파싱에서 보존
  2. `issue_2727_hwpx_line_mode_survives_roundtrip` — HWPX 직렬화→재파싱에서 보존
  3. `issue_2727_hancom_char_equation_stays_char` — 원본 CHAR 문서는 양 포맷 왕복 후에도 attr 0
  4. `issue_2727_hancom_hwpx_char_parses_as_zero_bit` — 한컴 원본 HWPX `lineMode="CHAR"` → bit0 clear
- `src/serializer/hwpx/section.rs::tests::equation_line_mode_reflects_ir` (신규 1건) —
  `render_equation` 출력에 기본값도 `lineMode="CHAR"` 가 나오고, IR bit0 set 시
  `baseUnit="1000" lineMode="LINE" font=""` 자리에 방출되는지 단언(선행 `equation_text_flow_reflects_ir` 패턴 정합).

## 4. 검증

### 4.1 red→green (실제 실행 캡처)

정정 4곳(`parser/control.rs`, `serializer/control.rs`, `parser/hwpx/section.rs`,
`serializer/hwpx/section.rs`)을 원상 복구한 뒤 실행했다(모델 필드는 컴파일을 위해 유지).

**RED — 통합 테스트**

```
running 4 tests
test issue_2727_hwp5_line_mode_survives_roundtrip ... FAILED
test issue_2727_hancom_hwpx_char_parses_as_zero_bit ... ok
test issue_2727_hwpx_line_mode_survives_roundtrip ... FAILED
test issue_2727_hancom_char_equation_stays_char ... ok

failures:

---- issue_2727_hwp5_line_mode_survives_roundtrip stdout ----
thread 'issue_2727_hwp5_line_mode_survives_roundtrip' (40324) panicked at tests\issue_2727_equation_line_mode.rs:71:5:
assertion `left == right` failed: EQEDIT attribute bit0(수식 범위 LINE)이 HWP5 왕복에서 보존돼야 한다. attr=0x00000000
  left: 0
 right: 1

---- issue_2727_hwpx_line_mode_survives_roundtrip stdout ----
thread 'issue_2727_hwpx_line_mode_survives_roundtrip' (36652) panicked at tests\issue_2727_equation_line_mode.rs:89:5:
assertion `left == right` failed: HWPX lineMode="LINE" 이 왕복에서 보존돼야 한다. attr=0x00000000
  left: 0
 right: 1

failures:
    issue_2727_hwp5_line_mode_survives_roundtrip
    issue_2727_hwpx_line_mode_survives_roundtrip

test result: FAILED. 2 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
error: test failed, to rerun pass `--test issue_2727_equation_line_mode`
```

**RED — 단위 테스트**

```
running 1 test
test serializer::hwpx::section::tests::equation_line_mode_reflects_ir ... FAILED

failures:

---- serializer::hwpx::section::tests::equation_line_mode_reflects_ir stdout ----
thread 'serializer::hwpx::section::tests::equation_line_mode_reflects_ir' (14068) panicked at src\serializer\hwpx\section.rs:2365:9:
기본값도 lineMode 속성을 방출해야 함(한컴 저장본 정합): <hp:equation id="0" zOrder="0" numberingType="EQUATION" textWrap="SQUARE" textFlow="BOTH_SIDES" lock="0" dropcapstyle="None" instid="0" version="" baseLine="0" textColor="#000000" baseUnit="0" font=""><hp:script></hp:script>...

failures:
    serializer::hwpx::section::tests::equation_line_mode_reflects_ir

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2471 filtered out; finished in 0.01s
error: test failed, to rerun pass `--lib`
```

**GREEN — 복구 후 재실행**

```
running 4 tests
test issue_2727_hwp5_line_mode_survives_roundtrip ... ok
test issue_2727_hancom_hwpx_char_parses_as_zero_bit ... ok
test issue_2727_hwpx_line_mode_survives_roundtrip ... ok
test issue_2727_hancom_char_equation_stays_char ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

```
running 1 test
test serializer::hwpx::section::tests::equation_line_mode_reflects_ir ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2471 filtered out; finished in 0.00s
```

### 4.2 CLI 실측 — 정정 전

한컴 원본 `samples/수식-문자처럼취급-아님.hwpx` 의 `lineMode="CHAR"` 한 곳만 `LINE` 으로 바꾼
**조작 입력**(정답지가 아님)을 만들어 측정했다.

```
=== A) hwpx(LINE) -> export-hwpx ===
<hp:equation id="1102112140" ... textColor="#000000" baseUnit="1600" font="HancomEQN">
   → lineMode 속성 소실

=== B) hwpx(LINE) -> convert -> hwp ===   (EQEDIT payload)
  0000: 00 00 00 00 2B 00 4C 00 41 00 44 00 44 00 45 00
        ^^^^^^^^^^^ 01 00 00 00 이어야 하는데 0

=== 정정 전 CLI: HWP5(attr=1) -> HWPX ===
lineMode 출현 0회
```

또한 한컴 원본 `.hwp` 를 그대로 `export-hwpx` 하면 한컴이 쓰는 `lineMode="CHAR"` 가 출력에서
빠져 있었다.

### 4.3 CLI 실측 — 정정 후

```
=== A) hwpx(LINE) -> export-hwpx ===
baseUnit="1600" lineMode="LINE" font="HancomEQN"

=== B) hwpx(LINE) -> convert -> hwp ===
  0000: 01 00 00 00 2B 00 4C 00 41 00 44 00 44 00 45 00

=== C) hwp(attr=1) -> convert -> hwp (HWP5 왕복) ===
  0000: 01 00 00 00 2B 00 4C 00 41 00 44 00 44 00 45 00

=== D) hwp(attr=1) -> export-hwpx ===
lineMode="LINE"
```

정직하게 밝히는 관찰: **손대지 않은 HWP5→HWP5 복사**(`convert a.hwp b.hwp`)는 `Section::raw_stream`
패스스루가 원본 바이트를 그대로 재생하므로 정정 전에도 attribute 가 살아남는다. 손실이 실현되는 것은
그 구역이 재직렬화될 때(수식 속성 변경·삽입·삭제 등 편집 API 가 `raw_stream = None` 을 실행한 뒤)이며,
그 조건을 재현한 것이 4.1 의 RED 통합 테스트다(`set_first_equation_line_mode` 가 실제 편집 API 와
동일하게 `section.raw_stream = None` 을 수행한다).

### 4.4 회귀 실측 (말뭉치 무변경)

```
=== 한컴 원본 hwp -> hwpx ===            lineMode="CHAR"
=== math-001.hwpx -> hwpx (수식 44개) ===  44 lineMode="CHAR"
=== math-001.hwp EQEDIT attr ===         0000: 00 00 00 00 2A 00 20 00 ...
```

정정 후 CHAR 문서는 값이 그대로이며, 종전에 빠져 있던 `lineMode="CHAR"` 가 한컴과 같은 자리에
추가로 방출된다(한컴 정합 방향).

### 4.5 CI 3종

| 검사 | 명령 | 결과 |
|------|------|------|
| clippy | `cargo clippy --all-targets -- -D warnings` | **통과** — `Finished dev profile ... in 1m 10s`, 경고 0 |
| 테스트 | `cargo test --profile release-test --tests` | **통과** — 아래 4.6 |
| fmt | 변경 `.rs` 전부 `rustfmt --edition 2021` 후 `git diff --name-only` | **통과** — 출력 없음 (커밋 상태 기준) |

`cargo fmt --all -- --check` 는 이 Windows 체크아웃에서 CRLF 파일에 대해 `Incorrect newline style`
만 출력하고 diff 를 내지 않아 거짓 통과를 만들므로 사용하지 않았다.

### 4.6 `cargo test --profile release-test --tests` 결과

종료 코드 0. 테스트 바이너리 292개 집계:

| 항목 | 수 |
|------|-----|
| passed | **3,485** |
| failed | **0** |
| ignored | 23 |
| `FAILED` 문자열 출현 | 0 |

- lib 단위 테스트: `test result: ok. 2465 passed; 0 failed; 7 ignored; 0 measured; 0 filtered out; finished in 9.29s`
- 신규 통합 테스트:

```
     Running tests\issue_2727_equation_line_mode.rs

running 4 tests
test issue_2727_hwp5_line_mode_survives_roundtrip ... ok
test issue_2727_hancom_hwpx_char_parses_as_zero_bit ... ok
test issue_2727_hwpx_line_mode_survives_roundtrip ... ok
test issue_2727_hancom_char_equation_stays_char ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## 5. 미실행 항목

- **한컴 한글 시각 판정 없음.** 이 환경에 한컴 한글이 없어 정정 후 `.hwp`/`.hwpx` 를 한글에서 열어
  수식 범위가 "줄 단위"로 보이는지 눈으로 확인하지 못했다. 검증은 바이트/XML 레벨과 왕복 IR 동등성에
  한정된다.
- **`LINE` 정답지 부재.** 말뭉치에 한컴이 만든 `lineMode="LINE"` 문서가 없어, LINE 값에 대한 검증은
  조작 입력(4.2)과 IR 왕복 테스트로만 수행했다. `CHAR` 쪽은 한컴 원본 23,138건 기준 무변경을 확인했다.

## 6. 잔여

1. **`affectLSpacing` 전면 유실** — `CommonObjAttr` 에 대응 필드가 없고
   `serializer/hwpx/section.rs:1913,2030`·`picture.rs:410`·`shape.rs:1004`·`table.rs:146` 이 모두
   `"0"` 하드코딩이다. `samples/issue1949_giant_cell_nested_tables_perf.hwpx` 의 수식 18개 중 **5개가
   `affectLSpacing="1"`** 이므로 실사례가 있는 실손실이다. 다만 공용 모델·공용 파서·공용 직렬화기를
   함께 고쳐야 해서 이번 수식 전용 범위 밖으로 뒀다.
2. **`allowOverlap` 하드코딩** — `render_equation` 이 `allowOverlap="0"` 고정. `common.allow_overlap`
   은 이미 IR 필드다(말뭉치 내 수식은 전부 0 이라 잠복).
3. **`<hp:script>` 자식 순서** — OWPML `EquationType` 은 `sz, pos, outMargin, caption, shapeComment,
   metaTag, script` 순서를 요구하고 한컴 출력도 `script` 를 마지막에 둔다. rhwp 는 여는 태그 직후에
   방출한다. 한컴 재적재 영향을 실측하지 못해 이번엔 건드리지 않았다.
4. **HML 경로** — `src/parser/hml/reader.rs:695-703` 이 `BaseLine|BaseUnit|TextColor|Version|Font`
   이외의 `EQUATION` 속성을 `UnsupportedEquationSemantics` 경고로 흘려보낸다. HML `LineMode` 매핑은
   HML 파서·직렬화기 동시 변경이 필요하다.
5. **수식 캡션** — 한컴 실측본의 `eqed` 자식에 `LIST_HEADER` 가 없어 이번 말뭉치에서는 확인 불가.
