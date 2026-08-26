"""남은 액션을 **하나씩** 걸어 갈래를 분류한다 — 안 끝남·무동작·움직임.

왜 필요한가: 남은 원장의 대부분이 액션인데, 어느 것이 답을 안 주고 어느 것이 관측 가능한지
목록이 없다. 하나씩 짧은 시한으로 걸어 그 지도를 만든다. 시한을 넘기면 한글을 죽이고 다음으로
간다 — 그 이름은 다시 걸지 않는다.

**`HANG` 은 "대화상자"가 아니다.** 화면을 찍어 보니 대화상자가 **안 보이는데도** 호출이 안
끝난다(`tools/hwpctrl_compat/screenshot.ps1` 로 확인). 관측한 사실은 "시한 안에 안 끝난다"
뿐이므로 그 이름만 쓴다 — 보지 않고 "대화상자"라고 적었다가 틀렸다.

    python tools/hwpctrl_compat/sweep_actions.py [--out 결과.tsv] [--limit N]

결과는 TSV 다: 이름, 갈래(HANG/NOOP/MOVED/CHANGED/FAIL), 무엇이 달라졌는지.
"""

from __future__ import annotations

import argparse
import io
import json
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
LEDGER = REPO / "npm" / "hwpctrl-ocx" / "spec" / "api_ledger.json"
SAMPLE = "samples/para-001.hwp"

# **한글을 죽이는** 이름들 — 걸면 COM 서버가 사라져 그 뒤 이름을 전부 잃는다. 실측으로
# 확인된 것만 넣는다.
#
# 예전에는 여기에 "이름을 보니 대화상자겠지" 싶은 것들도 함께 넣어 두었고, 분류기가 이 목록을
# **재 보지도 않고 `HANG` 으로** 채웠다. 그래서 `FindDlg`·`Close`·`SpellingCheck` 따위 39 개가
# "안 끝남"으로 세어졌는데, 하나씩 걸어 보니 **전부 멀쩡히 끝난다**(계획서 §4.70). 짐작을
# 목록에 넣으면 그 짐작이 지도가 된다.
KILLS_HANGUL = {
    "TableStringToTable", "CellBorder", "CellBorderFill",
}

# 재 보고 "시한 안에 안 끝난다"가 확인된 것들.
CONFIRMED_HANG = {
    "PutBullet", "PutParaNumber", "PutOutlineNumber", "ParaNumberBullet",
    "CharShapeHeight", "CharShapeWidth", "CharShapeSpacing",
    # **맥락을 맞춰야 멈춘다**(계획서 §4.73). 글만 있는 문단에서는 몇 초에 끝나 "안 멈춘다"고
    # 볼 뻔했다 — 개체를 고르거나 칸 블록을 잡고 걸어야 대화상자가 뜨고 그대로 멈춘다.
    "ShapeObjDialog", "TablePropertyDialog", "ModifyHyperlink",
}

FORBIDDEN = KILLS_HANGUL | CONFIRMED_HANG


def remaining_actions() -> list[str]:
    doc = json.loads(LEDGER.read_text(encoding="utf-8"))

    def walk(node):
        if isinstance(node, dict):
            if "id" in node and "status" in node:
                yield node
            for value in node.values():
                yield from walk(value)
        elif isinstance(node, list):
            for value in node:
                yield from walk(value)

    names = []
    for item in walk(doc):
        ident = item["id"]
        if not ident.startswith("Action.") or item.get("status") in ("verified", "substituted"):
            continue
        name = ident.split(".", 1)[1]
        if "." in name or name in FORBIDDEN:
            continue
        names.append(name)
    return sorted(names)


# 맥락 — 액션이 먹으려면 미리 만들어 둬야 하는 상태. 글만 있는 문서에서는 개체·표 액션이
# 전부 "무동작"으로 보이는데, 그건 액션이 아니라 **맥락이 없어서**다(실측: 65 중 46이 그랬다).
CONTEXTS = {
    "plain": {
        "sample": "samples/para-001.hwp",
        "setup": [["SetPos", [0, 0, 20]]],
        "reads": [
            ["GetPos", []],
            ["SelectionMode", []],
            ["CharShape.Item", ["Height"]],
            ["ParaShape.Item", ["AlignType"]],
        ],
        "labels": ["캐럿", "모드", "글자높이", "정렬"],
    },
    "object": {
        "sample": "samples/20250130-hongbo.hwp",
        "setup": [["SetPos", [0, 0, 0]], ["Run", ["SelectCtrlFront"]]],
        "reads": [
            ["GetPos", []],
            ["SelectionMode", []],
            ["CurSelectedCtrl.CtrlID", []],
            ["HeadCtrl.Next.Next.Properties.Item", ["Width"]],
            ["HeadCtrl.Next.Next.Properties.Item", ["Height"]],
            ["HeadCtrl.Next.Next.Properties.Item", ["Lock"]],
            ["HeadCtrl.Next.Next.Next.CtrlID", []],
        ],
        "labels": ["캐럿", "모드", "고른개체", "폭", "높이", "잠금", "다음컨트롤"],
    },
    # **그리기 개체**를 고른 상태. `object` 맥락은 hongbo 의 첫 개체가 **표**라, 옮기기·크기·
    # 뒤집기처럼 그리기에만 먹는 액션이 전부 무동작으로 보였다. 개체를 골랐다고 다 같은 개체가
    # 아니다 — 갈래까지 맞춰야 잣대가 닿는다.
    "drawing": {
        "sample": "samples/draw-group.hwp",
        "setup": [["SetPos", [0, 0, 0]], ["Run", ["SelectCtrlFront"]]],
        "reads": [
            ["GetPos", []],
            ["SelectionMode", []],
            ["CurSelectedCtrl.CtrlID", []],
            ["CurSelectedCtrl.UserDesc", []],
            ["HeadCtrl.Next.Next.Properties.Item", ["Width"]],
            ["HeadCtrl.Next.Next.Properties.Item", ["Height"]],
            ["HeadCtrl.Next.Next.Properties.Item", ["TextWrap"]],
            ["HeadCtrl.Next.Next.Properties.Item", ["Lock"]],
        ],
        "labels": ["캐럿", "모드", "고른개체", "이름", "폭", "높이", "배치", "잠금"],
    },
    # **자리차지** 그리기 개체를 고른 상태. `drawing` 맥락의 개체는 **글자처럼** 배치라
    # 옮기기가 안 먹는다 — 배치까지 맞춰야 그 계열이 보인다. 관측창에 위치 항목을 넣는다.
    "floating": {
        "sample": "samples/shape-group-02.hwp",
        "setup": [["SetPos", [0, 0, 0]], ["Run", ["SelectCtrlFront"]]],
        "reads": [
            ["GetPos", []],
            ["SelectionMode", []],
            ["CurSelectedCtrl.CtrlID", []],
            ["HeadCtrl.Next.Next.Properties.Item", ["HorzOffset"]],
            ["HeadCtrl.Next.Next.Properties.Item", ["VertOffset"]],
            ["HeadCtrl.Next.Next.Properties.Item", ["Width"]],
            ["HeadCtrl.Next.Next.Properties.Item", ["Height"]],
            ["HeadCtrl.Next.Next.Properties.Item", ["TextWrap"]],
        ],
        "labels": ["캐럿", "모드", "고른개체", "가로자리", "세로자리", "폭", "높이", "배치"],
    },
    # 셀 안이되 **칸 블록은 없는** 상태. 블록이 있어야 먹는 액션은 여기서도 무동작으로 보이므로
    # `cellblock` 으로 한 번 더 돌려야 한다 — 맥락을 하나만 두면 또 빈 곳을 재게 된다.
    #
    # ⚠ `CellShape` 는 표 크기 조절 뒤에 **한 박자 늦게** 답한다(§4.47). 그래서 `TableResize*`
    # 는 여기서 `NOOP` 으로 나오는데 그건 무동작이라는 뜻이 **아니다** — 관측창이 늦은 것이다.
    # 그 계열은 표 `Properties` 로 봐야 값이 움직이고, 그마저도 읽을 때마다 달라 판정 불가다.
    "cell": {
        "sample": "samples/21868765_별표2_보건소_분장사무.hwp",
        "setup": [["SetPos", [3, 0, 0]]],
        "reads": [
            ["GetPos", []],
            ["SelectionMode", []],
            ["CellShape.Item", ["Width"]],
            ["CellShape.Item", ["Height"]],
            ["CellShape.Item", ["VertAlign"]],
            ["HeadCtrl.Next.Next.CtrlID", []],
        ],
        "labels": ["캐럿", "모드", "칸폭", "칸높이", "세로정렬", "표컨트롤"],
    },
    "cellblock": {
        "sample": "samples/21868765_별표2_보건소_분장사무.hwp",
        "setup": [["SetPos", [3, 0, 0]], ["Run", ["TableCellBlock"]]],
        "reads": [
            ["GetPos", []],
            ["SelectionMode", []],
            ["CellShape.Item", ["Width"]],
            ["CellShape.Item", ["Height"]],
            ["CellShape.Item", ["VertAlign"]],
            ["HeadCtrl.Next.Next.CtrlID", []],
        ],
        "labels": ["캐럿", "모드", "칸폭", "칸높이", "세로정렬", "표컨트롤"],
    },
}


def observables(action: str, ctx: dict) -> list:
    """액션 앞뒤로 같은 것을 읽는다 — 무엇이 달라졌는지 갈래를 보려는 것."""
    return (
        list(ctx["setup"])
        + list(ctx["reads"])
        + list(ctx["setup"])
        + [["Run", [action]]]
        + list(ctx["reads"])
    )


def classify(before: list, after: list, labels: list[str]) -> tuple[str, str]:
    diffs = [
        f"{labels[i]}:{json.dumps(b, ensure_ascii=False)}→{json.dumps(a, ensure_ascii=False)}"
        for i, (b, a) in enumerate(zip(before, after))
        if b != a
    ]
    if not diffs:
        return "NOOP", ""
    only_caret = all(d.startswith("캐럿") for d in diffs)
    return ("MOVED" if only_caret else "CHANGED"), " | ".join(diffs)


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--out", type=Path, default=REPO / "output" / "poc" / "hwpctrl" / "sweep_actions.tsv")
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--timeout", type=int, default=70)
    ap.add_argument("--context", choices=sorted(CONTEXTS), default="plain",
                    help="액션이 먹을 맥락 — plain(글만) | object(개체 고름) | cell(셀 안)")
    ap.add_argument("--only-prefix", help="이 접두어로 시작하는 액션만")
    args = ap.parse_args()

    ctx = CONTEXTS[args.context]
    names = remaining_actions()
    if args.only_prefix:
        names = [n for n in names if n.startswith(args.only_prefix)]
    if args.limit:
        names = names[: args.limit]
    args.out.parent.mkdir(parents=True, exist_ok=True)
    tmp = Path(tempfile.mkdtemp(prefix="sweep-"))
    rows = []
    print(f"액션 {len(names)}개를 하나씩 건다 (시한 {args.timeout}초)")

    for i, action in enumerate(names, 1):
        scenario = {
            "id": f"sweep-{action}",
            "title": action,
            "ledger": [],
            "open": ctx["sample"],
            "calls": observables(action, ctx),
        }
        path = tmp / f"sweep-{action}.json"
        with io.open(path, "w", encoding="utf-8", newline="\n") as fh:
            json.dump(scenario, fh, ensure_ascii=False)
        subprocess.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True, check=False)
        try:
            proc = subprocess.run(
                [sys.executable, str(HERE / "runner_ocx.py"), str(path), "--out", str(tmp),
                 "--expect-version", "12"],
                capture_output=True, timeout=args.timeout, check=False,
            )
        except subprocess.TimeoutExpired:
            rows.append((action, "HANG", "시한 안에 안 끝났다"))
            subprocess.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True, check=False)
            print(f"  [{i}/{len(names)}] {action}: HANG")
            continue
        if proc.returncode != 0:
            rows.append((action, "FAIL", f"종료코드 {proc.returncode}"))
            print(f"  [{i}/{len(names)}] {action}: FAIL {proc.returncode}")
            continue
        data = json.loads((tmp / f"sweep-{action}.returns.json").read_text(encoding="utf-8"))
        read_names = {call[0] for call in ctx["reads"]}
        values = [c.get("value") for c in data["calls"] if c["call"] in read_names]
        half = len(ctx["reads"])
        kind, detail = classify(values[:half], values[half : half * 2], ctx["labels"])
        rows.append((action, kind, detail))
        print(f"  [{i}/{len(names)}] {action}: {kind} {detail[:70]}")

    with io.open(args.out, "w", encoding="utf-8", newline="\n") as fh:
        fh.write("action\tkind\tdetail\n")
        for row in rows:
            fh.write("\t".join(row) + "\n")
    counts: dict[str, int] = {}
    for _, kind, _ in rows:
        counts[kind] = counts.get(kind, 0) + 1
    print(f"\n갈래별: {counts}")
    print(f"→ {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
