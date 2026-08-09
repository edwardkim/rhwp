# API 레퍼런스 — `rhwp` 파이썬 패키지

전체 공개 API. 설계 근거는 [`python_binding_guide.md`](../../../mydocs/manual/python_binding_guide.md),
계약의 원천은 rhwp 본체의 `capabilities` 자기서술이다.

## 목차

- [모듈 수준](#모듈-수준)
- [1층 — 무상태 명령](#1층--무상태-명령)
- [2층 — 세션](#2층--세션)
- [3층 — 계획](#3층--계획)
- [IR 스키마](#ir-스키마)
- [모델](#모델)
- [예외](#예외)
- [저수준](#저수준)

---

## 모듈 수준

### `rhwp.__version__`

바인딩 패키지 버전. rhwp 본체 버전과 별개다.

### `rhwp.SUPPORTED_SCHEMA_VERSION`

이 바인딩이 검증한 봉투 스키마 버전(`"1.0"`). rhwp 본체가 major 를 올리면 여기도 올린다.

### `rhwp.find_binary(*, refresh=False) -> Path`

rhwp 실행 파일 경로를 돌려준다. 탐색 순서는 `RHWP_BIN` → 패키지 동봉 → `PATH`.

```python
>>> rhwp.find_binary()
PosixPath('/usr/local/bin/rhwp')
```

**예외**: `BinaryNotFoundError` — 세 경로 모두 실패. 메시지에 시도한 위치를 전부 담는다.

환경변수를 **줬는데 못 쓰면** 조용히 다음 경로로 넘어가지 않고 즉시 실패한다.

### `rhwp.clear_cache() -> None`

탐색 캐시를 비운다. 테스트에서 환경변수를 바꿔 가며 검사할 때 쓴다.

### `rhwp.ENV_VAR`

바이너리 경로 환경변수 이름(`"RHWP_BIN"`).

### `rhwp.binary_name()` / `rhwp.BUNDLED_DIR`

현재 플랫폼의 실행 파일 이름과 패키지 동봉 바이너리 디렉터리다. 배포 점검이나
테스트에서 실행 파일 탐색 위치를 확인할 때 쓴다.

---

## 1층 — 무상태 명령

호출 하나 = 프로세스 하나 = 문서 재파싱 하나. 같은 문서를 반복해서 만질 거라면
[2층](#2층--세션)이 빠르다.

모든 함수는 `timeout: Optional[float]` 을 받는다(기본 300초, `None` 이면 무제한)
그리고 [`Envelope`](#envelope) 를 돌려준다.

### 조회

| 함수 | 대응 CLI | 돌려주는 것 |
|---|---|---|
| `info(path)` | `info` | 포맷·쪽수·구역수·문단수·글꼴 |
| `export_text(path, *, page=None, max_chars=None)` | `export-text` | 쪽별 평문 |
| `export_structure(path, *, mode=None)` | `export-structure` | 제목 계층·절 |
| `export_tables(path)` | `export-tables` | 표 전량 + 셀 좌표 |
| `fields(path)` | `fields` | 누름틀 목록 |
| `capabilities(*, mcp=False)` | `capabilities` | 도구 자기서술 |

```python
meta = rhwp.info("보고서.hwp")
print(meta.page_count, meta.format)

tables = rhwp.export_tables("양식.hwpx").tables
print(tables[0].index, len(tables[0].cells))   # set_cell 에 쓸 좌표
```

### `search(path, query, *, case_sensitive=True, limit=None)`

주소가 붙은 검색 — 매치마다 (구역·문단·**쪽**·문자 오프셋).

```python
for m in rhwp.search("보고서.hwp", "예산").matches:
    print(f"{m.page}쪽 {m.paragraph}문단: {m.snippet}")
```

`-` 로 시작하는 검색어도 그대로 넘길 수 있다 — 내부에서 `--` 구분자를 쓴다.

### `digest(path, *, sections=False, pages=None, max_chars=None)`

요약·RAG 용 청킹. `sections=True` 는 주소를 보존한 절 단위, `pages="1-5"` 는 쪽 범위 창.

### 산출

| 함수 | 대응 CLI |
|---|---|
| `export_svg(path, *, out=None, page=None)` | `export-svg` |
| `export_pdf(path, *, out=None)` | `export-pdf` |
| `export_markdown(path, *, out=None)` | `export-markdown` |
| `export_hml(path, *, out=None)` | `export-hml` |
| `export_doclang(path, *, out=None)` | `export-doclang` |
| `thumbnail(path, *, out=None)` | `thumbnail` |
| `extract_pages(path, pages, *, out=None)` | `extract-pages` |
| `build_from_ingest(spec, *, out=None)` | `build-from-ingest` |

### 변환·비교

### `export_hwpx(path, *, out=None, verify=False, verify_pages=False, raise_on_verdict=False)`

HWP → HWPX 변환. `verify=True` 면 봉투에 `verify.identical` 이 담긴다.

```python
result = rhwp.export_hwpx("원본.hwp", out="변환본.hwpx", verify=True)
if not result.verify.identical:
    print(f"차이 {result.verify.diff_count}건")   # 예외가 아니라 판정이다
```

### `convert(path, *, out, verify=False, raise_on_verdict=False)`

HWPX → HWP 변환. `out`은 CLI의 위치 인자이며 필수다. 생략하면 프로세스를
실행하기 전에 `UsageError`가 발생한다.

### `ir_diff(a, b, *, section=None, paragraph=None)`

두 문서의 IR 차이를 범주별로.

### `render_diff(path, path_b=None, *, via=None, page=None, max_disp=None, raise_on_verdict=False)`

페이지 렌더 결과를 비교한다. `path_b`를 생략하면 한 문서의 렌더 왕복을 검사하고,
지정하면 두 문서의 전/후 렌더를 비교한다.

### 편집

세 함수 모두 `out`·`dry_run`·`verify`·`raise_on_verdict` 를 받는다.

### `fill_fields(path, data, *, out=None, dry_run=False, verify=False)`

누름틀 채우기(메일머지). `data` 는 `{"필드이름": "값"}`, 동명 필드는 `"이름#1"` 로 순번 지정.

```python
result = rhwp.fill_fields("서식.hwp", {"성명": "홍길동"}, out="제출본.hwp", verify=True)
print(result.filled_count, result.not_found, result.changed_pages)
```

### `replace_text(path, find, replace, *, occurrence=None, ignore_case=False, ...)`

문자열 치환. `occurrence` 를 주면 그 순번 하나만.

### `set_cell(path, table, row, col, text, *, keep_style=False, ...)`

표 셀 기록. 좌표는 `export_tables` 로 확인한다. 병합된 셀은 좌상단 좌표로만 접근한다.

### 대량

### `batch(subcommand, paths, *, extra_args=(), timeout=None)`

폴더/목록 일괄 처리. NDJSON 레코드 **목록**(dict)을 돌려준다.

```python
for r in rhwp.batch("export-text", ["a.hwp", "b.hwp"]):
    if "error" in r:
        print(f"실패: {r['source']} — {r['error']}")
```

부분 실패도 실패지만 **성공분은 남는다**. 스트림을 통째로 버리지 말 것.

---

## 2층 — 세션

### `rhwp.open(path, *, password=None, session=None, profile=None) -> Document`

문서를 열어 핸들을 돌려준다. `with` 문에서 쓰면 자동으로 닫힌다.

- `password`: 보호 문서 암호
- `session`: 이미 만든 `Session` 에 얹는다(주면 문서를 닫아도 세션은 남는다)
- `profile`: 새 세션의 역할 프로필(도구 노출 범위 제한)

### `class Document`

| 메서드 | 대응 도구 |
|---|---|
| `info()` | `hwp_doc_info` |
| `text(*, page=None)` | `hwp_doc_text` |
| `fields()` | `hwp_doc_fields` |
| `tables()` | `hwp_doc_tables` |
| `search(query, *, case_sensitive=True)` | `hwp_doc_search` |
| `render_page(page, output)` | `hwp_doc_render_page` |
| `fill_fields(data)` | `hwp_doc_fill_fields` |
| `replace_text(find, replace, *, case_sensitive=True)` | `hwp_doc_replace_text` |
| `set_cell(table, row, col, text)` | `hwp_doc_set_cell` |
| `save(output, *, verify=False)` | `hwp_doc_save` |
| `close()` | `hwp_close` |

```python
with rhwp.open("서식.hwp") as doc:
    doc.fill_fields({"성명": "홍길동"})
    saved = doc.save("제출본.hwp", verify=True)

    for page in saved.changed_pages or []:
        doc.render_page(page, f"확인_{page}.svg")   # 바뀐 쪽만 — 상수 비용
```

**속성**: `doc_id` — 서버가 발급한 핸들 식별자.

**예외**: `SessionClosedError` — 닫힌 핸들 재사용.

### `class Session(profile=None, timeout=300.0, cwd=None)`

`mcp-serve` 자식 프로세스 하나를 감싼 JSON-RPC 클라이언트. 여러 문서를 한 서버에서
열고 싶을 때만 직접 만든다.

- `call(name, arguments) -> Envelope` — 도구 하나 호출
- `close()` — 서버 정리(멱등)
- `cwd`: 자식 `mcp-serve`의 작업 디렉터리. 상대 경로 문서를 열 때 지정한다.

`timeout` 값은 생성자 호환용으로 보관되며, 현재 Python stdio 구현은 개별 요청의
블로킹 읽기를 중단하지 못한다. 요청 제한 시간은 1층 함수의 `timeout`을 사용하거나
별도 프로세스 경계를 둔다.

`with` 를 쓰면 예외로 빠져나가도 정리된다. **서버가 남으면 다음 작업이 파일을 못 연다.**

---

## 3층 — 계획

### `class Plan(input_path, output_path)`

체이닝으로 step 을 쌓는 빌더. 빌더는 **문법만** 검사하고(값 타입·필수 인자),
실행 가능성은 rhwp 의 선검증이 판정한다 — 판정자를 두 곳에 두면 어긋난다.

| 메서드 | 설명 |
|---|---|
| `fill_fields(data)` | 누름틀 채우기 |
| `replace_text(find, replace, *, occurrence=None, case_sensitive=True)` | 치환 |
| `set_cell(table, row, col, text, *, keep_style=False)` | 셀 기록 |
| `set_checkbox(occurrence)` | 빈 체크박스(□) 표시 |
| `verify(enabled=True)` | 저장 직후 자기검증 요구 |
| `require_all_fields_found(enabled=True)` | 못 찾은 필드 0 단언 |
| `to_dict(*, dry_run=False)` | 계획서 JSON 구조 |
| `check()` | **디스크 무변경** 검사 → `PlanResult` |
| `run()` | 실행 → `PlanResult` |

```python
plan = (rhwp.Plan("서식.hwp", "제출본.hwp")
        .fill_fields({"성명": "홍길동"})
        .set_checkbox(1)
        .verify())

preview = plan.check()
if preview.ok:
    plan.run()
else:
    print(preview.describe_violations())
```

### `class PlanResult(Envelope)`

| 속성 | 설명 |
|---|---|
| `ok` | 위반 없이 통과했는가 |
| `violations` | 선검증 위반 목록(`List[Envelope]`) |
| `is_dry_run` | 검사 전용 실행이었는가 |
| `preview` | 검사 모드의 step 별 미리보기 |
| `steps` | 실행 모드의 step 별 결과 |
| `describe_violations()` | 위반을 사람이 읽을 여러 줄로 |

**위반은 예외가 아니라 결과다** — 계획을 고쳐 다시 검사하는 것이 정상 흐름이다.

### `run_plan(plan, *, timeout=...)`

이미 만들어 둔 계획서(dict)를 그대로 실행한다. JSON 파일에서 읽어온 계획용.

---

## IR 스키마

### `rhwp.ir_schema(*, timeout=...) -> IrSchema`

`export-ir-schema` 를 읽어 온다. 문서를 입력으로 받지 않는다 — 스키마는 **타입의
자기서술**이지 특정 문서의 속성이 아니다.

### `rhwp.capabilities_schema(*, timeout=...) -> IrSchema`

`export-capabilities-schema` 를 읽어 온다. 명령·플래그·MCP 매니페스트를 설명하는
JSON Schema라서 외부 도구가 rhwp 명령 표면을 생성·검증할 때 쓴다. 문서를 입력으로
받지 않는다.

`rhwp.ir_schema_envelope()` 및 `rhwp.capabilities_schema_envelope()`은 각각의 원문
봉투와 `definitionCount` 같은 메타를 그대로 받는 경우에 사용한다.

### `class IrSchema`

| 멤버 | 설명 |
|---|---|
| `version` | IR 또는 명령 표면 스키마 버전(봉투 `schemaVersion` 과 별개) |
| `dialect` | JSON Schema 방언 URI |
| `root` | 루트 타입(`Document` 또는 `Capabilities`) |
| `names()` | 정의 이름 목록 |
| `dangling_references()` | 끊어진 `$ref` 를 (참조한 곳, 없는 이름) 으로 |
| `raw` | 원문 스키마 본문 |
| `schema[name]` / `name in schema` / `iter(schema)` / `len(schema)` | 매핑 프로토콜 |

### `class TypeDef`

| 멤버 | 설명 |
|---|---|
| `name`, `description` | 이름·설명 |
| `is_object`, `is_union` | 종류 |
| `variants` | 유니온이면 변형 이름 목록 |
| `fields` | 필드 목록(필수가 앞) |
| `field(name)` | 이름으로 하나 |

### `class FieldDef`

| 멤버 | 설명 |
|---|---|
| `name`, `description`, `required` | 기본 |
| `json_type` | JSON 타입 |
| `ref`, `item_ref` | 참조 대상 |
| `enum_values` | 열거형 허용 값 |
| `python_type` | 파이썬 타입 힌트 문자열(코드 생성기가 쓴다) |

---

## 모델

### `class Envelope`

봉투 하나를 감싸는 읽기 전용 매핑. 세 방식으로 같은 값에 닿는다.

```python
env.page_count      # 속성 (snake_case)
env["pageCount"]    # 원문 키
env["page_count"]   # 변환 키
```

| 멤버 | 설명 |
|---|---|
| `raw` | 원문 봉투 **사본** |
| `schema_version` | 봉투 스키마 버전 |
| `verify` | `VerifyReport` 또는 `None`(검증 안 함) |
| `verify_pages` | `VerifyPagesReport` 또는 `None`(쪽수 검증 안 함) |
| `changed_pages` | `List[int]` 또는 `None`(확정 불가) |
| `get_path("verify.identical", default=None)` | 점 경로 조회 |

**없는 필드는 조용히 `None` 이 되지 않는다** — `AttributeError`/`KeyError` 와 함께
있는 필드를 알려준다.

**읽기 전용** — 도구가 내놓은 판정을 호출자가 고치지 않는다.

### `class VerifyReport(Envelope)`

| 멤버 | 설명 |
|---|---|
| `identical` | 저장본이 메모리 IR 과 같은가(판정의 전부) |
| `diff_count` | 차이 개수. 재파싱 실패면 `None` |
| `reparse_error` | 저장본을 못 읽었을 때의 사유 |
| `__bool__` | `identical` 과 같다 |

### `class VerifyPagesReport(Envelope)`

| 멤버 | 설명 |
|---|---|
| `before` | 저장 전 메모리 IR 쪽수 |
| `after` | 저장 후 재파싱한 쪽수 |
| `identical` | 저장 전후 쪽수가 같은가 |
| `__bool__` | `identical` 과 같다 |

---

## 예외

```
RhwpError
├── BinaryNotFoundError    실행 파일 없음
├── UsageError             exit 2 — 호출 조립 버그 (.suggestion 으로 힌트)
├── RhwpRuntimeError       exit 1 — 읽기·파싱·렌더·쓰기 실패
├── VerdictFailed          exit 3/4 — raise_on_verdict=True 일 때만
├── ProtocolError          stdout 이 계약 위반
├── SessionClosedError     닫힌 핸들 재사용
└── RhwpTimeoutError       제한 시간 초과
```

새 코드는 내장 예외와 구별되는 `RhwpTimeoutError`를 사용한다. 이전 버전 호환을 위해
`TimeoutError`는 같은 클래스를 가리키는 별칭으로만 유지한다.

모든 예외가 갖는 것:

| 속성 | 설명 |
|---|---|
| `message` | 사람이 읽을 설명 |
| `argv` | 실행한 명령줄 |
| `command` | 재현 가능한 명령 문자열(버그 리포트용) |
| `exit_code` | 종료 코드 |
| `stderr` | 도구 진단 원문 |
| `envelope` | 파싱된 봉투(판정 근거 보존) |

### 종료 코드 상수

`EXIT_OK`(0) · `EXIT_RUNTIME`(1) · `EXIT_USAGE`(2) · `EXIT_VERIFY`(3) · `EXIT_VERIFY_PAGES`(4)

---

## 저수준

직접 쓸 일은 드물지만, 바인딩이 아직 감싸지 않은 명령을 부를 때 필요하다.

### `run_json(args, *, stdin=None, timeout=..., cwd=None, raise_on_verdict=False) -> dict`

`--json` 명령을 실행하고 봉투를 dict 로. 종료 코드 검사는 **파싱 뒤**에 한다 —
exit 3 일 때도 봉투에 판정 근거가 있기 때문이다.

### `run_ndjson(args, ...) -> List[dict]`

batch 계열. 부분 실패를 예외로 올리지 않는다.

### `run_raw(args, *, check=True, ...) -> CompletedRun`

원문 결과(`argv`·`exit_code`·`stdout`·`stderr`).

### `to_snake(name)` / `to_camel(name)`

이름 변환 규칙. 봉투 키와 파이썬 속성 사이를 잇는다.
