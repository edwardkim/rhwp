#!/usr/bin/env python
"""#2269 PDF 결정성 비교 — svg2pdf 폰트 HashMap 비결정을 정규화해 회귀 diff 가능하게.

배경: `export-pdf` 동일 입력 2회 실행이 바이트 diff 를 낸다. 원인은 상위 크레이트
svg2pdf-0.13 `util/context.rs:81 for font in self.fonts.values_mut()` 의 HashMap
반복 순서(프로세스별 랜덤 시드)로, 폰트 객체 번호와 `/foN` 리소스 이름이 실행마다
다른 폰트에 배정된다(시각 출력·파일 크기는 동일).

이 도구는 두 PDF 의 (a) 간접 객체 번호와 (b) `/foN` 폰트 리소스 이름을 **내용 기준
정준 순서**로 재배치해, 비결정 성분을 제거한 뒤 바이트 비교한다. 정규화 후 동일하면
"시각/구조 불변, 채번만 비결정" 으로 판정 — 회귀 diff 게이트가 이 정규화 형을 쓰면 된다.

사용:
    python tools/pdf_normalize_compare.py a.pdf b.pdf
    python tools/pdf_normalize_compare.py --emit a.pdf > a.norm   # 정규화 형 출력(게이트용)
종료코드: 0=정규화 동일, 1=정규화 후에도 상이(진짜 회귀), 2=인자 오류.
"""
import re
import sys

OBJ_DEF = re.compile(rb"(\d+) 0 obj")
OBJ_REF = re.compile(rb"(\d+) 0 R")
FONT_NAME = re.compile(rb"/fo(\d+)\b")


def parse_objects(data: bytes):
    """{obj_num: body_bytes} — 'N 0 obj' .. 'endobj' 구간."""
    objs = {}
    for m in re.finditer(rb"(\d+) 0 obj(.*?)endobj", data, re.S):
        objs[int(m.group(1))] = m.group(2)
    return objs


def canonical_ref_order(data: bytes):
    """객체 번호를 '바이트 스트림 최초 등장(정의 or 참조)' 순으로 정준화한 매핑."""
    order = []
    seen = set()
    for m in re.finditer(rb"\b(\d+) 0 (?:obj|R)\b", data):
        n = int(m.group(1))
        if n not in seen:
            seen.add(n)
            order.append(n)
    return {old: i + 1 for i, old in enumerate(order)}


def canonical_font_names(data: bytes):
    """/foN 이름을 각 이름이 가리키는 객체의 '정준 번호' 기준으로 재명명.

    /Font << /foN <obj> 0 R ... >> 에서 (이름 -> 대상 객체) 를 모아, 대상 객체의
    정준 번호로 정렬해 fo0..foK 재배정. 같은 폰트(같은 대상)는 실행 무관 동일 이름."""
    refmap = canonical_ref_order(data)
    # /foN <int> 0 R 페어 수집
    pairs = {}
    for m in re.finditer(rb"/fo(\d+)\s+(\d+) 0 R", data):
        name = int(m.group(1))
        target = int(m.group(2))
        pairs.setdefault(name, refmap.get(target, 10**9))
    # 대상 정준번호 -> 새 이름
    ordered = sorted(pairs.items(), key=lambda kv: kv[1])
    return {old_name: i for i, (old_name, _) in enumerate(ordered)}


def _mask(body: bytes) -> bytes:
    """정렬 키용 — 참조 번호·폰트 이름을 마스킹해 채번 무관 내용만 남긴다."""
    b = re.sub(rb"\b\d+ 0 R\b", b"REF", body)
    b = re.sub(rb"/fo\d+\b", b"/fo", b)
    return b


def normalize(data: bytes) -> bytes:
    """객체를 내용 기준으로 재정렬 + 채번/폰트이름 정준화한 정규형을 만든다.

    svg2pdf 의 HashMap 비결정은 (1) 객체 방출 순서 (2) 참조 번호 (3) /foN 이름에
    모두 나타나므로, 객체를 마스킹 내용으로 정렬해 순서를 고정하고, 그 정준 순서로
    번호·이름을 재배정한다. 동일 시각/구조면 정규형이 바이트 동일해진다."""
    objs = {}
    for m in re.finditer(rb"(\d+) 0 obj(.*?)endobj", data, re.S):
        objs[int(m.group(1))] = m.group(2)
    # 내용(마스킹) 기준 정렬 → 정준 순서
    order = sorted(objs.keys(), key=lambda n: (_mask(objs[n]), n))
    refmap = {old: i + 1 for i, old in enumerate(order)}
    # 폰트 이름: /foN -> 대상 객체 정준번호 순
    pairs = {}
    for m in re.finditer(rb"/fo(\d+)\s+(\d+) 0 R", data):
        pairs.setdefault(int(m.group(1)), refmap.get(int(m.group(2)), 10**9))
    namemap = {old: i for i, (old, _) in enumerate(sorted(pairs.items(), key=lambda kv: kv[1]))}

    def sort_font_dict(m) -> bytes:
        # /Font << /foN X 0 R ... >> 항목을 이름 기준 정렬 (dict 순서 비결정 제거)
        entries = re.findall(rb"/fo\d+\s+\d+ 0 R", m.group(1))
        entries.sort()
        return b"/Font <<" + b" ".join(entries) + b">>"

    def rewrite(body: bytes) -> bytes:
        b = re.sub(rb"\b(\d+) 0 R\b", lambda m: b"%d 0 R" % refmap.get(int(m.group(1)), 0), body)
        b = re.sub(rb"/fo(\d+)\b", lambda m: b"/fo%d" % namemap.get(int(m.group(1)), int(m.group(1))), b)
        # /Font 딕셔너리 항목 정렬
        b = re.sub(rb"/Font <<(.*?)>>", sort_font_dict, b, flags=re.S)
        return b

    parts = []
    for old in order:
        parts.append(b"%d 0 obj" % refmap[old] + rewrite(objs[old]) + b"endobj\n")
    return b"".join(parts)


def _read(path: str) -> bytes:
    """입력 파일 읽기 — 부재/접근불가 시 traceback 없이 인자 오류(exit 2)로 종료."""
    try:
        with open(path, "rb") as f:
            return f.read()
    except OSError as e:
        print(f"입력 파일을 열 수 없음: {path} ({e.strerror})", file=sys.stderr)
        raise SystemExit(2)


def main() -> int:
    try:
        sys.stdout.reconfigure(encoding="utf-8")
        sys.stderr.reconfigure(encoding="utf-8")
    except Exception:
        pass
    args = [a for a in sys.argv[1:] if a != "--emit"]
    emit = "--emit" in sys.argv
    if emit:
        if len(args) != 1:
            print("사용: pdf_normalize_compare.py --emit a.pdf", file=sys.stderr)
            return 2
        sys.stdout.buffer.write(normalize(_read(args[0])))
        return 0
    if len(args) != 2:
        print(__doc__, file=sys.stderr)
        return 2
    a = normalize(_read(args[0]))
    b = normalize(_read(args[1]))
    if a == b:
        print(f"정규화 동일 — 채번만 비결정(시각/구조 불변). len={len(a)}")
        return 0
    print(f"정규화 후에도 상이 — 진짜 회귀. lenA={len(a)} lenB={len(b)}", file=sys.stderr)
    for i, (x, y) in enumerate(zip(a, b)):
        if x != y:
            print(f"  첫 diff @ {i}\n  A: {a[max(0,i-40):i+40]!r}\n  B: {b[max(0,i-40):i+40]!r}", file=sys.stderr)
            break
    return 1


if __name__ == "__main__":
    sys.exit(main())
