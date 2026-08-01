# 요리책 — 실제로 하고 싶은 일들

각 레시피는 그대로 복사해 돌아간다. 설계 배경은
[`python_binding_guide.md`](../../../mydocs/manual/python_binding_guide.md).

## 목차

1. [서식 자동 채우기](#1-서식-자동-채우기)
2. [대량 메일머지](#2-대량-메일머지)
3. [RAG 색인 만들기](#3-rag-색인-만들기)
4. [표를 데이터셋으로](#4-표를-데이터셋으로)
5. [변환 품질 검증](#5-변환-품질-검증)
6. [눈검증 루프 닫기](#6-눈검증-루프-닫기)
7. [보호 문서 다루기](#7-보호-문서-다루기)
8. [아카이브 대장화](#8-아카이브-대장화)
9. [계획으로 안전하게 편집](#9-계획으로-안전하게-편집)
10. [문서 간 전사](#10-문서-간-전사)

---

## 1. 서식 자동 채우기

**하고 싶은 것**: 정부 양식의 누름틀을 채우고, 제대로 채워졌는지 확인한다.

```python
import rhwp

def fill_and_verify(form: str, data: dict, output: str) -> bool:
    # 먼저 어떤 칸이 있는지 본다 — 이름을 추측하지 않는다.
    available = {f.name for f in rhwp.fields(form).fields}
    unknown = set(data) - available
    if unknown:
        raise ValueError(f"없는 누름틀: {unknown}\n있는 것: {sorted(available)}")

    result = rhwp.fill_fields(form, data, out=output, verify=True)

    if result.not_found:
        print(f"채우지 못한 칸: {result.not_found}")
        return False

    # 판정은 예외가 아니라 값이다.
    verify = result.verify
    if verify is None or not verify.identical:
        print(f"저장본 검증 실패: {verify.raw if verify else '(검증 안 함)'}")
        return False

    print(f"{result.filled_count}칸 채움 → {output}")
    return True
```

**동명 누름틀**이 여러 개면 `#순번` 으로 지정한다.

```python
rhwp.fill_fields(form, {"성명#0": "홍길동", "성명#1": "김철수"}, out=out)
```

---

## 2. 대량 메일머지

**하고 싶은 것**: 서식 하나 + 데이터 N행 → 산출물 N개.

```python
import csv
from pathlib import Path

import rhwp

def mail_merge(form: str, rows_csv: str, out_dir: str) -> list[str]:
    out = Path(out_dir)
    out.mkdir(parents=True, exist_ok=True)
    made = []

    with open(rows_csv, encoding="utf-8-sig") as fh:
        for i, row in enumerate(csv.DictReader(fh), 1):
            target = out / f"{i:04d}_{row.get('성명', 'noname')}.hwp"
            result = rhwp.fill_fields(form, row, out=target, verify=True)

            verify = result.verify
            if verify and not verify.identical:
                print(f"  {target.name}: 검증 실패 (차이 {verify.diff_count})")
                continue
            made.append(str(target))
    return made
```

한 건 실패로 전체를 멈추지 않는다 — 실패한 행만 보고하고 나머지를 계속한다.

---

## 3. RAG 색인 만들기

**하고 싶은 것**: 문서를 청크로 나누되 **주소를 잃지 않는다**. 인용할 때 "몇 쪽"을
답할 수 있어야 한다.

```python
import rhwp

def index_document(path: str) -> list[dict]:
    # 절 단위 청킹 — 주소가 보존된다.
    digest = rhwp.digest(path, sections=True)

    chunks = []
    for section in digest.sections:
        chunks.append({
            "text": section.text,
            "source": path,
            "page": section.raw.get("page"),
            "heading": section.raw.get("heading"),
        })
    return chunks
```

**인용 검증** — 답변이 실제로 그 쪽에 있는지 되짚는다.

```python
def verify_citation(path: str, quote: str) -> list[int]:
    """인용문이 실제로 나오는 쪽 번호."""
    hits = rhwp.search(path, quote)
    return [m.page for m in hits.matches if m.raw.get("page") is not None]
```

평문을 추출해 외부에서 검색하면 주소가 소멸한다 — `search` 는 조판 엔진을 거치므로
"몇 쪽"에 답할 수 있다.

---

## 4. 표를 데이터셋으로

**하고 싶은 것**: 보고서 표들을 하나의 데이터프레임으로.

```python
import rhwp

def tables_to_rows(path: str) -> list[dict]:
    rows = []
    for table in rhwp.export_tables(path).tables:
        # 병합된 셀은 좌상단 좌표로만 나온다 — 덮인 좌표는 목록에 없다.
        grid: dict[tuple[int, int], str] = {}
        for cell in table.cells:
            grid[(cell.row, cell.col)] = cell.text

        max_row = max((r for r, _ in grid), default=-1)
        max_col = max((c for _, c in grid), default=-1)

        for r in range(max_row + 1):
            rows.append({
                "table": table.index,
                "row": r,
                **{f"col{c}": grid.get((r, c), "") for c in range(max_col + 1)},
            })
    return rows
```

**셀에 값을 쓸 때**는 같은 좌표를 그대로 쓴다.

```python
rhwp.set_cell(path, table=1, row=0, col=2, text="수정값", out="결과.hwpx")
```

---

## 5. 변환 품질 검증

**하고 싶은 것**: HWP → HWPX 변환이 내용을 잃지 않았는지 확인한다.

```python
import rhwp

def convert_safely(source: str, target: str) -> bool:
    result = rhwp.export_hwpx(source, out=target, verify=True, verify_pages=True)

    verify = result.verify
    if verify is None:
        raise RuntimeError("verify 를 요청했는데 보고가 없다")

    if verify.identical:
        return True

    if verify.reparse_error:
        # 저장본을 다시 읽지 못했다 — 판정 불가가 아니라 실패다.
        print(f"재파싱 실패: {verify.reparse_error}")
        return False

    print(f"IR 차이 {verify.diff_count}건 — 무엇이 달라졌는지 보려면:")
    diff = rhwp.ir_diff(source, target)
    for category, items in (diff.raw.get("categories") or {}).items():
        print(f"  {category}: {len(items) if isinstance(items, list) else items}")
    return False
```

---

## 6. 눈검증 루프 닫기

**하고 싶은 것**: 편집 후 **바뀐 쪽만** 그려서 확인한다. 전 쪽을 그리면 비용이 폭발한다.

```python
from pathlib import Path

import rhwp

def edit_and_show(source: str, target: str, data: dict) -> list[Path]:
    result = rhwp.fill_fields(source, data, out=target, verify=True)

    pages = result.changed_pages
    if pages is None:
        # 확정 불가 — 부분 목록보다 정직하다. 이럴 땐 전체를 보거나 포기한다.
        print("바뀐 쪽을 확정할 수 없습니다 (전체 확인 필요)")
        return []

    made = []
    with rhwp.open(target) as doc:
        for page in pages:
            svg = Path(target).with_suffix(f".p{page}.svg")
            doc.render_page(page, svg)
            made.append(svg)
    return made
```

`None`(모름)과 `[]`(바뀐 쪽 없음)을 구분하는 것이 요점이다. 둘을 falsy 로 뭉뚱그리면
"확인할 게 없다"고 잘못 결론 낸다.

---

## 7. 보호 문서 다루기

```python
import rhwp

with rhwp.open("보호문서.hwp", password="비밀번호") as doc:
    text = doc.text()
    print(text.raw.get("text", "")[:200])
```

암호가 틀리면 `RhwpRuntimeError` 가 난다 — 인자 문제(`UsageError`)가 아니라 문서를
열 수 없는 것이므로 런타임 실패다.

---

## 8. 아카이브 대장화

**하고 싶은 것**: 폴더의 문서 수백 개를 한 번에 조사한다.

```python
from pathlib import Path

import rhwp

def catalog(folder: str) -> tuple[list[dict], list[dict]]:
    paths = sorted(p for p in Path(folder).rglob("*") if p.suffix.lower() in (".hwp", ".hwpx"))
    records = rhwp.batch("info", paths)

    ok = [r for r in records if "error" not in r]
    failed = [r for r in records if "error" in r]
    return ok, failed
```

**부분 실패도 실패지만 성공분은 남는다.** 실패 하나로 스트림을 통째로 버리면
수백 건의 성공까지 잃는다.

대량 작업은 `timeout=None`(무제한)이 기본이다.

---

## 9. 계획으로 안전하게 편집

**하고 싶은 것**: 여러 편집 중 하나라도 불가능하면 **아무것도 하지 않는다**.

```python
import rhwp

def submit_form(form: str, output: str, name: str, dept: str) -> bool:
    plan = (
        rhwp.Plan(form, output)
        .fill_fields({"성명": name, "부서": dept})
        .replace_text("2025년", "2026년")
        .set_checkbox(0)
        .require_all_fields_found()
        .verify()
    )

    # 1) 검사 — 디스크를 건드리지 않는다.
    preview = plan.check()
    if not preview.ok:
        print("계획에 문제가 있습니다:")
        print(preview.describe_violations())
        return False

    for step in preview.preview:
        print(f"  예정: {step.raw}")

    # 2) 실행 — 전 step 이 메모리에서 통과해야 저장한다.
    journal = plan.run()
    verify = journal.verify
    return bool(verify and verify.identical)
```

**위반은 예외가 아니라 결과다.** 계획을 고쳐 다시 검사하는 것이 정상 흐름이다.

계획서를 파일로 저장해 두면 감사 추적·재현이 따라온다.

```python
import json

payload = plan.to_dict()
with open("제출계획.json", "w", encoding="utf-8") as fh:
    json.dump(payload, fh, ensure_ascii=False, indent=2)

# 나중에 그대로 재실행
with open("제출계획.json", encoding="utf-8") as fh:
    rhwp.run_plan(json.load(fh))
```

---

## 10. 문서 간 전사

**하고 싶은 것**: A 문서의 값을 읽어 B 서식에 옮긴다.

```python
import rhwp

def transcribe(source: str, form: str, output: str, mapping: dict[str, str]) -> bool:
    """mapping: {원본_누름틀: 대상_누름틀}"""
    source_values = {f.name: f.raw.get("value", "") for f in rhwp.fields(source).fields}

    data = {}
    for src_name, dst_name in mapping.items():
        if src_name not in source_values:
            print(f"원본에 '{src_name}' 이 없습니다")
            return False
        data[dst_name] = source_values[src_name]

    plan = rhwp.Plan(form, output).fill_fields(data).require_all_fields_found().verify()
    if not plan.check().ok:
        return False
    journal = plan.run()
    return bool(journal.verify and journal.verify.identical)
```

---

## 자주 하는 실수

### `verify` 를 요청하지 않고 통과로 읽기

```python
result = rhwp.fill_fields(form, data, out=out)     # verify 미요청
if result.verify:            # None → falsy → "실패"로 오독
    ...
```

`None` 은 "검증 안 함"이지 "검증 실패"가 아니다. 요청하지 않았으면 판정 자체가 없다.

### `changed_pages` 의 `None` 과 `[]` 를 섞기

```python
if not result.changed_pages:   # None 과 [] 를 같이 잡는다
    print("바뀐 게 없네")       # 틀렸다 — None 은 "모른다"
```

### 좌표를 추측하기

```python
rhwp.set_cell(path, table=0, row=0, col=0, text="값")   # 표 0 이 있다는 보장 없음
```

`export_tables` 로 실존 좌표를 먼저 확인한다. 최상위 표 인덱스가 0 에서 시작하지
않는 문서도 있다.

### 세션을 닫지 않기

```python
doc = rhwp.open("a.hwp")     # with 없이
# ... 예외 발생 ...
# 서버가 남아 파일을 잡고 있다 → 다음 작업이 막힌다
```

항상 `with` 를 쓴다.
