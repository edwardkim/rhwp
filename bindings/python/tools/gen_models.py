#!/usr/bin/env python3
"""IR 스키마 → 타입 있는 파이썬 모델 생성기.

`bindings_foundation.md` §3 이 못박은 규약을 코드로 강제한다:
**필드명은 봉투 키를 기계 변환한다 — 수기 개명 금지.**

사람이 이름을 다시 붙이기 시작하면 rhwp 가 IR 에 필드를 하나 더할 때마다 바인딩이
뒤처지고, 어느 쪽이 맞는지 아무도 모르게 된다. 생성기를 두면 스키마가 곧 진실이다.

사용법::

    python tools/gen_models.py                     # stdout 으로
    python tools/gen_models.py -o src/rhwp/ir.py   # 파일로
    python tools/gen_models.py --check             # 최신인지 검사 (CI 용)

``--check`` 는 생성 결과가 디스크와 다르면 exit 1 이다. IR 이 바뀌었는데 모델을
다시 만들지 않은 PR 을 CI 가 잡는다.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path
from typing import List, Optional, Sequence

# 설치 없이 돌 수 있게.
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from rhwp._naming import reserved_safe, to_snake  # noqa: E402
from rhwp.schema import IrSchema, TypeDef, ir_schema  # noqa: E402

HEADER = '''"""IR 타입 모델 — **자동 생성 파일. 손으로 고치지 마세요.**

생성: ``python tools/gen_models.py -o src/rhwp/ir.py``
출처: ``rhwp export-ir-schema`` (irSchemaVersion {version})

이 파일을 직접 수정하면 다음 생성 때 덮어써집니다. 모양을 바꾸려면 rhwp 본체의
``src/ir_schema.rs`` 를 고치세요 — 스키마가 단일 출처입니다.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional

#: 이 모델이 생성된 IR 스키마 버전.
IR_SCHEMA_VERSION = "{version}"

'''


def _indent(text: str, level: int = 1) -> str:
    pad = "    " * level
    return "\n".join(pad + line if line.strip() else line for line in text.splitlines())


def _docstring(text: str, level: int = 1) -> str:
    """설명을 docstring 으로. 여러 줄이면 접어서."""
    cleaned = " ".join(text.split())
    if not cleaned:
        return ""
    pad = "    " * level
    if len(cleaned) + len(pad) + 6 <= 96:
        return f'{pad}"""{cleaned}"""\n'
    # 긴 설명은 줄바꿈해 넣는다.
    words = cleaned.split()
    lines: List[str] = []
    current = ""
    for word in words:
        if len(current) + len(word) + 1 > 88 - len(pad):
            lines.append(current)
            current = word
        else:
            current = f"{current} {word}".strip()
    if current:
        lines.append(current)
    body = "\n".join(pad + line for line in lines)
    return f'{pad}"""\n{body}\n{pad}"""\n'


def _default_for(hint: str) -> Optional[str]:
    """선택 필드의 기본값. 가변 기본값은 ``field(default_factory=...)`` 로."""
    if hint.startswith("List["):
        return "field(default_factory=list)"
    if hint.startswith("Dict["):
        return "field(default_factory=dict)"
    if hint.startswith("Optional["):
        return "None"
    return "None"


def _optional(hint: str) -> str:
    """선택 필드의 타입 힌트 — 이미 Optional/컨테이너면 그대로."""
    if hint.startswith(("Optional[", "List[", "Dict[")):
        return hint
    return f"Optional[{hint}]"


def render_type(type_def: TypeDef) -> str:
    """정의 하나를 dataclass 로."""
    if type_def.is_union:
        variants = " | ".join(type_def.variants) if type_def.variants else "Any"
        doc = _docstring(type_def.description or f"{type_def.name} 유니온", 0)
        alias = f"{type_def.name} = Any  # oneOf: {variants}\n"
        return f"{doc}{alias}\n"

    lines: List[str] = ["@dataclass", f"class {type_def.name}:"]
    doc = _docstring(type_def.description or type_def.name)
    if doc:
        lines.append(doc.rstrip("\n"))

    fields = type_def.fields
    if not fields:
        lines.append("    pass")
        return "\n".join(lines) + "\n\n"

    for f in fields:
        attr = reserved_safe(to_snake(f.name))
        hint = f.python_type
        comment_parts: List[str] = []
        if attr != f.name:
            comment_parts.append(f"봉투 키: {f.name}")
        if f.enum_values:
            comment_parts.append("허용: " + "/".join(f.enum_values))
        comment = f"  # {' · '.join(comment_parts)}" if comment_parts else ""

        if f.required:
            lines.append(f"    {attr}: {hint}{comment}")
        else:
            lines.append(f"    {attr}: {_optional(hint)} = {_default_for(hint)}{comment}")

        if f.description:
            lines.append(_docstring(f.description, 1).rstrip("\n"))

    return "\n".join(lines) + "\n\n"


def _topological(schema: IrSchema) -> List[TypeDef]:
    """참조 순서로 정렬 — 전방 참조를 줄인다 (``from __future__`` 로 안전하지만
    읽기 좋게).
    """
    remaining = {t.name: t for t in schema}
    emitted: List[TypeDef] = []
    seen: set = set()

    def visit(name: str, stack: set) -> None:
        if name in seen or name not in remaining:
            return
        if name in stack:
            return  # 순환 — 전방 참조로 남긴다.
        stack.add(name)
        type_def = remaining[name]
        for f in type_def.fields:
            for target in (f.ref, f.item_ref):
                if target:
                    visit(target, stack)
        for variant in type_def.variants:
            visit(variant, stack)
        stack.discard(name)
        if name not in seen:
            seen.add(name)
            emitted.append(type_def)

    for name in sorted(remaining):
        visit(name, set())
    return emitted


def generate(schema: IrSchema) -> str:
    """스키마 전체를 파이썬 모듈 소스로."""
    dangling = schema.dangling_references()
    if dangling:
        detail = ", ".join(f"{src} → {target}" for src, target in dangling)
        raise SystemExit(f"끊어진 참조가 있어 생성을 중단합니다: {detail}")

    parts = [HEADER.format(version=schema.version)]
    for type_def in _topological(schema):
        parts.append(render_type(type_def))

    names = sorted(t.name for t in schema)
    exports = ",\n".join(f'    "{n}"' for n in names)
    parts.append(f'__all__ = [\n    "IR_SCHEMA_VERSION",\n{exports},\n]\n')
    return "".join(parts)


def main(argv: Optional[Sequence[str]] = None) -> int:
    parser = argparse.ArgumentParser(description="IR 스키마에서 파이썬 모델을 생성합니다.")
    parser.add_argument("-o", "--out", type=Path, help="출력 파일 (없으면 stdout)")
    parser.add_argument(
        "--check",
        action="store_true",
        help="생성 결과가 디스크와 같은지 검사만 한다 (CI 용, 다르면 exit 1)",
    )
    args = parser.parse_args(argv)

    if args.check and not args.out:
        parser.error("--check 는 -o 와 함께 써야 합니다")

    try:
        schema = ir_schema()
    except Exception as exc:  # noqa: BLE001 - CLI 진입점
        print(f"오류: IR 스키마를 읽지 못했습니다 — {exc}", file=sys.stderr)
        print("  rhwp 바이너리가 PATH 에 있는지, RHWP_BIN 이 맞는지 확인하세요.", file=sys.stderr)
        return 1

    source = generate(schema)

    if args.check:
        assert args.out is not None
        if not args.out.exists():
            print(f"오류: {args.out} 이 없습니다 — 생성기를 먼저 돌리세요.", file=sys.stderr)
            return 1
        current = args.out.read_text(encoding="utf-8")
        if current != source:
            print(
                f"오류: {args.out} 이 최신이 아닙니다.\n"
                f"  IR 스키마(v{schema.version})가 바뀌었습니다 — "
                f"python tools/gen_models.py -o {args.out} 를 다시 돌리세요.",
                file=sys.stderr,
            )
            return 1
        print(f"{args.out} 최신 (IR v{schema.version}, 정의 {len(schema)}개)")
        return 0

    if args.out:
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text(source, encoding="utf-8")
        print(f"생성: {args.out} (IR v{schema.version}, 정의 {len(schema)}개)")
    else:
        sys.stdout.write(source)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
