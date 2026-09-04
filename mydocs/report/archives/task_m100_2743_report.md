# task_m100_2743 처리결과 보고서 — HML 리소스 Id 할당 상한

- **이슈**: [#2743](https://github.com/edwardkim/rhwp/issues/2743)
- **브랜치**: `task/m100-2743-hml-resource-id-limit` (base `origin/devel` @ `1658d0bb`)
- **범위**: `src/parser/hml/reader.rs`, `tests/issue_2743_hml_resource_id_limit.rs`
- **분류**: 결함 수정 (malformed 입력에 대한 파서 robustness — 무검증 할당)

## 1. 문제

HML 리더의 `set_indexed()` 가 파일에서 온 `Id` 를 검증 없이 `resize_with(index + 1, ..)`
에 넘겨, 리소스 테이블 예약 크기가 `Id` 값에 선형 비례했다.

```rust
fn set_indexed<T: Default>(values: &mut Vec<T>, index: usize, value: T) {
    if values.len() <= index {
        values.resize_with(index + 1, T::default);   // index 는 파일에서 온 값
    }
    values[index] = value;
}
```

여섯 개 호출부가 전부 `parse_attribute::<usize>(element, b"Id")?` 결과를 그대로 넘긴다:
`FONT`, `BORDERFILL`, `CHARSHAPE`, `PARASHAPE`, `TABDEF`, `STYLE`.

### 1.1 #2722 와 다른 점 — **조용한 구간**이 따로 있다

| 입력 | 결과 | 종료 코드 |
|---|---|---|
| 382바이트, `CHARSHAPE Id="1000000"` | 힙 최대 **120,009,531 바이트** 예약 | **0 — 오류·경고 없음** |
| 385바이트, `CHARSHAPE Id="2000000000"` | `memory allocation of 240000000120 bytes failed` → abort | `0xC0000409` |

전자는 `parse_hml` 이 `Ok` 를 반환한다. 호출자가 정상 문서와 구별할 방법이 없었다.
이 결함의 본질은 abort 가 아니라 **이 조용함**이다.

### 1.2 같은 함수 안의 대조

`capture_border_fill` 은 같은 `Id` 속성에 대해 **너무 작은** 값(`Id=0`)은 하드 `Err` 로
거부하면서, `Id=2000000000` 은 그대로 통과시켜 240 GB 를 요구했다. 경계 검사가 한 방향만
있었다.

### 1.3 상한 기구는 이미 있었다

`HmlLimits` 는 `max_xml_bytes`/`max_depth`/`max_attributes`/`max_text_node_bytes` 네 개를
갖고 있으나 전부 **입력 크기** 상한이고 **인덱스(할당 크기) 상한이 없었다**. 재현 입력은
382바이트 / 깊이 6 / 속성 2개로 네 상한 전부의 한참 아래다.

## 2. 변경

`src/parser/hml/reader.rs` 한 파일 (92 insertions, 9 deletions).

1. **`HmlLimits::max_resource_id` 신설** (`:58`, 기본값 `65_535` at `:68`) — 새 기구를
   만들지 않고 이미 있는 상한 구조체에 필드 하나를 더했다.
2. **`set_indexed` 가 상한을 받아 `bool` 반환** (`:1583`) — 초과 시 **아무것도 할당하지
   않고** `false`. 상한 이하에서는 종전 코드와 동일하다.
3. **`ReadState.max_resource_id`** (`:203`) — `ReadState::new(xml, limits.max_resource_id)`
   (`:207`, `:1336`) 로 한 번만 복사. `capture_*` 시그니처를 건드리지 않아 디스패치 체인
   (`start`/`empty`/`capture_start`)은 그대로다.
4. **`warn_resource_id_out_of_range()`** (`:232`) — 건너뛴 리소스를 기존
   `HmlWarning::invalid_reference`(`HmlWarningCode::InvalidReference`)로 보고. 새 경고
   코드를 추가하지 않았다.
5. **여섯 호출부**(`:509` FONT, `:525` BORDERFILL, `:592` CHARSHAPE, `:630` PARASHAPE,
   `:669` TABDEF, `:688` STYLE)가 반환값을 확인해 실패 시 경고를 남긴다.
   `BORDERFILL` 은 추가로 `current_border_fill = None` 으로 되돌려, 뒤따르는
   `*BORDER` 자식이 존재하지 않는 테이블 항목을 가리키지 않게 했다.

`HmlLimits` 는 공개 타입이라 필드 추가가 **외부에서 구조체 리터럴로 전 필드를 나열하는
코드**에는 breaking 이다. 저장소 안의 구성 지점(`tests/hml_parser.rs:870`,
`src/serializer/hml/raw_fragment.rs:8`)은 전부 `HmlLimits::default()` /
`..HmlLimits::default()` 를 써서 영향이 없음을 확인했다. `#[non_exhaustive]` 부착은 별도
API 결정이라 이번 범위에 넣지 않았다.

### 2.1 하드 `Err` 가 아니라 skip + 경고로 한 이유

1. **오늘 열리는 파일이 내일 안 열리면 안 된다.** `Id="1000000"` 파일은 현재 `Ok` 로
   열린다(느리고 뚱뚱할 뿐). 하드 `Err` 는 회귀다.
2. **리더의 확립된 방침과 일치한다.** `HmlWarning::invalid_reference` 의 문구가 이미
   "잘못된 HML 리소스 참조를 기본값으로 대체했습니다" 다.
3. **조용함이 결함의 본질이므로 경고가 수정의 절반이다.** 지금까지는 아무 신호가 없었다.
4. **한 리소스를 건너뛰어도 다른 `Id` 위치가 밀리지 않는다.** `set_indexed` 는 절대
   인덱스에 배치하므로 나머지 테이블은 온전하고, 남은 참조는 기존 조회 경로가 기본값으로
   떨어뜨린다(`capture_border_line` 은 이미 `.get_mut()` 사용).

`capture_border_fill` 의 `Id=0` 하드 `Err` 는 그대로 뒀다 — 1-based 규약 위반(의미론
오류)이라 성격이 다르고, 바꾸면 기존 동작 회귀 위험이 있다.

### 2.2 상한값 `65_535` 근거 — 그리고 흔한 오해의 정정

**(a) 실측 코퍼스 조사** (이번 작업 중 직접 실행, `samples/` 345개 파일):

```
SURVEY files=345 (hml=2)
SURVEY max char_shapes=28193 (hwp3-sample10.hwp)
SURVEY max para_shapes=3378 (hwp3-sample16.hwp)
SURVEY max styles=208 (loading-fail-01.hwp)
SURVEY max border_fills=2705 (hwp3-sample16.hwp)
SURVEY max tab_defs=25 font_faces_per_lang=165
HML aligns.hml           cs=5 ps=12 st=14 bf=2 td=3
HML formatting_table.hml cs=7 ps=17 st=18 bf=3 td=3
```

관측 최댓값 **28,193** 대비 상한은 **2.32배**. 리소스 테이블 길이는 포맷 독립적인 DocInfo
자원이므로(같은 문서를 HML 로 내보내면 `CHARSHAPE Id="28192"` 가 나온다) HML 샘플이 2개뿐
이어도 이 조사가 `Id` 범위의 근거가 된다. HML 샘플 자체 최댓값은 18 로 상한의 1/3,641.

**(b) 참조 폭 — "65,535 초과는 어차피 참조 불가"는 부분적으로만 참이다.**

| 참조 필드 | 타입 | 65,535 초과 도달 |
|---|---|---|
| `Paragraph.para_shape_id` (`src/model/paragraph.rs:13`) | `u16` | 불가 |
| `Paragraph.style_id` (`:15`) | `u8` | 불가 |
| `Style.para_shape_id` (`src/model/style.rs:444`) | `u16` | 불가 |
| `Style.char_shape_id` (`src/model/style.rs:446`) | `u16` | 불가 |
| **`CharShapeRef.char_shape_id`** (`src/model/paragraph.rs:131`) | **`u32`** | **가능** |

문단의 글자모양 런 테이블은 `u32` 로 참조하므로 **char_shapes 에 대해서는 표현 한계가
아니라 (a) 실측 기반 정책 상한**이다. 그렇기 때문에 더더욱 하드 `Err` 가 아니라 경고를
남기는 skip 이어야 한다. (제안받은 근거를 그대로 쓰지 않고 확인 후 정정했다.)

**(c) 최악 예약량** (원소 크기는 `std::mem::size_of` 실측):

| 테이블 | 원소 | 65,536칸 |
|---|---|---|
| `font_faces[lang]` | `Font` 200 B | 13.1 MB |
| `border_fills` | `BorderFill` 160 B | 10.5 MB |
| `char_shapes` | `CharShape` 120 B | 7.9 MB |
| `para_shapes` | `ParaShape` 112 B | 7.3 MB |
| `styles` | `Style` 80 B | 5.2 MB |
| `tab_defs` | `TabDef` 56 B | 3.7 MB |

## 3. 검증

빌드: `cargo test --profile release-test` (릴리스 최적화).

### 3.1 재현 측정 방법

할당량은 테스트 바이너리에 **추적용 `#[global_allocator]`** 를 끼워 측정했다 — 샘플링
RSS 가 아니라 결정론적 누계다. (PowerShell `PeakWorkingSet64` 는 이 환경에서 빈 값을
돌려줘 사용하지 않았다.) 측정용 probe 는 커밋하지 않았다.

| 입력 | 수정 전 힙 최대 | 수정 후 힙 최대 |
|---|---|---|
| `Id="0"` (376 B, 기준선) | 20,172 B | 20,246 B |
| `Id="1000000"` (382 B) | **120,009,531 B (114.5 MB)** | **20,532 B** |
| `Id="2000000000"` (385 B) | abort (240,000,000,120 B 요구) | 정상 파싱 |

- 파일이 **6바이트** 늘었을 뿐인데 수정 전에는 힙이 **+119,989,359 바이트(+114.4 MB)**
  늘었다. 입력 대비 **314,161배**, 기준선 대비 **5,949배**.
- 수정 후에는 기준선과 사실상 동일(20,532 vs 20,246)해져 **5,845배** 줄었다.
- 할당 지점 확정: `240,000,000,120 / 120(= size_of::<CharShape>()) = 2,000,000,001 = Id + 1`
  — `resize_with(index + 1, ..)` 와 정확히 일치.

### 3.2 red→green 실증 (실제 실행 · 캡처 원문)

**RED 방법**: `max_resource_id` 기본값을 `usize::MAX` 로 되돌려 가드를 무력화했다
(코드 경로는 유지하고 효과만 종전과 동일하게 만든 것). abort 케이스가 바이너리를 죽이므로
2회로 나눠 실행했다.

**RED 1차 — abort 하지 않는 테스트들**

```
running 3 tests
test hml_resource_id_beyond_limit_is_skipped_with_warning ... FAILED
test hml_resource_id_boundary_accepts_limit_and_rejects_above ... FAILED
test hml_resource_ids_within_limit_are_unchanged ... ok

---- hml_resource_id_beyond_limit_is_skipped_with_warning stdout ----
thread 'hml_resource_id_beyond_limit_is_skipped_with_warning' (25692) panicked at tests\issue_2743_hml_resource_id_limit.rs:47:5:
상한 초과 CHARSHAPE 는 테이블을 늘리지 않아야 함 (실제 1000001칸)

---- hml_resource_id_boundary_accepts_limit_and_rejects_above stdout ----
thread 'hml_resource_id_boundary_accepts_limit_and_rejects_above' (2464) panicked at tests\issue_2743_hml_resource_id_limit.rs:122:5:
Id=65536 는 건너뛰어야 함

test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.09s
```

**RED 2차 — abort 케이스 단독**

```
running 1 test
memory allocation of 240000000120 bytes failed
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
error: test failed, to rerun pass `--test issue_2743_hml_resource_id_limit`

Caused by:
  process didn't exit successfully: `...\issue_2743_hml_resource_id_limit-49304a36aa4d5693.exe --exact hml_resource_id_far_beyond_limit_does_not_abort` (exit code: 0xc0000409, STATUS_STACK_BUFFER_OVERRUN)
note: test exited abnormally; to see the full output pass --no-capture to the harness.
```

**RED 3차 — 여섯 종류 동시 단독**

```
---- hml_all_six_resource_kinds_are_bounded stdout ----
thread 'hml_all_six_resource_kinds_are_bounded' (37736) panicked at tests\issue_2743_hml_resource_id_limit.rs:97:9:
char_shapes 가 상한 초과 Id 로 늘어남 (9000001칸)

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 4 filtered out; finished in 1.93s
```

이 케이스는 abort 하지 **않았다** — 즉 **797바이트 HML 이 여섯 테이블 합계
6,552,000,568 바이트(6.10 GiB)를 아무 오류 없이 예약하고 통과**했다. 조용한 구간이
GB 규모까지 확장된다는 직접 증거다.

**GREEN — 가드 복원 후 전체 재실행**

```
running 5 tests
test hml_resource_id_far_beyond_limit_does_not_abort ... ok
test hml_all_six_resource_kinds_are_bounded ... ok
test hml_resource_id_beyond_limit_is_skipped_with_warning ... ok
test hml_resource_ids_within_limit_are_unchanged ... ok
test hml_resource_id_boundary_accepts_limit_and_rejects_above ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### 3.3 red→green 회계 (통과/실패를 테스트별로 명시)

| 테스트 | RED | GREEN |
|---|---|---|
| `hml_resource_id_beyond_limit_is_skipped_with_warning` | FAILED — "실제 1000001칸" | ok |
| `hml_resource_id_boundary_accepts_limit_and_rejects_above` | FAILED — "Id=65536 는 건너뛰어야 함" | ok |
| `hml_all_six_resource_kinds_are_bounded` | FAILED — "9000001칸" (6.10 GiB 를 오류 없이 예약) | ok |
| `hml_resource_id_far_beyond_limit_does_not_abort` | **프로세스 abort** (`0xC0000409`) | ok |
| `hml_resource_ids_within_limit_are_unchanged` | **ok — 수정 전에도 통과** | ok |

마지막 항목은 red 가 되지 않는 것이 **정상**이다. 정상 범위 `Id`(0~2)의 동작이 수정 전후
완전히 같음을 고정하는 동작 불변 가드이므로, 양쪽에서 통과해야 의미가 있다.

조용한 구간을 잡기 위해 가드는 "죽지 않음"이 아니라 **결과 테이블 길이와 경고 개수**를
단언한다. 그렇게 하지 않으면 수정 전에도 통과해 red 가 성립하지 않는다.

### 3.4 CI 3종 (실측 결과)

| 검사 | 명령 | 결과 |
|---|---|---|
| clippy | `cargo clippy --all-targets -- -D warnings` | **exit 0** |
| 테스트 | `cargo test --profile release-test --tests` | **exit 0** — 테스트 타깃 293개 전부 `ok`, `FAILED` 0개, 합계 **3,495 passed / 0 failed / 23 ignored** |
| fmt | 변경 2개 파일에 write 모드 `rustfmt --edition 2021` 적용 후 재적용 시 md5 불변 | **IDEMPOTENT** |

테스트 타깃별 주요 결과 (정상 파일 불변 확인):

```
unittests src\lib.rs              ok. 2464 passed; 0 failed; 7 ignored
issue_2743_hml_resource_id_limit  ok. 5 passed; 0 failed; 0 ignored     ← 신규
hml_parser                        ok. 34 passed; 0 failed; 0 ignored
hml_serializer                    ok. 30 passed; 0 failed; 0 ignored
hwp5_roundtrip_baseline           ok. 3 passed; 0 failed; 0 ignored
hwpx_roundtrip_baseline           ok. 4 passed; 0 failed; 0 ignored
hwpx_roundtrip_integration        ok. 22 passed; 0 failed; 0 ignored
visual_roundtrip_baseline         ok. 3 passed; 0 failed; 0 ignored
```

특히 `hml_parser` 34개·`hml_serializer` 30개가 전부 통과했다 — 기존 HML 경고 개수를
단언하는 `assert_eq!(result.warnings.len(), 6)` (`tests/hml_parser.rs:642`) 포함이라,
정상 입력에 새 경고가 끼어들지 않음이 확인된다.

fmt 는 지시대로 `cargo fmt --all -- --check` 를 쓰지 않았다 — 이 Windows 체크아웃에서는
CRLF 파일에 대해 `Incorrect newline style` 만 출력하고 diff 를 내지 않아 거짓 통과가 된다.
`src/parser/hml/reader.rs` 는 하위 모듈 선언이 없어 rustfmt 가 다른 파일로 내려가지
않음을 사전에 확인했다(#2722 때 `mod tests` 로 내려간 사례 재발 방지).

## 4. 미실행 항목 (투명 고지)

- **wasm32 실기 실행은 하지 않았다.** 이슈의 wasm32 서술(`Layout::array` 가 32비트
  `usize` 초과 → capacity overflow → 트랩)은 `usize` 폭과 원소 크기 산술에서 도출한
  것이며 관측이 아니다.
- **PeakWorkingSet(RSS) 는 측정하지 못했다.** 이 환경의 PowerShell 이 빈 값을 돌려줘,
  대신 추적 `#[global_allocator]` 의 결정론적 누계를 썼다. 보고한 수치는 RSS 가 아니라
  힙 할당 최대치다.
- `set_indexed` 가 HML 리더 전용임은 `grep -rn "set_indexed" src/` 로 확인했다
  (다른 포맷 경로에는 이 결함이 없다).

## 5. 잔여 (범위 밖)

| # | 항목 | 이번에 다루지 않는 이유 |
|---|---|---|
| 1 | `capture_border_fill` 의 `Id=0` 하드 `Err` | 기존 동작. 2.1 참조 — 성격이 다르고 바꾸면 회귀 위험 |
| 2 | `<CHARSHAPELIST Count="N">` 선언 개수와 실제 항목 수의 교차 검증 | 정합성 기능이지 할당 상한이 아니다 |
| 3 | HWP5/HWPX 의 동일 리소스 테이블 | `set_indexed` 를 쓰지 않는다. 레코드 크기 검사로 이미 유계 |
| 4 | `HmlLimits` 를 CLI/공개 API 에서 조정하는 표면 | 기본값만으로 결함이 닫힌다 |
