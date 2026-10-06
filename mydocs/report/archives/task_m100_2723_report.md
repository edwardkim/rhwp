# Task M100 #2723 처리결과 보고서 — HML 표 셀 안 글상자 문단 귀속 수정

- 이슈: [#2723](https://github.com/edwardkim/rhwp/issues/2723)
- 브랜치: `task/m100-2723-hml-nested-textbox`
- 기준: `origin/devel@49f384463905afd59e59c1b596b70281cb4a8539`
- 작성일: 2026-07-21
- 상태: 구현·검증 완료, CI 3종 통과

## 1. 범위

`src/parser/hml/reader.rs` 의 `finish_paragraph` 문단 귀속 규칙 **하나만** 고친다. 같은 파일의
`set_indexed`·`HmlLimits` 는 별건(#2722)이라 손대지 않았다. 직렬화기·preflight·어댑터·다른 포맷 파서는
변경 대상이 아니다.

## 2. 문제

HML 리더가 `</P>` 에서 문단을 어느 컨테이너에 넣을지 **컨테이너 종류 우선순위(셀 먼저)** 로 정했다.

```rust
if let Some(cell) = self.cells.last_mut() {          // 셀이 열려 있으면 무조건 셀
    cell.paragraphs.push(paragraph);
} else if let Some(rectangle) = self.rectangles.last_mut() {
    rectangle.text_box.push(paragraph);
} else { /* 구역 */ }
```

`self.cells` 와 `self.rectangles` 는 둘 다 **열린 요소 스택**이라 동시에 비어 있지 않을 수 있다. 그때
실제 부모는 스택에서 더 안쪽인 쪽인데, 위 코드는 그 순서를 보지 않는다. 그래서
`CELL > … > RECTANGLE > DRAWINGOBJECT > DRAWTEXT > PARALIST > P` 중첩 — **표 셀 안에 놓인 글상자** —
에서 글상자 문단이 셀로 흘러들었다.

`adapter.rs:158-160` 이 빈 `text_box` 를 `None` 으로 접으므로 글상자는 텍스트뿐 아니라 **개체 자체가
IR 에서 사라진다.**

## 3. 분석 — 종류 우선순위 ≠ 최내곽

같은 파일 `reader.rs:855-859` 의 `nearest_object_is_table()` 은 **동일한 문제를 이미 올바르게** 푼다.

```rust
fn nearest_object_is_table(&self) -> bool {
    let rectangle = self.stack.iter().rposition(|name| name == "RECTANGLE");
    let table = self.stack.iter().rposition(|name| name == "TABLE");
    table.is_some_and(|table_index| rectangle.is_none_or(|rect_index| table_index > rect_index))
}
```

사용처 3곳 — `capture_object_size(:809)`, `capture_object_position(:822)`, `capture_shape_object(:861)` —
가 모두 이 판정을 쓴다. `capture_shape_object` 에는 "종전엔 rectangle 만 처리해 표의 TextWrap 이 HML
재로드 시 기본값 Square 로 유실됐다"는 주석까지 남아 있다. 컨테이너 선택이 필요한 함수 4개 중
`finish_paragraph` 만 예외였다.

의도된 설계라는 근거는 다음 확인 범위에서 나오지 않았다.

| 확인 | 결과 |
|---|---|
| `git log -S "finish_paragraph" -- src/parser/hml/reader.rs` | 최초 도입 커밋 `40cecb82` 1건뿐, 이후 수정 없음 |
| `mydocs/` 전체 `finish_paragraph` / `nearest_object_is_table` | 0건 |
| 종류 우선순위 귀속을 정당화하는 계획·보고서 | 0건 |
| 동일 규칙을 다루는 열린 이슈·PR | 0건 |

`</P>`(안쪽 글상자 문단) 시점의 스택 — `end()` 가 `finish_element()` 를 호출한 **뒤에** pop 하므로
닫히는 `P` 도 포함된다.

```
HWPML BODY SECTION P TEXT TABLE ROW CELL PARALIST P TEXT RECTANGLE DRAWINGOBJECT DRAWTEXT PARALIST P
                                    ^ rposition=7                               ^ rposition=13
```

`DRAWTEXT(13) > CELL(7)`. 구분에 필요한 정보는 스택에 이미 완전히 있었고, 쓰이지 않았을 뿐이다.

## 4. 변경

### 4.1 `src/parser/hml/reader.rs` (+22 / −3)

`nearest_object_is_table()` 바로 아래에 대칭 헬퍼를 추가했다.

```rust
fn nearest_paragraph_owner_is_cell(&self) -> bool {
    let draw_text = self.stack.iter().rposition(|name| name == "DRAWTEXT");
    let cell = self.stack.iter().rposition(|name| name == "CELL");
    cell.is_some_and(|cell_index| draw_text.is_none_or(|text_index| cell_index > text_index))
}
```

`finish_paragraph` 은 **글상자가 스택상 더 안쪽일 때만** 글상자를 고르고, 나머지는 종전 순서(셀 → 구역)를
그대로 둔다.

```rust
let owner_is_cell = self.nearest_paragraph_owner_is_cell();
if let Some(rectangle) = self.rectangles.last_mut().filter(|_| !owner_is_cell) {
    rectangle.text_box.push(paragraph);
} else if let Some(cell) = self.cells.last_mut() {
    cell.paragraphs.push(paragraph);
} else { /* 구역 */ }
```

비교 대상을 `RECTANGLE` 이 아니라 `DRAWTEXT` 로 둔 이유: 사각형이 문단을 받는 자식은 `DRAWTEXT` 뿐이라
`SHAPEOBJECT`/`SHAPECOMPONENT`/`LINESHAPE` 등 형제 요소가 열려 있는 동안의 오판 여지를 없앤다.

분기 순서를 뒤집어(사각형 → 셀 → 구역) `!owner_is_cell` 판정이 참이지만 `rectangles` 가 빈 비정상 입력
(예: `RECTANGLE` 없는 떠 있는 `DRAWTEXT`)에서도 종전과 동일하게 셀로 떨어지게 했다.

동작이 바뀌는 경우는 **`CELL` 과 `DRAWTEXT` 가 동시에 열려 있고 `DRAWTEXT` 가 더 안쪽인 하나뿐**이다.

| 스택 상태 | 종전 | 변경 후 |
|---|---|---|
| `CELL` 만 | 셀 | 셀 (동일) |
| `DRAWTEXT` 만 | 글상자 | 글상자 (동일) |
| 둘 다 없음 | 구역 | 구역 (동일) |
| `CELL` 이 더 안쪽 (글상자 안 표) | 셀 | 셀 (동일) |
| **`DRAWTEXT` 가 더 안쪽 (셀 안 글상자)** | **셀 (결함)** | **글상자 (수정)** |

### 4.2 `tests/hml_parser.rs` (+66)

기존 `nested_table_layout_does_not_overwrite_enclosing_rectangle`(글상자 **안에** 표) 바로 아래에 반대
방향 픽스처와 테스트 2개를 붙였다.

- `CELL_TEXTBOX_HML` — 셀 문단 1개가 사각형을 담고, 그 사각형 `DRAWTEXT` 안에 `BOXTEXT` 문단 1개.
- `first_cell_rectangle()` — 셀 문단 수가 1인지 단언하며 셀 안 사각형을 꺼내는 공용 헬퍼.
- `textbox_inside_table_cell_keeps_its_own_paragraphs` — 파싱 직후 `text_box` 가 `Some` 이고 문단
  `"BOXTEXT"` 1개.
- `textbox_inside_table_cell_survives_hml_export_and_reopen` — `export_hml_native()` 산출 XML의
  `<DRAWTEXT>` 가 1개이고, 되읽은 IR에서도 글상자가 살아 있음.

## 5. 검증

### 5.1 실측 재현 (기준 `origin/devel@49f38446`, 수정 전)

저장소 실제 한컴 파일 `samples/hml/formatting_table.hml` 의 첫 CELL PARALIST 에
`RECTANGLE/DRAWINGOBJECT/DRAWTEXT/PARALIST/P`(`<CHAR>BOXTEXT</CHAR>`) 하나를 주입해
`./target/debug/rhwp.exe dump` 를 돌렸다. 이 파일은 구역 직속 사각형(글상자 `"textbox"`)도 함께 갖고 있어
정상 경로와 결함 경로를 한 실행에서 대조할 수 있다.

구역 직속 사각형 — 정상:

```
--- 문단 0.0 --- cc=15, text_len=6, controls=1
  [0]   [사각형] round=0%
    글상자: list_attr=0x00000000, margins=(283,283,283,283), max_width=6611, paras=1
      p[0]: ps_id=0, cc=8, text="textbox", ls_count=0, ctrls=0
```

표 셀 안의 사각형 — 결함:

```
  [0]   셀[0] r=0,c=0 rs=1,cs=1 h=282 w=41956 pad=(510,510,141,141) valign=Center aim=true hdr=false bf=3 paras=3 text="table|BOXTEXT|"
  [0]     p[0] ps_id=0 ctrls=0 text_len=5
  [0]     p[1] ps_id=0 ctrls=0 text_len=7      <- 유입된 "BOXTEXT"
  [0]     p[2] ps_id=0 ctrls=1 text_len=0      <- 사각형을 담은 문단 (순서 역전)
  [0]       ctrl[0] 사각형: tac=true, wrap=Square
```

수정 후 같은 파일:

```
  [0]   셀[0] r=0,c=0 rs=1,cs=1 h=282 w=41956 pad=(510,510,141,141) valign=Center aim=true hdr=false bf=3 paras=2 text="table|"
  [0]     p[0] ps_id=0 ctrls=0 text_len=5
  [0]     p[1] ps_id=0 ctrls=1 text_len=0
  [0]       ctrl[0] 사각형: tac=true, wrap=Square
```

`paras=3 → 2`, 셀 텍스트 `"table|BOXTEXT|" → "table|"`.

### 5.2 export → 재열기 왕복 — 실측

`./target/debug/rhwp.exe export-hml <입력> -o <출력>` 을 같은 파일에 대해 수정 전/후로 각각 실행했다.

| | 수정 전 | 수정 후 |
|---|---|---|
| 종료 코드 | 0 (거부 없음) | 0 |
| 출력 `<DRAWTEXT>` 태그 | `['<DRAWTEXT>']` — **1개** | `['<DRAWTEXT>', '<DRAWTEXT>']` — 2개 |
| 셀 문단 수 | 3 (`table`, `BOXTEXT`, 사각형) | 2 (`table`, 사각형) |
| `BOXTEXT` 위치 | 셀 문단으로 승격 | `DRAWTEXT/PARALIST/P` 내부 |

수정 전 출력 CELL(요약) — `<DRAWTEXT>` 블록이 통째로 사라졌다.

```xml
<CELL …><PARALIST>
  <P …><TEXT CharShape="0"><CHAR>table</CHAR></TEXT></P>
  <P …><TEXT CharShape="0"><CHAR>BOXTEXT</CHAR></TEXT></P>
  <P …><TEXT CharShape="0"><RECTANGLE …><DRAWINGOBJECT>
    <SHAPECOMPONENT …/><LINESHAPE …/>
  </DRAWINGOBJECT></RECTANGLE></TEXT></P>
</PARALIST></CELL>
```

수정 후 출력 CELL(요약) — 원문 구조가 보존된다.

```xml
<CELL …><PARALIST>
  <P …><TEXT CharShape="0"><CHAR>table</CHAR></TEXT></P>
  <P …><TEXT CharShape="0"><RECTANGLE …><DRAWINGOBJECT>
    <SHAPECOMPONENT …/><LINESHAPE …/>
    <DRAWTEXT><TEXTMARGIN …/><PARALIST>
      <P …><TEXT CharShape="0"><CHAR>BOXTEXT</CHAR></TEXT></P>
    </PARALIST></DRAWTEXT>
  </DRAWINGOBJECT></RECTANGLE></TEXT></P>
</PARALIST></CELL>
```

즉 왕복 파괴는 **코드 확인이 아니라 측정으로 확인**됐고, 수정으로 해소됐다.

### 5.3 red → green (실제 실행 캡처)

수정 코드만 되돌리고(테스트는 그대로) 실행한 결과 — 원문 그대로.

```
running 3 tests
test nested_table_layout_does_not_overwrite_enclosing_rectangle ... ok
test textbox_inside_table_cell_keeps_its_own_paragraphs ... FAILED
test textbox_inside_table_cell_survives_hml_export_and_reopen ... FAILED

failures:

---- textbox_inside_table_cell_keeps_its_own_paragraphs stdout ----

thread 'textbox_inside_table_cell_keeps_its_own_paragraphs' (39816) panicked at tests\hml_parser.rs:867:5:
assertion `left == right` failed: 셀 문단은 사각형을 담은 1개뿐이어야 한다
  left: 2
 right: 1
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- textbox_inside_table_cell_survives_hml_export_and_reopen stdout ----

thread 'textbox_inside_table_cell_survives_hml_export_and_reopen' (38400) panicked at tests\hml_parser.rs:901:5:
assertion `left == right` failed
  left: 0
 right: 1


failures:
    textbox_inside_table_cell_keeps_its_own_paragraphs
    textbox_inside_table_cell_survives_hml_export_and_reopen

test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 33 filtered out; finished in 0.01s
```

수정 복구 후 같은 명령 — 원문 그대로.

```
running 3 tests
test textbox_inside_table_cell_keeps_its_own_paragraphs ... ok
test nested_table_layout_does_not_overwrite_enclosing_rectangle ... ok
test textbox_inside_table_cell_survives_hml_export_and_reopen ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 33 filtered out; finished in 0.01s
```

`left: 0 / right: 1` 은 `xml.matches("<DRAWTEXT>").count()` 로, 수정 전 직렬화기가 `<DRAWTEXT>` 를
**하나도** 쓰지 않았음을 뜻한다.

### 5.4 반대 방향 중첩 비회귀

`nested_table_layout_does_not_overwrite_enclosing_rectangle`(글상자 **안에** 표, `tests/hml_parser.rs`)
는 **red 실행과 green 실행 양쪽에서 모두 `ok`** 다(위 캡처 1행/2행). 이 방향은 `</CELL>` 이 이미 닫혀
`cells` 가 비어 있어 종전에도 정상이었고, 이번 변경 후에도 `rposition(CELL) > rposition(DRAWTEXT)` 로
동일하게 셀이 선택된다. 한 방향을 고치면서 반대 방향을 깨지 않았다.

`tests/hml_parser.rs` 전체도 통과한다 — `test result: ok. 36 passed; 0 failed; 0 ignored`.

### 5.5 CI 3종 (최종 트리 기준, 커밋 내용과 동일)

| 항목 | 명령 | 결과 |
|---|---|---|
| clippy | `cargo clippy --all-targets -- -D warnings` | `Finished dev profile … in 55.90s` — 경고 0 |
| test | `cargo test --profile release-test --tests` | **3482 passed, 0 failed, 23 ignored** (`test result:` 291줄, `FAILED` 0줄) |
| fmt | 변경 `.rs` 2개에 `rustfmt --edition 2021` 후 재실행 | md5 불변(멱등), 추가 변경 0 |

fmt 는 `cargo fmt --all -- --check` 를 쓰지 않았다. 이 Windows 체크아웃에서는 CRLF 파일에 대해
`Incorrect newline style` 만 출력하고 diff 를 내지 않아 거짓 통과가 된다.

## 6. 미실행 항목

- **렌더링 픽셀 대조.** IR 과 HML 직렬화 산출물까지만 측정했다. `export-svg`/`export-pdf` 결과를 한컴
  뷰어 출력과 픽셀 비교하지는 않았다.
- **한컴에서 저장한 "셀 안 글상자" 원본 파일 확보.** 저장소 실제 한컴 파일(`formatting_table.hml`)에
  해당 중첩을 주입해 재현했다. 한컴이 직접 저장한 동일 구조 원본으로는 대조하지 못했다.
- **HWP5/HWPX 경로 회귀.** 변경은 HML 리더 안에서 끝나므로 다른 포맷은 영향을 받지 않지만, 두 포맷의
  동일 중첩을 별도로 측정하지는 않았다.

## 7. 잔여 (범위 밖)

- **HWPX/HWP5 리더의 동일 패턴 점검.** 다른 포맷 리더가 같은 종류 우선순위 귀속을 쓰는지 미확인.
- **캡션 등 다른 문단 컨테이너.** 현행 HML 리더가 `CAPTION` 문단을 모델링하지 않아 당장 증상은 없으나,
  추가될 경우 같은 판정이 필요하다.
- **`export-hml` preflight 강화.** 이번 수정으로 왕복은 보존되지만, 구조 오귀속을 preflight
  (`validate_cell` → `validate_paragraph` → `validate_rectangle`)가 잡지 못한다는 사실은 남는다. 차단기
  추가는 별도 논의가 필요하다.
- **`dump` 출력의 셀 내부 컨트롤 상세도.** 셀 안 사각형은 한 줄 요약(`ctrl[0] 사각형: …`)만 나와 글상자
  유무가 보이지 않는다. 이번 결함을 CLI 만으로 알아채기 어려웠던 이유이며, 출력 보강은 별건이다.
