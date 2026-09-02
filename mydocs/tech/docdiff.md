---
kind: reference
status: active
canonical: mydocs/tech/docdiff.md
last_verified: 2026-08-16
---

# 문서 의미 diff 엔진 — `src/docdiff/`

두 `Document` IR 의 **구조적·의미적 차이**를 계산하는 재사용 라이브러리다. 회귀 검증,
왕복 시험, 편집 전후 확인, 변환 손실 설명이 전부 같은 질문 하나를 던진다 — *"두 문서가
어떻게 다른가."* 그 답을 **라이브러리로 부를 수 있게** 만든 것이 이 모듈이다.

ROADMAP.md 는 업스트림 책임에 "공통 품질과 운영 | 호환성 원칙, 재현용 샘플, **회귀·시각
검증**, 보안 수정, 문서와 공식 릴리스"를 적어 두었다(`ROADMAP.md` 192행). 회귀 검증이
근거를 대려면 "어디가 달라졌나"를 **값으로** 돌려주는 층이 필요하다.

관련 소스: `src/docdiff/mod.rs`(공개 표면·경계 문서), `model.rs`(결과 타입),
`compare.rs`(비교 엔진), `summary.rs`(집계).

## 1. 중복 조사 — 기존 장치와의 경계

새 모듈을 짓기 전에 저장소에 이미 있는 비교 장치를 전수 확인했다. **결론: 재사용 가능한
라이브러리 함수가 이미 하나 있고, 그것은 이 모듈과 층이 다르다.**

### 1.1 실측

| 장치 | 위치 | 소비 가능성 |
| --- | --- | --- |
| `roundtrip::diff_documents(a, b) -> IrDiff` | `src/serializer/hwpx/roundtrip.rs:731` | **공개 라이브러리 함수** — 이미 재사용 가능 |
| `roundtrip_ir_diff(bytes)` | `src/serializer/hwpx/roundtrip.rs:492` | 공개. 위 함수의 왕복 구동부 |
| `ir_diff(args) -> i32` (CLI `ir-diff`) | `src/main.rs:16670`, 디스패치 `src/main.rs:385` | **바이너리 안의 비공개 함수** — 라이브러리에서 못 부름 |
| `ir_diff_paragraph_fields(...)` | `src/main.rs:15871` | 비공개. CLI 의 문단 비교 본체 |
| `IrDiffEmitter` | `src/main.rs:15823` | 비공개. 출력 가드 + 카테고리 집계 |
| `render-diff` (`diff_render_geometry`) | `src/diagnostics/render_geom_diff.rs:441`, 디스패치 `src/main.rs:392` | 공개. **렌더 기하** 축이라 IR 비교가 아니다 |
| `hwpx-roundtrip` 배치 | `src/diagnostics/hwpx_roundtrip_batch.rs` | 공개. `roundtrip_ir_diff` 의 배치 구동부 |

### 1.2 그래서 무엇이 이미 있고, 무엇이 없나

**있다.** `roundtrip::diff_documents` 는 `(&Document, &Document) -> IrDiff` 시그니처의
공개 함수다. 이름까지 같으므로 착각하기 쉽다. 그러나 그것이 **묻는 질문**은 다르다 —
"저장했다 다시 읽었을 때 원본이 그대로 살아 있나"다. 비교 대상은 리소스 개수,
문단별 `char_shapes` 시퀀스, `line_segs` 9 필드, 섹션 `PageDef`·`visibility`, 표·개체
캡션, 필드 `parameters`, 그림 크기, `page_break`/`holdAnchorAndSO`/`flowWithText` 같은
직렬화 보존 플래그다. 직렬화기 게이트로서는 정확히 옳다.

**없다.** 그 함수는 **문단 텍스트를 아예 비교하지 않는다**(`roundtrip.rs` 전역에서
`Paragraph::text` 비교 없음 — 모듈 머리주석의 "Stage 1~5에서 비교 대상 필드를 누적
확장한다 (문단 텍스트 …)"가 아직 미도달인 상태다). 사람이 "문서가 어떻게 바뀌었나"를
물을 때 가장 먼저 알고 싶은 것이 빠져 있다.

그 자리를 CLI `ir-diff` 가 메우고 있다. 텍스트·`char_offsets`·`para_shape_id`·탭·
`line_segs`·컨트롤·표(`diff_table`)·`ParaShape`·`TabDef` 를 비교한다. 문제는 **그것이 전부
`src/main.rs` 안의 비공개 함수**라는 점이다. 라이브러리 소비자(다른 진단 모듈, MCP 도구,
회귀 테스트, `rhwp-agent`)는 한 줄도 못 쓴다. 게다가 결과가 값이 아니라 **출력 문자열**
이다 — `--json` 의 카테고리 집계마저 이미 만든 출력 줄의 앞부분을 되파싱해서
(`IrDiffEmitter::diff`) `BTreeMap<String, u32>` 를 만든다.

### 1.3 결정적 결함 — 자리끼리 맞대기

두 기존 장치는 **똑같은 방식으로 문단을 짝짓는다**: `a.paragraphs[i]` 대
`b.paragraphs[i]`, 남는 것은 개수 차이 한 줄
(`roundtrip.rs:772-782`, `main.rs:16825-16848`).

그래서 **문단 하나가 앞에 끼어들면 뒤따르는 문단 전부가 "달라졌다"로 보고된다.** 문단
200 개짜리 문서 맨 앞에 머리말 한 줄을 넣으면 차이 201 건이 나온다. 저장기 회귀
게이트로서는 무해하다(어차피 0 건이어야 하니까). 그러나 편집·변환 결과를 사람이나
에이전트에게 **설명**하는 자리에서는 그 잡음이 곧 쓸모없음이다.

### 1.4 경계 — 왜 둘 다 필요한가

| | 무엇을 묻나 | 실패의 뜻 |
| --- | --- | --- |
| `roundtrip::diff_documents` (충실도) | 한 비트라도 잃지 않았나 | 저장기 결함 |
| `rhwp ir-diff` (CLI, 필드 충실도) | 두 파일의 IR 필드가 다른가 | 변환 손실 후보 |
| **`docdiff`** (의미) | 사람이 보기에 무엇이 바뀌었나 | 내용이 달라졌다 |

충실도 게이트는 **관대하면 안 된다** — `line_segs` 한 칸이 어긋나면 한글이 본문을 통째로
버리므로 실패여야 한다. 의미 diff 는 **엄격하면 안 된다** — 문단 하나 넣었는데 201 건을
내면 아무도 안 읽는다. 두 요구는 한 함수에 담기지 않는다. 그래서 이 모듈은 셋째 장치가
아니라 **한 단계 위의 층**이다.

## 2. 공개 표면

```rust
pub fn diff_documents(a: &Document, b: &Document, opts: &DiffOptions) -> DocumentDiff;

pub struct DiffOptions { pub ignore_whitespace: bool, pub max_findings: Option<usize> }
pub struct DocumentDiff { pub identical: bool, pub findings: Vec<Finding>, pub truncated: bool }
pub struct Finding { pub path: NodePath, pub kind: FindingKind, pub detail: String }
pub enum FindingKind { SectionCountChanged, ParagraphAdded, ParagraphRemoved, TextChanged,
                       ParagraphStyleChanged, TableShapeChanged, ControlCountChanged,
                       ControlKindChanged, StyleCountChanged, StyleChanged }
pub struct NodePath { /* Vec<PathStep> */ }
pub enum PathStep { Section(usize), Paragraph(usize), Control(usize),
                    TableCell { row: u16, col: u16 }, Style(usize) }
pub struct DiffSummary { pub total: usize, pub truncated: bool,
                         pub by_kind: Vec<(FindingKind, usize)> }

impl DocumentDiff { pub fn summary(&self) -> DiffSummary; pub fn to_json(&self) -> Value; }
```

### 2.1 원 스케치에서 바꾼 것과 이유

- **`NodePath` 를 `Vec<PathStep>` 으로.** 문자열 경로(`ir-diff` 와 `roundtrip` 의
  `path: String`)는 소비자가 되파싱해야 한다. 타입이면 `path.section()` 으로 꺼낸다.
  표시형(`sec[0]/para[3]/ctrl[1]/cell[r2,c0]/para[0]`)은 `Display` 로 제공한다.
- **`FindingKind` 에 `ParagraphStyleChanged`·`ControlKindChanged`·`StyleCountChanged`
  추가.** 스케치의 `StyleChanged` 하나로는 "문단이 가리키는 모양이 바뀜"과 "스타일 정의
  자체가 바뀜"을 구별할 수 없었다. `ControlKindChanged` 는 표가 그림으로 바뀌는 것 같은
  변환 사고를 개수 차이와 분리한다.
- **`FindingKind::ALL` 과 `label()` 추가.** 집계 순서의 단일 출처이자 봉투의 안정 키다.
- **`to_json()` 추가(직렬화 계층).** CLI·MCP 가 봉투에 그대로 끼우도록. 스키마 버전은
  일부러 안 붙인다 — 봉투의 주인은 이 엔진이 아니라 명령이다.

## 3. 결과 계약과 불변식

1. **`identical == true` 이면 `findings` 는 비어 있고 `truncated` 도 `false` 다.**
   구현은 `identical = findings.is_empty() && !truncated` 다. `max_findings: Some(0)` 에
   차이가 있는 문서를 넣어도 `identical` 로 거짓말하지 않는다(테스트로 고정).
2. **`truncated == true` 는 "정말로 더 있었다"를 뜻한다.** 상한에 걸려도 **순회는 끝까지
   한다.** 조기 탈출하면 `truncated` 가 "더 있었을 수도"라는 약한 말이 된다.
3. **결정적이다.** 순회는 문서 순서(구역 → 문단 → 컨트롤 → 셀, 끝으로 문서 정보)로
   고정이고, LCS 되짚기는 갈림길에서 언제나 삭제를 먼저 고른다(`compare.rs` 의 `>=`).
   `HashMap` 순회에 기대는 곳이 **한 군데도 없다** — 집계는 `FindingKind::ALL` 배열을
   돈다. 같은 두 문서는 몇 번을 돌려도 같은 결과·같은 순서다(테스트로 고정).
4. **`DiffSummary` 는 보고된 것의 회계다.** 상한에 버려진 차이는 세지 않고, 그 사실은
   `summary.truncated` 가 말한다. `by_kind` 는 선언 순서이고 0 건 항목은 빠진다.
5. **`detail` 은 계약이 아니다.** 사람이 읽는 한 줄이다. 기계 판정은 `kind` 와 `path` 로
   한다. 미리보기는 40 **문자**(바이트 아님)에서 자르고 `…` 를 붙인다.
6. **경로 첨자의 기준.** 짝지어진 문단과 삭제된 문단은 A 기준, 추가된 문단은 B 기준
   첨자다(A 에 자리가 없으므로).

## 4. 정렬 — 이 엔진의 본체

```
공통 앞(prefix) 깎기 → 공통 뒤(suffix) 깎기 → 남은 가운데만 LCS → 맞닿은 삭제·추가 짝짓기
```

마지막 단계가 핵심이다. LCS 만 쓰면 한 글자 고친 문단이 `Removed` + `Added` 두 건으로
나온다. 사람이 보기엔 그건 **수정**이다. 그래서 앞뒤로 맞닿은 삭제 덩어리와 추가 덩어리를
앞에서부터 짝지어 `Pair` 로 승격시키고, 남는 쪽만 순수 추가·삭제로 남긴다.

결과:

| 입력 | 기존 자리끼리 맞대기 | `docdiff` |
| --- | --- | --- |
| 문단 1 개 한 글자 수정 | `TextChanged` 1 | `TextChanged` 1 |
| 문단 200 개 맨 앞에 1 줄 삽입 | 차이 201 건 | `ParagraphAdded` **1 건** |
| 가운데 1 줄 삭제 | 뒤 전부 오염 | `ParagraphRemoved` 1 건 |

**비용 상한.** LCS 표가 `LCS_CELL_BUDGET`(100만 칸, `u32` 로 4 MB)을 넘으면 자리끼리
맞대는 방식으로 물러선다. 앞뒤를 먼저 깎으므로 실제 회귀 검증에서 LCS 를 돌리는
"가운데"는 대개 몇 줄뿐이다. 물러선 경우에도 결과는 결정적이다.

**재귀 깊이.** 표 셀 안 문단은 같은 규칙으로 재귀한다. `MAX_DEPTH`(32)는 실제 문서의
중첩 깊이를 한참 넘는 스택 보호용이다.

## 5. 채택 시나리오

이 모듈은 **아무도 아직 쓰지 않는다**(신설 PR 이므로). 아래는 설계가 겨냥한 소비처다.

### 5.1 회귀 시험

`tests/` 의 왕복·변환 테스트가 실패할 때 "다르다" 대신 **근거**를 낸다.

```rust
let diff = docdiff::diff_documents(&before, &after, &DiffOptions::default().max_findings(20));
assert!(diff.identical, "편집 전후 의미 차이:\n{}",
        diff.findings.iter().map(|f| f.to_string()).collect::<Vec<_>>().join("\n"));
```

`Finding::to_string()` 이 `sec[0]/para[3] textChanged: A="..." B="..."` 로 나오므로 실패
메시지가 곧 좌표다. 기존 충실도 게이트를 **대체하지 않고** 그 옆에 선다 — 충실도가
실패했을 때 "그래서 사람이 보는 문서는 뭐가 달라졌나"를 같은 실패 메시지 안에서 답한다.

### 5.2 왕복 검증 (`convert --verify`, `export-hwpx --verify`)

현재 `--verify` 는 IR 차이가 있으면 exit 3 을 내고 상세는 `ir-diff` 로 미룬다
(`main.rs:7816` 등의 "상세는 --json 또는 ir-diff"). 여기에 의미 요약 한 줄을 덧붙일 수
있다 — "충실도 차이 47 건(직렬화 축), 의미 차이 0 건" 이면 사용자에게 **안심해도 된다**고
말할 수 있고, 반대면 진짜 손실이다. 이 판정은 지금 아무도 못 한다.

### 5.3 CLI

- 신규 `doc-diff` 명령(후속 PR): `to_json()` 을 `provenance::marked()` 로 감싸면 봉투
  완성. 차이 있으면 exit 3 — 기존 `ir-diff` 와 같은 게이트 규약.
- 기존 `ir-diff` 를 이 엔진 위로 옮기는 것은 **별도 후속 PR** 이다. 이 PR 은 `main.rs` 를
  건드리지 않는다.

### 5.4 MCP

`hwp_doc_diff` 도구의 결과 타입으로 그대로 쓴다. `FindingKind::label()` 이 안정 키이므로
도구 스키마의 `enum` 을 코드에서 생성할 수 있다(`capabilities --mcp` 가 도구 정의의 단일
출처라는 규약과 정합). 에이전트가 문자열을 파싱하지 않고 `kind` 로 분기한다.

### 5.5 편집 파이프라인 (`edit --verify`, `run` 계획서)

"의도한 것만 바뀌었나"를 **좌표로** 확인한다. 셀 하나를 고쳤으면 findings 는
`sec[0]/para[2]/ctrl[0]/cell[r3,c1]/para[0] textChanged` 정확히 1 건이어야 한다. 지금은
그 단언을 쓸 수단이 없다.

## 6. 비범위

- **파싱하지 않는다.** 입력은 `Document` IR 이다. 파일 열기는 호출자 몫이다.
- **렌더 기하를 보지 않는다.** 화면 위 픽셀 변위는 `render-diff`
  (`src/diagnostics/render_geom_diff.rs`)의 축이다. 두 축은 상호 보완이지 대체가 아니다.
- **직렬화 충실도를 재지 않는다.** `line_segs`·`char_shapes`·원본 바이트 보존은 왕복
  게이트가 본다.
- **글상자·각주·머리말 안으로 아직 재귀하지 않는다.** 표 셀까지다. 컨트롤 개수·종류
  차이는 잡히므로 손실이 조용히 통과하지는 않는다. 확장은 후속 작업.
- **스타일 정의 전 필드를 비교하지 않는다.** 이름·종류·모양 참조까지다. 필드 단위
  충실도는 왕복 게이트의 몫이다.
- **`main.rs` 를 고치지 않는다.** `ir-diff` 추출은 후속 PR 이다.

## 7. 검증

| 게이트 | 결과 |
| --- | --- |
| `cargo build --lib` | 통과 |
| `cargo test --lib docdiff::` | 22 통과 / 0 실패 |
| `cargo test --doc docdiff` | 2 통과 / 0 실패 |
| `cargo clippy --lib --tests -- -D warnings` | 통과 |
| `rustfmt --edition 2021 --check src/docdiff/*.rs` | 통과 |

단위 테스트가 고정하는 것: 같은 문서 → `identical`·차이 0, 빈 문서 대 빈 문서, 한 글자
변경, 가운데 문단 추가·삭제(비오염), 구역 수 변화, 표 행렬 변화, 표 셀 좌표, 컨트롤
개수·종류, 스타일 개수·정의, **결정성**(12 회 호출 동일), `max_findings` 상한과
`truncated`, 상한 0 경계, 불변식 전수, 요약 집계 정확성·순서, 공백 무시, 문단모양 변화,
경로 타입 접근, JSON 봉투 안정성, 카테고리 이름 고유성, 대형 문서 앞머리 삽입.

## 관련 문서

- [`parser_architecture.md`](parser_architecture.md) — 공통 `Document` IR 경계
- [`envelope_provenance.md`](envelope_provenance.md) — 봉투 출처 표지 계약
