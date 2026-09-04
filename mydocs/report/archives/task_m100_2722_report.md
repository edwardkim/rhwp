# task_m100_2722 처리결과 보고서 — 표 그리드 재구축 무한 할당 차단

- **이슈**: [#2722](https://github.com/edwardkim/rhwp/issues/2722)
- **브랜치**: `task/m100-2722-parser-robustness` (base `origin/devel` @ `49f38446`)
- **범위**: `src/model/table.rs`, `tests/issue_2722_table_grid_alloc.rs`
- **분류**: 결함 수정 (malformed 입력에 대한 파서 robustness — 무검증 할당)

## 1. 문제

`Table::rebuild_grid()` (`src/model/table.rs:513-516`, 수정 전) 이 파일에서 그대로 온
`row_count`/`col_count` 를 검증 없이 곱해 그리드를 예약했다.

```rust
pub fn rebuild_grid(&mut self) {
    let rc = self.row_count as usize;
    let cc = self.col_count as usize;
    self.cell_grid = vec![None; rc * cc];   // 상한 없음
    for (idx, cell) in self.cells.iter().enumerate() {
        ...
                if gi < self.cell_grid.len() {   // 쓰기에는 가드가 이미 있었다
```

두 필드 모두 `u16` 이므로 최대 곱은 `65535 × 65535 = 4,294,836,225` 칸이고,
원소 `Option<usize>` 가 x86_64 에서 16바이트이므로 **68,717,379,600 바이트(약 64 GiB)**
예약을 시도한다. `Vec` 은 할당 실패를 `Err` 로 돌려주지 않고 `handle_alloc_error` →
`abort()` 로 프로세스를 죽인다. `catch_unwind` 로도 잡히지 않는다.

`rebuild_grid()` 는 HWPX·HWP5·HML·HWP3 네 파서가 전부 지나는 **단일 chokepoint** 다.

| 포맷 | 값 출처 | `rebuild_grid()` 호출 | 파일이 직접 지정? |
|---|---|---|---|
| HWPX | `src/parser/hwpx/section.rs:1601-1602` (`rowCnt`/`colCnt` XML 속성) | `section.rs:1840` | 예 |
| HWP5 | `src/parser/control.rs:262-263` (`HWPTAG_TABLE` `read_u16()`) | `control.rs:244` | 예 |
| HML | `src/parser/hml/reader.rs:786-787` (`RowCount`/`ColCount`) | `hml/adapter.rs:262` | 예 |
| HWP3 | `src/parser/hwp3/mod.rs:680-688` (셀 좌표 집합에서 **유도**) | `hwp3/mod.rs:818` | 아니오 |

`if gi < self.cell_grid.len()` 가드가 **루프 안쪽에는 이미 있었다**는 점이 이 결함의
성격을 보여준다 — 인덱스는 의심했으나 그보다 먼저 실행되는 할당 크기는 검증 대상에서
빠져 있었다.

## 2. 분석

### 2.1 상위 경계가 막지 못하는 이유

- `row_count`/`col_count` 는 셀 데이터와 독립된 스칼라다. HWPX 는 4~5바이트 XML 속성,
  HWP5 는 `HWPTAG_TABLE` 안의 2바이트 필드 2개다. `record.rs` 의 레코드 크기 검사를
  완벽히 통과한다 — **입력 크기와 할당 크기 사이에 비례관계가 전혀 없다**.
- `HmlLimits` (`src/parser/hml/reader.rs:39-54`) 는 XML 바이트·깊이·속성 수·텍스트
  길이만 본다. 아래 250바이트 입력은 어떤 상한에도 걸리지 않는다.
- ZIP 폭탄 방어와도 무관하다. 2.3 재현은 압축률을 건드리지 않은 정상 ZIP 이다.
- `cells` 가 비어 있어도 무관하다. 그리드 크기는 선언된 행·열 수에만 의존한다.

### 2.2 같은 클래스를 이미 막고 있는 대조 지점

저장소는 "파일에서 온 count/size 를 할당 인자로 쓰기 전 상한" 정책을 이미 구현하고 있다.
표 그리드만 누락이었다.

| 지점 | 방식 | 주석이 밝힌 종전 규모 |
|---|---|---|
| `src/parser/record.rs:59-73` | `checked_add` + 경계 검사 | wasm32 랩어라운드 → 4GB |
| `src/parser/doc_info.rs:652-658` | `.min(r.remaining() / 8)` | ~34GB |
| `src/parser/control/shape.rs:980-985` | `.min(r.remaining() / 10)` | ~51GB |
| `src/parser/hwp3/mod.rs:60-78` | `check_record_count()` (#877, 256 MiB) | 3.69GB |

표 그리드의 **68GB** 는 이 중 최대이며, 유일하게 네 포맷 공통 경로다.

### 2.3 wasm32 관점

wasm32 는 `usize` 가 32비트다.

- `rc * cc = 4,294,836,225` 는 `u32::MAX = 4,294,967,295` 보다 **131,070 작다**. 곱셈
  자체는 아슬아슬하게 랩어라운드하지 않는다(만약 두 필드가 `u32` 였다면 `record.rs:59-73`
  과 동일한 랩어라운드 형태가 됐을 것이다).
- 그러나 `Layout::array::<Option<usize>>(4_294_836_225)` 는 `× 8 = 34,358,689,800` 으로
  32비트 `usize` 를 넘겨 `capacity overflow` **패닉**이 된다. wasm 에서 패닉은
  `unreachable` 트랩이므로 **모듈 인스턴스 전체가 사용 불능**이 된다.

## 3. 변경

`src/model/table.rs` 한 파일, 3개 변경.

1. **상한 상수 신설** (`table.rs:7-14`)
   ```rust
   pub const MAX_TABLE_GRID_CELLS: usize = 4_000_000;
   ```
   실측 근거는 4.4 참조 (정상 문서 최댓값의 75.8배).

2. **`rebuild_grid()` 에 가드** (`table.rs:531-545`)
   ```rust
   let requested = rc.saturating_mul(cc);
   let grid_len = if requested > MAX_TABLE_GRID_CELLS {
       self.addressed_grid_len(cc).min(MAX_TABLE_GRID_CELLS).min(requested)
   } else {
       requested
   };
   self.cell_grid = vec![None; grid_len];
   ```
   `requested <= MAX_TABLE_GRID_CELLS` 이면 `grid_len == rc * cc` 로 **종전 식과 동일**.
   분기 자체가 실행되지 않는다.

3. **`addressed_grid_len()` 신설** (`table.rs:558-578`)
   셀이 실제로 가리키는 마지막 그리드 인덱스 + 1 을 `saturating_*` 로만 계산한다.
   셀 0개면 0. 상한만 걸 경우 표 하나당 최대 64 MiB 를 여전히 허용해 "빈 표를 여러 개"
   변형이 남는데, 이 축소를 먼저 적용하면 그 여지가 사라진다.

### 3.1 설계 선택 근거

- **`row_count`/`col_count` 는 건드리지 않는다.** 모델 필드를 변형하면 직렬화·라운드트립
  계약에 영향이 갈 수 있다. 길이만 줄이고 `col_count`(stride)는 그대로 두므로
  `cell_index_at()`/`cell_at()` 은 범위 안 셀을 정확히 그대로 조회하고, 범위 밖은
  `Vec::get()` 이 `None` 을 준다 — 루프 안쪽 기존 가드와 동일한 의미다.
- **검토 후 채택하지 않은 안**: (a) `row_count`/`col_count` 를 셀이 쓰는 범위로 낮추기 —
  단일 셀이 `row_span=65535, col_span=65535` 를 선언하면 축소 목표가 다시 65535×65535 가
  되어 결국 하드 상한이 어차피 필요하다. (b) `rebuild_grid()` 를 `Result` 로 변경 —
  호출처가 4개 파서 + 편집 명령 다수라 전면 수정이 필요하고, 손상된 표 하나로 문서 전체
  파싱을 실패시키는 것은 기존 `unwrap_or(0)` graceful 파싱 방침과 어긋난다.

## 4. 검증

빌드 환경: `cargo test --profile release-test` (릴리스 최적화, `overflow-checks` 없음).

### 4.1 신규 테스트 — `tests/issue_2722_table_grid_alloc.rs` (4개)

| 테스트 | 내용 |
|---|---|
| `rebuild_grid_bounds_hostile_row_col_count` | 셀 0개 65535×65535 → 그리드 0칸, `row_count`/`col_count` 는 65535 그대로 |
| `rebuild_grid_bounds_hostile_counts_with_one_cell` | (0,0) 셀 1개 65535×65535 → 그리드 1칸, 앵커 인덱스 보존 |
| `hml_table_with_hostile_counts_parses_without_abort` | 250바이트 HML(`RowCount="65535" ColCount="65535"`) → `Ok`, 그리드 상한 이하 |
| `normal_table_grid_size_is_unchanged` | 정상 3×4 표 → 그리드 정확히 12칸, 12칸 전부 조회 성공 |

### 4.2 red→green 실증 (실제 실행 · 캡처 원문)

**RED** — `rebuild_grid()` 의 가드를 종전 `self.cell_grid = vec![None; rc * cc];` 로
되돌리고 동일 테스트 실행:

```
running 4 tests
memory allocation of 68717379600 bytes failed
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test normal_table_grid_size_is_unchanged ... error: test failed, to rerun pass `--test issue_2722_table_grid_alloc`

Caused by:
  process didn't exit successfully: `C:\Users\swsz9\Downloads\moneyflow\rhwp-wt-g\target\release-test\deps\issue_2722_table_grid_alloc-4b9540f9bd1bc640.exe` (exit code: 0xc0000409, STATUS_STACK_BUFFER_OVERRUN)
note: test exited abnormally; to see the full output pass --no-capture to the harness.
```

**이것은 일반적인 테스트 실패가 아니다.** `0xC0000409` 는 Windows 에서 Rust `abort()` 가
보고되는 코드이며, 정상 표만 검사하는 `normal_table_grid_size_is_unchanged` 조차 진행 중
함께 죽었다 — 바이너리 전체가 사망한다.

**GREEN** — 가드를 복원하고 동일 명령 재실행:

```
running 4 tests
test rebuild_grid_bounds_hostile_row_col_count ... ok
test normal_table_grid_size_is_unchanged ... ok
test rebuild_grid_bounds_hostile_counts_with_one_cell ... ok
test hml_table_with_hostile_counts_parses_without_abort ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### 4.3 추가 실측 — 실제 한컴 HWPX (커밋하지 않은 임시 재현)

`samples/3-09월_교육_통합_2022.hwpx` (4,113,254 바이트) 의 `Contents/section0.xml` 에서
**첫 번째 `<hp:tbl>` 태그의 속성 2개만** 바꾸고 ZIP 재패킹:

```
before: <hp:tbl id="1587285471" ... repeatHeader="1" rowCnt="2"     colCnt="4"     ...
after : <hp:tbl id="1587285471" ... repeatHeader="1" rowCnt="65535" colCnt="65535" ...
```

수정 전 `rhwp::parser::parse_document()`:

```
hwpx bytes = 4113254
memory allocation of 68717379600 bytes failed
...
  process didn't exit successfully: `...\wtg_repro_tblgrid-91198fa859c6fda8.exe --nocapture --exact repro_hwpx_patched` (exit code: 0xc0000409, STATUS_STACK_BUFFER_OVERRUN)
```

수정 후 동일 파일:

```
hwpx bytes = 4113254
hwpx parse ok = true
test repro_hwpx_patched ... ok
```

이 재현 파일은 스크래치 경로에 의존하므로 회귀 가드로 커밋하지 않았다. 커밋된 가드는
경로 의존이 없는 4.1 의 4개다.

### 4.4 상한값 실측 근거

`samples/` 전수 조사(중첩 표 포함, 이번 작업 중 직접 실행):

```
SURVEY files_parsed=343 tables=5353
SURVEY max_cells=52770 max_row_count=5277 max_col_count=10 file=issue2063_huge_cellbreak_table.hwp
```

- 실제 문서 5,353개 표 중 `row_count × col_count` 최댓값 **52,770** (5,277행 × 10열).
  파일명부터 `huge_cellbreak_table` 인 극단 케이스다.
- 상한 `4,000,000` 은 실측 최댓값의 **75.8배**. 정상 문서는 상한 분기에 닿지 않으므로
  **동작이 비트 단위로 불변**이다.
- 최악의 경우 예약량은 `4,000,000 × 16B = 64 MiB` (wasm32 는 8B → 32 MiB).

### 4.5 CI 3종 (실측 결과)

| 검사 | 명령 | 결과 |
|---|---|---|
| clippy | `cargo clippy --all-targets -- -D warnings` | **exit 0** (`Finished dev profile`) |
| 테스트 | `cargo test --profile release-test --tests` | **exit 0** — 테스트 타깃 292개 전부 `ok`, `FAILED` 0개, 합계 **3,484 passed / 0 failed / 23 ignored** |
| fmt | 변경 파일에 `rustfmt --edition 2021` 적용 후 재적용 시 md5 불변 확인 | **IDEMPOTENT** (두 파일 모두 해시 동일) |

테스트 타깃별 주요 결과:

```
unittests src\lib.rs          ok. 2464 passed; 0 failed; 7 ignored
issue_2722_table_grid_alloc   ok. 4 passed; 0 failed; 0 ignored     ← 신규
hwp5_roundtrip_baseline       ok. 3 passed; 0 failed; 0 ignored
hwpx_roundtrip_baseline       ok. 4 passed; 0 failed; 0 ignored
hwpx_roundtrip_integration    ok. 22 passed; 0 failed; 0 ignored
hwpx_form_roundtrip           ok. 1 passed; 0 failed; 0 ignored
hml_parser                    ok. 34 passed; 0 failed; 0 ignored
hml_serializer                ok. 30 passed; 0 failed; 0 ignored
visual_roundtrip_baseline     ok. 3 passed; 0 failed; 0 ignored
```

fmt 는 지시대로 `cargo fmt --all -- --check` 를 쓰지 않았다 — 이 Windows 체크아웃에서는
CRLF 파일에 대해 `Incorrect newline style` 만 출력하고 diff 를 내지 않아 **거짓 통과**가
된다. 대신 write 모드 `rustfmt` 를 적용한 뒤 재적용해도 md5 가 변하지 않음을 확인했다
(`src/model/table.rs` = `de5a2111...`, `tests/issue_2722_table_grid_alloc.rs` = `a9fdc62c...`).

### 4.6 정상 파일 불변 확인

- 4.4 실측대로 정상 문서 최대치(52,770)는 상한(4,000,000)의 1/75.8 이므로 새 분기가
  **실행되지 않는다**. `grid_len == rc * cc` 로 종전 식과 동일하다.
- `tests/` 의 실파일 라운드트립 가드가 **전부 통과**했다 — `hwp5_roundtrip_baseline` 3,
  `hwpx_roundtrip_baseline` 4, `hwpx_roundtrip_integration` 22, `hwpx_form_roundtrip` 1,
  `hml_serializer` 30, `hml_parser` 34, `visual_roundtrip_baseline` 3 (모두 0 failed).
  즉 정상 파일의 파싱→직렬화 왕복 바이트와 시각 baseline 이 종전과 동일하다.
- `row_count`/`col_count` 를 변형하지 않으므로 직렬화 입력은 파싱 결과 그대로다.

## 5. 미실행 항목 (투명 고지)

- **HWP5 (`.hwp`) 는 코드 확인만 했고 실측하지 않았다.** `src/parser/control.rs:262-263`
  이 `HWPTAG_TABLE` 에서 `read_u16()` 두 개를 그대로 대입하고 `:244` 에서
  `rebuild_grid()` 를 호출하므로 경로는 HWPX·HML 과 동일하지만, CFB + deflate 스트림을
  편집할 도구가 없어 악성 바이트를 실제로 만들지 못했다. **HWP5 에서 재현했다고 주장하지
  않는다.**
- HWP3 는 `row_count`/`col_count` 가 셀 좌표 집합에서 유도되므로 파일이 임의값을 직접
  지정할 수 없다 — 재현 시도 대상에서 제외했다.
- wasm32 실기 실행은 하지 않았다. 2.3 의 wasm32 서술은 `usize` 폭과 `Layout::array` 의
  산술로부터 도출한 것이며, 관측이 아니다.

## 6. 잔여 (범위 밖)

이슈 #2722 의 7장과 동일. 각각 별도 이슈감이며 이번 PR 에서 건드리지 않았다.

| # | 지점 | 내용 | 이번에 손대지 않는 이유 |
|---|---|---|---|
| 1 | `src/serializer/hml/body.rs:422`, `src/serializer/hwpx/table.rs:104` | `for row in 0..table.row_count` 중첩 루프 — 손상된 표가 파싱을 통과하면 저장 시 약 43억 회 반복 → 사실상 hang | 저장 경로 결함으로 원인이 다르다. 본 수정은 abort 제거(문서를 열 수 있게 하는 것)에 한정 |
| 2 | `src/model/table.rs` 채우기 루프 `cell.row..(cell.row + cell.row_span)` | `u16` 덧셈 오버플로. `release-test` 는 `overflow-checks` 가 꺼져 wrap 하고 `if gi < len` 가드가 막지만 debug 빌드에서는 패닉 | `release-test` 에서 관측 불가라 red→green 으로 증명할 수 없다 |
| 3 | `src/parser/hml/reader.rs:1504` `set_indexed()` | HML `Id` 속성이 `Vec::resize_with` 크기를 직접 지정 | 별도 결함 클래스(인덱스 상한 부재). 별도 이슈로 분리 |
| 4 | HWP5 실측 | 5장 참조 | CFB+deflate 편집 도구 부재 |
